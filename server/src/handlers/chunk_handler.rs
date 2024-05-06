use super::auth_handler::{AdminOnly, LoggedUser};
use crate::data::models::{
    ChatMessageProxy, ChunkMetadata, ChunkMetadataWithFileData, DatasetAndOrgWithSubAndPlan,
    IngestSpecificChunkMetadata, Pool, RedisPool, ScoreSlimChunks,
    SearchSlimChunkQueryResponseBody, ServerDatasetConfiguration, SlimChunkMetadata, UnifiedId,
};
use crate::errors::ServiceError;
use crate::get_env;
use crate::operators::chunk_operator::get_metadata_from_id_query;
use crate::operators::chunk_operator::*;
use crate::operators::group_operator::{
    check_group_ids_exist_query, get_groups_from_tracking_ids_query,
};
use crate::operators::qdrant_operator::recommend_qdrant_query;
use crate::operators::search_operator::{
    get_metadata_filter_condition, search_full_text_chunks, search_hybrid_chunks,
    search_semantic_chunks,
};
use actix_web::web::Bytes;
use actix_web::{web, HttpResponse};
use chrono::NaiveDateTime;
use dateparser::DateTimeUtc;
use itertools::Itertools;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::chat::{
    ChatCompletionParameters, ChatMessage, ChatMessageContent, Role,
};
use qdrant_client::qdrant;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use simple_server_timing_header::Timer;
use tokio_stream::StreamExt;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
#[schema(example = json!({
    "chunk_html": "<p>Some HTML content</p>",
    "link": "https://example.com",
    "tag_set": ["tag1", "tag2"],
    "file_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
    "metadata": {"key1": "value1", "key2": "value2"},
    "chunk_vector": [0.1, 0.2, 0.3],
    "tracking_id": "tracking_id",
    "upsert_by_tracking_id": true,
    "group_ids": ["d290f1ee-6c54-4b01-90e6-d701748f0851"],
    "group_tracking_ids": ["group_tracking_id"],
    "time_stamp": "2021-01-01T00:00:00",
    "weight": 0.5,
    "split_avg": false
}))]
pub struct ChunkData {
    /// HTML content of the chunk. This can also be plaintext. The innerText of the HTML will be used to create the embedding vector. The point of using HTML is for convienience, as some users have applications where users submit HTML content.
    pub chunk_html: Option<String>,
    /// Link to the chunk. This can also be any string. Frequently, this is a link to the source of the chunk. The link value will not affect the embedding creation.
    pub link: Option<String>,
    /// Tag set is a list of tags. This can be used to filter chunks by tag. Unlike with metadata filtering, HNSW indices will exist for each tag such that there is not a performance hit for filtering on them.
    pub tag_set: Option<Vec<String>>,
    /// File_uuid is the uuid of the file that the chunk is associated with. This is used to associate chunks with files. This is useful for when you want to delete a file and all of its associated chunks.
    pub file_id: Option<uuid::Uuid>,
    /// Metadata is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub metadata: Option<serde_json::Value>,
    /// Chunk_vector is a vector of floats which can be used instead of generating a new embedding. This is useful for when you are using a pre-embedded dataset. If this is not provided, the innerText of the chunk_html will be used to create the embedding.
    pub chunk_vector: Option<Vec<f32>>,
    /// Tracking_id is a string which can be used to identify a chunk. This is useful for when you are coordinating with an external system and want to use the tracking_id to identify the chunk.
    pub tracking_id: Option<String>,
    /// Upsert when a chunk with the same tracking_id exists. By default this is false, and the request will fail if a chunk with the same tracking_id exists. If this is true, the chunk will be updated if a chunk with the same tracking_id exists.
    pub upsert_by_tracking_id: Option<bool>,
    /// Group ids are the ids of the groups that the chunk should be placed into. This is useful for when you want to create a chunk and add it to a group or multiple groups in one request. Necessary because this route queues the chunk for ingestion and the chunk may not exist yet immediately after response.
    pub group_ids: Option<Vec<uuid::Uuid>>,
    /// Group tracking_ids are the tracking_ids of the groups that the chunk should be placed into. This is useful for when you want to create a chunk and add it to a group or multiple groups in one request. Necessary because this route queues the chunk for ingestion and the chunk may not exist yet immediately after response.
    pub group_tracking_ids: Option<Vec<String>>,
    /// Time_stamp should be an ISO 8601 combined date and time without timezone. It is used for time window filtering and recency-biasing search results.
    pub time_stamp: Option<String>,
    /// Weight is a float which can be used to bias search results. This is useful for when you want to bias search results for a chunk. The magnitude only matters relative to other chunks in the chunk's dataset dataset.
    pub weight: Option<f64>,
    /// Split avg is a boolean which tells the server to split the text in the chunk_html into smaller chunks and average their resulting vectors. This is useful for when you want to create a chunk from a large piece of text and want to split it into smaller chunks to create a more fuzzy average dense vector. The sparse vector will be generated normally with no averaging. By default this is false.
    pub split_avg: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct FailedChunk {
    index: usize,
    message: String,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[serde(untagged)]
pub enum ReturnQueuedChunk {
    /// All Chunks that have been queue'd with any errors filtered out
    Single(SingleQueuedChunkResponse),
    Batch(BatchQueuedChunkResponse),
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[schema(example = json!({
    "chunk_metadata": [{
        "content": "Some content",
        "link": "https://example.com",
        "tag_set": ["tag1", "tag2"],
        "file_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
        "metadata": {"key1": "value1", "key2": "value2"},
        "chunk_vector": [0.1, 0.2, 0.3],
        "tracking_id": "tracking_id",
        "time_stamp": "2021-01-01T00:00:00",
        "weight": 0.5
    }],
    "pos_in_queue": 1
}))]
pub struct SingleQueuedChunkResponse {
    /// The chunk that got queue'd
    pub chunk_metadata: ChunkMetadata,
    /// The current position the last access item is in the queue
    pub pos_in_queue: i32,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
#[schema(example = json!({
    "chunk_metadata": [{
        "content": "Some content",
        "link": "https://example.com",
        "tag_set": ["tag1", "tag2"],
        "file_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
        "metadata": {"key1": "value1", "key2": "value2"},
        "chunk_vector": [0.1, 0.2, 0.3],
        "tracking_id": "tracking_id",
        "time_stamp": "2021-01-01T00:00:00",
        "weight": 0.5
    },
    {
        "content": "Some content",
        "link": "https://example.com",
        "tag_set": ["tag1", "tag2"],
        "file_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
        "metadata": {"key1": "value1", "key2": "value2"},
        "chunk_vector": [0.1, 0.2, 0.3],
        "tracking_id": "tracking_id",
        "time_stamp": "2021-01-01T00:00:00",
        "weight": 0.5
    }],
    "pos_in_queue": 2
}))]
pub struct BatchQueuedChunkResponse {
    // All the chunks that got queue'd
    pub chunk_metadata: Vec<ChunkMetadata>,
    /// The current position the last access item is in the queue
    pub pos_in_queue: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UploadIngestionMessage {
    pub ingest_specific_chunk_metadata: IngestSpecificChunkMetadata,
    pub chunk: ChunkData,
    pub dataset_id: uuid::Uuid,
    pub dataset_config: ServerDatasetConfiguration,
    pub upsert_by_tracking_id: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "chunk_html": "<p>Some HTML content</p>",
    "link": "https://example.com",
    "tag_set": ["tag1", "tag2"],
    "file_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
    "metadata": {"key1": "value1", "key2": "value2"},
    "chunk_vector": [0.1, 0.2, 0.3],
    "tracking_id": "tracking_id",
    "upsert_by_tracking_id": true,
    "group_ids": ["d290f1ee-6c54-4b01-90e6-d701748f0851"],
    "group_tracking_ids": ["group_tracking_id"],
    "time_stamp": "2021-01-01T00:00:00",
    "weight": 0.5,
    "split_avg": false
}))]
pub struct CreateSingleChunkData(pub ChunkData);

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!([{
    "chunk_html": "<p>Some HTML content</p>",
    "link": "https://example.com",
    "tag_set": ["tag1", "tag2"],
    "file_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
    "metadata": {"key1": "value1", "key2": "value2"},
    "chunk_vector": [0.1, 0.2, 0.3],
    "tracking_id": "tracking_id",
    "upsert_by_tracking_id": true,
    "group_ids": ["d290f1ee-6c54-4b01-90e6-d701748f0851"],
    "group_tracking_ids": ["group_tracking_id"],
    "time_stamp": "2021-01-01T00:00:00",
    "weight": 0.5,
    "split_avg": false
}, {
    "chunk_html": "<p>Some more HTML content</p>",
    "link": "https://explain.com",
    "tag_set": ["tag3", "tag4"],
    "file_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
    "metadata": {"key1": "value1", "key2": "value2"},
    "chunk_vector": [0.1, 0.2, 0.3],
    "tracking_id": "tracking_id",
    "upsert_by_tracking_id": true,
    "group_ids": ["d290f1ee-6c54-4b01-90e6-d701748f0851"],
    "group_tracking_ids": ["group_tracking_id"],
    "time_stamp": "2021-01-01T00:00:00",
    "weight": 0.5,
    "split_avg": false
}]
))]
pub struct CreateBatchChunkData(pub Vec<ChunkData>);

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(untagged)]
pub enum CreateChunkData {
    Single(CreateSingleChunkData),
    Batch(CreateBatchChunkData),
}

/// Create or Upsert Chunk or Chunks
///
/// Create a new chunk. If the chunk has the same tracking_id as an existing chunk, the request will fail. Once a chunk is created, it can be searched for using the search endpoint.
#[utoipa::path(
    post,
    path = "/chunk",
    context_path = "/api",
    tag = "chunk",
    request_body(content = CreateChunkData, description = "JSON request payload to create a new chunk (chunk)", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON response payload containing the created chunk", body = ReturnQueuedChunk),
        (status = 426, description = "Error when upgrade is needed to process more chunks", body = ErrorResponseBody),
        (status = 400, description = "Error typically due to deserialization issues", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(redis_pool, pool))]
pub async fn create_chunk(
    create_chunk_data: web::Json<CreateChunkData>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let chunks = match create_chunk_data.clone() {
        CreateChunkData::Single(chunk) => vec![chunk.0],
        CreateChunkData::Batch(chunks) => chunks.0,
    };

    let count_dataset_id = dataset_org_plan_sub.dataset.id;

    let mut timer = Timer::new();
    let chunk_count = get_row_count_for_dataset_id_query(count_dataset_id, pool.clone()).await?;
    timer.add("get dataset count");

    if chunk_count + chunks.len()
        > dataset_org_plan_sub
            .organization
            .plan
            .unwrap_or_default()
            .chunk_count as usize
    {
        return Ok(HttpResponse::UpgradeRequired()
            .json(json!({"message": "Must upgrade your plan to add more chunks"})));
    }

    let server_dataset_configuration = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let mut ingestion_messages = vec![];

    let mut chunk_metadatas = vec![];

    for chunk in chunks {
        let chunk_tag_set = chunk.tag_set.clone().map(|tag_set| tag_set.join(","));

        let chunk_tracking_id = chunk
            .tracking_id
            .clone()
            .filter(|chunk_tracking| !chunk_tracking.is_empty());

        if !chunk.upsert_by_tracking_id.unwrap_or(false) && chunk_tracking_id.is_some() {
            let existing_chunk = get_optional_metadata_from_tracking_id_query(
                chunk_tracking_id
                    .clone()
                    .expect("tracking_id must exist at this point"),
                count_dataset_id,
                pool.clone(),
            )
            .await?;

            if existing_chunk.is_some() {
                return Err(ServiceError::BadRequest(
                    "Need to call with upsert_by_tracking_id set to true in request payload because a chunk with this tracking_id already exists".to_string(),
                )
                .into());
            }
        }

        let timestamp = {
            chunk
                .time_stamp
                .clone()
                .map(|ts| -> Result<NaiveDateTime, ServiceError> {
                    Ok(ts
                        .parse::<DateTimeUtc>()
                        .map_err(|_| {
                            ServiceError::BadRequest("Invalid timestamp format".to_string())
                        })?
                        .0
                        .with_timezone(&chrono::Local)
                        .naive_local())
                })
                .transpose()?
        };

        let chunk_metadata = ChunkMetadata::from_details(
            "".to_string(),
            &chunk.chunk_html,
            &chunk.link,
            &chunk_tag_set,
            None,
            chunk.metadata.clone(),
            chunk_tracking_id,
            timestamp,
            dataset_org_plan_sub.dataset.id,
            chunk.weight.unwrap_or(0.0),
        );
        chunk_metadatas.push(chunk_metadata.clone());

        // check if a group_id is not in existent_group_ids and return an error if it is not
        if let Some(group_ids) = chunk.group_ids.clone() {
            let existent_group_ids = check_group_ids_exist_query(
                chunk.group_ids.clone().unwrap_or_default(),
                count_dataset_id,
                pool.clone(),
            )
            .await?;

            for group_id in group_ids {
                if !existent_group_ids.contains(&group_id) {
                    return Err(ServiceError::BadRequest(format!(
                        "Group with id {} does not exist",
                        group_id
                    ))
                    .into());
                }
            }
        }

        let group_ids_from_group_tracking_ids = if let Some(group_tracking_ids) =
            chunk.group_tracking_ids.clone()
        {
            get_groups_from_tracking_ids_query(group_tracking_ids, count_dataset_id, pool.clone())
                .await
                .ok()
                .unwrap_or(vec![])
        } else {
            vec![]
        };

        let initial_group_ids = chunk.group_ids.clone().unwrap_or_default();
        let mut chunk_only_group_ids = chunk.clone();
        let deduped_group_ids = group_ids_from_group_tracking_ids
            .into_iter()
            .chain(initial_group_ids.into_iter())
            .unique()
            .collect::<Vec<uuid::Uuid>>();

        chunk_only_group_ids.group_ids = Some(deduped_group_ids);
        chunk_only_group_ids.group_tracking_ids = None;

        let upload_message = UploadIngestionMessage {
            ingest_specific_chunk_metadata: IngestSpecificChunkMetadata {
                id: chunk_metadata.id,
                qdrant_point_id: chunk_metadata.qdrant_point_id,
                dataset_id: count_dataset_id,
                attempt_number: 0,
            },
            chunk: chunk_only_group_ids.clone(),
            dataset_id: count_dataset_id,
            dataset_config: server_dataset_configuration.clone(),
            upsert_by_tracking_id: chunk.upsert_by_tracking_id.unwrap_or(false),
        };

        ingestion_messages.push(upload_message);
    }

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    timer.add("got redis connection");

    let serialized_messages: Vec<String> = ingestion_messages
        .iter()
        .filter_map(|msg| serde_json::to_string(&msg).ok())
        .collect();

    let pos_in_queue = redis::cmd("rpush")
        .arg("ingestion")
        .arg(&serialized_messages)
        .query_async(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let response = match create_chunk_data.into_inner() {
        CreateChunkData::Single(_) => ReturnQueuedChunk::Single(SingleQueuedChunkResponse {
            chunk_metadata: chunk_metadatas
                .get(0)
                .ok_or(ServiceError::BadRequest(
                    "Failed to queue a single chunk due to deriving 0 ingestion_messages from the request data".to_string(),
                ))?
                .clone(),
            pos_in_queue,
        }),
        CreateChunkData::Batch(_) => ReturnQueuedChunk::Batch(BatchQueuedChunkResponse {
            chunk_metadata: chunk_metadatas,
            pos_in_queue,
        }),
    };

    Ok(HttpResponse::Ok()
        .insert_header((Timer::header_key(), timer.header_value()))
        .json(response))
}

/// Delete Chunk
///
/// Delete a chunk by its id. If deleting a root chunk which has a collision, the most recently created collision will become a new root chunk.
#[utoipa::path(
    delete,
    path = "/chunk/{chunk_id}",
    context_path = "/api",
    tag = "chunk",
    responses(
        (status = 204, description = "Confirmation that the chunk with the id specified was deleted"),
        (status = 400, description = "Service error relating to finding a chunk by tracking_id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("chunk_id" = Option<uuid::Uuid>, Path, description = "Id of the chunk you want to fetch."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn delete_chunk(
    chunk_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let chunk_id = chunk_id.into_inner();

    delete_chunk_metadata_query(
        chunk_id,
        dataset_org_plan_sub.dataset,
        pool,
        server_dataset_config,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Delete Chunk By Tracking Id
///
/// Delete a chunk by tracking_id. This is useful for when you are coordinating with an external system and want to use the tracking_id to identify the chunk. If deleting a root chunk which has a collision, the most recently created collision will become a new root chunk.
#[utoipa::path(
    delete,
    path = "/chunk/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "chunk",
    responses(
        (status = 204, description = "Confirmation that the chunk with the tracking_id specified was deleted"),
        (status = 400, description = "Service error relating to finding a chunk by tracking_id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("tracking_id" = Option<String>, Path, description = "tracking_id of the chunk you want to delete"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn delete_chunk_by_tracking_id(
    tracking_id: web::Path<String>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let tracking_id_inner = tracking_id.into_inner();
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let chunk_metadata =
        get_metadata_from_tracking_id_query(tracking_id_inner, dataset_id, pool.clone()).await?;

    delete_chunk_metadata_query(
        chunk_metadata.id,
        dataset_org_plan_sub.dataset,
        pool,
        server_dataset_config,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "chunk_id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
    "link": "https://example.com",
    "chunk_html": "<p>Some HTML content</p>",
    "metadata": {"key1": "value1", "key2": "value2"},
    "time_stamp": "2021-01-01T00:00:00",
    "weight": 0.5,
    "group_ids": ["d290f1ee-6c54-4b01-90e6-d701748f0851"],
}))]
pub struct UpdateChunkData {
    /// Id of the chunk you want to update. You can provide either the chunk_id or the tracking_id. If both are provided, the chunk_id will be used.
    chunk_id: Option<uuid::Uuid>,
    /// Tracking_id of the chunk you want to update. This is required to match an existing chunk.
    tracking_id: Option<String>,
    /// Tag set is a list of tags. This can be used to filter chunks by tag. Unlike with metadata filtering, HNSW indices will exist for each tag such that there is not a performance hit for filtering on them. If no tag_set is provided, the existing tag_set will be used.
    tag_set: Option<Vec<String>>,
    /// Link of the chunk you want to update. This can also be any string. Frequently, this is a link to the source of the chunk. The link value will not affect the embedding creation. If no link is provided, the existing link will be used.
    link: Option<String>,
    /// HTML content of the chunk you want to update. This can also be plaintext. The innerText of the HTML will be used to create the embedding vector. The point of using HTML is for convienience, as some users have applications where users submit HTML content. If no chunk_html is provided, the existing chunk_html will be used.
    chunk_html: Option<String>,
    /// The metadata is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata. If no metadata is provided, the existing metadata will be used.
    metadata: Option<serde_json::Value>,
    /// Time_stamp should be an ISO 8601 combined date and time without timezone. It is used for time window filtering and recency-biasing search results. If no time_stamp is provided, the existing time_stamp will be used.
    time_stamp: Option<String>,
    /// Weight is a float which can be used to bias search results. This is useful for when you want to bias search results for a chunk. The magnitude only matters relative to other chunks in the chunk's dataset dataset. If no weight is provided, the existing weight will be used.
    weight: Option<f64>,
    /// Group ids are the ids of the groups that the chunk should be placed into. This is useful for when you want to update a chunk and add it to a group or multiple groups in one request.
    group_ids: Option<Vec<uuid::Uuid>>,
    /// Group tracking_ids are the tracking_ids of the groups that the chunk should be placed into. This is useful for when you want to update a chunk and add it to a group or multiple groups in one request.
    group_tracking_ids: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateIngestionMessage {
    pub chunk_metadata: ChunkMetadata,
    pub server_dataset_config: ServerDatasetConfiguration,
    pub dataset_id: uuid::Uuid,
    pub group_ids: Option<Vec<UnifiedId>>,
}

/// Update Chunk
///
/// Update a chunk. If you try to change the tracking_id of the chunk to have the same tracking_id as an existing chunk, the request will fail.
#[utoipa::path(
    put,
    path = "/chunk",
    context_path = "/api",
    tag = "chunk",
    request_body(content = UpdateChunkData, description = "JSON request payload to update a chunk (chunk)", content_type = "application/json"),
    responses(
        (status = 204, description = "No content Ok response indicating the chunk was updated as requested",),
        (status = 400, description = "Service error relating to to updating chunk, likely due to conflicting tracking_id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool, redis_pool))]
pub async fn update_chunk(
    chunk: web::Json<UpdateChunkData>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let dataset_id = dataset_org_plan_sub.dataset.id;
    let chunk_id = chunk.chunk_id;

    let chunk_metadata = if let Some(chunk_id) = chunk_id {
        get_metadata_from_id_query(chunk_id, dataset_id, pool).await?
    } else if let Some(tracking_id) = chunk.tracking_id.clone() {
        get_metadata_from_tracking_id_query(tracking_id.clone(), dataset_id, pool).await?
    } else {
        return Err(ServiceError::BadRequest(
            "Either chunk_id or tracking_id must be provided to update a chunk".into(),
        )
        .into());
    };

    let link = chunk
        .link
        .clone()
        .unwrap_or_else(|| chunk_metadata.link.clone().unwrap_or_default());
    let chunk_tracking_id = chunk
        .tracking_id
        .clone()
        .filter(|chunk_tracking| !chunk_tracking.is_empty());

    let chunk_tag_set = chunk.tag_set.clone().map(|tag_set| tag_set.join(","));

    let chunk_html = match chunk.chunk_html.clone() {
        Some(chunk_html) => Some(chunk_html),
        None => chunk_metadata.chunk_html,
    };

    let metadata = ChunkMetadata::from_details_with_id(
        chunk_metadata.id,
        "".to_string(),
        &chunk_html,
        &Some(link),
        &chunk_tag_set.or(chunk_metadata.tag_set),
        chunk_metadata.qdrant_point_id,
        <std::option::Option<serde_json::Value> as Clone>::clone(&chunk.metadata)
            .or(chunk_metadata.metadata),
        chunk_tracking_id,
        chunk
            .time_stamp
            .clone()
            .map(|ts| -> Result<NaiveDateTime, ServiceError> {
                //TODO: change all ts parsing to this crate
                Ok(ts
                    .parse::<DateTimeUtc>()
                    .map_err(|_| ServiceError::BadRequest("Invalid timestamp format".to_string()))?
                    .0
                    .with_timezone(&chrono::Local)
                    .naive_local())
            })
            .transpose()?
            .or(chunk_metadata.time_stamp),
        dataset_id,
        chunk.weight.unwrap_or(1.0),
    );

    let group_ids = if let Some(group_ids) = chunk.group_ids.clone() {
        Some(
            group_ids
                .into_iter()
                .map(UnifiedId::from)
                .collect::<Vec<UnifiedId>>(),
        )
    } else {
        chunk.group_tracking_ids.clone().map(|group_tracking_ids| {
            group_tracking_ids
                .into_iter()
                .map(UnifiedId::from)
                .collect::<Vec<UnifiedId>>()
        })
    };

    let message = UpdateIngestionMessage {
        chunk_metadata: metadata.clone(),
        server_dataset_config,
        dataset_id,
        group_ids,
    };

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    redis::cmd("rpush")
        .arg("ingestion")
        .arg(serde_json::to_string(&message)?)
        .query_async(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct UpdateChunkByTrackingIdData {
    /// Tracking_id of the chunk you want to update. This is required to match an existing chunk.
    tracking_id: String,
    /// Link of the chunk you want to update. This can also be any string. Frequently, this is a link to the source of the chunk. The link value will not affect the embedding creation. If no link is provided, the existing link will be used.
    link: Option<String>,
    /// HTML content of the chunk you want to update. This can also be plaintext. The innerText of the HTML will be used to create the embedding vector. The point of using HTML is for convienience, as some users have applications where users submit HTML content. If no chunk_html is provided, the existing chunk_html will be used.
    chunk_html: Option<String>,
    /// The metadata is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata. If no metadata is provided, the existing metadata will be used.
    metadata: Option<serde_json::Value>,
    /// Time_stamp should be an ISO 8601 combined date and time without timezone. It is used for time window filtering and recency-biasing search results. If no time_stamp is provided, the existing time_stamp will be used.
    time_stamp: Option<String>,
    /// Weight is a float which can be used to bias search results. This is useful for when you want to bias search results for a chunk. The magnitude only matters relative to other chunks in the chunk's dataset dataset. If no weight is provided, the existing weight will be used.
    weight: Option<f64>,
    /// Group ids are the ids of the groups that the chunk should be placed into. This is useful for when you want to update a chunk and add it to a group or multiple groups in one request.
    group_ids: Option<Vec<uuid::Uuid>>,
    /// Group tracking_ids are the tracking_ids of the groups that the chunk should be placed into. This is useful for when you want to update a chunk and add it to a group or multiple groups in one request.
    group_tracking_ids: Option<Vec<String>>,
}

/// Update Chunk By Tracking Id
///
/// Update a chunk by tracking_id. This is useful for when you are coordinating with an external system and want to use the tracking_id to identify the chunk.
#[utoipa::path(
    put,
    path = "/chunk/tracking_id/update",
    context_path = "/api",
    tag = "chunk",
    request_body(content = UpdateChunkByTrackingIdData, description = "JSON request payload to update a chunk by tracking_id (chunks)", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunk has been updated as per your request",),
        (status = 400, description = "Service error relating to to updating chunk", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool, redis_pool))]
pub async fn update_chunk_by_tracking_id(
    chunk: web::Json<UpdateChunkByTrackingIdData>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    if chunk.tracking_id.is_empty() {
        return Err(ServiceError::BadRequest(
            "Tracking id must be provided to update by tracking_id".into(),
        )
        .into());
    }
    let tracking_id = chunk.tracking_id.clone();

    let dataset_id = dataset_org_plan_sub.dataset.id;

    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let chunk_metadata =
        get_metadata_from_tracking_id_query(tracking_id, dataset_id, pool.clone()).await?;

    let link = chunk
        .link
        .clone()
        .unwrap_or_else(|| chunk_metadata.link.clone().unwrap_or_default());

    let chunk_html = match chunk.chunk_html.clone() {
        Some(chunk_html) => Some(chunk_html),
        None => chunk_metadata.chunk_html,
    };

    let metadata = ChunkMetadata::from_details_with_id(
        chunk_metadata.id,
        "".to_string(),
        &chunk_html,
        &Some(link),
        &chunk_metadata.tag_set,
        chunk_metadata.qdrant_point_id,
        <std::option::Option<serde_json::Value> as Clone>::clone(&chunk.metadata)
            .or(chunk_metadata.metadata),
        Some(chunk.tracking_id.clone()),
        chunk
            .time_stamp
            .clone()
            .map(|ts| -> Result<NaiveDateTime, ServiceError> {
                //TODO: change all ts parsing to this crate
                Ok(ts
                    .parse::<DateTimeUtc>()
                    .map_err(|_| ServiceError::BadRequest("Invalid timestamp format".to_string()))?
                    .0
                    .with_timezone(&chrono::Local)
                    .naive_local())
            })
            .transpose()?
            .or(chunk_metadata.time_stamp),
        dataset_org_plan_sub.dataset.id,
        chunk.weight.unwrap_or(1.0),
    );
    let group_ids = if let Some(group_ids) = chunk.group_ids.clone() {
        Some(
            group_ids
                .into_iter()
                .map(UnifiedId::from)
                .collect::<Vec<UnifiedId>>(),
        )
    } else {
        chunk.group_tracking_ids.clone().map(|group_tracking_ids| {
            group_tracking_ids
                .into_iter()
                .map(UnifiedId::from)
                .collect::<Vec<UnifiedId>>()
        })
    };

    let message = UpdateIngestionMessage {
        chunk_metadata: metadata.clone(),
        server_dataset_config,
        dataset_id,
        group_ids,
    };

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    redis::cmd("rpush")
        .arg("ingestion")
        .arg(serde_json::to_string(&message)?)
        .query_async(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(untagged)]
pub enum RangeCondition {
    String(String),
    Float(f64),
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "gte": 0.0,
    "lte": 1.0,
    "gt": 0.0,
    "lt": 1.0
}))]
pub struct Range {
    // gte is the lower bound of the range. This is inclusive.
    pub gte: Option<RangeCondition>,
    // lte is the upper bound of the range. This is inclusive.
    pub lte: Option<RangeCondition>,
    // gt is the lower bound of the range. This is exclusive.
    pub gt: Option<RangeCondition>,
    // lt is the upper bound of the range. This is exclusive.
    pub lt: Option<RangeCondition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(untagged)]
pub enum MatchCondition {
    Text(String),
    Integer(i64),
}

impl MatchCondition {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            MatchCondition::Text(text) => text.clone(),
            MatchCondition::Integer(int) => int.to_string(),
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self {
            MatchCondition::Text(text) => text.parse().unwrap(),
            MatchCondition::Integer(int) => *int,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "field": "metadata.key1",
    "match": ["value1", "value2"],
    "range": {
        "gte": 0.0,
        "lte": 1.0,
        "gt": 0.0,
        "lt": 1.0
    }
}))]
pub struct FieldCondition {
    /// Field is the name of the field to filter on. The field value will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata. To access fields inside of the metadata that you provide with the card, prefix the field name with `metadata.`.
    pub field: String,
    /// Match is the value to match on the field. The match value will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata.
    pub r#match: Option<Vec<MatchCondition>>,
    /// Range is a JSON object which can be used to filter chunks by a range of values. This only works for numerical fields. You can specify this if you want values in a certain range.
    pub range: Option<Range>,
}

fn convert_to_date_time(time_stamp: String) -> Result<Option<f64>, ServiceError> {
    Ok(Some(
        time_stamp
            .parse::<DateTimeUtc>()
            .map_err(|_| ServiceError::BadRequest("Invalid timestamp format".to_string()))?
            .0
            .with_timezone(&chrono::Local)
            .naive_local()
            .timestamp() as f64,
    ))
}

pub fn get_range(range: Range) -> Result<qdrant::Range, ServiceError> {
    #[derive(PartialEq)]
    enum RangeValueType {
        Float,
        String,
        None,
    }

    // Determine the type of the range value if present
    let mut range_value_type = RangeValueType::None;

    // First pass to determine the consistent type of the range, if any
    let range_values = [&range.gt, &range.gte, &range.lt, &range.lte];
    for value in range_values {
        match value {
            Some(RangeCondition::Float(_)) => {
                if range_value_type == RangeValueType::String {
                    return Err(ServiceError::BadRequest(
                        "Mixed types in range conditions".to_string(),
                    ));
                }
                range_value_type = RangeValueType::Float;
            }
            Some(RangeCondition::String(_)) => {
                if range_value_type == RangeValueType::Float {
                    return Err(ServiceError::BadRequest(
                        "Mixed types in range conditions".to_string(),
                    ));
                }
                range_value_type = RangeValueType::String;
            }
            None => {}
        }
    }

    // Based on the determined type, process the values
    match range_value_type {
        RangeValueType::Float | RangeValueType::String => {
            let gt = match &range.gt {
                Some(RangeCondition::Float(value)) => Some(*value),
                Some(RangeCondition::String(value)) => convert_to_date_time(value.to_string())?,
                None => None,
            };
            let gte = match &range.gte {
                Some(RangeCondition::Float(value)) => Some(*value),
                Some(RangeCondition::String(value)) => convert_to_date_time(value.to_string())?,
                None => None,
            };
            let lt = match &range.lt {
                Some(RangeCondition::Float(value)) => Some(*value),
                Some(RangeCondition::String(value)) => convert_to_date_time(value.to_string())?,
                None => None,
            };
            let lte = match &range.lte {
                Some(RangeCondition::Float(value)) => Some(*value),
                Some(RangeCondition::String(value)) => convert_to_date_time(value.to_string())?,
                None => None,
            };

            Ok(qdrant::Range { gt, gte, lt, lte })
        }
        RangeValueType::None => Err(ServiceError::BadRequest(
            "No range conditions provided".to_string(),
        )),
    }
}

impl FieldCondition {
    pub async fn convert_to_qdrant_condition(
        &self,
        pool: web::Data<Pool>,
        dataset_id: uuid::Uuid,
    ) -> Result<Option<qdrant::Condition>, ServiceError> {
        if self.r#match.is_none() && self.range.is_none() {
            return Ok(None);
        }

        if self.r#match.is_some() && self.range.is_some() {
            return Err(ServiceError::BadRequest(
                "Cannot have both match and range conditions".to_string(),
            ));
        }

        if self.field.starts_with("metadata.") {
            return Ok(Some(
                get_metadata_filter_condition(&self, dataset_id, pool)
                    .await?
                    .into(),
            ));
        }

        if let Some(range) = self.range.clone() {
            let range = get_range(range)?;
            return Ok(Some(qdrant::Condition::range(self.field.as_str(), range)));
        };

        let matches = match self.r#match.clone() {
            Some(matches) => matches,
            // Return nothing, there isn't a
            None => return Ok(None),
        };

        match matches.first().ok_or(ServiceError::BadRequest(
            "Should have at least one value for match".to_string(),
        ))? {
            MatchCondition::Text(_) => Ok(Some(qdrant::Condition::matches(
                self.field.as_str(),
                matches.iter().map(|x| x.to_string()).collect_vec(),
            ))),
            MatchCondition::Integer(_) => Ok(Some(qdrant::Condition::matches(
                self.field.as_str(),
                matches.iter().map(|x| x.to_i64()).collect_vec(),
            ))),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "should": [
        {
            "field": "metadata.key1",
            "match": ["value1", "value2"],
            "range": {
                "gte": 0.0,
                "lte": 1.0,
                "gt": 0.0,
                "lt": 1.0
            }
        }
    ],
    "must": [
        {
            "field": "metadata.key2",
            "match": ["value3", "value4"],
            "range": {
                "gte": 0.0,
                "lte": 1.0,
                "gt": 0.0,
                "lt": 1.0
            }
        }
    ],
    "must_not": [
        {
            "field": "metadata.key3",
            "match": ["value5", "value6"],
            "range": {
                "gte": 0.0,
                "lte": 1.0,
                "gt": 0.0,
                "lt": 1.0
            }
        }
    ]
}))]
pub struct ChunkFilter {
    /// Only one of these field conditions has to match for the chunk to be included in the result set.
    pub should: Option<Vec<FieldCondition>>,
    /// All of these field conditions have to match for the chunk to be included in the result set.
    pub must: Option<Vec<FieldCondition>>,
    /// None of these field conditions can match for the chunk to be included in the result set.
    pub must_not: Option<Vec<FieldCondition>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
#[schema(example = json!({
    "search_type": "semantic",
    "query": "Some search query",
    "page": 1,
    "page_size": 10,
    "filters": {
        "should": [
            {
                "field": "metadata.key1",
                "match": ["value1", "value2"],
                "range": {
                    "gte": 0.0,
                    "lte": 1.0,
                    "gt": 0.0,
                    "lt": 1.0
                }
            }
        ],
        "must": [
            {
                "field": "metadata.key2",
                "match": ["value3", "value4"],
                "range": {
                    "gte": 0.0,
                    "lte": 1.0,
                    "gt": 0.0,
                    "lt": 1.0
                }
            }
        ],
        "must_not": [
            {
                "field": "metadata.key3",
                "match": ["value5", "value6"],
                "range": {
                    "gte": 0.0,
                    "lte": 1.0,
                    "gt": 0.0,
                    "lt": 1.0
                }
            }
        ]
    },
    "date_bias": true,
    "use_weights": true,
    "get_collisions": true,
    "highlight_results": true,
    "highlight_delimiters": ["?", ",", ".", "!"],
    "score_threshold": 0.5
}))]
pub struct SearchChunkData {
    /// Can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using BAAI/bge-reranker-large. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE.
    pub search_type: String,
    /// Query is the search query. This can be any string. The query will be used to create an embedding vector and/or SPLADE vector which will be used to find the result set.
    pub query: String,
    /// Page of chunks to fetch. Each page is 10 chunks. Support for custom page size is coming soon.
    pub page: Option<u64>,
    /// Page size is the number of chunks to fetch. This can be used to fetch more than 10 chunks at a time.
    pub page_size: Option<u64>,
    /// Get total page count for the query accounting for the applied filters. Defaults to true, but can be set to false to reduce latency in edge cases performance.
    pub get_total_pages: Option<bool>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Set date_bias to true to bias search results towards more recent chunks. This will work best in hybrid search mode.
    pub date_bias: Option<bool>,
    /// Set use_weights to true to use the weights of the chunks in the result set in order to sort them. If not specified, this defaults to true.
    pub use_weights: Option<bool>,
    /// Set get_collisions to true to get the collisions for each chunk. This will only apply if environment variable COLLISIONS_ENABLED is set to true.
    pub get_collisions: Option<bool>,
    /// Set highlight_results to true to highlight the results. If not specified, this defaults to true.
    pub highlight_results: Option<bool>,
    /// Set highlight_delimiters to a list of strings to use as delimiters for highlighting. If not specified, this defaults to ["?", ",", ".", "!"].
    pub highlight_delimiters: Option<Vec<String>>,
    /// Turn on quote words and negated words to search for exact phrases and exclude words from the search results. Default is false.
    pub quote_negated_words: Option<bool>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold.
    pub score_threshold: Option<f32>,
    /// Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement. Default is false.
    pub slim_chunks: Option<bool>,
}

impl Default for SearchChunkData {
    fn default() -> Self {
        SearchChunkData {
            search_type: "hybrid".to_string(),
            query: "".to_string(),
            page: Some(1),
            get_total_pages: None,
            page_size: Some(10),
            filters: None,
            date_bias: None,
            use_weights: None,
            get_collisions: None,
            highlight_results: None,
            highlight_delimiters: None,
            score_threshold: None,
            slim_chunks: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
#[schema(example = json!({
    "metadata": [
        {
            "id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
            "content": "Some content",
            "link": "https://example.com",
            "chunk_html": "<p>Some HTML content</p>",
            "metadata": {"key1": "value1", "key2": "value2"},
            "time_stamp": "2021-01-01T00:00:00",
            "weight": 0.5,
        }
    ],
    "score": 0.5
}))]
pub struct ScoreChunkDTO {
    pub metadata: Vec<ChunkMetadataWithFileData>,
    pub score: f64,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
pub enum ChunkMetadataTypes {
    IDs(Vec<SlimChunkMetadata>),
    MetadataWithFileData(Vec<ChunkMetadataWithFileData>),
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct SearchChunkQueryResponseBody {
    pub score_chunks: Vec<ScoreChunkDTO>,
    pub total_chunk_pages: i64,
}

#[derive(Clone, Debug)]
pub struct ParsedQuery {
    pub query: String,
    pub quote_words: Option<Vec<String>>,
    pub negated_words: Option<Vec<String>>,
}
pub fn parse_query(query: String) -> ParsedQuery {
    let re = Regex::new(r#""(?:[^"\\]|\\.)*""#).expect("Regex pattern is always valid");
    let quote_words: Vec<String> = re
        .captures_iter(&query)
        .map(|capture| capture[0].to_string())
        .filter(|word| !word.is_empty())
        .collect::<Vec<String>>();

    let quote_words = if quote_words.is_empty() {
        None
    } else {
        Some(quote_words)
    };

    let negated_words: Vec<String> = query
        .split_whitespace()
        .filter(|word| word.starts_with('-'))
        .map(|word| word.strip_prefix('-').unwrap().to_string())
        .collect::<Vec<String>>();

    let negated_words = if negated_words.is_empty() {
        None
    } else {
        Some(negated_words)
    };

    ParsedQuery {
        query,
        quote_words,
        negated_words,
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[serde(untagged)]
pub enum SearchChunkResponseTypes {
    SearchSlimChunkQueryResponseBody(SearchSlimChunkQueryResponseBody),
    SearchChunkQueryResponseBody(SearchChunkQueryResponseBody),
}

/// Search
///
/// This route provides the primary search functionality for the API. It can be used to search for chunks by semantic similarity, full-text similarity, or a combination of both. Results' `chunk_html` values will be modified with `<b>` tags for sub-sentence highlighting.
#[utoipa::path(
    post,
    path = "/chunk/search",
    context_path = "/api",
    tag = "chunk",
    request_body(content = SearchChunkData, description = "JSON request payload to semantically search for chunks (chunks)", content_type = "application/json"),
    responses(
        (status = 200, description = "Chunks with embedding vectors which are similar to those in the request body", body = SearchChunkResponseTypes),

        (status = 400, description = "Service error relating to searching", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn search_chunk(
    data: web::Json<SearchChunkData>,
    _user: LoggedUser,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let page = data.page.unwrap_or(1);
    let get_total_pages = data.get_total_pages.unwrap_or(true);

    let mut parsed_query = ParsedQuery {
        query: data.query.clone(),
        quote_words: None,
        negated_words: None,
    };
    if data.quote_negated_words.unwrap_or(false) {
        parsed_query = parse_query(data.query.clone());
    }

    let tx_ctx = sentry::TransactionContext::new("search", "search_chunks");
    let transaction = sentry::start_transaction(tx_ctx);
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone().into())));
    let mut timer = Timer::new();

    let result_chunks = match data.search_type.as_str() {
        "fulltext" => {
            if !server_dataset_config.FULLTEXT_ENABLED {
                return Err(ServiceError::BadRequest(
                    "Fulltext search is not enabled for this dataset".into(),
                )
                .into());
            }

            search_full_text_chunks(
                data.clone(),
                parsed_query,
                page,
                get_total_pages,
                pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
            )
            .await?
        }
        "hybrid" => {
            search_hybrid_chunks(
                data.clone(),
                parsed_query,
                page,
                get_total_pages,
                pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
            )
            .await?
        }
        _ => {
            search_semantic_chunks(
                data.clone(),
                parsed_query,
                page,
                get_total_pages,
                pool,
                dataset_org_plan_sub.dataset,
                &mut timer,
                server_dataset_config,
            )
            .await?
        }
    };

    transaction.finish();

    if data.slim_chunks.unwrap_or(false) {
        let ids = result_chunks
            .score_chunks
            .iter()
            .map(|score_chunk| <ScoreChunkDTO as Clone>::clone(score_chunk).into())
            .collect::<Vec<ScoreSlimChunks>>();

        let res = SearchSlimChunkQueryResponseBody {
            score_chunks: ids,
            total_chunk_pages: result_chunks.total_chunk_pages,
        };

        return Ok(HttpResponse::Ok()
            .insert_header((Timer::header_key(), timer.header_value()))
            .json(res));
    }

    Ok(HttpResponse::Ok()
        .insert_header((Timer::header_key(), timer.header_value()))
        .json(result_chunks))
}

/// Get Chunk By Id
///
/// Get a singular chunk by id.
#[utoipa::path(
    get,
    path = "/chunk/{chunk_id}",
    context_path = "/api",
    tag = "chunk",
    responses(
        (status = 200, description = "chunk with the id that you were searching for", body = ChunkMetadata),
        (status = 400, description = "Service error relating to fidning a chunk by tracking_id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("chunk_id" = Option<uuid::Uuid>, Path, description = "Id of the chunk you want to fetch."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_chunk_by_id(
    chunk_id: web::Path<uuid::Uuid>,
    _user: LoggedUser,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let chunk_id = chunk_id.into_inner();
    let chunk = get_metadata_from_id_query(chunk_id, dataset_org_plan_sub.dataset.id, pool).await?;

    Ok(HttpResponse::Ok().json(chunk))
}

/// Get Chunk By Tracking Id
///
/// Get a singular chunk by tracking_id. This is useful for when you are coordinating with an external system and want to use your own id as the primary reference for a chunk.
#[utoipa::path(
    get,
    path = "/chunk/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "chunk",
    responses(
        (status = 200, description = "chunk with the tracking_id that you were searching for", body = ChunkMetadata),
        (status = 400, description = "Service error relating to fidning a chunk by tracking_id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("tracking_id" = Option<String>, Path, description = "tracking_id of the chunk you want to fetch"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_chunk_by_tracking_id(
    tracking_id: web::Path<String>,
    _user: LoggedUser,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let chunk = get_metadata_from_tracking_id_query(
        tracking_id.into_inner(),
        dataset_org_plan_sub.dataset.id,
        pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(chunk))
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct RecommendChunksRequest {
    /// The ids of the chunks to be used as positive examples for the recommendation. The chunks in this array will be used to find similar chunks.
    pub positive_chunk_ids: Option<Vec<uuid::Uuid>>,
    /// The ids of the chunks to be used as negative examples for the recommendation. The chunks in this array will be used to filter out similar chunks.
    pub negative_chunk_ids: Option<Vec<uuid::Uuid>>,
    /// The tracking_ids of the chunks to be used as positive examples for the recommendation. The chunks in this array will be used to find similar chunks.
    pub positive_tracking_ids: Option<Vec<String>>,
    /// The tracking_ids of the chunks to be used as negative examples for the recommendation. The chunks in this array will be used to filter out similar chunks.
    pub negative_tracking_ids: Option<Vec<String>>,
    /// Filters to apply to the chunks to be recommended. This is a JSON object which contains the filters to apply to the chunks to be recommended. The default is None.
    pub filters: Option<ChunkFilter>,
    /// The number of chunks to return. This is the number of chunks which will be returned in the response. The default is 10.
    pub limit: Option<u64>,
    /// Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement. Default is false.
    pub slim_chunks: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[serde(untagged)]
pub enum RecommendChunksResponseTypes {
    #[schema(example = json!([{
        "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "content": "Hello, world!",
        "link": "https://trieve.ai",
        "qdrant_point_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "created_at": "2021-01-01T00:00:00",
        "updated_at": "2021-01-01T00:00:00",
        "tag_set": "tag1,tag2",
        "chunk_html": "<p>Hello, world!</p>",
        "metadata": {"key": "value"},
        "tracking_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "time_stamp": "2021-01-01T00:00:00",
        "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "weight": 0.5,
    }]))]
    Chunks(Vec<ChunkMetadata>),
    #[schema(example = json!([{
        "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "link": "https://trieve.ai",
        "qdrant_point_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "created_at": "2021-01-01T00:00:00",
        "updated_at": "2021-01-01T00:00:00",
        "tag_set": "tag1,tag2",
        "metadata": {"key": "value"},
        "tracking_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "time_stamp": "2021-01-01T00:00:00",
        "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "weight": 0.5,
    }]))]
    SlimChunks(Vec<SlimChunkMetadata>),
}

/// Get Recommended Chunks
///
/// Get recommendations of chunks similar to the chunks in the request. Think about this as a feature similar to the "add to playlist" recommendation feature on Spotify. This request pairs especially well with our groups endpoint.
#[utoipa::path(
    post,
    path = "/chunk/recommend",
    context_path = "/api",
    tag = "chunk",
    request_body(content = RecommendChunksRequest, description = "JSON request payload to get recommendations of chunks similar to the chunks in the request", content_type = "application/json"),
    responses(

        (status = 200, description = "Chunks with embedding vectors which are similar to those in the request body", body = RecommendChunksResponseTypes),
        (status = 400, description = "Service error relating to to getting similar chunks", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_recommended_chunks(
    data: web::Json<RecommendChunksRequest>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let positive_chunk_ids = data.positive_chunk_ids.clone();
    let negative_chunk_ids = data.negative_chunk_ids.clone();
    let positive_tracking_ids = data.positive_tracking_ids.clone();
    let negative_tracking_ids = data.negative_tracking_ids.clone();
    let limit = data.limit.unwrap_or(10);
    let server_dataset_config =
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);

    let dataset_id = dataset_org_plan_sub.dataset.id;

    let mut positive_qdrant_ids = vec![];

    if let Some(positive_chunk_ids) = positive_chunk_ids {
        positive_qdrant_ids.extend(
            get_point_ids_from_unified_chunk_ids(
                positive_chunk_ids
                    .into_iter()
                    .map(UnifiedId::TrieveUuid)
                    .collect(),
                dataset_id,
                pool.clone(),
            )
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not get positive qdrant_point_ids from positive_chunk_ids: {}",
                    err
                ))
            })?,
        )
    }

    if let Some(positive_chunk_tracking_ids) = positive_tracking_ids {
        positive_qdrant_ids.extend(
            get_point_ids_from_unified_chunk_ids(
                positive_chunk_tracking_ids
                    .into_iter()
                    .map(UnifiedId::TrackingId)
                    .collect(),
                dataset_id,
                pool.clone(),
            )
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not get positive qdrant_point_ids from positive_tracking_ids: {}",
                    err
                ))
            })?,
        )
    }

    let mut negative_qdrant_ids = vec![];

    if let Some(negative_chunk_ids) = negative_chunk_ids {
        negative_qdrant_ids.extend(
            get_point_ids_from_unified_chunk_ids(
                negative_chunk_ids
                    .into_iter()
                    .map(UnifiedId::TrieveUuid)
                    .collect(),
                dataset_id,
                pool.clone(),
            )
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not get negative qdrant_point_ids from negative_chunk_ids: {}",
                    err
                ))
            })?,
        )
    }

    if let Some(negative_chunk_tracking_ids) = negative_tracking_ids {
        negative_qdrant_ids.extend(
            get_point_ids_from_unified_chunk_ids(
                negative_chunk_tracking_ids
                    .into_iter()
                    .map(UnifiedId::TrackingId)
                    .collect(),
                dataset_id,
                pool.clone(),
            )
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not get negative qdrant_point_ids from negative_tracking_ids: {}",
                    err
                ))
            })?,
        )
    }

    let recommended_qdrant_point_ids = recommend_qdrant_query(
        positive_qdrant_ids,
        negative_qdrant_ids,
        data.filters.clone(),
        limit,
        dataset_org_plan_sub.dataset.id,
        server_dataset_config,
        pool.clone(),
    )
    .await
    .map_err(|err| {
        ServiceError::BadRequest(format!("Could not get recommended chunks: {}", err))
    })?;

    let recommended_chunk_metadatas =
        get_metadata_from_point_ids(recommended_qdrant_point_ids, pool)
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not get recommended chunk_metadas from qdrant_point_ids: {}",
                    err
                ))
            })?;

    if data.slim_chunks.unwrap_or(false) {
        let res = recommended_chunk_metadatas
            .into_iter()
            .map(|chunk| chunk.into())
            .collect::<Vec<SlimChunkMetadata>>();

        return Ok(HttpResponse::Ok().json(res));
    }

    Ok(HttpResponse::Ok().json(recommended_chunk_metadatas))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "model": "text-embedding-small",
    "prev_messages": [
        {
            "role": "user",
            "content": "I am going to provide several pieces of information (docs) for you to use in response to a request or question.",
        }
    ],
    "chunk_ids": ["d290f1ee-6c54-4b01-90e6-d701748f0851"],
    "prompt": "Respond to the instruction and include the doc numbers that you used in square brackets at the end of the sentences that you used the docs for:",
    "stream_response": true
}))]
pub struct GenerateChunksRequest {
    /// The model to use for the chat. This can be any model from the openrouter model list. If no model is provided, gpt-3.5-turbo will be used.
    pub model: Option<String>,
    /// The previous messages to be placed into the chat history. The last message in this array will be the prompt for the model to inference on. The length of this array must be at least 1.
    pub prev_messages: Vec<ChatMessageProxy>,
    /// The ids of the chunks to be retrieved and injected into the context window for RAG.
    pub chunk_ids: Vec<uuid::Uuid>,
    /// Prompt for the last message in the prev_messages array. This will be used to generate the next message in the chat. The default is 'Respond to the instruction and include the doc numbers that you used in square brackets at the end of the sentences that you used the docs for:'. You can also specify an empty string to leave the final message alone such that your user's final message can be used as the prompt. See docs.trieve.ai or contact us for more information.
    pub prompt: Option<String>,
    /// Whether or not to stream the response. If this is set to true or not included, the response will be a stream. If this is set to false, the response will be a normal JSON response. Default is true.
    pub stream_response: Option<bool>,
}

/// RAG on Specified Chunks
///
/// This endpoint exists as an alternative to the topic+message concept where our API handles chat memory. With this endpoint, the user is responsible for providing the context window and the prompt. See more in the "search before generate" page at docs.trieve.ai.
#[utoipa::path(
    post,
    path = "/chunk/generate",
    context_path = "/api",
    tag = "chunk",
    request_body(content = GenerateChunksRequest, description = "JSON request payload to perform RAG on some chunks (chunks)", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream of a string, check the chat or search UI for an example how to process this. Response if streaming.",),
        (status = 200, description = "This will be a JSON response of a string containing the LLM's generated inference. Response if not streaming.", body = String),
        (status = 400, description = "Service error relating to to updating chunk, likely due to conflicting tracking_id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn generate_off_chunks(
    data: web::Json<GenerateChunksRequest>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let prev_messages = data.prev_messages.clone();

    if prev_messages.iter().len() < 1 {
        return Err(
            ServiceError::BadRequest("There needs to be at least 1 prior message".into()).into(),
        );
    };

    let chunk_ids = data.chunk_ids.clone();
    let prompt = data.prompt.clone();
    let stream_response = data.stream_response;

    let mut chunks =
        get_metadata_from_ids_query(chunk_ids, dataset_org_plan_sub.dataset.id, pool).await?;

    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);
    let base_url = dataset_config.LLM_BASE_URL;
    let default_model = dataset_config.LLM_DEFAULT_MODEL;

    let base_url = if base_url.is_empty() {
        "https://openrouter.ai/api/v1".into()
    } else {
        base_url
    };

    let llm_api_key = if base_url.contains("openai.com") {
        get_env!("OPENAI_API_KEY", "OPENAI_API_KEY for openai should be set").into()
    } else {
        get_env!(
            "LLM_API_KEY",
            "LLM_API_KEY for openrouter or self-hosted should be set"
        )
        .into()
    };

    let client = Client {
        api_key: llm_api_key,
        http_client: Some(reqwest::Client::new()),
        base_url,
        organization: None,
    };

    let mut messages: Vec<ChatMessage> = vec![];

    messages.truncate(prev_messages.len() - 1);
    messages.push(ChatMessage {
        role: Role::User,
        content: ChatMessageContent::Text("I am going to provide several pieces of information (docs) for you to use in response to a request or question.".to_string()),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    });
    messages.push(ChatMessage {
        role: Role::Assistant,
        content: ChatMessageContent::Text(
            "Understood, I will use the provided docs as information to respond to any future questions or instructions."
                .to_string(),
        ),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    });
    chunks.sort_by(|a, b| {
        data.chunk_ids
            .iter()
            .position(|&id| id == a.id)
            .unwrap()
            .cmp(&data.chunk_ids.iter().position(|&id| id == b.id).unwrap())
    });
    chunks.iter().enumerate().for_each(|(idx, bookmark)| {
        let first_240_words = bookmark
            .content
            .split_whitespace()
            .take(240)
            .collect::<Vec<_>>()
            .join(" ");

        messages.push(ChatMessage {
            role: Role::User,
            content: ChatMessageContent::Text(format!("Doc {}: {}", idx + 1, first_240_words)),
            tool_calls: None,
            name: None,
            tool_call_id: None,
        });
        messages.push(ChatMessage {
            role: Role::Assistant,
            content: ChatMessageContent::Text("".to_string()),
            tool_calls: None,
            name: None,
            tool_call_id: None,
        });
    });

    let last_prev_message = prev_messages
        .last()
        .expect("There needs to be at least 1 prior message");
    let mut prev_messages = prev_messages.clone();
    prev_messages.truncate(prev_messages.len() - 1);

    prev_messages
        .iter()
        .for_each(|message| messages.push(ChatMessage::from(message.clone())));

    let prompt = prompt.unwrap_or("Respond to the question or instruction using the docs and include the doc numbers that you used in square brackets at the end of the sentences that you used the docs for:\n\n".to_string());

    messages.push(ChatMessage {
        role: Role::User,
        content: ChatMessageContent::Text(format!(
            "{} {}",
            prompt,
            last_prev_message.content.clone()
        )),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    });

    let parameters = ChatCompletionParameters {
        model: data.model.clone().unwrap_or(default_model),
        stream: stream_response,
        messages,
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        max_tokens: None,
        presence_penalty: Some(0.8),
        frequency_penalty: Some(0.8),
        logit_bias: None,
        user: None,
        response_format: None,
        tools: None,
        tool_choice: None,
        logprobs: None,
        top_logprobs: None,
        seed: None,
        instance_id: None,
    };

    if !stream_response.unwrap_or(true) {
        let assistant_completion =
            client
                .chat()
                .create(parameters.clone())
                .await
                .map_err(|err| {
                    ServiceError::BadRequest(format!(
                        "Bad response from LLM server provider: {}",
                        err
                    ))
                })?;

        let chat_content = assistant_completion.choices[0].message.content.clone();
        return Ok(HttpResponse::Ok().json(chat_content));
    }

    let stream = client
        .chat()
        .create_stream(parameters.clone())
        .await
        .unwrap();

    Ok(HttpResponse::Ok().streaming(stream.map(
        move |response| -> Result<Bytes, actix_web::Error> {
            if let Ok(response) = response {
                let chat_content = response.choices[0].delta.content.clone();
                return Ok(Bytes::from(chat_content.unwrap_or("".to_string())));
            }
            Err(ServiceError::InternalServerError(
                "Model Response Error. Please try again later".into(),
            )
            .into())
        },
    )))
}
