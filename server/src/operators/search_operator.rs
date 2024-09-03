use super::chunk_operator::{
    get_chunk_metadatas_and_collided_chunks_from_point_ids_query,
    get_content_chunk_from_point_ids_query, get_highlights, get_highlights_with_exact_match,
    get_qdrant_ids_from_chunk_ids_query, get_slim_chunks_from_point_ids_query, HighlightStrategy,
};
use super::group_operator::{
    get_group_ids_from_tracking_ids_query, get_groups_from_group_ids_query,
};
use super::model_operator::{
    cross_encoder, get_bm25_embeddings, get_dense_vector, get_sparse_vector,
};
use super::qdrant_operator::{
    count_qdrant_query, search_over_groups_query, GroupSearchResults, QdrantSearchQuery, VectorType,
};
use super::typo_operator::correct_query;
use crate::data::models::{
    convert_to_date_time, ChunkGroup, ChunkGroupAndFileId, ChunkMetadata, ChunkMetadataTypes,
    ConditionType, ContentChunkMetadata, Dataset, DatasetConfiguration, GeoInfoWithBias,
    HasIDCondition, QdrantSortBy, QueryTypes, ReRankOptions, RedisPool, ScoreChunk, ScoreChunkDTO,
    SearchMethod, SlimChunkMetadata, SortByField, SortBySearchType, SortOrder, UnifiedId,
};
use crate::handlers::chunk_handler::{
    AutocompleteReqPayload, ChunkFilter, CountChunkQueryResponseBody, CountChunksReqPayload,
    ParsedQuery, ParsedQueryTypes, ScoringOptions, SearchChunkQueryResponseBody,
    SearchChunksReqPayload,
};
use crate::handlers::group_handler::{
    SearchOverGroupsReqPayload, SearchWithinGroupReqPayload, SearchWithinGroupResults,
};
use crate::operators::qdrant_operator::search_qdrant_query;
use crate::{
    data::models::{get_range, FieldCondition, MatchCondition, Pool},
    errors::ServiceError,
};
use actix_web::web;
use diesel::dsl::sql;
use diesel::sql_types::{Bool, Float, Text};
use diesel::{ExpressionMethods, JoinOnDsl, PgArrayExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;

use itertools::Itertools;
use simple_server_timing_header::Timer;
use utoipa::ToSchema;

use qdrant_client::qdrant::condition::ConditionOneOf::HasId;
use qdrant_client::qdrant::Filter;
use qdrant_client::qdrant::{Condition, HasIdCondition, PointId};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub score: f32,
    pub point_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchChunkQueryResult {
    pub search_results: Vec<SearchResult>,
    pub total_chunk_pages: i64,
    pub batch_lengths: Vec<usize>,
}

async fn convert_group_tracking_ids_to_group_ids(
    condition: FieldCondition,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<FieldCondition, ServiceError> {
    if condition.field == "group_tracking_ids" {
        if let Some(match_any) = condition.r#match_any {
            let matches = match_any
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<String>>();

            let correct_matches: Vec<MatchCondition> =
                get_group_ids_from_tracking_ids_query(matches, dataset_id, pool.clone())
                    .await?
                    .iter()
                    .map(|(id, _)| MatchCondition::Text(id.to_string()))
                    .collect();

            Ok(FieldCondition {
                field: "group_ids".to_string(),
                match_any: Some(correct_matches),
                match_all: None,
                date_range: None,
                range: None,
                geo_bounding_box: None,
                geo_polygon: None,
                geo_radius: None,
            })
        } else if let Some(match_all) = condition.match_all {
            let matches = match_all
                .iter()
                .map(|item| item.to_string())
                .collect::<Vec<String>>();

            let correct_matches: Vec<MatchCondition> =
                get_group_ids_from_tracking_ids_query(matches, dataset_id, pool.clone())
                    .await?
                    .iter()
                    .map(|(id, _)| MatchCondition::Text(id.to_string()))
                    .collect();

            Ok(FieldCondition {
                field: "group_ids".to_string(),
                match_any: None,
                match_all: Some(correct_matches),
                date_range: None,
                range: None,
                geo_bounding_box: None,
                geo_polygon: None,
                geo_radius: None,
            })
        } else {
            Err(ServiceError::BadRequest(
                "match_any key not found for group_tracking_ids".to_string(),
            ))?
        }
    } else {
        Ok(condition)
    }
}

pub async fn get_qdrant_ids_from_condition(
    cond: HasIDCondition,
    pool: web::Data<Pool>,
) -> Result<Vec<String>, ServiceError> {
    if let Some(ids) = cond.ids {
        Ok(get_qdrant_ids_from_chunk_ids_query(
            ids.into_iter().map(UnifiedId::TrieveUuid).collect(),
            pool.clone(),
        )
        .await?
        .into_iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>())
    } else if let Some(tracking_ids) = cond.tracking_ids {
        Ok(get_qdrant_ids_from_chunk_ids_query(
            tracking_ids
                .into_iter()
                .map(UnifiedId::TrackingId)
                .collect(),
            pool.clone(),
        )
        .await?
        .into_iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>())
    } else {
        Err(ServiceError::BadRequest(
            "ids or tracking_ids must be provided".to_string(),
        ))?
    }
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool))]
pub async fn assemble_qdrant_filter(
    filters: Option<ChunkFilter>,
    quote_words: Option<Vec<String>>,
    negated_words: Option<Vec<String>>,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Filter, ServiceError> {
    let mut filter = Filter::default();

    filter
        .must
        .push(Condition::matches("dataset_id", dataset_id.to_string()));

    if let Some(filters) = filters {
        if let Some(should_filters) = filters.should {
            for should_condition in should_filters {
                match should_condition {
                    ConditionType::Field(cond) => {
                        let should_condition =
                            convert_group_tracking_ids_to_group_ids(cond, dataset_id, pool.clone())
                                .await?;

                        let qdrant_condition = should_condition
                            .convert_to_qdrant_condition(
                                filters.jsonb_prefilter,
                                dataset_id,
                                pool.clone(),
                            )
                            .await?;

                        if let Some(condition) = qdrant_condition {
                            filter.should.push(condition);
                        }
                    }
                    ConditionType::HasID(cond) => {
                        filter.should.push(Condition::has_id(
                            get_qdrant_ids_from_condition(cond, pool.clone()).await?,
                        ));
                    }
                }
            }
        }

        if let Some(must_filters) = filters.must {
            for must_condition in must_filters {
                match must_condition {
                    ConditionType::Field(cond) => {
                        let must_condition =
                            convert_group_tracking_ids_to_group_ids(cond, dataset_id, pool.clone())
                                .await?;

                        let qdrant_condition = must_condition
                            .convert_to_qdrant_condition(
                                filters.jsonb_prefilter,
                                dataset_id,
                                pool.clone(),
                            )
                            .await?;

                        if let Some(condition) = qdrant_condition {
                            filter.must.push(condition);
                        }
                    }
                    ConditionType::HasID(cond) => {
                        filter.must.push(Condition::has_id(
                            get_qdrant_ids_from_condition(cond, pool.clone()).await?,
                        ));
                    }
                }
            }
        }

        if let Some(must_not_filters) = filters.must_not {
            for must_not_condition in must_not_filters {
                match must_not_condition {
                    ConditionType::Field(cond) => {
                        let must_not_condition =
                            convert_group_tracking_ids_to_group_ids(cond, dataset_id, pool.clone())
                                .await?;

                        let qdrant_condition = must_not_condition
                            .convert_to_qdrant_condition(
                                filters.jsonb_prefilter,
                                dataset_id,
                                pool.clone(),
                            )
                            .await?;

                        if let Some(condition) = qdrant_condition {
                            filter.must_not.push(condition);
                        }
                    }
                    ConditionType::HasID(cond) => {
                        filter.must_not.push(Condition::has_id(
                            get_qdrant_ids_from_condition(cond, pool.clone()).await?,
                        ));
                    }
                }
            }
        }
    };

    if let Some(quote_words) = quote_words {
        for quote_word in quote_words {
            filter
                .must
                .push(Condition::matches_text("content", quote_word));
        }
    }

    if let Some(negated_words) = negated_words {
        for negated_word in negated_words {
            filter
                .must_not
                .push(Condition::matches_text("content", negated_word));
        }
    }

    Ok(filter)
}

#[derive(Debug)]
pub struct RetrievePointQuery {
    vector: VectorType,
    score_threshold: Option<f32>,
    limit: u64,
    sort_by: Option<SortByField>,
    rerank_by: Option<SortBySearchType>,
    filter: Option<ChunkFilter>,
}

impl RetrievePointQuery {
    pub async fn into_qdrant_query(
        self,
        parsed_query: ParsedQueryTypes,
        dataset_id: uuid::Uuid,
        group_id: Option<uuid::Uuid>,
        config: &DatasetConfiguration,
        pool: web::Data<Pool>,
    ) -> Result<QdrantSearchQuery, ServiceError> {
        let parsed_query = match parsed_query {
            ParsedQueryTypes::Single(parsed_query) => Some(parsed_query),
            ParsedQueryTypes::Multi(_) => None,
        };

        let mut filter = assemble_qdrant_filter(
            self.filter,
            parsed_query
                .as_ref()
                .map(|query| query.quote_words.clone())
                .clone()
                .flatten(),
            parsed_query
                .as_ref()
                .map(|query| query.negated_words.clone())
                .clone()
                .flatten(),
            dataset_id,
            pool,
        )
        .await?;

        if let Some(group_id) = group_id {
            filter
                .must
                .push(Condition::matches("group_ids", group_id.to_string()));
        }

        let rerank_query = if let Some(parsed_query) = parsed_query {
            if let Some(rerank_by) = self.rerank_by {
                match rerank_by.rerank_type {
                    ReRankOptions::Fulltext => {
                        let data = SearchChunksReqPayload {
                            query: QueryTypes::Single(
                                rerank_by
                                    .rerank_query
                                    .clone()
                                    .unwrap_or(parsed_query.query.clone()),
                            ),
                            search_type: SearchMethod::FullText,
                            ..Default::default()
                        };

                        let vector = get_qdrant_vector(
                            data.search_type,
                            ParsedQueryTypes::Single(parsed_query),
                            None,
                            config,
                        )
                        .await?;
                        Some(QdrantSearchQuery {
                            vector,
                            score_threshold: self.score_threshold,
                            limit: rerank_by.prefetch_amount.unwrap_or(1000),
                            rerank_by: Box::new(None),
                            sort_by: None,
                            filter: filter.clone(),
                        })
                    }
                    ReRankOptions::Semantic => {
                        let data = SearchChunksReqPayload {
                            query: QueryTypes::Single(
                                rerank_by
                                    .rerank_query
                                    .clone()
                                    .unwrap_or(parsed_query.query.clone()),
                            ),
                            search_type: SearchMethod::Semantic,
                            ..Default::default()
                        };

                        let vector = get_qdrant_vector(
                            data.search_type,
                            ParsedQueryTypes::Single(parsed_query),
                            None,
                            config,
                        )
                        .await?;
                        Some(QdrantSearchQuery {
                            vector,
                            score_threshold: self.score_threshold,
                            limit: rerank_by.prefetch_amount.unwrap_or(1000),
                            rerank_by: Box::new(None),
                            sort_by: None,
                            filter: filter.clone(),
                        })
                    }
                    ReRankOptions::BM25 => {
                        let data = SearchChunksReqPayload {
                            query: QueryTypes::Single(
                                rerank_by
                                    .rerank_query
                                    .clone()
                                    .unwrap_or(parsed_query.query.clone()),
                            ),
                            search_type: SearchMethod::BM25,
                            ..Default::default()
                        };

                        let vector = get_qdrant_vector(
                            data.search_type,
                            ParsedQueryTypes::Single(parsed_query),
                            None,
                            config,
                        )
                        .await?;

                        Some(QdrantSearchQuery {
                            vector,
                            score_threshold: self.score_threshold,
                            limit: rerank_by.prefetch_amount.unwrap_or(1000),
                            rerank_by: Box::new(None),
                            sort_by: None,
                            filter: filter.clone(),
                        })
                    }
                    ReRankOptions::CrossEncoder => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(QdrantSearchQuery {
            vector: self.vector,
            score_threshold: self.score_threshold,
            limit: self.limit,
            rerank_by: Box::new(rerank_query),
            sort_by: self.sort_by,
            filter: filter.clone(),
        })
    }
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument]
pub async fn retrieve_qdrant_points_query(
    qdrant_searches: Vec<QdrantSearchQuery>,
    page: u64,
    get_total_pages: bool,
    config: &DatasetConfiguration,
) -> Result<SearchChunkQueryResult, ServiceError> {
    let parent_span = sentry::configure_scope(|scope| scope.get_span());
    let transaction: sentry::TransactionOrSpan = match &parent_span {
        Some(parent) => parent
            .start_child("Qdrant Points Query", "retrieve_qdrant_points_query")
            .into(),
        None => {
            let ctx = sentry::TransactionContext::new(
                "Qdrant Points Query",
                "retrieve_qdrant_points_query",
            );
            sentry::start_transaction(ctx).into()
        }
    };
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone())));

    let page = if page == 0 { 1 } else { page };

    let (point_ids, count, batch_lengths) = search_qdrant_query(
        page,
        qdrant_searches.clone(),
        config.clone(),
        get_total_pages,
    )
    .await?;

    let limit = qdrant_searches
        .iter()
        .map(|query| query.limit)
        .min()
        .unwrap_or(10);

    let pages = (count as f64 / limit as f64).ceil() as i64;

    Ok(SearchChunkQueryResult {
        search_results: point_ids,
        total_chunk_pages: pages,
        batch_lengths,
    })
}

#[tracing::instrument(skip(pool))]
pub async fn get_metadata_filter_condition(
    filter: &FieldCondition,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Filter, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let key = filter
        .field
        .strip_prefix("metadata.")
        .unwrap_or(&filter.field)
        .to_string();

    let mut conn = pool.get().await.unwrap();

    let mut query = chunk_metadata_columns::chunk_metadata
        .select(chunk_metadata_columns::qdrant_point_id)
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
        .into_boxed();

    if let Some(matches) = &filter.match_any {
        if let Some(first_val) = matches.get(0) {
            match first_val {
                MatchCondition::Text(string_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, string_val
                    )));
                }
                MatchCondition::Integer(id_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
                MatchCondition::Float(id_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
            }
        }

        for match_condition in matches.iter().skip(1) {
            match match_condition {
                MatchCondition::Text(string_val) => {
                    query = query.or_filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, string_val
                    )));
                }
                MatchCondition::Integer(id_val) => {
                    query = query.or_filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
                MatchCondition::Float(id_val) => {
                    query = query.or_filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
            }
        }
    } else if let Some(matches) = &filter.match_all {
        if let Some(first_val) = matches.get(0) {
            match first_val {
                MatchCondition::Text(string_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, string_val
                    )));
                }
                MatchCondition::Integer(id_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
                MatchCondition::Float(id_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
            }
        }

        for match_condition in matches.iter().skip(1) {
            match match_condition {
                MatchCondition::Text(string_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, string_val
                    )));
                }
                MatchCondition::Integer(id_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
                MatchCondition::Float(id_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
            }
        }
    }

    if let Some(range) = &filter.range {
        let range_filter = get_range(range.clone())?;
        if let Some(gt) = range_filter.gt {
            query = query.filter(
                sql::<Bool>(&format!("(chunk_metadata.metadata->>'{}')::float > ", key))
                    .bind::<Float, _>(gt as f32),
            );
        };

        if let Some(gte) = range_filter.gte {
            query = query.filter(
                sql::<Bool>(&format!("(chunk_metadata.metadata->>'{}')::float >= ", key))
                    .bind::<Float, _>(gte as f32),
            );
        };

        if let Some(lt) = range_filter.lt {
            query = query.filter(
                sql::<Bool>(&format!("(chunk_metadata.metadata->>'{}')::float < ", key))
                    .bind::<Float, _>(lt as f32),
            );
        };

        if let Some(lte) = range_filter.lte {
            query = query.filter(
                sql::<Bool>(&format!("(chunk_metadata.metadata->>'{}')::float <= ", key))
                    .bind::<Float, _>(lte as f32),
            );
        };
    }

    if let Some(date_range) = &filter.date_range {
        if let Some(gt) = &date_range.gt {
            query = query.filter(
                sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
                    .gt(convert_to_date_time(Some(gt.clone()))?.unwrap().to_string()),
            );
        };

        if let Some(gte) = &date_range.gte {
            query = query.filter(
                sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key)).ge(
                    convert_to_date_time(Some(gte.clone()))?
                        .unwrap()
                        .to_string(),
                ),
            );
        };

        if let Some(lt) = &date_range.lt {
            query = query.filter(
                sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
                    .lt(convert_to_date_time(Some(lt.clone()))?.unwrap().to_string()),
            );
        };

        if let Some(lte) = &date_range.lte {
            query = query.filter(
                sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key)).le(
                    convert_to_date_time(Some(lte.clone()))?
                        .unwrap()
                        .to_string(),
                ),
            );
        };
    }

    let qdrant_point_ids: Vec<uuid::Uuid> = query
        .load::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

    let matching_point_ids: Vec<PointId> = qdrant_point_ids
        .iter()
        .map(|uuid| uuid.to_string())
        .collect::<HashSet<String>>()
        .iter()
        .map(|uuid| (*uuid).clone().into())
        .collect::<Vec<PointId>>();

    let mut metadata_filter = Filter::default();
    metadata_filter.must.push(Condition {
        condition_one_of: Some(HasId(HasIdCondition {
            has_id: matching_point_ids,
        })),
    });

    Ok(metadata_filter)
}

#[tracing::instrument(skip(pool))]
pub async fn get_group_metadata_filter_condition(
    filter: &FieldCondition,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Filter, ServiceError> {
    let mut metadata_filter = Filter::default();

    let key = filter
        .field
        .strip_prefix("group_metadata.")
        .unwrap_or(&filter.field)
        .to_string();

    use crate::data::schema::chunk_group::dsl as chunk_group_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    let mut query =
        chunk_metadata_columns::chunk_metadata
            .left_outer_join(chunk_group_bookmarks_columns::chunk_group_bookmarks.on(
                chunk_metadata_columns::id.eq(chunk_group_bookmarks_columns::chunk_metadata_id),
            ))
            .left_outer_join(
                chunk_group_columns::chunk_group
                    .on(chunk_group_bookmarks_columns::group_id.eq(chunk_group_columns::id)),
            )
            .select(chunk_metadata_columns::qdrant_point_id)
            .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
            .into_boxed();

    if let Some(matches) = &filter.match_any {
        if let Some(first_val) = matches.get(0) {
            match first_val {
                MatchCondition::Text(string_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, string_val
                    )));
                }
                MatchCondition::Integer(id_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
                MatchCondition::Float(id_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
            }
        }

        for match_condition in matches.iter().skip(1) {
            match match_condition {
                MatchCondition::Text(string_val) => {
                    query = query.or_filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, string_val
                    )));
                }
                MatchCondition::Integer(id_val) => {
                    query = query.or_filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
                MatchCondition::Float(id_val) => {
                    query = query.or_filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
            }
        }
    } else if let Some(matches) = &filter.match_all {
        if let Some(first_val) = matches.get(0) {
            match first_val {
                MatchCondition::Text(string_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, string_val
                    )));
                }
                MatchCondition::Integer(id_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
                MatchCondition::Float(id_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
            }
        }

        for match_condition in matches.iter().skip(1) {
            match match_condition {
                MatchCondition::Text(string_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, string_val
                    )));
                }
                MatchCondition::Integer(id_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
                MatchCondition::Float(id_val) => {
                    query = query.filter(sql::<Bool>(&format!(
                        "chunk_metadata.metadata @> '{{\"{}\":\"{}\"}}'",
                        key, id_val
                    )));
                }
            }
        }
    };

    if let Some(range) = &filter.range {
        let range_filter = get_range(range.clone())?;
        if let Some(gt) = range_filter.gt {
            query = query.filter(
                sql::<Text>(&format!("chunk_group.metadata->>'{}'", key)).gt(gt.to_string()),
            );
        };

        if let Some(gte) = range_filter.gte {
            query = query.filter(
                sql::<Text>(&format!("chunk_group.metadata->>'{}'", key)).ge(gte.to_string()),
            );
        };

        if let Some(lt) = range_filter.lt {
            query = query.filter(
                sql::<Text>(&format!("chunk_group.metadata->>'{}'", key)).lt(lt.to_string()),
            );
        };

        if let Some(lte) = range_filter.lte {
            query = query.filter(
                sql::<Text>(&format!("chunk_group.metadata->>'{}'", key)).le(lte.to_string()),
            );
        };
    }

    let qdrant_point_ids: Vec<uuid::Uuid> = query
        .load::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

    let matching_point_ids: Vec<PointId> = qdrant_point_ids
        .iter()
        .map(|uuid| uuid.to_string())
        .collect::<HashSet<String>>()
        .iter()
        .map(|uuid| (*uuid).clone().into())
        .collect::<Vec<PointId>>();

    metadata_filter.must.push(Condition {
        condition_one_of: Some(HasId(HasIdCondition {
            has_id: matching_point_ids,
        })),
    });

    Ok(metadata_filter)
}

#[tracing::instrument(skip(pool))]
pub async fn get_group_tag_set_filter_condition(
    filter: &FieldCondition,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Filter, ServiceError> {
    let mut metadata_filter = Filter::default();

    use crate::data::schema::chunk_group::dsl as chunk_group_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    let mut query =
        chunk_metadata_columns::chunk_metadata
            .left_outer_join(chunk_group_bookmarks_columns::chunk_group_bookmarks.on(
                chunk_metadata_columns::id.eq(chunk_group_bookmarks_columns::chunk_metadata_id),
            ))
            .left_outer_join(
                chunk_group_columns::chunk_group
                    .on(chunk_group_bookmarks_columns::group_id.eq(chunk_group_columns::id)),
            )
            .select(chunk_metadata_columns::qdrant_point_id)
            .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
            .into_boxed();

    if let Some(matches) = &filter.match_any {
        if let Some(first_val) = matches.get(0) {
            match first_val {
                MatchCondition::Text(string_val) => {
                    query = query
                        .filter(chunk_group_columns::tag_set.contains(vec![string_val.clone()]));
                }
                MatchCondition::Integer(int_val) => {
                    query = query
                        .filter(chunk_group_columns::tag_set.contains(vec![int_val.to_string()]));
                }
                MatchCondition::Float(float_val) => {
                    query = query
                        .filter(chunk_group_columns::tag_set.contains(vec![float_val.to_string()]));
                }
            }
        }

        for match_condition in matches.iter().skip(1) {
            match match_condition {
                MatchCondition::Text(string_val) => {
                    query = query
                        .or_filter(chunk_group_columns::tag_set.contains(vec![string_val.clone()]));
                }
                MatchCondition::Integer(int_val) => {
                    query = query.or_filter(
                        chunk_group_columns::tag_set.contains(vec![int_val.to_string()]),
                    );
                }
                MatchCondition::Float(float_val) => {
                    query = query.or_filter(
                        chunk_group_columns::tag_set.contains(vec![float_val.to_string()]),
                    );
                }
            }
        }
    } else if let Some(matches) = &filter.match_all {
        if let Some(first_val) = matches.get(0) {
            match first_val {
                MatchCondition::Text(string_val) => {
                    query = query
                        .filter(chunk_group_columns::tag_set.contains(vec![string_val.clone()]));
                }
                MatchCondition::Integer(int_val) => {
                    query = query
                        .filter(chunk_group_columns::tag_set.contains(vec![int_val.to_string()]));
                }
                MatchCondition::Float(float_val) => {
                    query = query
                        .filter(chunk_group_columns::tag_set.contains(vec![float_val.to_string()]));
                }
            }
        }

        for match_condition in matches.iter().skip(1) {
            match match_condition {
                MatchCondition::Text(string_val) => {
                    query = query
                        .filter(chunk_group_columns::tag_set.contains(vec![string_val.clone()]));
                }
                MatchCondition::Integer(int_val) => {
                    query = query
                        .filter(chunk_group_columns::tag_set.contains(vec![int_val.to_string()]));
                }
                MatchCondition::Float(float_val) => {
                    query = query
                        .filter(chunk_group_columns::tag_set.contains(vec![float_val.to_string()]));
                }
            }
        }
    }

    if filter.range.is_some() {
        "Range filter not supported for group_tag_set".to_string();
    }

    let qdrant_point_ids: Vec<uuid::Uuid> = query
        .load::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

    let matching_point_ids: Vec<PointId> = qdrant_point_ids
        .iter()
        .map(|uuid| uuid.to_string())
        .collect::<HashSet<String>>()
        .iter()
        .map(|uuid| (*uuid).clone().into())
        .collect::<Vec<PointId>>();

    metadata_filter.must.push(Condition {
        condition_one_of: Some(HasId(HasIdCondition {
            has_id: matching_point_ids,
        })),
    });

    Ok(metadata_filter)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchOverGroupsQueryResult {
    pub search_results: Vec<GroupSearchResults>,
    pub corrected_query: Option<String>,
    pub total_chunk_pages: i64,
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool))]
pub async fn retrieve_group_qdrant_points_query(
    vector: VectorType,
    page: u64,
    get_total_pages: bool,
    filters: Option<ChunkFilter>,
    limit: u64,
    score_threshold: Option<f32>,
    group_size: u32,
    parsed_query: ParsedQueryTypes,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
    config: &DatasetConfiguration,
) -> Result<SearchOverGroupsQueryResult, ServiceError> {
    let page = if page == 0 { 1 } else { page };
    let parsed_query = match parsed_query {
        ParsedQueryTypes::Single(parsed_query) => Some(parsed_query),
        ParsedQueryTypes::Multi(_) => None,
    };

    let filter = assemble_qdrant_filter(
        filters.clone(),
        parsed_query
            .as_ref()
            .map(|query| query.quote_words.clone())
            .clone()
            .flatten(),
        parsed_query
            .as_ref()
            .map(|query| query.negated_words.clone())
            .clone()
            .flatten(),
        dataset_id,
        pool,
    )
    .await?;

    let (point_ids, count) = search_over_groups_query(
        page,
        filter.clone(),
        limit,
        score_threshold,
        group_size,
        vector.clone(),
        config.clone(),
        get_total_pages,
    )
    .await?;

    let pages = (count as f64 / limit as f64).ceil() as i64;

    Ok(SearchOverGroupsQueryResult {
        search_results: point_ids,
        corrected_query: None,
        total_chunk_pages: pages,
    })
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct FullTextDocIds {
    pub doc_ids: Option<uuid::Uuid>,
    pub total_count: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[schema(title = "V1")]
pub struct GroupScoreChunk {
    pub group_id: uuid::Uuid,
    pub group_name: Option<String>,
    pub group_description: Option<String>,
    pub group_created_at: chrono::NaiveDateTime,
    pub group_updated_at: chrono::NaiveDateTime,
    pub group_tracking_id: Option<String>,
    pub group_metadata: Option<serde_json::Value>,
    pub group_tag_set: Option<Vec<Option<String>>>,
    pub group_dataset_id: uuid::Uuid,
    pub metadata: Vec<ScoreChunkDTO>,
    pub file_id: Option<uuid::Uuid>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[schema(title = "V2")]
pub struct SearchOverGroupsResults {
    pub group: ChunkGroup,
    pub chunks: Vec<ScoreChunk>,
    pub file_id: Option<uuid::Uuid>,
}

impl From<GroupScoreChunk> for SearchOverGroupsResults {
    fn from(val: GroupScoreChunk) -> Self {
        SearchOverGroupsResults {
            group: ChunkGroup {
                id: val.group_id,
                name: val.group_name.unwrap_or("".to_string()),
                description: val.group_description.unwrap_or("".to_string()),
                created_at: val.group_created_at,
                updated_at: val.group_updated_at,
                dataset_id: val.group_dataset_id,
                tracking_id: val.group_tracking_id,
                metadata: val.group_metadata,
                tag_set: val.group_tag_set,
            },
            chunks: val
                .metadata
                .into_iter()
                .map(|score_chunk| score_chunk.into())
                .collect(),
            file_id: val.file_id,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(title = "V1")]
pub struct DeprecatedSearchOverGroupsResponseBody {
    pub group_chunks: Vec<GroupScoreChunk>,
    pub corrected_query: Option<String>,
    pub total_chunk_pages: i64,
}

impl DeprecatedSearchOverGroupsResponseBody {
    pub fn into_v2(self, search_id: uuid::Uuid) -> SearchOverGroupsResponseBody {
        SearchOverGroupsResponseBody {
            id: search_id,
            results: self
                .group_chunks
                .into_iter()
                .map(|chunk| chunk.into())
                .collect(),
            corrected_query: self.corrected_query,
            total_pages: self.total_chunk_pages,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(title = "V2")]
pub struct SearchOverGroupsResponseBody {
    pub id: uuid::Uuid,
    pub results: Vec<SearchOverGroupsResults>,
    pub corrected_query: Option<String>,
    pub total_pages: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum SearchOverGroupsResponseTypes {
    #[schema(title = "V2")]
    V2(SearchOverGroupsResponseBody),
    #[schema(title = "V1")]
    V1(DeprecatedSearchOverGroupsResponseBody),
}

#[tracing::instrument(skip(pool))]
pub async fn retrieve_chunks_for_groups(
    search_over_groups_query_result: SearchOverGroupsQueryResult,
    data: &SearchOverGroupsReqPayload,
    pool: web::Data<Pool>,
) -> Result<DeprecatedSearchOverGroupsResponseBody, ServiceError> {
    let point_ids = search_over_groups_query_result
        .search_results
        .clone()
        .iter()
        .flat_map(|hit| hit.hits.iter().map(|point| point.point_id).collect_vec())
        .collect_vec();

    let metadata_chunks = match data.slim_chunks.unwrap_or(false)
        && data.search_type != SearchMethod::Hybrid
    {
        true => get_slim_chunks_from_point_ids_query(point_ids, pool.clone()).await?,
        _ => {
            get_chunk_metadatas_and_collided_chunks_from_point_ids_query(point_ids, pool.clone())
                .await?
        }
    };

    let groups = get_groups_from_group_ids_query(
        search_over_groups_query_result
            .search_results
            .iter()
            .map(|group| group.group_id)
            .collect(),
        pool.clone(),
    )
    .await?;

    let group_chunks: Vec<GroupScoreChunk> = search_over_groups_query_result
        .search_results
        .iter()
        .map(|group_search_result| {
            let score_chunks: Vec<ScoreChunkDTO> = group_search_result
                .hits
                .iter()
                .filter_map(|search_result| {
                    let mut chunk: ChunkMetadataTypes =
                        match metadata_chunks.iter().find(|metadata_chunk| {
                            metadata_chunk.metadata().qdrant_point_id == search_result.point_id
                        }) {
                            Some(metadata_chunk) => metadata_chunk.clone(),
                            None => {
                                log::error!(
                                    "Failed to find chunk for qdrant_point_id for retrieve_chunks_for_groups: {:?}",
                                    search_result.point_id
                                );
                                sentry::capture_message(
                                    &format!("Failed to find chunk for qdrant_point_id for retrieve_chunks_for_groups: {:?}", search_result.point_id),
                                    sentry::Level::Error,
                                );

                                return None;
                            },
                        };

                    let mut highlights: Option<Vec<String>> = None;
                    if let Some(highlight_options)  = &data.highlight_options {
                        if highlight_options.highlight_results.unwrap_or(true) && !data.slim_chunks.unwrap_or(false) && !matches!(data.query, QueryTypes::Multi(_)) {
                            let (highlighted_chunk, highlighted_snippets) = match highlight_options.highlight_strategy {
                                Some(HighlightStrategy::V1) => {
                                    get_highlights(
                                            chunk.clone().into(),
                                            data.query.clone().to_single_query().expect("Should never be multi query"),
                                            highlight_options.highlight_threshold,
                                            highlight_options.highlight_delimiters.clone().unwrap_or(vec![
                                                ".".to_string(),
                                                "!".to_string(),
                                                "?".to_string(),
                                                "\n".to_string(),
                                                "\t".to_string(),
                                                ",".to_string(),
                                            ]),
                                            highlight_options.highlight_max_length,
                                            highlight_options.highlight_max_num,
                                            highlight_options.highlight_window
                                        )
                                        .unwrap_or((chunk.clone().into(), vec![]))
                                },
                                _ => {
                                    get_highlights_with_exact_match(
                                            chunk.clone().into(),
                                            data.query.clone().to_single_query().expect("Should never be multi query"),
                                            highlight_options.highlight_threshold,
                                            highlight_options.highlight_delimiters.clone().unwrap_or(vec![
                                                ".".to_string(),
                                                "!".to_string(),
                                                "?".to_string(),
                                                "\n".to_string(),
                                                "\t".to_string(),
                                                ",".to_string(),
                                            ]),
                                            highlight_options.highlight_max_length,
                                            highlight_options.highlight_max_num,
                                            highlight_options.highlight_window,
                                        )
                                        .unwrap_or((chunk.clone().into(), vec![]))
                                },
                            };

                            highlights = Some(highlighted_snippets);

                            match chunk {
                                ChunkMetadataTypes::Metadata(_) => chunk = highlighted_chunk.into(),
                                ChunkMetadataTypes::Content(_) => {
                                    chunk =
                                        <ChunkMetadata as Into<ContentChunkMetadata>>::into(highlighted_chunk)
                                            .into()
                                }
                                _ => unreachable!(
                                    "If slim_chunks is false, then chunk must be either Metadata or Content"
                                ),
                            }
                        }
                }


                    Some(ScoreChunkDTO {
                        metadata: vec![chunk],
                        highlights,
                        score: search_result.score.into(),
                    })
                })
                .sorted_by(|a, b| b.score.partial_cmp(&a.score).unwrap())
                .collect_vec();

            let group_data = groups.iter().find(|group| group.id == group_search_result.group_id);

            GroupScoreChunk {
                group_id: group_search_result.group_id,
                group_name: group_data.map(|group| group.name.clone()),
                group_description: group_data.map(|group| group.description.clone()),
                group_created_at: group_data.map(|group| group.created_at).unwrap_or_default(),
                group_updated_at: group_data.map(|group| group.updated_at).unwrap_or_default(),
                group_tracking_id: group_data.and_then(|group| group.tracking_id.clone()),
                group_metadata: group_data.and_then(|group| group.metadata.clone()),
                group_tag_set: group_data.and_then(|group| group.tag_set.clone()),
                group_dataset_id: group_data.map(|group| group.dataset_id).unwrap_or_default(),
                metadata: score_chunks,
                file_id: group_data.and_then(|group| group.file_id),
            }
        })
        .collect_vec();

    Ok(DeprecatedSearchOverGroupsResponseBody {
        group_chunks,
        corrected_query: None,
        total_chunk_pages: search_over_groups_query_result.total_chunk_pages,
    })
}

#[tracing::instrument(skip(pool))]
pub async fn get_metadata_from_groups(
    search_over_groups_query_result: SearchOverGroupsQueryResult,
    slim_chunks: Option<bool>,
    pool: web::Data<Pool>,
) -> Result<Vec<GroupScoreChunk>, actix_web::Error> {
    let point_ids = search_over_groups_query_result
        .search_results
        .iter()
        .flat_map(|hit| hit.hits.iter().map(|point| point.point_id).collect_vec())
        .collect_vec();

    let chunk_metadatas = match slim_chunks {
        Some(true) => get_slim_chunks_from_point_ids_query(point_ids, pool.clone()).await?,
        _ => {
            get_chunk_metadatas_and_collided_chunks_from_point_ids_query(point_ids, pool.clone())
                .await?
        }
    };

    let groups = get_groups_from_group_ids_query(
        search_over_groups_query_result
            .search_results
            .iter()
            .map(|group| group.group_id)
            .collect(),
        pool.clone(),
    )
    .await?;

    let group_chunks: Vec<GroupScoreChunk> = search_over_groups_query_result
        .search_results
        .iter()
        .map(|group_search_result| {
            let score_chunk: Vec<ScoreChunkDTO> = group_search_result
                .hits
                .iter()
                .filter_map(|search_result| {
                    let chunk: ChunkMetadataTypes =
                        match chunk_metadatas.iter().find(|metadata_chunk| {
                            metadata_chunk.metadata().qdrant_point_id == search_result.point_id
                        }) {
                            Some(metadata_chunk) => metadata_chunk.clone(),
                            None => {
                                log::error!(
                                    "Failed to find chunk for qdrant_point_id for get_metadata_from_groups: {:?}",
                                    search_result.point_id
                                );
                                sentry::capture_message(
                                    &format!("Failed to find chunk for qdrant_point_id for get_metadata_from_groups: {:?}", search_result.point_id),
                                    sentry::Level::Error,
                                );

                                return None;
                            },
                        };

                    Some(ScoreChunkDTO {
                        metadata: vec![chunk],
                        highlights: None,
                        score: search_result.score.into(),
                    })
                })
                .collect_vec();

            let group_data = groups.iter().find(|group| group.id == group_search_result.group_id);

            GroupScoreChunk {
                group_id: group_search_result.group_id,
                group_name: group_data.map(|group| group.name.clone()),
                group_description: group_data.map(|group| group.description.clone()),
                group_created_at: group_data.map(|group| group.created_at).unwrap_or_default(),
                group_updated_at: group_data.map(|group| group.updated_at).unwrap_or_default(),
                group_tracking_id: group_data.and_then(|group| group.tracking_id.clone()),
                group_metadata: group_data.and_then(|group| group.metadata.clone()),
                group_tag_set: group_data.and_then(|group| group.tag_set.clone()),
                group_dataset_id: group_data.map(|group| group.dataset_id).unwrap_or_default(),
                metadata: score_chunk,
                file_id: group_data.and_then(|group| group.file_id),
            }
        })
        .collect_vec();

    Ok(group_chunks)
}

#[tracing::instrument(skip(pool, timer))]
#[inline(never)]
/// Retrieve chunks from point ids, DOES NOT GUARD AGAINST DATASET ACCESS PERMISSIONS
pub async fn retrieve_chunks_from_point_ids(
    search_chunk_query_results: SearchChunkQueryResult,
    timer: Option<&mut Timer>,
    data: &SearchChunksReqPayload,
    pool: web::Data<Pool>,
) -> Result<SearchChunkQueryResponseBody, actix_web::Error> {
    let parent_span = sentry::configure_scope(|scope| scope.get_span());
    let transaction: sentry::TransactionOrSpan = match &parent_span {
        Some(parent) => parent
            .start_child(
                "Retrieve Chunks from point IDS",
                "Retrieve Chunks from point IDS",
            )
            .into(),
        None => {
            let ctx = sentry::TransactionContext::new(
                "Retrieve Chunks from point IDS",
                "Retrieve Chunks from point IDS",
            );
            sentry::start_transaction(ctx).into()
        }
    };
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone())));

    let point_ids = search_chunk_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();

    let metadata_chunks =
        if data.slim_chunks.unwrap_or(false) && data.search_type != SearchMethod::Hybrid {
            get_slim_chunks_from_point_ids_query(point_ids, pool.clone()).await?
        } else if data.content_only.unwrap_or(false) {
            get_content_chunk_from_point_ids_query(point_ids, pool.clone()).await?
        } else {
            get_chunk_metadatas_and_collided_chunks_from_point_ids_query(point_ids, pool.clone())
                .await?
        };

    let timer = if let Some(timer) = timer {
        timer.add("fetched from postgres");

        Some(timer)
    } else {
        None
    };

    let score_chunks: Vec<ScoreChunkDTO> = search_chunk_query_results
        .search_results
        .iter()
        .filter_map(|search_result| {
            let mut chunk: ChunkMetadataTypes =
                match metadata_chunks.iter().find(|metadata_chunk| {
                    metadata_chunk.metadata().qdrant_point_id == search_result.point_id
                }) {
                    Some(metadata_chunk) => metadata_chunk.clone(),
                    None => {
                        log::error!(
                            "Failed to find chunk from qdrant_point_id for retrieve_chunks_from_point_ids: {:?}",
                            search_result.point_id
                        );
                        sentry::capture_message(
                            &format!(
                                "Failed to find chunk from qdrant_point_id for retrieve_chunks_from_point_ids: {:?}",
                                search_result.point_id
                            ),
                            sentry::Level::Error,
                        );

                        return None;
                    }
                };

            let mut highlights: Option<Vec<String>> = None;
                if let Some(highlight_options)  = &data.highlight_options {
                        if highlight_options.highlight_results.unwrap_or(true) && !data.slim_chunks.unwrap_or(false) && !matches!(data.query, QueryTypes::Multi(_)) {
                            let (highlighted_chunk, highlighted_snippets) = match highlight_options.highlight_strategy {
                                Some(HighlightStrategy::V1) => {
                                    get_highlights(
                                            chunk.clone().into(),
                                            data.query.clone().to_single_query().expect("Should never be multi query"),
                                            highlight_options.highlight_threshold,
                                            highlight_options.highlight_delimiters.clone().unwrap_or(vec![
                                                ".".to_string(),
                                                "!".to_string(),
                                                "?".to_string(),
                                                "\n".to_string(),
                                                "\t".to_string(),
                                                ",".to_string(),
                                            ]),
                                            highlight_options.highlight_max_length,
                                            highlight_options.highlight_max_num,
                                            highlight_options.highlight_window
                                        )
                                        .unwrap_or((chunk.clone().into(), vec![]))
                                },
                                _ => {
                                    get_highlights_with_exact_match(
                                            chunk.clone().into(),
                                            data.query.clone().to_single_query().expect("Should never be multi query"),
                                            highlight_options.highlight_threshold,
                                            highlight_options.highlight_delimiters.clone().unwrap_or(vec![
                                                ".".to_string(),
                                                "!".to_string(),
                                                "?".to_string(),
                                                "\n".to_string(),
                                                "\t".to_string(),
                                                ",".to_string(),
                                            ]),
                                            highlight_options.highlight_max_length,
                                            highlight_options.highlight_max_num,
                                            highlight_options.highlight_window,
                                        )
                                        .unwrap_or((chunk.clone().into(), vec![]))
                                },
                            };

                            highlights = Some(highlighted_snippets);

                            match chunk {
                                ChunkMetadataTypes::Metadata(_) => chunk = highlighted_chunk.into(),
                                ChunkMetadataTypes::Content(_) => {
                                    chunk =
                                        <ChunkMetadata as Into<ContentChunkMetadata>>::into(highlighted_chunk)
                                            .into()
                                }
                                _ => unreachable!(
                                    "If slim_chunks is false, then chunk must be either Metadata or Content"
                                ),
                            }
                        }
                }

            Some(ScoreChunkDTO {
                metadata: vec![chunk],
                highlights,
                score: search_result.score.into(),
            })
        })
        .collect();

    if let Some(timer) = timer {
        timer.add("highlight chunks");
    }

    Ok(SearchChunkQueryResponseBody {
        score_chunks,
        corrected_query: None,
        total_chunk_pages: search_chunk_query_results.total_chunk_pages,
    })
}

#[tracing::instrument]
pub fn rerank_chunks(
    chunks: Vec<ScoreChunkDTO>,
    sort_by: Option<SortByField>,
    tag_weights: Option<HashMap<String, f32>>,
    use_weights: Option<bool>,
    query_location: Option<GeoInfoWithBias>,
) -> Vec<ScoreChunkDTO> {
    let mut reranked_chunks = Vec::new();
    if use_weights.unwrap_or(true) {
        chunks.into_iter().for_each(|mut chunk| {
            if chunk.metadata[0].metadata().weight == 0.0 {
                chunk.score *= 1.0;
            } else {
                chunk.score *= chunk.metadata[0].metadata().weight;
            }
            reranked_chunks.push(chunk);
        });
    } else {
        reranked_chunks = chunks;
    }

    if query_location.is_some() && query_location.unwrap().bias > 0.0 {
        let info_with_bias = query_location.unwrap();
        let query_location = info_with_bias.location;
        let location_bias = info_with_bias.bias;
        let distances = reranked_chunks
            .iter()
            .filter_map(|chunk| chunk.metadata[0].metadata().location)
            .map(|location| query_location.haversine_distance_to(&location));
        let max_distance = distances.clone().max_by(|a, b| a.partial_cmp(b).unwrap());
        let min_distance = distances.clone().min_by(|a, b| a.partial_cmp(b).unwrap());
        let max_score = reranked_chunks
            .iter()
            .map(|chunk| chunk.score)
            .max_by(|a, b| a.partial_cmp(b).unwrap());
        let min_score = reranked_chunks
            .iter()
            .map(|chunk| chunk.score)
            .min_by(|a, b| a.partial_cmp(b).unwrap());

        reranked_chunks = reranked_chunks
            .iter_mut()
            .map(|chunk| {
                let normalized_distance = (chunk.metadata[0]
                    .metadata()
                    .location
                    .map(|location| query_location.haversine_distance_to(&location))
                    .unwrap_or(0.0)
                    - min_distance.unwrap_or(0.0))
                    / (max_distance.unwrap_or(1.0) - min_distance.unwrap_or(0.0));
                let normalized_chunk_score = (chunk.score - min_score.unwrap_or(0.0))
                    / (max_score.unwrap_or(1.0) - min_score.unwrap_or(0.0));
                chunk.score = (normalized_chunk_score * (1.0 - location_bias))
                    + (location_bias * (1.0 - normalized_distance));
                chunk.clone()
            })
            .collect::<Vec<ScoreChunkDTO>>();
    }

    if let Some(tag_weights) = tag_weights {
        reranked_chunks = reranked_chunks
            .iter_mut()
            .map(|chunk| {
                let mut tag_score = 1.0;
                for (tag, weight) in tag_weights.iter() {
                    if let Some(metadata) = chunk.metadata.get(0) {
                        if let Some(metadata_tags) = metadata.metadata().tag_set {
                            if metadata_tags.contains(&Some(tag.clone())) {
                                tag_score *= weight;
                            }
                        }
                    }
                }
                chunk.score *= tag_score as f64;
                chunk.clone()
            })
            .collect::<Vec<ScoreChunkDTO>>();
    }
    reranked_chunks.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(sort_by) = sort_by {
        match sort_by.field.as_str() {
            "time_stamp" => {
                if sort_by.direction.is_some_and(|dir| dir == SortOrder::Asc) {
                    reranked_chunks.sort_by(|a, b| {
                        a.metadata[0]
                            .metadata()
                            .time_stamp
                            .partial_cmp(&b.metadata[0].metadata().time_stamp)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                } else {
                    reranked_chunks.sort_by(|a, b| {
                        b.metadata[0]
                            .metadata()
                            .time_stamp
                            .partial_cmp(&a.metadata[0].metadata().time_stamp)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
            }
            "num_value" => {
                if sort_by.direction.is_some_and(|dir| dir == SortOrder::Asc) {
                    reranked_chunks.sort_by(|a, b| {
                        a.metadata[0]
                            .metadata()
                            .num_value
                            .partial_cmp(&b.metadata[0].metadata().num_value)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                } else {
                    reranked_chunks.sort_by(|a, b| {
                        b.metadata[0]
                            .metadata()
                            .num_value
                            .partial_cmp(&a.metadata[0].metadata().num_value)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                }
            }
            _ => {
                log::error!("Unsupported sort_by field: {}", sort_by.field);
                sentry::capture_message(
                    &format!("Unsupported sort_by field: {}", sort_by.field),
                    sentry::Level::Error,
                );
            }
        }
    }

    reranked_chunks
}

async fn get_qdrant_vector(
    search_type: SearchMethod,
    parsed_query: ParsedQueryTypes,
    scoring_options: Option<ScoringOptions>,
    config: &DatasetConfiguration,
) -> Result<VectorType, ServiceError> {
    match search_type {
        SearchMethod::Semantic => {
            if !config.SEMANTIC_ENABLED {
                return Err(ServiceError::BadRequest(
                    "Semantic search is not enabled for this dataset".to_string(),
                ));
            }
            let semantic_boost = scoring_options
                .clone()
                .map(|options| options.semantic_boost)
                .unwrap_or(None);

            let embedding_vector = match parsed_query {
                ParsedQueryTypes::Single(query) => {
                    get_dense_vector(query.query.clone(), semantic_boost, "query", config.clone())
                        .await?
                }
                ParsedQueryTypes::Multi(queries) => {
                    let mut embedding_futures = Vec::new();
                    let embedding_vector = Vec::new();

                    for (query, _) in &queries {
                        embedding_futures.push(get_dense_vector(
                            query.query.clone(),
                            None,
                            "query",
                            config.clone(),
                        ));
                    }

                    let boost_factors = queries.into_iter().map(|(_, boost)| boost).collect_vec();

                    let embeddings: Vec<Vec<f32>> =
                        futures::future::try_join_all(embedding_futures).await?;

                    let embeddings = embeddings.into_iter().zip(boost_factors).collect_vec();

                    let embedding_vector = embeddings.into_iter().fold(
                        embedding_vector,
                        |mut final_vector, (vec, boost)| {
                            if final_vector.is_empty() {
                                final_vector = vec.into_iter().map(|v| v * boost).collect();
                            } else {
                                final_vector = final_vector
                                    .iter()
                                    .zip(vec)
                                    .map(|(vec_elem, boost_vec_elem)| {
                                        vec_elem + boost * boost_vec_elem
                                    })
                                    .collect();
                            }

                            final_vector
                        },
                    );

                    embedding_vector
                }
            };
            Ok(VectorType::Dense(embedding_vector))
        }
        SearchMethod::BM25 => {
            if std::env::var("BM25_ACTIVE").unwrap_or("false".to_string()) != "true" {
                return Err(ServiceError::BadRequest(
                    "BM25 search is not enabled for this dataset".to_string(),
                ));
            }
            let fulltext_boost = scoring_options
                .clone()
                .map(|options| options.fulltext_boost)
                .unwrap_or(None);

            let sparse_vectors = match parsed_query {
                ParsedQueryTypes::Single(query) => get_bm25_embeddings(
                    vec![(query.query.clone(), fulltext_boost)],
                    config.BM25_AVG_LEN,
                    config.BM25_B,
                    config.BM25_K,
                ),
                ParsedQueryTypes::Multi(_) => {
                    return Err(ServiceError::BadRequest(
                        "BM25 search does not support multi queries".to_string(),
                    ));
                }
            };
            let sparse_vector = sparse_vectors.get(0).expect("Vector will always exist");

            Ok(VectorType::BM25Sparse(sparse_vector.clone()))
        }
        SearchMethod::FullText => {
            if !config.FULLTEXT_ENABLED {
                return Err(ServiceError::BadRequest(
                    "Full text search is not enabled for this dataset".to_string(),
                ));
            }

            let fulltext_boost = scoring_options
                .clone()
                .map(|options| options.fulltext_boost)
                .unwrap_or(None);

            let sparse_vector = match parsed_query {
                ParsedQueryTypes::Single(query) => {
                    get_sparse_vector(query.query.clone(), fulltext_boost, "query").await?
                }
                ParsedQueryTypes::Multi(_) => {
                    return Err(ServiceError::BadRequest(
                        "Full text search does not support multi queries".to_string(),
                    ));
                }
            };

            Ok(VectorType::SpladeSparse(sparse_vector))
        }
        SearchMethod::Hybrid => Err(ServiceError::BadRequest(
            "Hybrid search is not supported for this endpoint".to_string(),
        )),
    }
}

#[tracing::instrument(skip(timer, pool, redis_pool))]
pub async fn search_chunks_query(
    mut data: SearchChunksReqPayload,
    parsed_query: ParsedQueryTypes,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset: Dataset,
    config: &DatasetConfiguration,
    timer: &mut Timer,
) -> Result<SearchChunkQueryResponseBody, actix_web::Error> {
    let parent_span = sentry::configure_scope(|scope| scope.get_span());
    let transaction: sentry::TransactionOrSpan = match &parent_span {
        Some(parent) => parent
            .start_child("semantic search", "Search Semantic Chunks")
            .into(),
        None => {
            let ctx = sentry::TransactionContext::new("semantic search", "Search Semantic Chunks");
            sentry::start_transaction(ctx).into()
        }
    };
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone())));

    let mut parsed_query = parsed_query.clone();
    let mut corrected_query = None;

    if let Some(options) = &data.typo_options {
        timer.add("start correcting query");
        match parsed_query {
            ParsedQueryTypes::Single(ref mut query) => {
                corrected_query =
                    correct_query(query.query.clone(), dataset.id, redis_pool, options).await?;
                query.query = corrected_query.clone().unwrap_or(query.query.clone());
                data.query = QueryTypes::Single(query.query.clone());
            }
            ParsedQueryTypes::Multi(ref mut queries) => {
                for (query, _) in queries {
                    corrected_query =
                        correct_query(query.query.clone(), dataset.id, redis_pool.clone(), options)
                            .await?;
                    query.query = corrected_query.clone().unwrap_or(query.query.clone());
                }
            }
        }
        timer.add("corrected query");
    }

    timer.add("start to create query vector");

    let vector = get_qdrant_vector(
        data.clone().search_type,
        parsed_query.clone(),
        data.clone().scoring_options,
        config,
    )
    .await?;

    timer.add("computed query vector");

    let (sort_by, rerank_by) = match data.sort_options.as_ref().map(|d| d.sort_by.clone()) {
        Some(Some(sort_by)) => match sort_by {
            QdrantSortBy::Field(field) => (Some(field.clone()), None),
            QdrantSortBy::SearchType(search_type) => (None, Some(search_type)),
        },
        _ => (None, None),
    };

    let qdrant_query = RetrievePointQuery {
        vector,
        score_threshold: if rerank_by.clone().map(|r| r.rerank_type)
            == Some(ReRankOptions::CrossEncoder)
        {
            None
        } else {
            data.score_threshold
        },
        limit: data.page_size.unwrap_or(10),
        sort_by: sort_by.clone(),
        rerank_by: rerank_by.clone(),
        filter: data.filters.clone(),
    }
    .into_qdrant_query(parsed_query, dataset.id, None, config, pool.clone())
    .await?;

    let search_chunk_query_results = retrieve_qdrant_points_query(
        vec![qdrant_query],
        data.page.unwrap_or(1),
        data.get_total_pages.unwrap_or(false),
        config,
    )
    .await?;

    timer.add("fetched from qdrant");

    let mut result_chunks = retrieve_chunks_from_point_ids(
        search_chunk_query_results,
        Some(timer),
        &data,
        pool.clone(),
    )
    .await?;

    let rerank_chunks_input = if let Some(rerank_by) = rerank_by {
        match rerank_by.rerank_type {
            ReRankOptions::CrossEncoder => {
                let mut cross_encoder_results = cross_encoder(
                    data.query.clone().to_single_query()?,
                    data.page_size.unwrap_or(10),
                    result_chunks.score_chunks,
                    config,
                )
                .await?;

                if let Some(score_threshold) = data.score_threshold {
                    cross_encoder_results.retain(|chunk| chunk.score >= score_threshold.into());
                }

                cross_encoder_results
            }
            _ => result_chunks.score_chunks,
        }
    } else {
        result_chunks.score_chunks
    };

    result_chunks.score_chunks = rerank_chunks(
        rerank_chunks_input,
        sort_by,
        data.sort_options
            .as_ref()
            .map(|d| d.tag_weights.clone())
            .unwrap_or_default(),
        data.sort_options
            .as_ref()
            .map(|d| d.use_weights)
            .unwrap_or_default(),
        data.sort_options
            .as_ref()
            .map(|d| d.location_bias)
            .unwrap_or_default(),
    );

    timer.add("reranking");
    transaction.finish();

    result_chunks.corrected_query = corrected_query;

    Ok(result_chunks)
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(timer, pool, redis_pool))]
pub async fn search_hybrid_chunks(
    mut data: SearchChunksReqPayload,
    parsed_query: ParsedQuery,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset: Dataset,
    config: &DatasetConfiguration,
    timer: &mut Timer,
) -> Result<SearchChunkQueryResponseBody, actix_web::Error> {
    let parent_span = sentry::configure_scope(|scope| scope.get_span());
    let transaction: sentry::TransactionOrSpan = match &parent_span {
        Some(parent) => parent
            .start_child("hybrid search", "Search Hybrid Chunks")
            .into(),
        None => {
            let ctx = sentry::TransactionContext::new("hybrid search", "Search Hybrid Chunks");
            sentry::start_transaction(ctx).into()
        }
    };
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone())));

    let mut parsed_query = parsed_query.clone();
    let mut corrected_query = None;

    if let Some(options) = &data.typo_options {
        timer.add("start correcting query");
        corrected_query =
            correct_query(parsed_query.query.clone(), dataset.id, redis_pool, options).await?;
        parsed_query.query = corrected_query
            .clone()
            .unwrap_or(parsed_query.query.clone());
        data.query = QueryTypes::Single(parsed_query.query.clone());

        timer.add("corrected query");
    }

    let dataset_config = DatasetConfiguration::from_json(dataset.server_configuration.clone());

    let semantic_boost = data
        .scoring_options
        .clone()
        .map(|options| options.semantic_boost)
        .unwrap_or(None);
    let fulltext_boost = data
        .scoring_options
        .clone()
        .map(|options| options.fulltext_boost)
        .unwrap_or(None);

    let dense_query_vector_future = get_dense_vector(
        data.query.clone().to_single_query()?,
        semantic_boost,
        "query",
        dataset_config.clone(),
    );

    let sparse_query_vector_future =
        get_sparse_vector(parsed_query.query.clone(), fulltext_boost, "query");

    let (dense_vector, sparse_vector) =
        futures::try_join!(dense_query_vector_future, sparse_query_vector_future)?;

    timer.add("computed sparse and dense embeddings");

    let (sort_by, rerank_by) = match data.sort_options.as_ref().map(|d| d.sort_by.clone()) {
        Some(Some(sort_by)) => match sort_by {
            QdrantSortBy::Field(field) => (Some(field.clone()), None),
            QdrantSortBy::SearchType(search_type) => (None, Some(search_type)),
        },
        _ => (None, None),
    };

    let qdrant_queries = vec![
        RetrievePointQuery {
            vector: VectorType::Dense(dense_vector),
            score_threshold: None,
            sort_by: sort_by.clone(),
            rerank_by: rerank_by.clone(),
            limit: data.page_size.unwrap_or(10),
            filter: data.filters.clone(),
        }
        .into_qdrant_query(
            ParsedQueryTypes::Single(parsed_query.clone()),
            dataset.id,
            None,
            config,
            pool.clone(),
        )
        .await?,
        RetrievePointQuery {
            vector: VectorType::SpladeSparse(sparse_vector),
            score_threshold: None,
            sort_by: sort_by.clone(),
            rerank_by: rerank_by.clone(),
            limit: data.page_size.unwrap_or(10),
            filter: data.filters.clone(),
        }
        .into_qdrant_query(
            ParsedQueryTypes::Single(parsed_query.clone()),
            dataset.id,
            None,
            config,
            pool.clone(),
        )
        .await?,
    ];

    let search_chunk_query_results = retrieve_qdrant_points_query(
        qdrant_queries,
        data.page.unwrap_or(1),
        data.get_total_pages.unwrap_or(false),
        config,
    )
    .await?;

    let result_chunks = retrieve_chunks_from_point_ids(
        search_chunk_query_results,
        Some(timer),
        &data,
        pool.clone(),
    )
    .await?;

    timer.add("fetched metadata from postgres");

    let mut reranked_chunks = {
        let mut reranked_chunks = {
            let mut cross_encoder_results = cross_encoder(
                data.query.clone().to_single_query()?,
                data.page_size.unwrap_or(10),
                result_chunks.score_chunks,
                config,
            )
            .await?;

            if let Some(score_threshold) = data.score_threshold {
                cross_encoder_results.retain(|chunk| chunk.score >= score_threshold.into());
            }

            rerank_chunks(
                cross_encoder_results,
                sort_by,
                data.sort_options
                    .as_ref()
                    .map(|d| d.tag_weights.clone())
                    .unwrap_or_default(),
                data.sort_options
                    .as_ref()
                    .map(|d| d.use_weights)
                    .unwrap_or_default(),
                data.sort_options
                    .as_ref()
                    .map(|d| d.location_bias)
                    .unwrap_or_default(),
            )
        };

        reranked_chunks.truncate(data.page_size.unwrap_or(10) as usize);

        timer.add("reranking");

        SearchChunkQueryResponseBody {
            score_chunks: reranked_chunks,
            corrected_query,
            total_chunk_pages: result_chunks.total_chunk_pages,
        }
    };

    if data.slim_chunks.unwrap_or(false) {
        reranked_chunks.score_chunks = reranked_chunks
            .score_chunks
            .into_iter()
            .map(|score_chunk| ScoreChunkDTO {
                metadata: score_chunk
                    .metadata
                    .into_iter()
                    .map(|metadata| {
                        let slim_chunk = SlimChunkMetadata::from(metadata.metadata());

                        match metadata {
                            ChunkMetadataTypes::Metadata(_) => slim_chunk.into(),
                            ChunkMetadataTypes::Content(_) => slim_chunk.into(),
                            ChunkMetadataTypes::ID(_) => metadata,
                        }
                    })
                    .collect(),
                highlights: score_chunk.highlights,
                score: score_chunk.score,
            })
            .collect();
    }

    transaction.finish();
    Ok(reranked_chunks)
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool, timer, redis_pool))]
pub async fn search_groups_query(
    mut data: SearchWithinGroupReqPayload,
    parsed_query: ParsedQueryTypes,
    group: ChunkGroupAndFileId,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset: Dataset,
    config: &DatasetConfiguration,
    timer: &mut Timer,
) -> Result<SearchWithinGroupResults, actix_web::Error> {
    let vector =
        get_qdrant_vector(data.clone().search_type, parsed_query.clone(), None, config).await?;

    let mut parsed_query = parsed_query.clone();
    let mut corrected_query = None;

    if let Some(options) = &data.typo_options {
        timer.add("start correcting query");
        match parsed_query {
            ParsedQueryTypes::Single(ref mut query) => {
                corrected_query =
                    correct_query(query.query.clone(), dataset.id, redis_pool, options).await?;
                query.query = corrected_query.clone().unwrap_or(query.query.clone());
                data.query = QueryTypes::Single(query.query.clone());
            }
            ParsedQueryTypes::Multi(ref mut queries) => {
                for (query, _) in queries {
                    corrected_query =
                        correct_query(query.query.clone(), dataset.id, redis_pool.clone(), options)
                            .await?;
                    query.query = corrected_query.clone().unwrap_or(query.query.clone());
                }
            }
        }
        timer.add("corrected query");
    }

    let (sort_by, rerank_by) = match data.sort_options.as_ref().map(|d| d.sort_by.clone()) {
        Some(Some(sort_by)) => match sort_by {
            QdrantSortBy::Field(field) => (Some(field.clone()), None),
            QdrantSortBy::SearchType(search_type) => (None, Some(search_type)),
        },
        _ => (None, None),
    };

    let qdrant_query = RetrievePointQuery {
        vector,
        score_threshold: if rerank_by.clone().map(|r| r.rerank_type)
            == Some(ReRankOptions::CrossEncoder)
        {
            None
        } else {
            data.score_threshold
        },
        limit: data.page_size.unwrap_or(10),
        sort_by: sort_by.clone(),
        rerank_by: rerank_by.clone(),
        filter: data.filters.clone(),
    }
    .into_qdrant_query(parsed_query, dataset.id, None, config, pool.clone())
    .await?;

    let search_semantic_chunk_query_results = retrieve_qdrant_points_query(
        vec![qdrant_query],
        data.page.unwrap_or(1),
        data.get_total_pages.unwrap_or(false),
        config,
    )
    .await?;

    let mut result_chunks = retrieve_chunks_from_point_ids(
        search_semantic_chunk_query_results,
        None,
        &web::Json(data.clone().into()),
        pool.clone(),
    )
    .await?;

    let rerank_chunks_input = if let Some(rerank_by) = rerank_by {
        match rerank_by.rerank_type {
            ReRankOptions::CrossEncoder => {
                let mut cross_encoder_results = cross_encoder(
                    data.query.clone().to_single_query()?,
                    data.page_size.unwrap_or(10),
                    result_chunks.score_chunks,
                    config,
                )
                .await?;

                if let Some(score_threshold) = data.score_threshold {
                    cross_encoder_results.retain(|chunk| chunk.score >= score_threshold.into());
                }

                cross_encoder_results
            }
            _ => result_chunks.score_chunks,
        }
    } else {
        result_chunks.score_chunks
    };

    result_chunks.score_chunks = rerank_chunks(
        rerank_chunks_input,
        sort_by,
        data.sort_options
            .as_ref()
            .map(|d| d.tag_weights.clone())
            .unwrap_or_default(),
        data.sort_options
            .as_ref()
            .map(|d| d.use_weights)
            .unwrap_or_default(),
        data.sort_options
            .as_ref()
            .map(|d| d.location_bias)
            .unwrap_or_default(),
    );

    Ok(SearchWithinGroupResults {
        bookmarks: result_chunks.score_chunks,
        group,
        corrected_query,
        total_pages: result_chunks.total_chunk_pages,
    })
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool, timer, redis_pool))]
pub async fn search_hybrid_groups(
    mut data: SearchWithinGroupReqPayload,
    parsed_query: ParsedQuery,
    group: ChunkGroupAndFileId,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset: Dataset,
    config: &DatasetConfiguration,
    timer: &mut Timer,
) -> Result<SearchWithinGroupResults, actix_web::Error> {
    let dataset_config = DatasetConfiguration::from_json(dataset.server_configuration.clone());

    let mut parsed_query = parsed_query.clone();
    let mut corrected_query = None;

    if let Some(options) = &data.typo_options {
        timer.add("start correcting query");
        corrected_query =
            correct_query(parsed_query.query.clone(), dataset.id, redis_pool, options).await?;
        parsed_query.query = corrected_query
            .clone()
            .unwrap_or(parsed_query.query.clone());
        data.query = QueryTypes::Single(parsed_query.query.clone());

        timer.add("corrected query");
    }

    let dense_vector_future = get_dense_vector(
        parsed_query.query.clone(),
        None,
        "query",
        dataset_config.clone(),
    );

    let sparse_vector_future = get_sparse_vector(parsed_query.query.clone(), None, "query");

    let (dense_vector, sparse_vector) =
        futures::try_join!(dense_vector_future, sparse_vector_future)?;

    let (sort_by, rerank_by) = match data.sort_options.as_ref().map(|d| d.sort_by.clone()) {
        Some(Some(sort_by)) => match sort_by {
            QdrantSortBy::Field(field) => (Some(field.clone()), None),
            QdrantSortBy::SearchType(search_type) => (None, Some(search_type)),
        },
        _ => (None, None),
    };

    let qdrant_queries = vec![
        RetrievePointQuery {
            vector: VectorType::Dense(dense_vector),
            score_threshold: None,
            sort_by: sort_by.clone(),
            rerank_by: rerank_by.clone(),
            limit: data.page_size.unwrap_or(10),
            filter: data.filters.clone(),
        }
        .into_qdrant_query(
            ParsedQueryTypes::Single(parsed_query.clone()),
            dataset.id,
            Some(group.id),
            config,
            pool.clone(),
        )
        .await?,
        RetrievePointQuery {
            vector: VectorType::SpladeSparse(sparse_vector),
            score_threshold: None,
            sort_by: sort_by.clone(),
            rerank_by: rerank_by.clone(),
            limit: data.page_size.unwrap_or(10),
            filter: data.filters.clone(),
        }
        .into_qdrant_query(
            ParsedQueryTypes::Single(parsed_query.clone()),
            dataset.id,
            Some(group.id),
            config,
            pool.clone(),
        )
        .await?,
    ];

    let mut qdrant_results = retrieve_qdrant_points_query(
        qdrant_queries,
        data.page.unwrap_or(1),
        data.get_total_pages.unwrap_or(false),
        config,
    )
    .await?;

    qdrant_results.search_results = qdrant_results
        .search_results
        .iter()
        .unique_by(|chunk| chunk.point_id)
        .cloned()
        .collect();

    let result_chunks = retrieve_chunks_from_point_ids(
        qdrant_results,
        None,
        &web::Json(data.clone().into()),
        pool.clone(),
    )
    .await?;

    let reranked_chunks = {
        let mut reranked_chunks = if result_chunks.score_chunks.len() > 20 {
            let split_results = result_chunks
                .score_chunks
                .chunks(20)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<Vec<ScoreChunkDTO>>>();

            let cross_encoder_results = cross_encoder(
                data.query.clone().to_single_query()?,
                data.page_size.unwrap_or(10),
                split_results
                    .get(0)
                    .expect("Split results must exist")
                    .to_vec(),
                config,
            )
            .await?;
            let score_chunks = rerank_chunks(
                cross_encoder_results,
                sort_by,
                data.sort_options
                    .as_ref()
                    .map(|d| d.tag_weights.clone())
                    .unwrap_or_default(),
                data.sort_options
                    .as_ref()
                    .map(|d| d.use_weights)
                    .unwrap_or_default(),
                data.sort_options
                    .as_ref()
                    .map(|d| d.location_bias)
                    .unwrap_or_default(),
            );

            score_chunks
                .iter()
                .chain(split_results.get(1).unwrap().iter())
                .cloned()
                .collect::<Vec<ScoreChunkDTO>>()
        } else {
            let cross_encoder_results = cross_encoder(
                data.query.clone().to_single_query()?,
                data.page_size.unwrap_or(10),
                result_chunks.score_chunks.clone(),
                config,
            )
            .await?;

            rerank_chunks(
                cross_encoder_results,
                sort_by,
                data.sort_options
                    .as_ref()
                    .map(|d| d.tag_weights.clone())
                    .unwrap_or_default(),
                data.sort_options
                    .as_ref()
                    .map(|d| d.use_weights)
                    .unwrap_or_default(),
                data.sort_options
                    .as_ref()
                    .map(|d| d.location_bias)
                    .unwrap_or_default(),
            )
        };

        if let Some(score_threshold) = data.score_threshold {
            reranked_chunks.retain(|chunk| chunk.score >= score_threshold.into());
        }

        SearchChunkQueryResponseBody {
            score_chunks: reranked_chunks,
            corrected_query: None,
            total_chunk_pages: result_chunks.total_chunk_pages,
        }
    };

    Ok(SearchWithinGroupResults {
        bookmarks: reranked_chunks.score_chunks,
        group,
        corrected_query,
        total_pages: result_chunks.total_chunk_pages,
    })
}

#[tracing::instrument(skip(timer, pool, redis_pool))]
pub async fn semantic_search_over_groups(
    mut data: SearchOverGroupsReqPayload,
    parsed_query: ParsedQueryTypes,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset: Dataset,
    config: &DatasetConfiguration,
    timer: &mut Timer,
) -> Result<DeprecatedSearchOverGroupsResponseBody, actix_web::Error> {
    let dataset_config = DatasetConfiguration::from_json(dataset.server_configuration.clone());

    let mut parsed_query = parsed_query.clone();
    let mut corrected_query = None;

    if let Some(options) = &data.typo_options {
        timer.add("start correcting query");
        match parsed_query {
            ParsedQueryTypes::Single(ref mut query) => {
                corrected_query =
                    correct_query(query.query.clone(), dataset.id, redis_pool, options).await?;
                query.query = corrected_query.clone().unwrap_or(query.query.clone());
                data.query = QueryTypes::Single(query.query.clone());
            }
            ParsedQueryTypes::Multi(ref mut queries) => {
                for (query, _) in queries {
                    corrected_query =
                        correct_query(query.query.clone(), dataset.id, redis_pool.clone(), options)
                            .await?;
                    query.query = corrected_query.clone().unwrap_or(query.query.clone());
                }
            }
        }
        timer.add("corrected query");
    }

    timer.add("start to create dense embedding vector");

    let embedding_vector = get_qdrant_vector(
        data.clone().search_type,
        parsed_query.clone(),
        None,
        &dataset_config.clone(),
    )
    .await?;

    timer.add("computed dense embedding");

    let search_over_groups_qdrant_result = retrieve_group_qdrant_points_query(
        embedding_vector,
        data.page.unwrap_or(1),
        data.get_total_pages.unwrap_or(false),
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.group_size.unwrap_or(3),
        parsed_query,
        dataset.id,
        pool.clone(),
        config,
    )
    .await?;

    timer.add("fetched from qdrant");

    let mut result_chunks = retrieve_chunks_for_groups(
        search_over_groups_qdrant_result.clone(),
        &data,
        pool.clone(),
    )
    .await?;

    result_chunks.group_chunks = search_over_groups_qdrant_result
        .search_results
        .iter()
        .filter_map(|search_result| {
            result_chunks
                .group_chunks
                .iter()
                .find(|group| group.group_id == search_result.group_id)
                .cloned()
        })
        .collect();

    timer.add("fetched from postgres");

    //TODO: rerank for groups
    result_chunks.corrected_query = corrected_query;

    Ok(result_chunks)
}

#[tracing::instrument(skip(timer, pool, redis_pool))]
pub async fn full_text_search_over_groups(
    mut data: SearchOverGroupsReqPayload,
    parsed_query: ParsedQueryTypes,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset: Dataset,
    config: &DatasetConfiguration,
    timer: &mut Timer,
) -> Result<DeprecatedSearchOverGroupsResponseBody, actix_web::Error> {
    timer.add("start to get sparse vector");

    let embedding_vector = get_qdrant_vector(
        data.clone().search_type,
        parsed_query.clone(),
        None,
        &config.clone(),
    )
    .await?;

    timer.add("computed sparse vector");

    let mut parsed_query = parsed_query.clone();
    let mut corrected_query = None;

    if let Some(options) = &data.typo_options {
        timer.add("start correcting query");
        match parsed_query {
            ParsedQueryTypes::Single(ref mut query) => {
                corrected_query =
                    correct_query(query.query.clone(), dataset.id, redis_pool, options).await?;
                query.query = corrected_query.clone().unwrap_or(query.query.clone());
                data.query = QueryTypes::Single(query.query.clone());
            }
            ParsedQueryTypes::Multi(ref mut queries) => {
                for (query, _) in queries {
                    corrected_query =
                        correct_query(query.query.clone(), dataset.id, redis_pool.clone(), options)
                            .await?;
                    query.query = corrected_query.clone().unwrap_or(query.query.clone());
                }
            }
        }
        timer.add("corrected query");
    }

    let search_over_groups_qdrant_result = retrieve_group_qdrant_points_query(
        embedding_vector,
        data.page.unwrap_or(1),
        data.get_total_pages.unwrap_or(false),
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.group_size.unwrap_or(3),
        parsed_query,
        dataset.id,
        pool.clone(),
        config,
    )
    .await?;

    timer.add("fetched from qdrant");

    let mut result_groups_with_chunk_hits = retrieve_chunks_for_groups(
        search_over_groups_qdrant_result.clone(),
        &data,
        pool.clone(),
    )
    .await?;

    result_groups_with_chunk_hits.group_chunks = search_over_groups_qdrant_result
        .search_results
        .iter()
        .filter_map(|search_result| {
            result_groups_with_chunk_hits
                .group_chunks
                .iter()
                .find(|group| group.group_id == search_result.group_id)
                .cloned()
        })
        .collect();

    timer.add("fetched from postgres");

    //TODO: rerank for groups
    result_groups_with_chunk_hits.corrected_query = corrected_query;

    Ok(result_groups_with_chunk_hits)
}

async fn cross_encoder_for_groups(
    query: String,
    page_size: u64,
    groups_chunks: Vec<GroupScoreChunk>,
    config: &DatasetConfiguration,
) -> Result<Vec<GroupScoreChunk>, actix_web::Error> {
    let score_chunks = groups_chunks
        .iter()
        .map(|group| {
            group
                .metadata
                .clone()
                .get(0)
                .expect("Metadata should have one element")
                .clone()
        })
        .collect_vec();

    let cross_encoder_results = cross_encoder(query, page_size, score_chunks, config).await?;
    let mut group_results = cross_encoder_results
        .into_iter()
        .map(|score_chunk| {
            let mut group = groups_chunks
                .iter()
                .find(|group| {
                    group.metadata.iter().any(|chunk| {
                        chunk.metadata[0].metadata().id == score_chunk.metadata[0].metadata().id
                    })
                })
                .expect("Group not found")
                .clone();
            group.metadata[0].score = score_chunk.score;
            group
        })
        .collect_vec();

    group_results.dedup_by(|a, b| a.group_id == b.group_id);

    group_results.sort_by(|a, b| {
        b.metadata[0]
            .score
            .partial_cmp(&a.metadata[0].score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    group_results = group_results
        .into_iter()
        .map(|mut group| {
            group.metadata = group
                .metadata
                .into_iter()
                .sorted_by(|a, b| b.score.partial_cmp(&a.score).unwrap())
                .collect();
            group
        })
        .collect_vec();

    Ok(group_results)
}

#[tracing::instrument(skip(timer, pool, redis_pool))]
pub async fn hybrid_search_over_groups(
    mut data: SearchOverGroupsReqPayload,
    parsed_query: ParsedQuery,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset: Dataset,
    config: &DatasetConfiguration,
    timer: &mut Timer,
) -> Result<DeprecatedSearchOverGroupsResponseBody, actix_web::Error> {
    let dataset_config = DatasetConfiguration::from_json(dataset.server_configuration.clone());

    let mut parsed_query = parsed_query.clone();
    let mut corrected_query = None;

    if let Some(options) = &data.typo_options {
        timer.add("start correcting query");
        corrected_query =
            correct_query(parsed_query.query.clone(), dataset.id, redis_pool, options).await?;
        parsed_query.query = corrected_query
            .clone()
            .unwrap_or(parsed_query.query.clone());
        data.query = QueryTypes::Single(parsed_query.query.clone());

        timer.add("corrected query");
    }

    timer.add("start to create dense embedding vector and sparse vector");

    let dense_embedding_vectors_future = get_dense_vector(
        data.query.clone().to_single_query()?,
        None,
        "query",
        dataset_config.clone(),
    );

    let sparse_embedding_vector_future =
        get_sparse_vector(data.query.clone().to_single_query()?, None, "query");

    let (dense_vector, sparse_vector) = futures::try_join!(
        dense_embedding_vectors_future,
        sparse_embedding_vector_future
    )?;

    timer.add("computed dense embedding");

    let semantic_future = retrieve_group_qdrant_points_query(
        VectorType::Dense(dense_vector),
        data.page.unwrap_or(1),
        data.get_total_pages.unwrap_or(false),
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        None,
        data.group_size.unwrap_or(3),
        ParsedQueryTypes::Single(parsed_query.clone()),
        dataset.id,
        pool.clone(),
        config,
    );

    let full_text_future = retrieve_group_qdrant_points_query(
        VectorType::SpladeSparse(sparse_vector),
        data.page.unwrap_or(1),
        data.get_total_pages.unwrap_or(false),
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        None,
        data.group_size.unwrap_or(3),
        ParsedQueryTypes::Single(parsed_query.clone()),
        dataset.id,
        pool.clone(),
        config,
    );

    let (semantic_results, full_text_results) = futures::join!(semantic_future, full_text_future);

    let semantic_results = semantic_results?;

    let full_text_results = full_text_results?;

    let combined_results = semantic_results
        .clone()
        .search_results
        .iter()
        .zip(full_text_results.search_results.iter())
        .flat_map(|(x, y)| vec![x.clone(), y.clone()])
        .unique_by(|chunk| chunk.group_id)
        .collect::<Vec<GroupSearchResults>>();

    let combined_search_chunk_query_results = SearchOverGroupsQueryResult {
        search_results: combined_results,
        corrected_query: None,
        total_chunk_pages: semantic_results.total_chunk_pages,
    };

    timer.add("fetched from qdrant");

    let combined_result_chunks = retrieve_chunks_for_groups(
        combined_search_chunk_query_results.clone(),
        &data,
        pool.clone(),
    )
    .await?;

    timer.add("fetched from postgres");

    let mut reranked_chunks = if combined_result_chunks.group_chunks.len() > 20 {
        let split_results = combined_result_chunks
            .group_chunks
            .chunks(20)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<Vec<GroupScoreChunk>>>();

        let cross_encoder_results = cross_encoder_for_groups(
            data.query.clone().to_single_query()?,
            data.page_size.unwrap_or(10),
            split_results
                .get(0)
                .expect("Split results must exist")
                .to_vec(),
            config,
        )
        .await?;

        cross_encoder_results
            .iter()
            .chain(split_results.get(1).unwrap().iter())
            .cloned()
            .collect::<Vec<GroupScoreChunk>>()
    } else {
        cross_encoder_for_groups(
            data.query.clone().to_single_query()?,
            data.page_size.unwrap_or(10),
            combined_result_chunks.group_chunks.clone(),
            config,
        )
        .await?
    };

    timer.add("reranking");

    if let Some(score_threshold) = data.score_threshold {
        reranked_chunks.retain(|chunk| {
            chunk.metadata.get(0).map(|m| m.score).unwrap_or(0.0) >= score_threshold.into()
        });
        reranked_chunks.iter_mut().for_each(|chunk| {
            chunk
                .metadata
                .retain(|metadata| metadata.score >= score_threshold.into())
        });
    }

    let result_chunks = DeprecatedSearchOverGroupsResponseBody {
        group_chunks: reranked_chunks,
        corrected_query,
        total_chunk_pages: combined_search_chunk_query_results.total_chunk_pages,
    };

    //TODO: rerank for groups

    Ok(result_chunks)
}

#[tracing::instrument(skip(timer, pool, redis_pool))]
pub async fn autocomplete_chunks_query(
    mut data: AutocompleteReqPayload,
    parsed_query: ParsedQuery,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset: Dataset,
    config: &DatasetConfiguration,
    timer: &mut Timer,
) -> Result<SearchChunkQueryResponseBody, actix_web::Error> {
    let parent_span = sentry::configure_scope(|scope| scope.get_span());

    let mut parsed_query = parsed_query.clone();
    let mut corrected_query = None;

    if let Some(options) = &data.typo_options {
        timer.add("start correcting query");
        corrected_query =
            correct_query(parsed_query.query.clone(), dataset.id, redis_pool, options).await?;
        parsed_query.query = corrected_query
            .clone()
            .unwrap_or(parsed_query.query.clone());
        data.query.clone_from(&parsed_query.query);

        timer.add("corrected query");
    }

    let transaction: sentry::TransactionOrSpan = match &parent_span {
        Some(parent) => parent
            .start_child("semantic search", "Search Semantic Chunks")
            .into(),
        None => {
            let ctx = sentry::TransactionContext::new("semantic search", "Search Semantic Chunks");
            sentry::start_transaction(ctx).into()
        }
    };
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone())));

    timer.add("start to create dense embedding vector");

    timer.add("computed dense embedding");

    let (sort_by, rerank_by) = match data.sort_options.as_ref().map(|d| d.sort_by.clone()) {
        Some(Some(sort_by)) => match sort_by {
            QdrantSortBy::Field(field) => (Some(field.clone()), None),
            QdrantSortBy::SearchType(search_type) => (None, Some(search_type)),
        },
        _ => (None, None),
    };

    let vector = get_qdrant_vector(
        data.clone().search_type,
        ParsedQueryTypes::Single(parsed_query.clone()),
        data.clone().scoring_options,
        config,
    )
    .await?;

    let mut qdrant_query = vec![
        RetrievePointQuery {
            vector: vector.clone(),
            score_threshold: data.score_threshold,
            sort_by: sort_by.clone(),
            rerank_by: rerank_by.clone(),
            limit: data.page_size.unwrap_or(10),
            filter: data.filters.clone(),
        }
        .into_qdrant_query(
            ParsedQueryTypes::Single(parsed_query.clone()),
            dataset.id,
            None,
            config,
            pool.clone(),
        )
        .await?,
    ];

    if let Some(q) = qdrant_query.get_mut(0) {
        q.filter
            .must
            .push(Condition::matches_text("content", data.query.clone()));
    }

    if data.extend_results.unwrap_or(false) {
        qdrant_query.push(
            RetrievePointQuery {
                vector,
                score_threshold: data.score_threshold,
                sort_by: sort_by.clone(),
                rerank_by: rerank_by.clone(),
                limit: data.page_size.unwrap_or(10),
                filter: data.filters.clone(),
            }
            .into_qdrant_query(
                ParsedQueryTypes::Single(parsed_query.clone()),
                dataset.id,
                None,
                config,
                pool.clone(),
            )
            .await?,
        );
    };

    let search_chunk_query_results =
        retrieve_qdrant_points_query(qdrant_query, 1, false, config).await?;

    timer.add("fetching from qdrant");

    let mut result_chunks = retrieve_chunks_from_point_ids(
        search_chunk_query_results.clone(),
        None,
        &data.clone().into(),
        pool.clone(),
    )
    .await?;

    timer.add("fetching from postgres");

    let first_increase = *search_chunk_query_results
        .batch_lengths
        .get(0)
        .unwrap_or(&0) as usize;

    let (before_increase, after_increase) = result_chunks.score_chunks.split_at(first_increase);
    let mut reranked_chunks = rerank_chunks(
        before_increase.to_vec(),
        sort_by.clone(),
        data.sort_options
            .as_ref()
            .map(|d| d.tag_weights.clone())
            .unwrap_or_default(),
        data.sort_options
            .as_ref()
            .map(|d| d.use_weights)
            .unwrap_or_default(),
        data.sort_options
            .as_ref()
            .map(|d| d.location_bias)
            .unwrap_or_default(),
    );
    reranked_chunks.extend(rerank_chunks(
        after_increase.to_vec(),
        sort_by,
        data.sort_options
            .as_ref()
            .map(|d| d.tag_weights.clone())
            .unwrap_or_default(),
        data.sort_options
            .as_ref()
            .map(|d| d.use_weights)
            .unwrap_or_default(),
        data.sort_options
            .as_ref()
            .map(|d| d.location_bias)
            .unwrap_or_default(),
    ));

    result_chunks.score_chunks = reranked_chunks;

    timer.add("reranking");
    transaction.finish();

    result_chunks.corrected_query = corrected_query;

    Ok(result_chunks)
}

#[tracing::instrument(skip(pool))]
pub async fn count_chunks_query(
    data: CountChunksReqPayload,
    parsed_query: ParsedQueryTypes,
    pool: web::Data<Pool>,
    dataset: Dataset,
    config: &DatasetConfiguration,
) -> Result<CountChunkQueryResponseBody, actix_web::Error> {
    let vector = get_qdrant_vector(
        data.clone().search_type.into(),
        parsed_query.clone(),
        None,
        config,
    )
    .await?;

    let qdrant_query = RetrievePointQuery {
        vector,
        score_threshold: data.score_threshold,
        sort_by: None,
        rerank_by: None,
        limit: data.limit.unwrap_or(100000_u64),
        filter: data.filters.clone(),
    }
    .into_qdrant_query(parsed_query, dataset.id, None, config, pool.clone())
    .await?;

    let count = count_qdrant_query(
        data.limit.unwrap_or(100000_u64),
        vec![qdrant_query],
        config.clone(),
    )
    .await? as u32;

    Ok(CountChunkQueryResponseBody { count })
}
