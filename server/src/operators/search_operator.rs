use super::chunk_operator::{
    find_relevant_sentence, get_metadata_and_collided_chunks_from_point_ids_query,
    get_metadata_from_point_ids,
};
use super::model_operator::{create_embedding, cross_encoder};
use super::qdrant_operator::{search_over_groups_query, GroupSearchResults, VectorType};
use crate::data::models::{
    ChunkFileWithName, ChunkGroup, ChunkMetadataWithFileData, Dataset, FullTextSearchResult,
    ServerDatasetConfiguration,
};
use crate::data::schema::{self};
use crate::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use crate::errors::ServiceError;
use crate::get_env;
use crate::handlers::chunk_handler::{
    ParsedQuery, ScoreChunkDTO, SearchChunkData, SearchChunkQueryResponseBody, SearchGroupsData,
    SearchGroupsResult, SearchOverGroupsData,
};
use crate::operators::model_operator::get_splade_query_embedding;
use crate::operators::qdrant_operator::{get_qdrant_connection, search_qdrant_query};
use crate::{data::models::Pool, errors::DefaultError};
use actix_web::web;
use dateparser::DateTimeUtc;
use diesel::{dsl::sql, sql_types::Text};
use diesel::{
    BoolExpressionMethods, JoinOnDsl, NullableExpressionMethods, PgTextExpressionMethods,
};
use itertools::Itertools;
use simple_server_timing_header::Timer;
use utoipa::ToSchema;

use qdrant_client::qdrant::condition::ConditionOneOf::HasId;
use qdrant_client::qdrant::Range;
use qdrant_client::qdrant::{
    point_id::PointIdOptions, Condition, Filter, HasIdCondition, PointId, SearchPoints,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchResult {
    pub score: f32,
    pub point_id: uuid::Uuid,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchChunkQueryResult {
    pub search_results: Vec<SearchResult>,
    pub total_chunk_pages: i64,
}

#[allow(clippy::too_many_arguments)]
pub fn assemble_qdrant_filter(
    tag_set: Option<Vec<String>>,
    link: Option<Vec<String>>,
    time_range: Option<(String, String)>,
    filters: Option<serde_json::Value>,
    quote_words: Option<Vec<String>>,
    negated_words: Option<Vec<String>>,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Filter, DefaultError> {
    let mut filter = Filter::default();

    let tag_set_inner = tag_set.unwrap_or_default();
    let link_inner = link.unwrap_or_default();

    filter
        .must
        .push(Condition::matches("dataset_id", dataset_id.to_string()));
    //TODO: fix this after new qdrant rust client gets released
    if !tag_set_inner.is_empty() {
        filter
            .must
            .push(Condition::matches("tag_set", tag_set_inner));
    }

    if !link_inner.is_empty() {
        filter.must.push(Condition::matches("link", link_inner));
    }

    if let Some(time_range) = time_range {
        if !time_range.0.is_empty() && !time_range.1.is_empty() {
            filter.must.push(Condition::range(
                "time_stamp",
                Range {
                    gt: None,
                    lt: None,
                    gte: Some(
                        time_range
                            .0
                            .clone()
                            .parse::<DateTimeUtc>()
                            .map_err(|_| DefaultError {
                                message: "Failed to parse time range",
                            })?
                            .0
                            .with_timezone(&chrono::Local)
                            .naive_local()
                            .timestamp() as f64,
                    ),
                    lte: Some(
                        time_range
                            .1
                            .clone()
                            .parse::<DateTimeUtc>()
                            .map_err(|_| DefaultError {
                                message: "Failed to parse time range",
                            })?
                            .0
                            .with_timezone(&chrono::Local)
                            .naive_local()
                            .timestamp() as f64,
                    ),
                },
            ));
        } else if time_range.1.is_empty() {
            filter.must.push(Condition::range(
                "time_stamp",
                Range {
                    gt: None,
                    lt: None,
                    gte: Some(
                        time_range
                            .0
                            .clone()
                            .parse::<DateTimeUtc>()
                            .map_err(|_| DefaultError {
                                message: "Failed to parse time range",
                            })?
                            .0
                            .with_timezone(&chrono::Local)
                            .naive_local()
                            .timestamp() as f64,
                    ),
                    lte: None,
                },
            ));
        } else if time_range.0.is_empty() {
            filter.must.push(Condition::range(
                "time_stamp",
                Range {
                    gt: None,
                    lt: None,
                    gte: None,
                    lte: Some(
                        time_range
                            .1
                            .clone()
                            .parse::<DateTimeUtc>()
                            .map_err(|_| DefaultError {
                                message: "Failed to parse time range",
                            })?
                            .0
                            .with_timezone(&chrono::Local)
                            .naive_local()
                            .timestamp() as f64,
                    ),
                },
            ));
        }
    }

    if let Some(serde_json::Value::Object(obj)) = &filters {
        for key in obj.keys() {
            let value = obj.get(key).expect("Value should exist");
            match value {
                serde_json::Value::Array(arr) => {
                    filter.must.push(Condition::matches(
                        &format!("metadata.{}", key),
                        arr.iter()
                            .map(|item| item.to_string())
                            .collect::<Vec<String>>(),
                    ));
                }
                _ => {
                    filter.must.push(Condition::matches(
                        &format!("metadata.{}", key),
                        value.to_string().replace('\"', ""),
                    ));
                }
            }
        }
    }

    if quote_words.is_some() || negated_words.is_some() {
        let available_qdrant_ids = get_qdrant_point_ids_from_pg_for_quote_negated_words(
            quote_words,
            negated_words,
            dataset_id,
            pool,
        )?;

        let available_point_ids = available_qdrant_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<HashSet<String>>()
            .iter()
            .map(|id| (*id).clone().into())
            .collect::<Vec<PointId>>();

        filter.should.push(Condition {
            condition_one_of: Some(HasId(HasIdCondition {
                has_id: (available_point_ids).to_vec(),
            })),
        });
    }

    Ok(filter)
}

#[allow(clippy::too_many_arguments)]
pub async fn retrieve_qdrant_points_query(
    vector: VectorType,
    page: u64,
    link: Option<Vec<String>>,
    tag_set: Option<Vec<String>>,
    time_range: Option<(String, String)>,
    limit: u64,
    score_threshold: Option<f32>,
    filters: Option<serde_json::Value>,
    parsed_query: ParsedQuery,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<SearchChunkQueryResult, DefaultError> {
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

    let filter = assemble_qdrant_filter(
        tag_set,
        link,
        time_range,
        filters,
        parsed_query.quote_words,
        parsed_query.negated_words,
        dataset_id,
        pool,
    )?;

    let point_ids = search_qdrant_query(page, filter, limit, score_threshold, vector).await?;
    transaction.finish();

    Ok(SearchChunkQueryResult {
        search_results: point_ids.clone(),
        //FIXME: dont have total results now
        total_chunk_pages: (point_ids.len() as f64 / 10.0).ceil() as i64,
    })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchOverGroupsQueryResult {
    pub search_results: Vec<GroupSearchResults>,
    pub total_chunk_pages: i64,
}

#[allow(clippy::too_many_arguments)]
pub async fn retrieve_group_qdrant_points_query(
    vector: VectorType,
    page: u64,
    link: Option<Vec<String>>,
    tag_set: Option<Vec<String>>,
    time_range: Option<(String, String)>,
    filters: Option<serde_json::Value>,
    limit: u32,
    score_threshold: Option<f32>,
    group_size: u32,
    parsed_query: ParsedQuery,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<SearchOverGroupsQueryResult, DefaultError> {
    let page = if page == 0 { 1 } else { page };

    let filter = assemble_qdrant_filter(
        tag_set,
        link,
        time_range,
        filters,
        parsed_query.quote_words,
        parsed_query.negated_words,
        dataset_id,
        pool,
    )?;

    let point_ids =
        search_over_groups_query(page, filter, limit, score_threshold, group_size, vector).await?;

    Ok(SearchOverGroupsQueryResult {
        search_results: point_ids.clone(),
        total_chunk_pages: (point_ids.len() as f64 / 10.0).ceil() as i64,
    })
}

pub async fn global_unfiltered_top_match_query(
    embedding_vector: Vec<f32>,
    dataset_id: uuid::Uuid,
) -> Result<SearchResult, DefaultError> {
    let qdrant = get_qdrant_connection().await?;

    let qdrant_group = get_env!(
        "QDRANT_COLLECTION",
        "QDRANT_COLLECTION should be set if this is called"
    )
    .to_string();

    let mut dataset_filter = Filter::default();
    dataset_filter
        .must
        .push(Condition::matches("dataset_id", dataset_id.to_string()));

    let vector_name = match embedding_vector.len() {
        384 => "384_vectors",
        768 => "768_vectors",
        1024 => "1024_vectors",
        1536 => "1536_vectors",
        _ => {
            return Err(DefaultError {
                message: "Invalid embedding vector size",
            })
        }
    };

    let data = qdrant
        .search_points(&SearchPoints {
            collection_name: qdrant_group,
            vector: embedding_vector,
            vector_name: Some(vector_name.to_string()),
            limit: 1,
            with_payload: None,
            filter: Some(dataset_filter),
            ..Default::default()
        })
        .await
        .map_err(|e| {
            log::error!("Failed to search points on Qdrant {:?}", e);
            DefaultError {
                message: "Failed to search points on Qdrant",
            }
        })?;

    let top_search_result: SearchResult = match data.result.first() {
        Some(point) => match point.clone().id {
            Some(point_id) => match point_id.point_id_options {
                Some(PointIdOptions::Uuid(id)) => SearchResult {
                    score: point.score,
                    point_id: uuid::Uuid::parse_str(&id).map_err(|_| DefaultError {
                        message: "Failed to parse uuid",
                    })?,
                },
                Some(PointIdOptions::Num(_)) => {
                    return Err(DefaultError {
                        message: "Failed to parse uuid",
                    })
                }
                None => {
                    return Err(DefaultError {
                        message: "Failed to parse uuid",
                    })
                }
            },
            None => {
                return Err(DefaultError {
                    message: "Failed to parse uuid",
                })
            }
        },
        // This only happens when there are no chunks in the database
        None => SearchResult {
            score: 0.0,
            point_id: uuid::Uuid::nil(),
        },
    };

    Ok(top_search_result)
}

#[allow(clippy::too_many_arguments)]
pub async fn search_semantic_chunk_groups_query(
    embedding_vector: Vec<f32>,
    page: u64,
    pool: web::Data<Pool>,
    link: Option<Vec<String>>,
    tag_set: Option<Vec<String>>,
    filters: Option<serde_json::Value>,
    limit: u64,
    score_threshold: Option<f32>,
    group_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    parsed_query: ParsedQuery,
) -> Result<SearchChunkQueryResult, DefaultError> {
    let page = if page == 0 { 1 } else { page };
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().unwrap();

    let mut query = chunk_metadata_columns::chunk_metadata
        .left_outer_join(
            chunk_collisions_columns::chunk_collisions
                .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
        )
        .left_outer_join(
            chunk_group_bookmarks_columns::chunk_group_bookmarks.on(chunk_metadata_columns::id
                .eq(chunk_group_bookmarks_columns::chunk_metadata_id)
                .and(chunk_group_bookmarks_columns::group_id.eq(group_id))),
        )
        .select((
            chunk_metadata_columns::qdrant_point_id,
            chunk_collisions_columns::collision_qdrant_id.nullable(),
        ))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
        .filter(chunk_group_bookmarks_columns::group_id.eq(group_id))
        .distinct()
        .into_boxed();
    let tag_set_inner = tag_set.unwrap_or_default();
    let link_inner = link.unwrap_or_default();

    if let Some(tag) = tag_set_inner.first() {
        query = query.filter(chunk_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }
    for tag in tag_set_inner.iter().skip(1) {
        query = query.or_filter(chunk_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }

    if let Some(link_inner) = link_inner.first() {
        query = query.filter(chunk_metadata_columns::link.ilike(format!("%{}%", link_inner)));
    }
    for link_url in link_inner.iter().skip(1) {
        query = query.or_filter(chunk_metadata_columns::link.ilike(format!("%{}%", link_url)));
    }

    if let Some(serde_json::Value::Object(obj)) = &filters {
        for key in obj.keys() {
            if let Some(value) = obj.get(key) {
                match value {
                    serde_json::Value::Array(arr) => {
                        if let Some(first_val) = arr.first() {
                            if let Some(string_val) = first_val.as_str() {
                                query = query.filter(
                                    sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
                                        .ilike(format!("%{}%", string_val)),
                                );
                            }
                        }

                        for item in arr.iter().skip(1) {
                            if let Some(string_val) = item.as_str() {
                                query = query.or_filter(
                                    sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
                                        .ilike(format!("%{}%", string_val)),
                                );
                            }
                        }
                    }
                    serde_json::Value::String(string_val) => {
                        query = query.filter(
                            sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
                                .ilike(format!("%{}%", string_val)),
                        );
                    }
                    _ => (),
                }
            }
        }
    }

    if let Some(quote_words) = parsed_query.quote_words {
        for word in quote_words.iter() {
            query = query.filter(chunk_metadata_columns::content.ilike(format!("%{}%", word)));
        }
    }

    if let Some(negated_words) = parsed_query.negated_words {
        for word in negated_words.iter() {
            query = query.filter(chunk_metadata_columns::content.not_ilike(format!("%{}%", word)));
        }
    }

    let filtered_option_ids: Vec<(Option<uuid::Uuid>, Option<uuid::Uuid>)> =
        query.load(&mut conn).map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    let filtered_point_ids: &Vec<PointId> = &filtered_option_ids
        .iter()
        .map(|uuid| {
            uuid.0
                .unwrap_or(uuid.1.unwrap_or(uuid::Uuid::nil()))
                .to_string()
        })
        // remove duplicates
        .collect::<HashSet<String>>()
        .iter()
        .map(|uuid| (*uuid).clone().into())
        .collect::<Vec<PointId>>();

    let mut filter = Filter::default();
    filter.should.push(Condition {
        condition_one_of: Some(HasId(HasIdCondition {
            has_id: (filtered_point_ids).to_vec(),
        })),
    });

    let point_ids: Vec<SearchResult> = search_qdrant_query(
        page,
        filter,
        limit,
        score_threshold,
        VectorType::Dense(embedding_vector),
    )
    .await?;

    Ok(SearchChunkQueryResult {
        search_results: point_ids,
        total_chunk_pages: (filtered_option_ids.len() as f64 / 10.0).ceil() as i64,
    })
}

pub fn get_metadata_query(
    chunk_metadata: Vec<FullTextSearchResult>,
    mut conn: r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
) -> Result<Vec<ChunkMetadataWithFileData>, DefaultError> {
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::files::dsl as files_columns;

    let all_datas = chunk_metadata_columns::chunk_metadata
        .filter(
            chunk_metadata_columns::id.eq_any(
                chunk_metadata
                    .iter()
                    .map(|chunk| chunk.id)
                    .collect::<Vec<uuid::Uuid>>()
                    .as_slice(),
            ),
        )
        .left_outer_join(
            chunk_files_columns::chunk_files
                .on(chunk_metadata_columns::id.eq(chunk_files_columns::chunk_id)),
        )
        .left_outer_join(
            files_columns::files.on(chunk_files_columns::file_id.eq(files_columns::id)),
        )
        .left_outer_join(
            chunk_collisions_columns::chunk_collisions
                .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
        )
        .select((
            (
                chunk_files_columns::chunk_id,
                chunk_files_columns::file_id,
                files_columns::file_name,
            )
                .nullable(),
            (
                chunk_metadata_columns::id,
                chunk_collisions_columns::collision_qdrant_id.nullable(),
            ),
        ))
        .load::<(Option<ChunkFileWithName>, (uuid::Uuid, Option<uuid::Uuid>))>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    #[allow(clippy::type_complexity)]
    let (file_ids, chunk_collisions): (
        Vec<Option<ChunkFileWithName>>,
        Vec<(uuid::Uuid, Option<uuid::Uuid>)>,
    ) = itertools::multiunzip(all_datas);

    let chunk_metadata_with_file_id: Vec<ChunkMetadataWithFileData> = chunk_metadata
        .into_iter()
        .map(|metadata| {
            let chunk_with_file_name = file_ids
                .iter()
                .flatten()
                .find(|file| file.chunk_id == metadata.id);

            let qdrant_point_id = match metadata.qdrant_point_id {
                Some(id) => id,
                None => {
                    chunk_collisions
                                    .iter()
                                    .find(|collision| collision.0 == metadata.id) // Match chunk id
                                    .expect("Qdrant point id does not exist for root chunk or collision")
                                    .1
                                    .expect("Collision Qdrant point id must exist if there is no root qdrant point id")
                },
            };

            ChunkMetadataWithFileData {
                id: metadata.id,
                content: metadata.content,
                link: metadata.link,
                tag_set: metadata.tag_set,
                qdrant_point_id,
                created_at: metadata.created_at,
                updated_at: metadata.updated_at,
                chunk_html: metadata.chunk_html,
                file_id: chunk_with_file_name.map(|file| file.file_id),
                file_name: chunk_with_file_name.map(|file| file.file_name.to_string()),
                metadata: metadata.metadata,
                tracking_id: metadata.tracking_id,
                time_stamp: metadata.time_stamp,
                weight: metadata.weight
            }
        })
        .collect();
    Ok(chunk_metadata_with_file_id)
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct FullTextDocIds {
    pub doc_ids: Option<uuid::Uuid>,
    pub total_count: i64,
}

#[allow(clippy::too_many_arguments)]
pub async fn search_full_text_group_query(
    user_query: String,
    page: u64,
    pool: web::Data<Pool>,
    filters: Option<serde_json::Value>,
    limit: u64,
    score_threshold: Option<f32>,
    link: Option<Vec<String>>,
    tag_set: Option<Vec<String>>,
    group_id: uuid::Uuid,
    parsed_query: ParsedQuery,
    dataset_uuid: uuid::Uuid,
) -> Result<SearchChunkQueryResult, DefaultError> {
    let page = if page == 0 { 1 } else { page };
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let second_join = diesel::alias!(schema::chunk_metadata as second_join);

    let mut conn = pool.get().unwrap();
    // SELECT
    //     chunk_metadata.qdrant_point_id,
    //     second_join.qdrant_point_id
    // FROM
    //     chunk_metadata
    // LEFT OUTER JOIN chunk_collisions ON
    //     chunk_metadata.id = chunk_collisions.chunk_id
    //     AND chunk_metadata.private = false
    // LEFT OUTER JOIN chunk_metadata AS second_join ON
    //     second_join.qdrant_point_id = chunk_collisions.collision_qdrant_id
    //     AND second_join.private = true
    // WHERE
    //     chunk_metadata.private = false
    //     and (second_join.qdrant_point_id notnull or chunk_metadata.qdrant_point_id notnull);
    let mut query = chunk_metadata_columns::chunk_metadata
        .left_outer_join(
            chunk_collisions_columns::chunk_collisions
                .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
        )
        .left_outer_join(
            second_join.on(second_join
                .field(schema::chunk_metadata::qdrant_point_id)
                .eq(chunk_collisions_columns::collision_qdrant_id)),
        )
        .left_outer_join(
            chunk_group_bookmarks_columns::chunk_group_bookmarks.on(chunk_metadata_columns::id
                .eq(chunk_group_bookmarks_columns::chunk_metadata_id)
                .and(chunk_group_bookmarks_columns::group_id.eq(group_id))),
        )
        .filter(chunk_group_bookmarks_columns::group_id.eq(group_id))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_uuid))
        .select((
            chunk_metadata_columns::qdrant_point_id,
            second_join
                .field(schema::chunk_metadata::qdrant_point_id)
                .nullable(),
        ))
        .distinct_on((
            chunk_metadata_columns::qdrant_point_id,
            second_join
                .field(schema::chunk_metadata::qdrant_point_id)
                .nullable(),
        ))
        .into_boxed();

    let tag_set_inner = tag_set.unwrap_or_default();
    let link_inner = link.unwrap_or_default();

    if let Some(tag) = tag_set_inner.first() {
        query = query.filter(chunk_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }
    for tag in tag_set_inner.iter().skip(1) {
        query = query.or_filter(chunk_metadata_columns::tag_set.ilike(format!("%{}%", tag)));
    }

    if let Some(link_inner) = link_inner.first() {
        query = query.filter(chunk_metadata_columns::link.ilike(format!("%{}%", link_inner)));
    }
    for link_url in link_inner.iter().skip(1) {
        query = query.or_filter(chunk_metadata_columns::link.ilike(format!("%{}%", link_url)));
    }

    if let Some(serde_json::Value::Object(obj)) = &filters {
        for key in obj.keys() {
            if let Some(value) = obj.get(key) {
                match value {
                    serde_json::Value::Array(arr) => {
                        if let Some(first_val) = arr.first() {
                            if let Some(string_val) = first_val.as_str() {
                                query = query.filter(
                                    sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
                                        .ilike(format!("%{}%", string_val)),
                                );
                            }
                        }

                        for item in arr.iter().skip(1) {
                            if let Some(string_val) = item.as_str() {
                                query = query.or_filter(
                                    sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
                                        .ilike(format!("%{}%", string_val)),
                                );
                            }
                        }
                    }
                    serde_json::Value::String(string_val) => {
                        query = query.filter(
                            sql::<Text>(&format!("chunk_metadata.metadata->>'{}'", key))
                                .ilike(format!("%{}%", string_val)),
                        );
                    }
                    _ => (),
                }
            }
        }
    }

    if let Some(quote_words) = parsed_query.quote_words {
        for word in quote_words.iter() {
            query = query.filter(chunk_metadata_columns::content.ilike(format!("%{}%", word)));
        }
    }

    if let Some(negated_words) = parsed_query.negated_words {
        for word in negated_words.iter() {
            query = query.filter(chunk_metadata_columns::content.not_ilike(format!("%{}%", word)));
        }
    }

    query = query.order((
        chunk_metadata_columns::qdrant_point_id,
        second_join.field(schema::chunk_metadata::qdrant_point_id),
    ));

    let matching_qdrant_point_ids: Vec<(Option<uuid::Uuid>, Option<uuid::Uuid>)> =
        query.load(&mut conn).map_err(|_| DefaultError {
            message: "Failed to load full-text searched chunks",
        })?;

    let matching_point_ids: Vec<PointId> = matching_qdrant_point_ids
        .iter()
        .map(|uuid| {
            uuid.0
                .unwrap_or(uuid.1.unwrap_or(uuid::Uuid::nil()))
                .to_string()
        })
        .collect::<HashSet<String>>()
        .iter()
        .map(|uuid| (*uuid).clone().into())
        .collect::<Vec<PointId>>();

    let mut filter = Filter::default();
    filter.should.push(Condition {
        condition_one_of: Some(HasId(HasIdCondition {
            has_id: (matching_point_ids).to_vec(),
        })),
    });

    let embedding_vector =
        get_splade_query_embedding(&user_query)
            .await
            .map_err(|_| DefaultError {
                message: "Failed to get splade query embedding",
            })?;

    let point_ids = search_qdrant_query(
        page,
        filter,
        limit,
        score_threshold,
        VectorType::Sparse(embedding_vector),
    )
    .await;

    Ok(SearchChunkQueryResult {
        search_results: point_ids?,
        total_chunk_pages: (matching_qdrant_point_ids.len() as f64 / 10.0).ceil() as i64,
    })
}

pub async fn retrieve_chunks_from_point_ids_without_collsions(
    search_chunk_query_results: SearchChunkQueryResult,
    data: &web::Json<SearchChunkData>,
    pool: web::Data<Pool>,
) -> Result<SearchChunkQueryResponseBody, actix_web::Error> {
    let point_ids = search_chunk_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();

    let metadata_chunks = get_metadata_from_point_ids(point_ids, pool)
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let score_chunks: Vec<ScoreChunkDTO> = search_chunk_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut chunk: ChunkMetadataWithFileData = match metadata_chunks
                .iter()
                .find(|metadata_chunk| metadata_chunk.qdrant_point_id == search_result.point_id)
            {
                Some(metadata_chunk) => metadata_chunk.clone(),
                None => ChunkMetadataWithFileData {
                    id: uuid::Uuid::default(),
                    qdrant_point_id: uuid::Uuid::default(),
                    created_at: chrono::Utc::now().naive_local(),
                    updated_at: chrono::Utc::now().naive_local(),
                    file_id: None,
                    file_name: None,
                    content: "".to_string(),
                    chunk_html: Some("".to_string()),
                    link: Some("".to_string()),
                    tag_set: Some("".to_string()),
                    metadata: None,
                    tracking_id: None,
                    time_stamp: None,
                    weight: 1.0,
                },
            };
            if data.highlight_results.unwrap_or(true) {
                chunk = find_relevant_sentence(
                    chunk.clone(),
                    data.query.clone(),
                    data.highlight_delimiters.clone().unwrap_or(vec![
                        ".".to_string(),
                        "!".to_string(),
                        "?".to_string(),
                        "\n".to_string(),
                        "\t".to_string(),
                        ",".to_string(),
                    ]),
                )
                .unwrap_or(chunk);
            }

            ScoreChunkDTO {
                metadata: vec![chunk],
                score: search_result.score.into(),
            }
        })
        .collect();

    Ok(SearchChunkQueryResponseBody {
        score_chunks,
        total_chunk_pages: search_chunk_query_results.total_chunk_pages,
    })
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct GroupScoreChunkDTO {
    pub group_id: uuid::Uuid,
    pub metadata: Vec<ScoreChunkDTO>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SearchOverGroupsResponseBody {
    pub group_chunks: Vec<GroupScoreChunkDTO>,
    pub total_chunk_pages: i64,
}

pub async fn retrieve_chunks_for_groups(
    search_over_groups_query_result: SearchOverGroupsQueryResult,
    data: &web::Json<SearchOverGroupsData>,
    pool: web::Data<Pool>,
) -> Result<SearchOverGroupsResponseBody, actix_web::Error> {
    let point_ids = search_over_groups_query_result
        .search_results
        .iter()
        .flat_map(|hit| hit.hits.iter().map(|point| point.point_id).collect_vec())
        .collect_vec();

    let (metadata_chunks, collided_chunks) = get_metadata_and_collided_chunks_from_point_ids_query(
        point_ids,
        data.get_collisions.unwrap_or(false),
        pool,
    )
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let group_chunks: Vec<GroupScoreChunkDTO> = search_over_groups_query_result
        .search_results
        .iter()
        .map(|group| {
            let score_chunk: Vec<ScoreChunkDTO> = group
                .hits
                .iter()
                .map(|search_result| {
                    let mut chunk: ChunkMetadataWithFileData =
                        match metadata_chunks.iter().find(|metadata_chunk| {
                            metadata_chunk.qdrant_point_id == search_result.point_id
                        }) {
                            Some(metadata_chunk) => metadata_chunk.clone(),
                            None => ChunkMetadataWithFileData {
                                id: uuid::Uuid::default(),
                                qdrant_point_id: uuid::Uuid::default(),
                                created_at: chrono::Utc::now().naive_local(),
                                updated_at: chrono::Utc::now().naive_local(),
                                file_id: None,
                                file_name: None,
                                content: "".to_string(),
                                chunk_html: Some("".to_string()),
                                link: Some("".to_string()),
                                tag_set: Some("".to_string()),
                                metadata: None,
                                tracking_id: None,
                                time_stamp: None,
                                weight: 1.0,
                            },
                        };

                    if data.highlight_results.unwrap_or(true) {
                        chunk = find_relevant_sentence(
                            chunk.clone(),
                            data.query.clone(),
                            data.highlight_delimiters.clone().unwrap_or(vec![
                                ".".to_string(),
                                "!".to_string(),
                                "?".to_string(),
                                "\n".to_string(),
                                "\t".to_string(),
                                ",".to_string(),
                            ]),
                        )
                        .unwrap_or(chunk);
                    }

                    let mut collided_chunks: Vec<ChunkMetadataWithFileData> = collided_chunks
                        .iter()
                        .filter(|chunk| chunk.qdrant_id == search_result.point_id)
                        .map(|chunk| chunk.metadata.clone())
                        .collect();

                    collided_chunks.insert(0, chunk);

                    ScoreChunkDTO {
                        metadata: collided_chunks,
                        score: search_result.score.into(),
                    }
                })
                .collect_vec();

            GroupScoreChunkDTO {
                group_id: group.group_id,
                metadata: score_chunk,
            }
        })
        .collect_vec();

    Ok(SearchOverGroupsResponseBody {
        group_chunks,
        total_chunk_pages: search_over_groups_query_result.total_chunk_pages,
    })
}

/// Retrieve chunks from point ids, DOES NOT GUARD AGAINST DATASET ACCESS PERMISSIONS
pub async fn retrieve_chunks_from_point_ids(
    search_chunk_query_results: SearchChunkQueryResult,
    data: &web::Json<SearchChunkData>,
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

    let (metadata_chunks, collided_chunks) = get_metadata_and_collided_chunks_from_point_ids_query(
        point_ids,
        data.get_collisions.unwrap_or(false),
        pool,
    )
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let score_chunks: Vec<ScoreChunkDTO> = search_chunk_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut chunk: ChunkMetadataWithFileData = match metadata_chunks
                .iter()
                .find(|metadata_chunk| metadata_chunk.qdrant_point_id == search_result.point_id)
            {
                Some(metadata_chunk) => metadata_chunk.clone(),
                None => ChunkMetadataWithFileData {
                    id: uuid::Uuid::default(),
                    qdrant_point_id: uuid::Uuid::default(),
                    created_at: chrono::Utc::now().naive_local(),
                    updated_at: chrono::Utc::now().naive_local(),
                    file_id: None,
                    file_name: None,
                    content: "".to_string(),
                    chunk_html: Some("".to_string()),
                    link: Some("".to_string()),
                    tag_set: Some("".to_string()),
                    metadata: None,
                    tracking_id: None,
                    time_stamp: None,
                    weight: 1.0,
                },
            };

            if data.highlight_results.unwrap_or(true) {
                chunk = find_relevant_sentence(
                    chunk.clone(),
                    data.query.clone(),
                    data.highlight_delimiters.clone().unwrap_or(vec![
                        ".".to_string(),
                        "!".to_string(),
                        "?".to_string(),
                        "\n".to_string(),
                        "\t".to_string(),
                        ",".to_string(),
                    ]),
                )
                .unwrap_or(chunk);
            }

            let mut collided_chunks: Vec<ChunkMetadataWithFileData> = collided_chunks
                .iter()
                .filter(|chunk| chunk.qdrant_id == search_result.point_id)
                .map(|chunk| chunk.metadata.clone())
                .collect();

            collided_chunks.insert(0, chunk);

            ScoreChunkDTO {
                metadata: collided_chunks,
                score: search_result.score.into(),
            }
        })
        .collect();

    transaction.finish();

    Ok(SearchChunkQueryResponseBody {
        score_chunks,
        total_chunk_pages: search_chunk_query_results.total_chunk_pages,
    })
}

pub fn rerank_chunks(chunks: Vec<ScoreChunkDTO>, date_bias: Option<bool>) -> Vec<ScoreChunkDTO> {
    let mut reranked_chunks = Vec::new();
    chunks.into_iter().for_each(|mut chunk| {
        if chunk.metadata[0].weight == 0.0 {
            chunk.metadata[0].weight = 1.0;
        }
        chunk.score *= chunk.metadata[0].weight;
        reranked_chunks.push(chunk);
    });

    if date_bias.is_some() && date_bias.unwrap() {
        reranked_chunks.sort_by(|a, b| {
            if let (Some(time_stamp_a), Some(time_stamp_b)) =
                (a.metadata[0].time_stamp, b.metadata[0].time_stamp)
            {
                return time_stamp_a.timestamp().cmp(&time_stamp_b.timestamp());
            }
            a.score.total_cmp(&b.score)
        });
    }

    reranked_chunks.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    reranked_chunks
}

pub async fn search_semantic_chunks(
    data: web::Json<SearchChunkData>,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
    timer: &mut Timer,
) -> Result<SearchChunkQueryResponseBody, actix_web::Error> {
    timer.add("Reached semantic_chunks");
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

    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());

    let embedding_vector = create_embedding(&data.query, dataset_config.clone()).await?;
    timer.add("Created Embedding vector");

    let search_chunk_query_results = retrieve_qdrant_points_query(
        VectorType::Dense(embedding_vector),
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.filters.clone(),
        parsed_query,
        dataset.id,
        pool.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    timer.add("Fetch from qdrant");

    let mut result_chunks =
        retrieve_chunks_from_point_ids(search_chunk_query_results, &data, pool.clone()).await?;

    timer.add("Fetch from postgres");

    result_chunks.score_chunks = rerank_chunks(result_chunks.score_chunks, data.date_bias);
    timer.add("Rerank (algo)");
    transaction.finish();

    Ok(result_chunks)
}

pub async fn search_full_text_chunks(
    data: web::Json<SearchChunkData>,
    mut parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchChunkQueryResponseBody, actix_web::Error> {
    let parent_span = sentry::configure_scope(|scope| scope.get_span());
    let transaction: sentry::TransactionOrSpan = match &parent_span {
        Some(parent) => parent
            .start_child("full text search", "Search Full Text Chunks")
            .into(),
        None => {
            let ctx =
                sentry::TransactionContext::new("full text search", "Search Full Text Chunks");
            sentry::start_transaction(ctx).into()
        }
    };
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone())));

    parsed_query.query = parsed_query
        .query
        .split_whitespace()
        .join(" AND ")
        .replace('\"', "");

    let embedding_vector = get_splade_query_embedding(&parsed_query.query)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to get splade query embedding".into()))?;

    let search_chunk_query_results = retrieve_qdrant_points_query(
        VectorType::Sparse(embedding_vector),
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.filters.clone(),
        parsed_query,
        dataset.id,
        pool.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let mut result_chunks =
        retrieve_chunks_from_point_ids(search_chunk_query_results, &data, pool).await?;

    result_chunks.score_chunks = rerank_chunks(result_chunks.score_chunks, data.date_bias);

    transaction.finish();
    Ok(result_chunks)
}

#[allow(clippy::too_many_arguments)]
pub async fn search_hybrid_chunks(
    data: web::Json<SearchChunkData>,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
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

    let pool1 = pool.clone();
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());

    let embedding_vector = create_embedding(&data.query, dataset_config.clone()).await?;

    let search_chunk_query_results = retrieve_qdrant_points_query(
        VectorType::Dense(embedding_vector),
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.filters.clone(),
        parsed_query.clone(),
        dataset.id,
        pool.clone(),
    );

    let full_text_handler_results =
        search_full_text_chunks(web::Json(data.clone()), parsed_query, page, pool, dataset);

    let (search_chunk_query_results, full_text_handler_results) =
        futures::join!(search_chunk_query_results, full_text_handler_results);

    let search_chunk_query_results =
        search_chunk_query_results.map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let full_text_handler_results =
        full_text_handler_results.map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let point_ids = search_chunk_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();

    let (metadata_chunks, collided_chunks) = get_metadata_and_collided_chunks_from_point_ids_query(
        point_ids,
        data.get_collisions.unwrap_or(false),
        pool1,
    )
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let semantic_score_chunks: Vec<ScoreChunkDTO> = search_chunk_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut chunk: ChunkMetadataWithFileData = match metadata_chunks
                .iter()
                .find(|metadata_chunk| metadata_chunk.qdrant_point_id == search_result.point_id)
            {
                Some(metadata_chunk) => metadata_chunk.clone(),
                None => ChunkMetadataWithFileData {
                    id: uuid::Uuid::default(),
                    qdrant_point_id: uuid::Uuid::default(),
                    created_at: chrono::Utc::now().naive_local(),
                    updated_at: chrono::Utc::now().naive_local(),
                    file_id: None,
                    file_name: None,
                    content: "".to_string(),
                    chunk_html: Some("".to_string()),
                    link: Some("".to_string()),
                    tag_set: Some("".to_string()),
                    metadata: None,
                    tracking_id: None,
                    time_stamp: None,
                    weight: 1.0,
                },
            };

            if data.highlight_results.unwrap_or(true) {
                chunk = find_relevant_sentence(
                    chunk.clone(),
                    data.query.clone(),
                    data.highlight_delimiters.clone().unwrap_or(vec![
                        ".".to_string(),
                        "!".to_string(),
                        "?".to_string(),
                        "\n".to_string(),
                        "\t".to_string(),
                        ",".to_string(),
                    ]),
                )
                .unwrap_or(chunk);
            }

            let mut collided_chunks: Vec<ChunkMetadataWithFileData> = collided_chunks
                .iter()
                .filter(|chunk| chunk.qdrant_id == search_result.point_id)
                .map(|chunk| chunk.metadata.clone())
                .collect();

            collided_chunks.insert(0, chunk);

            ScoreChunkDTO {
                metadata: collided_chunks,
                score: search_result.score as f64 * 0.5,
            }
        })
        .collect();

    let mut result_chunks = {
        let combined_results = semantic_score_chunks
            .iter()
            .zip(full_text_handler_results.score_chunks.iter())
            .flat_map(|(x, y)| vec![x.clone(), y.clone()])
            .unique_by(|score_chunk| score_chunk.metadata[0].id)
            .collect::<Vec<ScoreChunkDTO>>();

        let reranked_chunks = if combined_results.len() > 20 {
            let split_results = combined_results
                .chunks(20)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<Vec<ScoreChunkDTO>>>();

            let cross_encoder_results = cross_encoder(
                data.query.clone(),
                data.page_size.unwrap_or(10),
                split_results
                    .first()
                    .expect("Split results must exist")
                    .to_vec(),
            )
            .await?;

            cross_encoder_results
                .iter()
                .chain(split_results.get(1).unwrap().iter())
                .cloned()
                .collect::<Vec<ScoreChunkDTO>>()
        } else {
            cross_encoder(
                data.query.clone(),
                data.page_size.unwrap_or(10),
                combined_results,
            )
            .await?
        };

        SearchChunkQueryResponseBody {
            score_chunks: reranked_chunks,
            total_chunk_pages: search_chunk_query_results.total_chunk_pages,
        }
    };

    result_chunks.score_chunks = rerank_chunks(result_chunks.score_chunks, data.date_bias);

    transaction.finish();
    Ok(result_chunks)
}

#[allow(clippy::too_many_arguments)]
pub async fn search_semantic_groups(
    data: web::Json<SearchGroupsData>,
    parsed_query: ParsedQuery,
    group: ChunkGroup,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchGroupsResult, actix_web::Error> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());
    let embedding_vector: Vec<f32> = create_embedding(&data.query, dataset_config.clone()).await?;

    let search_semantic_chunk_query_results = search_semantic_chunk_groups_query(
        embedding_vector,
        page,
        pool.clone(),
        data.link.clone(),
        data.tag_set.clone(),
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.group_id,
        dataset.id,
        parsed_query,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let mut result_chunks = retrieve_chunks_from_point_ids_without_collsions(
        search_semantic_chunk_query_results,
        &web::Json(data.clone().into()),
        pool.clone(),
    )
    .await?;

    result_chunks.score_chunks = rerank_chunks(result_chunks.score_chunks, data.date_bias);

    Ok(SearchGroupsResult {
        bookmarks: result_chunks.score_chunks,
        group,
        total_pages: result_chunks.total_chunk_pages,
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn search_full_text_groups(
    data: web::Json<SearchGroupsData>,
    parsed_query: ParsedQuery,
    group: ChunkGroup,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchGroupsResult, actix_web::Error> {
    let data_inner = data.clone();

    let search_chunk_query_results = search_full_text_group_query(
        data_inner.query.clone(),
        page,
        pool.clone(),
        data_inner.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data_inner.link.clone(),
        data_inner.tag_set.clone(),
        data_inner.group_id,
        parsed_query,
        dataset.id,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let mut result_chunks = retrieve_chunks_from_point_ids_without_collsions(
        search_chunk_query_results,
        &web::Json(data.clone().into()),
        pool.clone(),
    )
    .await?;

    result_chunks.score_chunks = rerank_chunks(result_chunks.score_chunks, data.date_bias);

    Ok(SearchGroupsResult {
        bookmarks: result_chunks.score_chunks,
        group,
        total_pages: result_chunks.total_chunk_pages,
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn search_hybrid_groups(
    data: web::Json<SearchGroupsData>,
    parsed_query: ParsedQuery,
    group: ChunkGroup,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchGroupsResult, actix_web::Error> {
    let data_inner = data.clone();
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());

    let embedding_vector = create_embedding(&data.query, dataset_config.clone()).await?;

    let semantic_future = search_semantic_chunk_groups_query(
        embedding_vector,
        page,
        pool.clone(),
        data.link.clone(),
        data.tag_set.clone(),
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.group_id,
        dataset.id,
        parsed_query.clone(),
    );

    let full_text_future = search_full_text_group_query(
        data_inner.query.clone(),
        page,
        pool.clone(),
        data_inner.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data_inner.link.clone(),
        data_inner.tag_set.clone(),
        data_inner.group_id,
        parsed_query.clone(),
        dataset.id,
    );

    let (semantic_results, full_text_results) = futures::join!(semantic_future, full_text_future);

    let semantic_results =
        semantic_results.map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let full_text_results =
        full_text_results.map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let combined_results = semantic_results
        .clone()
        .search_results
        .iter()
        .zip(full_text_results.search_results.iter())
        .flat_map(|(x, y)| vec![x.clone(), y.clone()])
        .unique_by(|chunk| chunk.point_id)
        .collect::<Vec<SearchResult>>();

    let combined_search_chunk_query_results = SearchChunkQueryResult {
        search_results: combined_results,
        total_chunk_pages: semantic_results.total_chunk_pages,
    };

    let combined_result_chunks = retrieve_chunks_from_point_ids_without_collsions(
        combined_search_chunk_query_results,
        &web::Json(data.clone().into()),
        pool.clone(),
    )
    .await?;

    let mut result_chunks = {
        let reranked_chunks = if combined_result_chunks.score_chunks.len() > 20 {
            let split_results = combined_result_chunks
                .score_chunks
                .chunks(20)
                .map(|chunk| chunk.to_vec())
                .collect::<Vec<Vec<ScoreChunkDTO>>>();

            let cross_encoder_results = cross_encoder(
                data.query.clone(),
                data.page_size.unwrap_or(10),
                split_results
                    .first()
                    .expect("Split results must exist")
                    .to_vec(),
            )
            .await?;

            cross_encoder_results
                .iter()
                .chain(split_results.get(1).unwrap().iter())
                .cloned()
                .collect::<Vec<ScoreChunkDTO>>()
        } else {
            cross_encoder(
                data.query.clone(),
                data.page_size.unwrap_or(10),
                combined_result_chunks.score_chunks.clone(),
            )
            .await?
        };

        SearchChunkQueryResponseBody {
            score_chunks: reranked_chunks,
            total_chunk_pages: combined_result_chunks.total_chunk_pages,
        }
    };

    result_chunks.score_chunks = rerank_chunks(result_chunks.score_chunks, data.date_bias);

    Ok(SearchGroupsResult {
        bookmarks: combined_result_chunks.score_chunks,
        group,
        total_pages: combined_result_chunks.total_chunk_pages,
    })
}

pub async fn semantic_search_over_groups(
    data: web::Json<SearchOverGroupsData>,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchOverGroupsResponseBody, actix_web::Error> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());
    let embedding_vector = create_embedding(&data.query, dataset_config.clone()).await?;

    let search_chunk_query_results = retrieve_group_qdrant_points_query(
        VectorType::Dense(embedding_vector),
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.group_size.unwrap_or(3),
        parsed_query,
        dataset.id,
        pool.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let result_chunks =
        retrieve_chunks_for_groups(search_chunk_query_results, &data, pool.clone()).await?;

    //TODO: rerank for groups

    Ok(result_chunks)
}

pub async fn full_text_search_over_groups(
    data: web::Json<SearchOverGroupsData>,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchOverGroupsResponseBody, actix_web::Error> {
    let embedding_vector = get_splade_query_embedding(&data.query)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to get splade query embedding".into()))?;

    let search_chunk_query_results = retrieve_group_qdrant_points_query(
        VectorType::Sparse(embedding_vector),
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.group_size.unwrap_or(3),
        parsed_query,
        dataset.id,
        pool.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let result_chunks =
        retrieve_chunks_for_groups(search_chunk_query_results, &data, pool.clone()).await?;

    //TODO: rerank for groups

    Ok(result_chunks)
}

async fn cross_encoder_for_groups(
    query: String,
    page_size: u64,
    groups_chunks: Vec<GroupScoreChunkDTO>,
) -> Result<Vec<GroupScoreChunkDTO>, actix_web::Error> {
    let score_chunks = groups_chunks
        .iter()
        .flat_map(|group| group.metadata.clone().into_iter().collect_vec())
        .collect_vec();
    let cross_encoder_results = cross_encoder(query, page_size, score_chunks).await?;
    let mut group_results = cross_encoder_results
        .into_iter()
        .map(|score_chunk| {
            let group = groups_chunks
                .iter()
                .find(|group| {
                    group
                        .metadata
                        .iter()
                        .any(|chunk| chunk.metadata[0].id == score_chunk.metadata[0].id)
                })
                .expect("Group not found");
            group.clone()
        })
        .collect_vec();
    group_results.dedup_by(|a, b| a.group_id == b.group_id);
    Ok(group_results)
}

pub async fn hybrid_search_over_groups(
    data: web::Json<SearchOverGroupsData>,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
) -> Result<SearchOverGroupsResponseBody, actix_web::Error> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());
    let dense_embedding_vector = create_embedding(&data.query, dataset_config.clone()).await?;
    let sparse_embedding_vector = get_splade_query_embedding(&data.query)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to get splade query embedding".into()))?;

    let semantic_future = retrieve_group_qdrant_points_query(
        VectorType::Dense(dense_embedding_vector),
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.group_size.unwrap_or(3),
        parsed_query.clone(),
        dataset.id,
        pool.clone(),
    );

    let full_text_future = retrieve_group_qdrant_points_query(
        VectorType::Sparse(sparse_embedding_vector),
        page,
        data.link.clone(),
        data.tag_set.clone(),
        data.time_range.clone(),
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.group_size.unwrap_or(3),
        parsed_query.clone(),
        dataset.id,
        pool.clone(),
    );

    let (semantic_results, full_text_results) = futures::join!(semantic_future, full_text_future);

    let semantic_results =
        semantic_results.map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let full_text_results =
        full_text_results.map_err(|err| ServiceError::BadRequest(err.to_string()))?;

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
        total_chunk_pages: semantic_results.total_chunk_pages,
    };

    let combined_result_chunks = retrieve_chunks_for_groups(
        combined_search_chunk_query_results.clone(),
        &data,
        pool.clone(),
    )
    .await?;

    let reranked_chunks = if combined_result_chunks.group_chunks.len() > 20 {
        let split_results = combined_result_chunks
            .group_chunks
            .chunks(20)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<Vec<GroupScoreChunkDTO>>>();

        let cross_encoder_results = cross_encoder_for_groups(
            data.query.clone(),
            data.page_size.unwrap_or(10).into(),
            split_results
                .first()
                .expect("Split results must exist")
                .to_vec(),
        )
        .await?;

        cross_encoder_results
            .iter()
            .chain(split_results.get(1).unwrap().iter())
            .cloned()
            .collect::<Vec<GroupScoreChunkDTO>>()
    } else {
        cross_encoder_for_groups(
            data.query.clone(),
            data.page_size.unwrap_or(10).into(),
            combined_result_chunks.group_chunks.clone(),
        )
        .await?
    };

    let result_chunks = SearchOverGroupsResponseBody {
        group_chunks: reranked_chunks,
        total_chunk_pages: combined_search_chunk_query_results.total_chunk_pages,
    };

    //TODO: rerank for groups

    Ok(result_chunks)
}

pub fn get_qdrant_point_ids_from_pg_for_quote_negated_words(
    quote_words: Option<Vec<String>>,
    negated_words: Option<Vec<String>>,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<uuid::Uuid>, DefaultError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    let mut conn = pool.get().unwrap();
    let mut query = chunk_metadata_columns::chunk_metadata
        .select(chunk_metadata_columns::qdrant_point_id)
        .filter(chunk_metadata_columns::qdrant_point_id.is_not_null())
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
        .into_boxed();

    if let Some(quote_words) = quote_words {
        for word in quote_words.iter() {
            let word_without_quotes = word.trim_matches('\"');
            query = query.filter(
                chunk_metadata_columns::chunk_html.ilike(format!("%{}%", word_without_quotes)),
            );
        }
    }

    if let Some(negated_words) = negated_words {
        for word in negated_words.iter() {
            let word_without_negation = word.trim_matches('-');
            query = query.filter(
                chunk_metadata_columns::chunk_html
                    .not_ilike(format!("%{}%", word_without_negation)),
            );
        }
    }

    let matching_qdrant_point_ids: Vec<Option<uuid::Uuid>> =
        query.load(&mut conn).map_err(|_| DefaultError {
            message: "Failed to load full-text searched chunks",
        })?;

    let matching_qdrant_point_ids = matching_qdrant_point_ids
        .into_iter()
        .flatten()
        .collect::<Vec<uuid::Uuid>>();

    Ok(matching_qdrant_point_ids)
}
