use super::chunk_operator::{
    find_relevant_sentence, get_metadata_and_collided_chunks_from_point_ids_query,
    get_metadata_from_point_ids,
};
use super::group_operator::{
    get_group_tracking_ids_from_group_ids_query, get_groups_from_tracking_ids_query,
};
use super::model_operator::{create_embeddings, cross_encoder};
use super::qdrant_operator::{
    get_point_count_qdrant_query, search_over_groups_query, GroupSearchResults, VectorType,
};
use crate::data::models::{
    ChunkFileWithName, ChunkGroup, ChunkMetadataWithFileData, Dataset, FullTextSearchResult,
    ServerDatasetConfiguration,
};
use crate::errors::ServiceError;
use crate::handlers::chunk_handler::{
    ChunkFilter, FieldCondition, MatchCondition, ParsedQuery, ScoreChunkDTO, SearchChunkData,
    SearchChunkQueryResponseBody,
};
use crate::handlers::group_handler::{
    SearchGroupsResult, SearchOverGroupsData, SearchWithinGroupData,
};
use crate::operators::model_operator::get_splade_embedding;
use crate::operators::qdrant_operator::{get_qdrant_connection, search_qdrant_query};
use crate::{data::models::Pool, errors::DefaultError};
use actix_web::web;
use itertools::Itertools;
use simple_server_timing_header::Timer;
use utoipa::ToSchema;

use qdrant_client::qdrant::condition::ConditionOneOf::HasId;
use qdrant_client::qdrant::Filter;
use qdrant_client::qdrant::{
    point_id::PointIdOptions, Condition, HasIdCondition, PointId, SearchPoints,
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

async fn convert_group_tracking_ids_to_group_ids(
    condition: FieldCondition,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<FieldCondition, DefaultError> {
    if condition.field == "group_tracking_ids" {
        let matches = condition
            .r#match
            .ok_or(DefaultError {
                message: "match key not found for group_tracking_ids",
            })?
            .iter()
            .map(|item| item.to_string())
            .collect();

        let correct_matches: Vec<MatchCondition> =
            get_groups_from_tracking_ids_query(matches, dataset_id, pool.clone())
                .await?
                .iter()
                .map(|ids| MatchCondition::Text(ids.to_string()))
                .collect();

        Ok(FieldCondition {
            field: "group_ids".to_string(),
            r#match: Some(correct_matches),
            range: None,
        })
    } else {
        Ok(condition)
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
) -> Result<Filter, DefaultError> {
    let mut filter = Filter::default();

    filter
        .must
        .push(Condition::matches("dataset_id", dataset_id.to_string()));
    //TODO: fix this after new qdrant rust client gets released

    if let Some(filters) = filters {
        if let Some(should_filters) = filters.should {
            for should_condition in should_filters {
                let should_condition = convert_group_tracking_ids_to_group_ids(
                    should_condition,
                    dataset_id,
                    pool.clone(),
                )
                .await?;

                let qdrant_condition = should_condition.convert_to_qdrant_condition()?;

                if let Some(condition) = qdrant_condition {
                    filter.should.push(condition.into());
                }
            }
        }

        if let Some(must_filters) = filters.must {
            for must_condition in must_filters {
                let must_condition = convert_group_tracking_ids_to_group_ids(
                    must_condition,
                    dataset_id,
                    pool.clone(),
                )
                .await?;

                let qdrant_condition = must_condition.convert_to_qdrant_condition()?;

                if let Some(condition) = qdrant_condition {
                    filter.must.push(condition.into());
                }
            }
        }

        if let Some(must_not_filters) = filters.must_not {
            for must_not_condition in must_not_filters {
                let must_not_condition = convert_group_tracking_ids_to_group_ids(
                    must_not_condition,
                    dataset_id,
                    pool.clone(),
                )
                .await?;

                let qdrant_condition = must_not_condition.convert_to_qdrant_condition()?;

                if let Some(condition) = qdrant_condition {
                    filter.must_not.push(condition.into());
                }
            }
        }
    };

    if quote_words.is_some() || negated_words.is_some() {
        let available_qdrant_ids = get_qdrant_point_ids_from_pg_for_quote_negated_words(
            quote_words,
            negated_words,
            dataset_id,
            pool,
        )
        .await?;

        let available_point_ids = available_qdrant_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<HashSet<String>>()
            .iter()
            .map(|id| (*id).clone().into())
            .collect::<Vec<PointId>>();

        filter.must.push(Condition {
            condition_one_of: Some(HasId(HasIdCondition {
                has_id: (available_point_ids).to_vec(),
            })),
        });
    }

    Ok(filter)
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool))]
pub async fn retrieve_qdrant_points_query(
    vector: VectorType,
    page: u64,
    limit: u64,
    score_threshold: Option<f32>,
    filters: Option<ChunkFilter>,
    parsed_query: ParsedQuery,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
    config: ServerDatasetConfiguration,
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
        filters,
        parsed_query.quote_words,
        parsed_query.negated_words,
        dataset_id,
        pool,
    )
    .await?;

    let point_ids_future = search_qdrant_query(
        page,
        filter.clone(),
        limit,
        score_threshold,
        vector,
        config.clone(),
    );

    let count_future = get_point_count_qdrant_query(filter, config);

    let (point_ids, count) = futures::join!(point_ids_future, count_future);

    let pages = (count.map_err(|e| {
        log::error!("Failed to get search count from Qdrant {:?}", e);
        DefaultError {
            message: "Failed to get point count from Qdrant",
        }
    })? as f64
        / limit as f64)
        .ceil() as i64;

    Ok(SearchChunkQueryResult {
        search_results: point_ids.map_err(|e| {
            log::error!("Failed to get points from Qdrant {:?}", e);
            DefaultError {
                message: "Failed to get points from Qdrant",
            }
        })?,
        total_chunk_pages: pages,
    })
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchOverGroupsQueryResult {
    pub search_results: Vec<GroupSearchResults>,
    pub total_chunk_pages: i64,
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool))]
pub async fn retrieve_group_qdrant_points_query(
    vector: VectorType,
    page: u64,
    filters: Option<ChunkFilter>,
    limit: u32,
    score_threshold: Option<f32>,
    group_size: u32,
    parsed_query: ParsedQuery,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
    config: ServerDatasetConfiguration,
) -> Result<SearchOverGroupsQueryResult, DefaultError> {
    let page = if page == 0 { 1 } else { page };

    let filter = assemble_qdrant_filter(
        filters,
        parsed_query.quote_words,
        parsed_query.negated_words,
        dataset_id,
        pool,
    )
    .await?;

    let point_id_future = search_over_groups_query(
        page,
        filter.clone(),
        limit,
        score_threshold,
        group_size,
        vector,
        config.clone(),
    );

    let count_future = get_point_count_qdrant_query(filter, config);

    let (point_ids, count) = futures::join!(point_id_future, count_future);

    let pages = (count.map_err(|e| {
        log::error!("Failed to get point count from Qdrant {:?}", e);
        DefaultError {
            message: "Failed to get point count from Qdrant",
        }
    })? as f64
        / limit as f64)
        .ceil() as i64;

    Ok(SearchOverGroupsQueryResult {
        search_results: point_ids.map_err(|e| {
            log::error!("Failed to get point count from Qdrant {:?}", e);
            DefaultError {
                message: "Failed to get point count from Qdrant",
            }
        })?,
        total_chunk_pages: pages,
    })
}

#[tracing::instrument(skip(embedding_vector))]
pub async fn global_unfiltered_top_match_query(
    embedding_vector: Vec<f32>,
    dataset_id: uuid::Uuid,
    config: ServerDatasetConfiguration,
) -> Result<SearchResult, DefaultError> {
    let qdrant_collection = config.QDRANT_COLLECTION_NAME;

    let qdrant =
        get_qdrant_connection(Some(&config.QDRANT_URL), Some(&config.QDRANT_API_KEY)).await?;

    let mut dataset_filter = Filter::default();
    dataset_filter
        .must
        .push(Condition::matches("dataset_id", dataset_id.to_string()));

    let vector_name = match embedding_vector.len() {
        384 => "384_vectors",
        512 => "512_vectors",
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
            collection_name: qdrant_collection,
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
#[tracing::instrument(skip(pool))]
pub async fn search_within_chunk_group_query(
    embedding_vector: VectorType,
    page: u64,
    pool: web::Data<Pool>,
    filters: Option<ChunkFilter>,
    limit: u64,
    score_threshold: Option<f32>,
    group_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    parsed_query: ParsedQuery,
    config: ServerDatasetConfiguration,
) -> Result<SearchChunkQueryResult, DefaultError> {
    let page = if page == 0 { 1 } else { page };
    let mut filter = assemble_qdrant_filter(
        filters,
        parsed_query.quote_words,
        parsed_query.negated_words,
        dataset_id,
        pool,
    )
    .await?;

    filter
        .must
        .push(Condition::matches("group_ids", group_id.to_string()));

    let point_ids_future = search_qdrant_query(
        page,
        filter.clone(),
        limit,
        score_threshold,
        embedding_vector,
        config.clone(),
    );

    let count_future = get_point_count_qdrant_query(filter, config);

    let (point_ids, count) = futures::join!(point_ids_future, count_future);

    let pages = (count.map_err(|e| {
        log::error!("Failed to get point count from Qdrant {:?}", e);
        DefaultError {
            message: "Failed to get point count from Qdrant",
        }
    })? as f64
        / limit as f64)
        .ceil() as i64;

    Ok(SearchChunkQueryResult {
        search_results: point_ids.map_err(|e| {
            log::error!("Failed to get point count from Qdrant {:?}", e);
            DefaultError {
                message: "Failed to get point count from Qdrant",
            }
        })?,
        total_chunk_pages: pages,
    })
}

#[tracing::instrument(skip(pool))]
pub async fn get_metadata_query(
    chunk_metadata: Vec<FullTextSearchResult>,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkMetadataWithFileData>, DefaultError> {
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::files::dsl as files_columns;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let mut conn = pool.get().await.expect("DB connection");

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
        .await
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

#[tracing::instrument(skip(pool))]
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
        .await
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
                None => {
                    log::error!(
                        "Failed to find metadata chunk for point id without collisions: {:?}",
                        search_result.point_id
                    );
                    sentry::capture_message(
                        &format!(
                            "Failed to find metadata chunk for point id without collisions {:?}",
                            search_result.point_id
                        ),
                        sentry::Level::Error,
                    );

                    ChunkMetadataWithFileData {
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
                    }
                }
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
#[schema(example = json!({
    "group_id": "e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "metadata": [
        {
            "metadata": [
                {
                    "id": "e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
                    "content": "This is a test content",
                    "link": "https://www.google.com",
                    "tag_set": "test",
                    "metadata": {
                        "key": "value"
                    },
                    "tracking_id": "e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
                    "time_stamp": "2021-01-01T00:00:00Z",
                    "weight": 1.0
                }
            ],
            "score": 0.5
        }
    ]
}))]
pub struct GroupScoreChunkDTO {
    pub group_id: uuid::Uuid,
    pub group_tracking_id: Option<String>,
    pub metadata: Vec<ScoreChunkDTO>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SearchOverGroupsResponseBody {
    pub group_chunks: Vec<GroupScoreChunkDTO>,
    pub total_chunk_pages: i64,
}

#[tracing::instrument(skip(pool))]
pub async fn retrieve_chunks_for_groups(
    search_over_groups_query_result: SearchOverGroupsQueryResult,
    data: &SearchOverGroupsData,
    pool: web::Data<Pool>,
) -> Result<SearchOverGroupsResponseBody, ServiceError> {
    let point_ids = search_over_groups_query_result
        .search_results
        .clone()
        .iter()
        .flat_map(|hit| hit.hits.iter().map(|point| point.point_id).collect_vec())
        .collect_vec();

    let (metadata_chunks, collided_chunks) = get_metadata_and_collided_chunks_from_point_ids_query(
        point_ids,
        data.get_collisions.unwrap_or(false),
        pool.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let group_tracking_ids = get_group_tracking_ids_from_group_ids_query(
        search_over_groups_query_result
            .search_results
            .iter()
            .map(|group| group.group_id)
            .collect(),
        pool,
    )
    .await?;

    let group_chunks: Vec<GroupScoreChunkDTO> = search_over_groups_query_result
        .search_results
        .iter()
        .enumerate()
        .map(|(i, group)| {
            let score_chunk: Vec<ScoreChunkDTO> = group
                .hits
                .iter()
                .map(|search_result| {
                    let mut chunk: ChunkMetadataWithFileData =
                        match metadata_chunks.iter().find(|metadata_chunk| {
                            metadata_chunk.qdrant_point_id == search_result.point_id
                        }) {
                            Some(metadata_chunk) => metadata_chunk.clone(),
                            None => {
                                log::error!(
                                    "Failed to find metadata chunk for point id for chunks with groups: {:?}",
                                    search_result.point_id
                                );
                                sentry::capture_message(
                                    &format!("Failed to find metadata chunk for point id for chunks with groups: {:?}", search_result.point_id),
                                    sentry::Level::Error,
                                );

                                ChunkMetadataWithFileData {
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
                            }},
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

            let group_tracking_id = group_tracking_ids.get(i).map(|x| x.clone()).flatten();

            GroupScoreChunkDTO {
                group_id: group.group_id,
                group_tracking_id,
                metadata: score_chunk,
            }
        })
        .collect_vec();

    Ok(SearchOverGroupsResponseBody {
        group_chunks,
        total_chunk_pages: search_over_groups_query_result.total_chunk_pages,
    })
}

pub async fn get_metadata_from_groups(
    search_over_groups_query_result: SearchOverGroupsQueryResult,
    get_collisions: Option<bool>,
    pool: web::Data<Pool>,
) -> Result<Vec<GroupScoreChunkDTO>, actix_web::Error> {
    let point_ids = search_over_groups_query_result
        .search_results
        .iter()
        .flat_map(|hit| hit.hits.iter().map(|point| point.point_id).collect_vec())
        .collect_vec();

    let (metadata_chunks, collided_chunks) = get_metadata_and_collided_chunks_from_point_ids_query(
        point_ids,
        get_collisions.unwrap_or(false),
        pool.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let group_tracking_ids = get_group_tracking_ids_from_group_ids_query(
        search_over_groups_query_result
            .search_results
            .iter()
            .map(|group| group.group_id)
            .collect(),
        pool,
    )
    .await?;

    let group_chunks: Vec<GroupScoreChunkDTO> = search_over_groups_query_result
        .search_results
        .iter()
        .enumerate()
        .map(|(i, group)| {
            let score_chunk: Vec<ScoreChunkDTO> = group
                .hits
                .iter()
                .map(|search_result| {
                    let chunk: ChunkMetadataWithFileData =
                        match metadata_chunks.iter().find(|metadata_chunk| {
                            metadata_chunk.qdrant_point_id == search_result.point_id
                        }) {
                            Some(metadata_chunk) => metadata_chunk.clone(),
                            None => {
                                log::error!(
                                    "Failed to find metadata chunk for point id for metadata with groups: {:?}",
                                    search_result.point_id
                                );
                                sentry::capture_message(
                                    &format!("Failed to find metadata chunk for point id for metadata with groups: {:?}", search_result.point_id),
                                    sentry::Level::Error,
                                );

                                ChunkMetadataWithFileData {
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
                            }},
                        };

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

            let group_tracking_id = group_tracking_ids.get(i).map(|x| x.clone()).flatten();

            GroupScoreChunkDTO {
                group_id: group.group_id,
                group_tracking_id,
                metadata: score_chunk,
            }
        })
        .collect_vec();

    Ok(group_chunks)
}

/// Retrieve chunks from point ids, DOES NOT GUARD AGAINST DATASET ACCESS PERMISSIONS
#[tracing::instrument(skip(pool))]
pub async fn retrieve_chunks_from_point_ids(
    search_chunk_query_results: SearchChunkQueryResult,
    data: &SearchChunkData,
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
    .await
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
                None => {
                    log::error!(
                        "Failed to find metadata chunk from point ids: {:?}",
                        search_result.point_id
                    );
                    sentry::capture_message(
                        &format!(
                            "Failed to find metadata chunk from point ids: {:?}",
                            search_result.point_id
                        ),
                        sentry::Level::Error,
                    );

                    ChunkMetadataWithFileData {
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
                    }
                }
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

#[tracing::instrument]
pub fn rerank_chunks(
    chunks: Vec<ScoreChunkDTO>,
    date_bias: Option<bool>,
    use_weights: Option<bool>,
) -> Vec<ScoreChunkDTO> {
    let mut reranked_chunks = Vec::new();
    if use_weights.unwrap_or(true) {
        chunks.into_iter().for_each(|mut chunk| {
            if chunk.metadata[0].weight == 0.0 {
                chunk.metadata[0].weight = 1.0;
            }
            chunk.score *= chunk.metadata[0].weight;
            reranked_chunks.push(chunk);
        });
    } else {
        reranked_chunks = chunks;
    }

    if date_bias.is_some() && date_bias.unwrap() {
        reranked_chunks.sort_by(|a, b| {
            if let (Some(time_stamp_a), Some(time_stamp_b)) =
                (a.metadata[0].time_stamp, b.metadata[0].time_stamp)
            {
                return time_stamp_b.timestamp().cmp(&time_stamp_a.timestamp());
            }
            a.score.total_cmp(&b.score)
        });
    } else {
        reranked_chunks.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    reranked_chunks
}

#[tracing::instrument(skip(timer, pool))]
pub async fn search_semantic_chunks(
    data: SearchChunkData,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
    timer: &mut Timer,
    config: ServerDatasetConfiguration,
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

    let embedding_vectors =
        create_embeddings(vec![data.query.clone()], "query", dataset_config.clone()).await?;
    let embedding_vector = embedding_vectors
        .first()
        .ok_or(ServiceError::BadRequest(
            "Failed to get embedding vector due to empty vec response from create_embedding"
                .to_string(),
        ))?
        .clone();

    timer.add("Created Embedding vector");

    let search_chunk_query_results = retrieve_qdrant_points_query(
        VectorType::Dense(embedding_vector),
        page,
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.filters.clone(),
        parsed_query,
        dataset.id,
        pool.clone(),
        config,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    timer.add("Fetch from qdrant");

    let mut result_chunks =
        retrieve_chunks_from_point_ids(search_chunk_query_results, &data, pool.clone()).await?;

    timer.add("Fetch from postgres");

    result_chunks.score_chunks =
        rerank_chunks(result_chunks.score_chunks, data.date_bias, data.use_weights);
    timer.add("Rerank (algo)");
    transaction.finish();

    Ok(result_chunks)
}

#[tracing::instrument(skip(pool))]
pub async fn search_full_text_chunks(
    data: SearchChunkData,
    mut parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
    config: ServerDatasetConfiguration,
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

    let embedding_vector = get_splade_embedding(&parsed_query.query, "query")
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to get splade query embedding".into()))?;

    let search_chunk_query_results = retrieve_qdrant_points_query(
        VectorType::Sparse(embedding_vector),
        page,
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.filters.clone(),
        parsed_query,
        dataset.id,
        pool.clone(),
        config,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let mut result_chunks =
        retrieve_chunks_from_point_ids(search_chunk_query_results, &data, pool).await?;

    result_chunks.score_chunks =
        rerank_chunks(result_chunks.score_chunks, data.date_bias, data.use_weights);

    if data.slim_chunks.unwrap_or(false) {
        result_chunks.score_chunks = result_chunks
            .score_chunks
            .into_iter()
            .map(|score_chunk| ScoreChunkDTO {
                metadata: vec![score_chunk.metadata.first().unwrap().clone()],
                score: score_chunk.score,
            })
            .collect();
    }

    transaction.finish();
    Ok(result_chunks)
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool))]
pub async fn search_hybrid_chunks(
    data: SearchChunkData,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
    config: ServerDatasetConfiguration,
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

    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());

    let embedding_vectors =
        create_embeddings(vec![data.query.clone()], "query", dataset_config.clone()).await?;
    let embedding_vector = embedding_vectors
        .first()
        .ok_or(ServiceError::BadRequest(
            "Failed to get embedding vector due to empty vec response from create_embedding"
                .to_string(),
        ))?
        .clone();

    let search_chunk_query_results = retrieve_qdrant_points_query(
        VectorType::Dense(embedding_vector),
        page,
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.filters.clone(),
        parsed_query.clone(),
        dataset.id,
        pool.clone(),
        config.clone(),
    );

    let full_text_handler_results = search_full_text_chunks(
        data.clone(),
        parsed_query,
        page,
        pool.clone(),
        dataset,
        config,
    );

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
        pool,
    )
    .await
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
                None => {
                    log::error!(
                        "Failed to find metadata chunk for point id for hybrid chunks: {:?}",
                        search_result.point_id
                    );
                    sentry::capture_message(
                        &format!(
                            "Failed to find metadata for point id for hybrid chunk: {:?}",
                            search_result.point_id
                        ),
                        sentry::Level::Error,
                    );

                    ChunkMetadataWithFileData {
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
                    }
                }
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

    let result_chunks = {
        let combined_results = semantic_score_chunks
            .iter()
            .zip(full_text_handler_results.score_chunks.iter())
            .flat_map(|(x, y)| vec![x.clone(), y.clone()])
            .unique_by(|score_chunk| score_chunk.metadata[0].id)
            .collect::<Vec<ScoreChunkDTO>>();

        let mut reranked_chunks = if combined_results.len() > 20 {
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

            let score_chunks =
                rerank_chunks(cross_encoder_results, data.date_bias, data.use_weights);

            score_chunks
                .iter()
                .chain(split_results.get(1).unwrap().iter())
                .cloned()
                .collect::<Vec<ScoreChunkDTO>>()
        } else {
            let cross_encoder_results = cross_encoder(
                data.query.clone(),
                data.page_size.unwrap_or(10),
                combined_results,
            )
            .await?;

            rerank_chunks(cross_encoder_results, data.date_bias, data.use_weights)
        };

        reranked_chunks.truncate(data.page_size.unwrap_or(10) as usize);

        SearchChunkQueryResponseBody {
            score_chunks: reranked_chunks,
            total_chunk_pages: search_chunk_query_results.total_chunk_pages,
        }
    };

    transaction.finish();
    Ok(result_chunks)
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool))]
pub async fn search_semantic_groups(
    data: SearchWithinGroupData,
    parsed_query: ParsedQuery,
    group: ChunkGroup,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
    config: ServerDatasetConfiguration,
) -> Result<SearchGroupsResult, actix_web::Error> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());

    let embedding_vectors =
        create_embeddings(vec![data.query.clone()], "query", dataset_config.clone()).await?;
    let embedding_vector = embedding_vectors
        .first()
        .ok_or(ServiceError::BadRequest(
            "Failed to get embedding vector due to empty vec response from create_embedding"
                .to_string(),
        ))?
        .clone();

    let search_semantic_chunk_query_results = search_within_chunk_group_query(
        VectorType::Dense(embedding_vector),
        page,
        pool.clone(),
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        group.id,
        dataset.id,
        parsed_query,
        config,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let mut result_chunks = retrieve_chunks_from_point_ids_without_collsions(
        search_semantic_chunk_query_results,
        &web::Json(data.clone().into()),
        pool.clone(),
    )
    .await?;

    result_chunks.score_chunks =
        rerank_chunks(result_chunks.score_chunks, data.date_bias, data.use_weights);

    Ok(SearchGroupsResult {
        bookmarks: result_chunks.score_chunks,
        group,
        total_pages: result_chunks.total_chunk_pages,
    })
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool))]
pub async fn search_full_text_groups(
    data: SearchWithinGroupData,
    parsed_query: ParsedQuery,
    group: ChunkGroup,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
    config: ServerDatasetConfiguration,
) -> Result<SearchGroupsResult, actix_web::Error> {
    let data_inner = data.clone();
    let embedding_vector = get_splade_embedding(&data.query, "query").await?;

    let search_chunk_query_results = search_within_chunk_group_query(
        VectorType::Sparse(embedding_vector),
        page,
        pool.clone(),
        data_inner.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        group.id,
        dataset.id,
        parsed_query,
        config,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let mut result_chunks = retrieve_chunks_from_point_ids_without_collsions(
        search_chunk_query_results,
        &web::Json(data.clone().into()),
        pool.clone(),
    )
    .await?;

    result_chunks.score_chunks =
        rerank_chunks(result_chunks.score_chunks, data.date_bias, data.use_weights);

    Ok(SearchGroupsResult {
        bookmarks: result_chunks.score_chunks,
        group,
        total_pages: result_chunks.total_chunk_pages,
    })
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool))]
pub async fn search_hybrid_groups(
    data: SearchWithinGroupData,
    parsed_query: ParsedQuery,
    group: ChunkGroup,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
    config: ServerDatasetConfiguration,
) -> Result<SearchGroupsResult, actix_web::Error> {
    let data_inner = data.clone();
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());

    let dense_embedding_vectors =
        create_embeddings(vec![data.query.clone()], "query", dataset_config.clone()).await?;
    let dense_embedding_vector = dense_embedding_vectors
        .first()
        .ok_or(ServiceError::BadRequest(
            "Failed to get embedding vector due to empty vec response from create_embedding"
                .to_string(),
        ))?
        .clone();

    let sparse_embedding_vector = get_splade_embedding(&data.query, "query").await?;

    let semantic_future = search_within_chunk_group_query(
        VectorType::Dense(dense_embedding_vector),
        page,
        pool.clone(),
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        group.id,
        dataset.id,
        parsed_query.clone(),
        config.clone(),
    );

    let full_text_future = search_within_chunk_group_query(
        VectorType::Sparse(sparse_embedding_vector),
        page,
        pool.clone(),
        data_inner.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        group.id,
        dataset.id,
        parsed_query.clone(),
        config,
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

    let result_chunks = {
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
            let score_chunks =
                rerank_chunks(cross_encoder_results, data.date_bias, data.use_weights);

            score_chunks
                .iter()
                .chain(split_results.get(1).unwrap().iter())
                .cloned()
                .collect::<Vec<ScoreChunkDTO>>()
        } else {
            let cross_encoder_results = cross_encoder(
                data.query.clone(),
                data.page_size.unwrap_or(10),
                combined_result_chunks.score_chunks.clone(),
            )
            .await?;

            rerank_chunks(cross_encoder_results, data.date_bias, data.use_weights)
        };

        SearchChunkQueryResponseBody {
            score_chunks: reranked_chunks,
            total_chunk_pages: combined_result_chunks.total_chunk_pages,
        }
    };

    Ok(SearchGroupsResult {
        bookmarks: result_chunks.score_chunks,
        group,
        total_pages: combined_result_chunks.total_chunk_pages,
    })
}

#[tracing::instrument(skip(pool))]
pub async fn semantic_search_over_groups(
    data: SearchOverGroupsData,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
    config: ServerDatasetConfiguration,
) -> Result<SearchOverGroupsResponseBody, actix_web::Error> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());
    let embedding_vectors =
        create_embeddings(vec![data.query.clone()], "query", dataset_config.clone()).await?;
    let embedding_vector = embedding_vectors
        .first()
        .ok_or(ServiceError::BadRequest(
            "Failed to get embedding vector due to empty array from create_embedding".to_string(),
        ))?
        .clone();

    let search_chunk_query_results = retrieve_group_qdrant_points_query(
        VectorType::Dense(embedding_vector),
        page,
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.group_size.unwrap_or(3),
        parsed_query,
        dataset.id,
        pool.clone(),
        config,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let result_chunks =
        retrieve_chunks_for_groups(search_chunk_query_results, &data, pool.clone()).await?;

    //TODO: rerank for groups

    Ok(result_chunks)
}

#[tracing::instrument(skip(pool))]
pub async fn full_text_search_over_groups(
    data: SearchOverGroupsData,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
    config: ServerDatasetConfiguration,
) -> Result<SearchOverGroupsResponseBody, actix_web::Error> {
    let embedding_vector = get_splade_embedding(&data.query, "query")
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to get splade query embedding".into()))?;

    let search_chunk_query_results = retrieve_group_qdrant_points_query(
        VectorType::Sparse(embedding_vector),
        page,
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.group_size.unwrap_or(3),
        parsed_query,
        dataset.id,
        pool.clone(),
        config,
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
        .map(|group| {
            group
                .metadata
                .clone()
                .first()
                .expect("Metadata should have one element")
                .clone()
        })
        .collect_vec();

    let cross_encoder_results = cross_encoder(query, page_size, score_chunks).await?;
    let mut group_results = cross_encoder_results
        .into_iter()
        .map(|score_chunk| {
            let mut group = groups_chunks
                .iter()
                .find(|group| {
                    group
                        .metadata
                        .iter()
                        .any(|chunk| chunk.metadata[0].id == score_chunk.metadata[0].id)
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

    Ok(group_results)
}

#[tracing::instrument(skip(pool))]
pub async fn hybrid_search_over_groups(
    data: SearchOverGroupsData,
    parsed_query: ParsedQuery,
    page: u64,
    pool: web::Data<Pool>,
    dataset: Dataset,
    config: ServerDatasetConfiguration,
) -> Result<SearchOverGroupsResponseBody, actix_web::Error> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());

    let dense_embedding_vectors =
        create_embeddings(vec![data.query.clone()], "query", dataset_config.clone()).await?;
    let dense_embedding_vector = dense_embedding_vectors
        .first()
        .ok_or(ServiceError::BadRequest(
            "Failed to get embedding vector due to empty array from create_embedding".to_string(),
        ))?
        .clone();

    let sparse_embedding_vector = get_splade_embedding(&data.query, "query")
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to get splade query embedding".into()))?;

    let semantic_future = retrieve_group_qdrant_points_query(
        VectorType::Dense(dense_embedding_vector),
        page,
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.group_size.unwrap_or(3),
        parsed_query.clone(),
        dataset.id,
        pool.clone(),
        config.clone(),
    );

    let full_text_future = retrieve_group_qdrant_points_query(
        VectorType::Sparse(sparse_embedding_vector),
        page,
        data.filters.clone(),
        data.page_size.unwrap_or(10),
        data.score_threshold,
        data.group_size.unwrap_or(3),
        parsed_query.clone(),
        dataset.id,
        pool.clone(),
        config,
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

#[tracing::instrument(skip(pool))]
pub async fn get_qdrant_point_ids_from_pg_for_quote_negated_words(
    quote_words: Option<Vec<String>>,
    negated_words: Option<Vec<String>>,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<uuid::Uuid>, DefaultError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let mut conn = pool.get().await.unwrap();
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
        query.load(&mut conn).await.map_err(|_| DefaultError {
            message: "Failed to load full-text searched chunks",
        })?;

    let matching_qdrant_point_ids = matching_qdrant_point_ids
        .into_iter()
        .flatten()
        .collect::<Vec<uuid::Uuid>>();

    Ok(matching_qdrant_point_ids)
}
