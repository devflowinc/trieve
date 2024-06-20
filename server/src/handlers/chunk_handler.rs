use std::collections::HashMap;

use super::auth_handler::{AdminOnly, LoggedUser};
use crate::data::models::{
    ChatMessageProxy, ChunkMetadata, ChunkMetadataStringTagSet, ChunkMetadataWithScore,
    ConditionType, DatasetAndOrgWithSubAndPlan, GeoInfo, IngestSpecificChunkMetadata, Pool,
    RedisPool, ScoreChunkDTO, ServerDatasetConfiguration, SlimChunkMetadataWithScore, UnifiedId,
};
use crate::errors::ServiceError;
use crate::get_env;
use crate::operators::chunk_operator::get_metadata_from_id_query;
use crate::operators::chunk_operator::*;
use crate::operators::parse_operator::convert_html_to_text;
use crate::operators::qdrant_operator::{point_ids_exists_in_qdrant, recommend_qdrant_query};
use crate::operators::search_operator::{
    autocomplete_fulltext_chunks, autocomplete_semantic_chunks, search_full_text_chunks,
    search_hybrid_chunks, search_semantic_chunks,
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
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use simple_server_timing_header::Timer;
use tokio_stream::StreamExt;
use utoipa::ToSchema;

/// Boost is useful for when you want to boost certain phrases in the fulltext search results for official listings. I.e. making sure that the listing for AirBNB itself ranks higher than companies who make software for AirBNB hosts by boosting the AirBNB token for its official listing.
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct BoostPhrase {
    /// The phrase to boost in the fulltext document frequency index
    pub phrase: String,
    /// Amount to multiplicatevly increase the frequency of the tokens in the phrase by
    pub boost_factor: f64,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
#[schema(example = json!({
    "chunk_html": "<p>Some HTML content</p>",
    "link": "https://example.com",
    "tag_set": ["tag1", "tag2"],
    "metadata": {"key1": "value1", "key2": "value2"},
    "chunk_vector": [0.1, 0.2, 0.3],
    "tracking_id": "tracking_id",
    "upsert_by_tracking_id": true,
    "group_ids": ["d290f1ee-6c54-4b01-90e6-d701748f0851"],
    "group_tracking_ids": ["group_tracking_id"],
    "time_stamp": "2021-01-01T00:00:00",
    "location": {
        "lat": -34,
        "lon": 151
    },
    "image_urls": ["https://example.com/red", "https://example.com/blue"],
    "weight": 0.5,
    "split_avg": false,
    "convert_html_to_text": false,
    "boost_phrase": {"phrase": "HTML", "boost": 5.0}
}))]
pub struct ChunkReqPayload {
    /// HTML content of the chunk. This can also be plaintext. The innerText of the HTML will be used to create the embedding vector. The point of using HTML is for convienience, as some users have applications where users submit HTML content.
    pub chunk_html: Option<String>,
    /// Link to the chunk. This can also be any string. Frequently, this is a link to the source of the chunk. The link value will not affect the embedding creation.
    pub link: Option<String>,
    /// Tag set is a list of tags. This can be used to filter chunks by tag. Unlike with metadata filtering, HNSW indices will exist for each tag such that there is not a performance hit for filtering on them.
    pub tag_set: Option<Vec<String>>,
    /// Num value is an arbitrary numerical value that can be used to filter chunks. This is useful for when you want to filter chunks by numerical value. There is no performance hit for filtering on num_value.
    pub num_value: Option<f64>,
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
    /// Location is a GeoInfo object which lets you specify a latitude and longitude which can be used later to filter results.
    pub location: Option<GeoInfo>,
    /// Image urls are a list of urls to images that are associated with the chunk. This is useful for when you want to associate images with a chunk.
    pub image_urls: Option<Vec<String>>,
    /// Weight is a float which can be used to bias search results. This is useful for when you want to bias search results for a chunk. The magnitude only matters relative to other chunks in the chunk's dataset dataset.
    pub weight: Option<f64>,
    /// Split avg is a boolean which tells the server to split the text in the chunk_html into smaller chunks and average their resulting vectors. This is useful for when you want to create a chunk from a large piece of text and want to split it into smaller chunks to create a more fuzzy average dense vector. The sparse vector will be generated normally with no averaging. By default this is false.
    pub split_avg: Option<bool>,
    /// Convert HTML to raw text before processing to avoid adding noise to the vector embeddings. By default this is true. If you are using HTML content that you want to be included in the vector embeddings, set this to false.
    pub convert_html_to_text: Option<bool>,
    /// Boost is useful for when you want to boost certain phrases in the fulltext search results for official listings. I.e. making sure that the listing for AirBNB itself ranks higher than companies who make software for AirBNB hosts by boosting the AirBNB token for its official listing.
    pub boost_phrase: Option<BoostPhrase>,
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
    pub chunk: ChunkReqPayload,
    pub dataset_id: uuid::Uuid,
    pub dataset_config: ServerDatasetConfiguration,
    pub upsert_by_tracking_id: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BulkUploadIngestionMessage {
    pub attempt_number: usize,
    pub dataset_id: uuid::Uuid,
    pub dataset_configuration: ServerDatasetConfiguration,
    pub ingestion_messages: Vec<UploadIngestionMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "chunk_html": "<p>Some HTML content</p>",
    "link": "https://example.com",
    "tag_set": ["tag1", "tag2"],
    "metadata": {"key1": "value1", "key2": "value2"},
    "chunk_vector": [0.1, 0.2, 0.3],
    "tracking_id": "tracking_id",
    "upsert_by_tracking_id": true,
    "group_ids": ["d290f1ee-6c54-4b01-90e6-d701748f0851"],
    "group_tracking_ids": ["group_tracking_id"],
    "time_stamp": "2021-01-01T00:00:00",
    "location": {
        "lat": -34,
        "lon": 151
    },
    "weight": 0.5,
    "split_avg": false,
    "convert_html_to_text": false,
}))]
pub struct CreateSingleChunkReqPayload(pub ChunkReqPayload);

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!([{
    "chunk_html": "<p>Some HTML content</p>",
    "link": "https://example.com",
    "tag_set": ["tag1", "tag2"],
    "metadata": {"key1": "value1", "key2": "value2"},
    "chunk_vector": [0.1, 0.2, 0.3],
    "tracking_id": "tracking_id",
    "upsert_by_tracking_id": true,
    "group_ids": ["d290f1ee-6c54-4b01-90e6-d701748f0851"],
    "group_tracking_ids": ["group_tracking_id"],
    "time_stamp": "2021-01-01T00:00:00",
    "image_urls": ["https://example.com/red", "https://example.com/blue"],
    "location": {
        "lat": -34,
        "lon": 151
    },
    "weight": 0.5,
    "split_avg": false,
    "convert_html_to_text": false,
}, {
    "chunk_html": "<p>Some more HTML content</p>",
    "link": "https://explain.com",
    "tag_set": ["tag3", "tag4"],
    "metadata": {"key1": "value1", "key2": "value2"},
    "chunk_vector": [0.1, 0.2, 0.3],
    "tracking_id": "tracking_id",
    "upsert_by_tracking_id": true,
    "group_ids": ["d290f1ee-6c54-4b01-90e6-d701748f0851"],
    "group_tracking_ids": ["group_tracking_id"],
    "time_stamp": "2021-01-01T00:00:00",
    "image_urls": [],
    "location": {
        "lat": -34,
        "lon": 151
    },
    "weight": 0.5,
    "split_avg": false,
    "convert_html_to_text": false,
}]
))]
pub struct CreateBatchChunkReqPayload(pub Vec<ChunkReqPayload>);

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(untagged)]
pub enum CreateChunkReqPayloadEnum {
    Single(CreateSingleChunkReqPayload),
    Batch(CreateBatchChunkReqPayload),
}

/// Create or Upsert Chunk or Chunks
///
/// Create a new chunk. If the chunk has the same tracking_id as an existing chunk, the request will fail. Once a chunk is created, it can be searched for using the search endpoint.
/// If uploading in bulk, the maximum amount of chunks that can be uploaded at once is 120 chunks. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/chunk",
    context_path = "/api",
    tag = "chunk",
    request_body(content = CreateChunkReqPayloadEnum, description = "JSON request payload to create a new chunk (chunk)", content_type = "application/json"),
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
    create_chunk_data: web::Json<CreateChunkReqPayloadEnum>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let chunks = match create_chunk_data.clone() {
        CreateChunkReqPayloadEnum::Single(chunk) => vec![chunk.0],
        CreateChunkReqPayloadEnum::Batch(chunks) => chunks.0,
    };

    let count_dataset_id = dataset_org_plan_sub.dataset.id;

    let mut timer = Timer::new();
    let chunk_count = get_row_count_for_dataset_id_query(count_dataset_id, pool.clone()).await?;
    timer.add("get dataset count");

    if chunks.len() > 120 {
        return Err(ServiceError::BadRequest(
            "Too many chunks provided in bulk. The limit is 120 chunks per bulk upload".to_string(),
        )
        .into());
    }

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

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    timer.add("got redis connection");

    let (ingestion_message, chunk_metadatas) = create_chunk_metadata(
        chunks,
        dataset_org_plan_sub.dataset.id,
        server_dataset_configuration,
        pool,
    )
    .await?;

    let serialized_message: String = serde_json::to_string(&ingestion_message).map_err(|_| {
        ServiceError::BadRequest("Failed to Serialize BulkUploadMessage".to_string())
    })?;

    let pos_in_queue = redis::cmd("lpush")
        .arg("ingestion")
        .arg(&serialized_message)
        .query_async(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let response = match create_chunk_data.into_inner() {
        CreateChunkReqPayloadEnum::Single(_) => ReturnQueuedChunk::Single(SingleQueuedChunkResponse {
            chunk_metadata: chunk_metadatas
                .get(0)
                .ok_or(ServiceError::BadRequest(
                    "Failed to queue a single chunk due to deriving 0 ingestion_messages from the request data".to_string(),
                ))?
                .clone(),
            pos_in_queue,
        }),
        CreateChunkReqPayloadEnum::Batch(_) => ReturnQueuedChunk::Batch(BatchQueuedChunkResponse {
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
/// Delete a chunk by its id. If deleting a root chunk which has a collision, the most recently created collision will become a new root chunk. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
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
/// Delete a chunk by tracking_id. This is useful for when you are coordinating with an external system and want to use the tracking_id to identify the chunk. If deleting a root chunk which has a collision, the most recently created collision will become a new root chunk. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
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
pub struct UpdateChunkReqPayload {
    /// Id of the chunk you want to update. You can provide either the chunk_id or the tracking_id. If both are provided, the chunk_id will be used.
    chunk_id: Option<uuid::Uuid>,
    /// Tracking_id of the chunk you want to update. This is required to match an existing chunk.
    tracking_id: Option<String>,
    /// Tag set is a list of tags. This can be used to filter chunks by tag. Unlike with metadata filtering, HNSW indices will exist for each tag such that there is not a performance hit for filtering on them. If no tag_set is provided, the existing tag_set will be used.
    tag_set: Option<Vec<String>>,
    /// Link of the chunk you want to update. This can also be any string. Frequently, this is a link to the source of the chunk. The link value will not affect the embedding creation. If no link is provided, the existing link will be used.
    link: Option<String>,
    ///Num value is an arbitrary numerical value that can be used to filter chunks. This is useful for when you want to filter chunks by numerical value. If no num_value is provided, the existing num_value will be used.
    num_value: Option<f64>,
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
    /// Location is a GeoInfo object which lets you specify a latitude and longitude which can be used later to filter results.
    location: Option<GeoInfo>,
    /// Image urls are a list of urls to images that are associated with the chunk. This is useful for when you want to associate images with a chunk. If no image_urls are provided, the existing image_urls will be used.
    image_urls: Option<Vec<String>>,
    /// Convert HTML to raw text before processing to avoid adding noise to the vector embeddings. By default this is true. If you are using HTML content that you want to be included in the vector embeddings, set this to false.
    convert_html_to_text: Option<bool>,
    /// Boost is useful for when you want to boost certain phrases in the fulltext search results for official listings. I.e. making sure that the listing for AirBNB itself ranks higher than companies who make software for AirBNB hosts by boosting the AirBNB token for its official listing.
    pub boost_phrase: Option<BoostPhrase>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateIngestionMessage {
    pub chunk_metadata: ChunkMetadata,
    pub server_dataset_config: ServerDatasetConfiguration,
    pub dataset_id: uuid::Uuid,
    pub group_ids: Option<Vec<UnifiedId>>,
    pub convert_html_to_text: Option<bool>,
    pub boost_phrase: Option<BoostPhrase>,
}

/// Update Chunk
///
/// Update a chunk. If you try to change the tracking_id of the chunk to have the same tracking_id as an existing chunk, the request will fail. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    put,
    path = "/chunk",
    context_path = "/api",
    tag = "chunk",
    request_body(content = UpdateChunkReqPayload, description = "JSON request payload to update a chunk (chunk)", content_type = "application/json"),
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
    update_chunk_data: web::Json<UpdateChunkReqPayload>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let dataset_id = dataset_org_plan_sub.dataset.id;
    let chunk_id = update_chunk_data.chunk_id;

    let chunk_metadata = if let Some(chunk_id) = chunk_id {
        get_metadata_from_id_query(chunk_id, dataset_id, pool).await?
    } else if let Some(tracking_id) = update_chunk_data.tracking_id.clone() {
        get_metadata_from_tracking_id_query(tracking_id.clone(), dataset_id, pool).await?
    } else {
        return Err(ServiceError::BadRequest(
            "Either chunk_id or tracking_id must be provided to update a chunk".into(),
        )
        .into());
    };

    let link = update_chunk_data
        .link
        .clone()
        .unwrap_or_else(|| chunk_metadata.link.clone().unwrap_or_default());

    let chunk_tracking_id = update_chunk_data
        .tracking_id
        .clone()
        .filter(|chunk_tracking| !chunk_tracking.is_empty());

    let chunk_tag_set = update_chunk_data.tag_set.clone().map(|tag_set| {
        tag_set
            .into_iter()
            .map(Some)
            .collect::<Vec<Option<String>>>()
    });

    let chunk_html = match update_chunk_data.chunk_html.clone() {
        Some(chunk_html) => Some(chunk_html),
        None => chunk_metadata.chunk_html,
    };

    let chunk_metadata = ChunkMetadata::from_details_with_id(
        chunk_metadata.id,
        chunk_html,
        &Some(link),
        &chunk_tag_set.or(chunk_metadata.tag_set),
        chunk_metadata.qdrant_point_id,
        <std::option::Option<serde_json::Value> as Clone>::clone(&update_chunk_data.metadata)
            .or(chunk_metadata.metadata),
        chunk_tracking_id,
        update_chunk_data
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
        update_chunk_data.location.or(chunk_metadata.location),
        update_chunk_data.image_urls.clone().or(chunk_metadata
            .image_urls
            .map(|x| x.into_iter().map(|x| x.unwrap()).collect())),
        dataset_id,
        update_chunk_data.weight.unwrap_or(chunk_metadata.weight),
        update_chunk_data.num_value.or(chunk_metadata.num_value),
    );

    let group_ids = if let Some(group_ids) = update_chunk_data.group_ids.clone() {
        Some(
            group_ids
                .into_iter()
                .map(UnifiedId::from)
                .collect::<Vec<UnifiedId>>(),
        )
    } else {
        update_chunk_data
            .group_tracking_ids
            .clone()
            .map(|group_tracking_ids| {
                group_tracking_ids
                    .into_iter()
                    .map(UnifiedId::from)
                    .collect::<Vec<UnifiedId>>()
            })
    };

    let message = UpdateIngestionMessage {
        chunk_metadata: chunk_metadata.clone(),
        server_dataset_config,
        dataset_id,
        group_ids,
        convert_html_to_text: update_chunk_data.convert_html_to_text,
        boost_phrase: update_chunk_data.boost_phrase.clone(),
    };

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    redis::cmd("lpush")
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
    /// Convert HTML to raw text before processing to avoid adding noise to the vector embeddings. By default this is true. If you are using HTML content that you want to be included in the vector embeddings, set this to false.
    pub convert_html_to_text: Option<bool>,
}

/// Update Chunk By Tracking Id
///
/// Update a chunk by tracking_id. This is useful for when you are coordinating with an external system and want to use the tracking_id to identify the chunk. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
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
#[deprecated]
#[tracing::instrument(skip(pool, redis_pool))]
pub async fn update_chunk_by_tracking_id(
    update_chunk_data: web::Json<UpdateChunkByTrackingIdData>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    if update_chunk_data.tracking_id.is_empty() {
        return Err(ServiceError::BadRequest(
            "Tracking id must be provided to update by tracking_id".into(),
        )
        .into());
    }
    let tracking_id = update_chunk_data.tracking_id.clone();

    let dataset_id = dataset_org_plan_sub.dataset.id;

    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let chunk_metadata =
        get_metadata_from_tracking_id_query(tracking_id, dataset_id, pool.clone()).await?;

    let link = update_chunk_data
        .link
        .clone()
        .unwrap_or_else(|| chunk_metadata.link.clone().unwrap_or_default());

    let chunk_html = match update_chunk_data.chunk_html.clone() {
        Some(chunk_html) => Some(chunk_html),
        None => chunk_metadata.chunk_html,
    };

    let metadata = ChunkMetadata::from_details_with_id(
        chunk_metadata.id,
        chunk_html,
        &Some(link),
        &chunk_metadata.tag_set,
        chunk_metadata.qdrant_point_id,
        <std::option::Option<serde_json::Value> as Clone>::clone(&update_chunk_data.metadata)
            .or(chunk_metadata.metadata),
        Some(update_chunk_data.tracking_id.clone()),
        update_chunk_data
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
        None,
        None,
        dataset_org_plan_sub.dataset.id,
        update_chunk_data.weight.unwrap_or(1.0),
        None,
    );
    let group_ids = if let Some(group_ids) = update_chunk_data.group_ids.clone() {
        Some(
            group_ids
                .into_iter()
                .map(UnifiedId::from)
                .collect::<Vec<UnifiedId>>(),
        )
    } else {
        update_chunk_data
            .group_tracking_ids
            .clone()
            .map(|group_tracking_ids| {
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
        convert_html_to_text: update_chunk_data.convert_html_to_text,
        boost_phrase: None,
    };

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    redis::cmd("lpush")
        .arg("ingestion")
        .arg(serde_json::to_string(&message)?)
        .query_async(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(HttpResponse::NoContent().finish())
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
    pub should: Option<Vec<ConditionType>>,
    /// All of these field conditions have to match for the chunk to be included in the result set.
    pub must: Option<Vec<ConditionType>>,
    /// None of these field conditions can match for the chunk to be included in the result set.
    pub must_not: Option<Vec<ConditionType>>,
    /// JOSNB prefilter tells the server to perform a full scan over the metadata JSONB column instead of using the filtered HNSW. Datasets on the enterprise plan with custom metadata indices will perform better with the filtered HNSW instead. When false, the server will use the filtered HNSW index to filter chunks. When true, the server will perform a full scan over the metadata JSONB column to filter chunks. Default is true.
    pub jsonb_prefilter: Option<bool>,
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
            }
        ],
        "must": [
            {
                "field": "num_value",
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
pub struct SearchChunksReqPayload {
    /// Can be either "semantic", "fulltext", or "hybrid". If specified as "hybrid", it will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using BAAI/bge-reranker-large. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE.
    pub search_type: String,
    /// Query is the search query. This can be any string. The query will be used to create an embedding vector and/or SPLADE vector which will be used to find the result set.
    pub query: String,
    /// Page of chunks to fetch. Page is 1-indexed.
    pub page: Option<u64>,
    /// Page size is the number of chunks to fetch. This can be used to fetch more than 10 chunks at a time.
    pub page_size: Option<u64>,
    /// Get total page count for the query accounting for the applied filters. Defaults to false, but can be set to true when the latency penalty is acceptable (typically 50-200ms).
    pub get_total_pages: Option<bool>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Recency Bias lets you determine how much of an effect the recency of chunks will have on the search results. If not specified, this defaults to 0.0. We recommend setting this to 1.0 for a gentle reranking of the results, >3.0 for a strong reranking of the results.
    pub recency_bias: Option<f32>,
    /// Set use_weights to true to use the weights of the chunks in the result set in order to sort them. If not specified, this defaults to true.
    pub use_weights: Option<bool>,
    /// Tag weights is a JSON object which can be used to boost the ranking of chunks with certain tags. This is useful for when you want to be able to bias towards chunks with a certain tag on the fly. The keys are the tag names and the values are the weights.
    pub tag_weights: Option<HashMap<String, f32>>,
    /// Set get_collisions to true to get the collisions for each chunk. This will only apply if environment variable COLLISIONS_ENABLED is set to true.
    pub get_collisions: Option<bool>,
    /// Set highlight_results to false for a slight latency improvement (1-10ms). If not specified, this defaults to true. This will add `<b><mark>` tags to the chunk_html of the chunks to highlight matching splits and return the highlights on each scored chunk in the response.
    pub highlight_results: Option<bool>,
    /// Set highlight_threshold to a lower or higher value to adjust the sensitivity of the highlights applied to the chunk html. If not specified, this defaults to 0.8. The range is 0.0 to 1.0.
    pub highlight_threshold: Option<f64>,
    /// Set highlight_delimiters to a list of strings to use as delimiters for highlighting. If not specified, this defaults to ["?", ",", ".", "!"]. These are the characters that will be used to split the chunk_html into splits for highlighting. These are the characters that will be used to split the chunk_html into splits for highlighting.
    pub highlight_delimiters: Option<Vec<String>>,
    /// Set highlight_max_length to control the maximum number of tokens (typically whitespace separated strings, but sometimes also word stems) which can be present within a single highlight. If not specified, this defaults to 8. This is useful to shorten large splits which may have low scores due to length compared to the query. Set to something very large like 100 to highlight entire splits.
    pub highlight_max_length: Option<u32>,
    /// Set highlight_max_num to control the maximum number of highlights per chunk. If not specified, this defaults to 3. It may be less than 3 if no snippets score above the highlight_threshold.
    pub highlight_max_num: Option<u32>,
    /// Set highlight_window to a number to control the amount of words that are returned around the matched phrases. If not specified, this defaults to 0. This is useful for when you want to show more context around the matched words. When specified, window/2 whitespace separated words are added before and after each highlight in the response's highlights array. If an extended highlight overlaps with another highlight, the overlapping words are only included once.
    pub highlight_window: Option<u32>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold.
    pub score_threshold: Option<f32>,
    /// Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typically 10-50ms). Default is false.
    pub slim_chunks: Option<bool>,
    /// Set content_only to true to only returning the chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typically 10-50ms). Default is false.
    pub content_only: Option<bool>,
}

impl Default for SearchChunksReqPayload {
    fn default() -> Self {
        SearchChunksReqPayload {
            search_type: "hybrid".to_string(),
            query: "".to_string(),
            page: Some(1),
            get_total_pages: None,
            page_size: Some(10),
            filters: None,
            recency_bias: None,
            use_weights: None,
            tag_weights: None,
            get_collisions: None,
            highlight_results: None,
            highlight_threshold: None,
            highlight_delimiters: None,
            highlight_max_length: None,
            highlight_max_num: None,
            highlight_window: None,
            score_threshold: None,
            slim_chunks: None,
            content_only: None,
        }
    }
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

/// Search
///
/// This route provides the primary search functionality for the API. It can be used to search for chunks by semantic similarity, full-text similarity, or a combination of both. Results' `chunk_html` values will be modified with `<b><mark>` tags for sub-sentence highlighting.
#[utoipa::path(
    post,
    path = "/chunk/search",
    context_path = "/api",
    tag = "chunk",
    request_body(content = SearchChunksReqPayload, description = "JSON request payload to semantically search for chunks (chunks)", content_type = "application/json"),
    responses(
        (status = 200, description = "Chunks with embedding vectors which are similar to those in the request body", body = SearchChunkQueryResponseBody),

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
pub async fn search_chunks(
    data: web::Json<SearchChunksReqPayload>,
    _user: LoggedUser,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let parsed_query = parse_query(data.query.clone());

    let tx_ctx = sentry::TransactionContext::new("search", "search_chunks");
    let transaction = sentry::start_transaction(tx_ctx);
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone().into())));
    let mut timer = Timer::new();

    let result_chunks = match data.search_type.as_str() {
        "fulltext" | "full-text" | "full_text" | "full text" => {
            if !server_dataset_config.FULLTEXT_ENABLED {
                return Err(ServiceError::BadRequest(
                    "Fulltext search is not enabled for this dataset".into(),
                )
                .into());
            }

            search_full_text_chunks(
                data.clone(),
                parsed_query,
                pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
                &mut timer,
            )
            .await?
        }
        "hybrid" => {
            search_hybrid_chunks(
                data.clone(),
                parsed_query,
                pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
                &mut timer,
            )
            .await?
        }
        "semantic" => {
            search_semantic_chunks(
                data.clone(),
                parsed_query,
                pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
                &mut timer,
            )
            .await?
        }
        _ => {
            return Err(ServiceError::BadRequest(
                "Invalid search type. Must be one of 'semantic', 'fulltext', or 'hybrid'".into(),
            )
            .into())
        }
    };

    transaction.finish();

    Ok(HttpResponse::Ok()
        .insert_header((Timer::header_key(), timer.header_value()))
        .json(result_chunks))
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
pub struct AutocompleteReqPayload {
    /// Can be either "semantic", or "fulltext". "semantic" will pull in one page_size of the nearest cosine distant vectors. "fulltext" will pull in one page_size of full-text results based on SPLADE.
    pub search_type: String,
    /// If specified to true, this will extend the search results to include non-exact prefix matches of the same search_type such that a full page_size of results are returned. Default is false.
    pub extend_results: Option<bool>,
    /// Query is the search query. This can be any string. The query will be used to create an embedding vector and/or SPLADE vector which will be used to find the result set.
    pub query: String,
    /// Page size is the number of chunks to fetch. This can be used to fetch more than 10 chunks at a time.
    pub page_size: Option<u64>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Recency Bias lets you determine how much of an effect the recency of chunks will have on the search results. If not specified, this defaults to 0.0. We recommend setting this to 1.0 for a gentle reranking of the results, >3.0 for a strong reranking of the results.
    pub recency_bias: Option<f32>,
    /// Set use_weights to true to use the weights of the chunks in the result set in order to sort them. If not specified, this defaults to true.
    pub use_weights: Option<bool>,
    /// Tag weights is a JSON object which can be used to boost the ranking of chunks with certain tags. This is useful for when you want to be able to bias towards chunks with a certain tag on the fly. The keys are the tag names and the values are the weights.
    pub tag_weights: Option<HashMap<String, f32>>,
    /// Set highlight_results to false for a slight latency improvement (1-10ms). If not specified, this defaults to true. This will add `<b><mark>` tags to the chunk_html of the chunks to highlight matching splits and return the highlights on each scored chunk in the response.
    pub highlight_results: Option<bool>,
    /// Set highlight_threshold to a lower or higher value to adjust the sensitivity of the highlights applied to the chunk html. If not specified, this defaults to 0.8. The range is 0.0 to 1.0.
    pub highlight_threshold: Option<f64>,
    /// Set highlight_delimiters to a list of strings to use as delimiters for highlighting. If not specified, this defaults to ["?", ",", ".", "!"]. These are the characters that will be used to split the chunk_html into splits for highlighting.
    pub highlight_delimiters: Option<Vec<String>>,
    /// Set highlight_max_length to control the maximum number of tokens (typically whitespace separated strings, but sometimes also word stems) which can be present within a single highlight. If not specified, this defaults to 8. This is useful to shorten large splits which may have low scores due to length compared to the query. Set to something very large like 100 to highlight entire splits.
    pub highlight_max_length: Option<u32>,
    /// Set highlight_max_num to control the maximum number of highlights per chunk. If not specified, this defaults to 3. It may be less than 3 if no snippets score above the highlight_threshold.
    pub highlight_max_num: Option<u32>,
    /// Set highlight_window to a number to control the amount of words that are returned around the matched phrases. If not specified, this defaults to 0. This is useful for when you want to show more context around the matched words. When specified, window/2 whitespace separated words are added before and after each highlight in the response's highlights array. If an extended highlight overlaps with another highlight, the overlapping words are only included once.
    pub highlight_window: Option<u32>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold.
    pub score_threshold: Option<f32>,
    /// Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typically 10-50ms). Default is false.
    pub slim_chunks: Option<bool>,
    /// Set content_only to true to only returning the chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typically 10-50ms). Default is false.
    pub content_only: Option<bool>,
}

impl From<AutocompleteReqPayload> for SearchChunksReqPayload {
    fn from(autocomplete_data: AutocompleteReqPayload) -> Self {
        SearchChunksReqPayload {
            search_type: autocomplete_data.search_type,
            query: autocomplete_data.query,
            page: Some(1),
            get_total_pages: None,
            page_size: autocomplete_data.page_size,
            filters: autocomplete_data.filters,
            recency_bias: autocomplete_data.recency_bias,
            use_weights: autocomplete_data.use_weights,
            tag_weights: autocomplete_data.tag_weights,
            get_collisions: None,
            highlight_results: autocomplete_data.highlight_results,
            highlight_threshold: autocomplete_data.highlight_threshold,
            highlight_delimiters: Some(
                autocomplete_data
                    .highlight_delimiters
                    .unwrap_or(vec![" ".to_string()]),
            ),
            highlight_max_length: autocomplete_data.highlight_max_length,
            highlight_max_num: autocomplete_data.highlight_max_num,
            highlight_window: autocomplete_data.highlight_window,
            score_threshold: autocomplete_data.score_threshold,
            slim_chunks: autocomplete_data.slim_chunks,
            content_only: autocomplete_data.content_only,
        }
    }
}

/// Autocomplete
///
/// This route provides the primary autocomplete functionality for the API. This prioritize prefix matching with semantic or full-text search.
#[utoipa::path(
    post,
    path = "/chunk/autocomplete",
    context_path = "/api",
    tag = "chunk",
    request_body(content = AutocompleteReqPayload, description = "JSON request payload to semantically search for chunks (chunks)", content_type = "application/json"),
    responses(
        (status = 200, description = "Chunks with embedding vectors which are similar to those in the request body", body = SearchChunkQueryResponseBody),

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
pub async fn autocomplete(
    data: web::Json<AutocompleteReqPayload>,
    _user: LoggedUser,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let parsed_query = parse_query(data.query.clone());

    let tx_ctx = sentry::TransactionContext::new("search", "search_chunks");
    let transaction = sentry::start_transaction(tx_ctx);
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone().into())));
    let mut timer = Timer::new();

    let result_chunks = match data.search_type.as_str() {
        "fulltext" | "full-text" | "full_text" | "full text" => {
            if !server_dataset_config.FULLTEXT_ENABLED {
                return Err(ServiceError::BadRequest(
                    "Fulltext search is not enabled for this dataset".into(),
                )
                .into());
            }

            autocomplete_fulltext_chunks(
                data.clone(),
                parsed_query,
                pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
                &mut timer,
            )
            .await?
        }
        "semantic" => {
            autocomplete_semantic_chunks(
                data.clone(),
                parsed_query,
                pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
                &mut timer,
            )
            .await?
        }
        _ => {
            return Err(ServiceError::BadRequest(
                "Invalid search type. Must be one of 'semantic', or 'fulltext'".into(),
            )
            .into())
        }
    };

    transaction.finish();

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
        (status = 200, description = "chunk with the id that you were searching for", body = ChunkMetadataStringTagSet),
        (status = 400, description = "Service error relating to fidning a chunk by tracking_id", body = ErrorResponseBody),
        (status = 404, description = "Chunk not found", body = ErrorResponseBody)
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
) -> Result<HttpResponse, ServiceError> {
    let chunk_id = chunk_id.into_inner();

    let dataset_configuration = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let chunk = get_metadata_from_id_query(chunk_id, dataset_org_plan_sub.dataset.id, pool).await?;
    let chunk_string_tag_set = ChunkMetadataStringTagSet::from(chunk);

    let point_id = chunk_string_tag_set.qdrant_point_id;
    let pointid_exists = if let Some(point_id) = point_id {
        point_ids_exists_in_qdrant(vec![point_id], dataset_configuration).await?
    } else {
        // This is a collision, assume collisions always exist in qdrant
        true
    };

    if pointid_exists {
        Ok(HttpResponse::Ok().json(chunk_string_tag_set))
    } else {
        Err(ServiceError::NotFound("Chunk not found".to_string()))
    }
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
        (status = 200, description = "chunk with the tracking_id that you were searching for", body = ChunkMetadataStringTagSet),
        (status = 400, description = "Service error relating to fidning a chunk by tracking_id", body = ErrorResponseBody),
        (status = 404, description = "Chunk not found", body = ErrorResponseBody)
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
) -> Result<HttpResponse, ServiceError> {
    let dataset_configuration = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let chunk = get_metadata_from_tracking_id_query(
        tracking_id.into_inner(),
        dataset_org_plan_sub.dataset.id,
        pool,
    )
    .await?;
    let chunk_tag_set_string = ChunkMetadataStringTagSet::from(chunk);

    let point_id = chunk_tag_set_string.qdrant_point_id;

    let pointid_exists = if let Some(point_id) = point_id {
        point_ids_exists_in_qdrant(vec![point_id], dataset_configuration).await?
    } else {
        // This is a collision, assume collisions always exist in qdrant
        true
    };

    if pointid_exists {
        Ok(HttpResponse::Ok().json(chunk_tag_set_string))
    } else {
        Err(ServiceError::NotFound("Chunk not found".to_string()))
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GetChunksData {
    pub ids: Vec<uuid::Uuid>,
}

/// Get Chunks By Ids
///
/// Get multiple chunks by multiple ids.
#[utoipa::path(
    post,
    path = "/chunks",
    context_path = "/api",
    tag = "chunk",
    request_body(content = GetChunksData, description = "JSON request payload to get the chunks in the request", content_type = "application/json"),
    responses(
        (status = 200, description = "chunks with the id that you were searching for", body = Vec<ChunkMetadataStringTagSet>),
        (status = 400, description = "Service error relating to fidning a chunk by tracking_id", body = ErrorResponseBody),
        (status = 404, description = "Any one of the specified chunks not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_chunks_by_ids(
    chunk_payload: web::Json<GetChunksData>,
    _user: LoggedUser,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let dataset_configuration = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let chunks = get_metadata_from_ids_query(
        chunk_payload.ids.clone(),
        dataset_org_plan_sub.dataset.id,
        pool,
    )
    .await?;
    let chunk_string_tag_sets = chunks
        .into_iter()
        .map(ChunkMetadataStringTagSet::from)
        .collect::<Vec<ChunkMetadataStringTagSet>>();

    let point_ids = chunk_string_tag_sets
        .iter()
        .filter_map(|x| x.qdrant_point_id)
        .collect();

    let pointids_exists = point_ids_exists_in_qdrant(point_ids, dataset_configuration).await?;

    if pointids_exists {
        Ok(HttpResponse::Ok().json(chunk_string_tag_sets))
    } else {
        Err(ServiceError::NotFound(
            "Any one of the specified chunks not found".to_string(),
        ))
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GetTrackingChunksData {
    pub tracking_ids: Vec<String>,
}

/// Get Chunks By TrackingIds
///
/// Get multiple chunks by ids.
#[utoipa::path(
    post,
    path = "/chunks/tracking",
    context_path = "/api",
    tag = "chunk",
    request_body(content = GetTrackingChunksData, description = "JSON request payload to get the chunks in the request", content_type = "application/json"),
    responses(
        (status = 200, description = "Chunks with one the ids which were specified", body = Vec<ChunkMetadataStringTagSet>),
        (status = 400, description = "Service error relating to finding a chunk by tracking_id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_chunks_by_tracking_ids(
    chunk_payload: web::Json<GetTrackingChunksData>,
    _user: LoggedUser,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let dataset_configuration = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let chunks = get_metadata_from_tracking_ids_query(
        chunk_payload.tracking_ids.clone(),
        dataset_org_plan_sub.dataset.id,
        pool,
    )
    .await?;
    let chunk_string_tag_sets = chunks
        .into_iter()
        .map(ChunkMetadataStringTagSet::from)
        .collect::<Vec<ChunkMetadataStringTagSet>>();

    let point_ids = chunk_string_tag_sets
        .iter()
        .filter_map(|x| x.qdrant_point_id)
        .collect();

    let pointids_exists = point_ids_exists_in_qdrant(point_ids, dataset_configuration).await?;

    if pointids_exists {
        Ok(HttpResponse::Ok().json(chunk_string_tag_sets))
    } else {
        Err(ServiceError::NotFound(
            "Any one of the specified chunks not found".to_string(),
        ))
    }
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
    /// Strategy to use for recommendations, either "average_vector" or "best_score". The default is "average_vector". The "average_vector" strategy will construct a single average vector from the positive and negative samples then use it to perform a pseudo-search. The "best_score" strategy is more advanced and navigates the HNSW with a heuristic of picking edges where the point is closer to the positive samples than it is the negatives.
    pub strategy: Option<String>,
    /// The type of recommendation to make. This lets you choose whether to recommend based off of `semantic` or `fulltext` similarity. The default is `semantic`.
    pub recommend_type: Option<String>,
    /// Filters to apply to the chunks to be recommended. This is a JSON object which contains the filters to apply to the chunks to be recommended. The default is None.
    pub filters: Option<ChunkFilter>,
    /// The number of chunks to return. This is the number of chunks which will be returned in the response. The default is 10.
    pub limit: Option<u64>,
    /// Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typicall 10-50ms). Default is false.
    pub slim_chunks: Option<bool>,
}

/// Get Recommended Chunks
///
/// Get recommendations of chunks similar to the positive samples in the request and dissimilar to the negative. You must provide at least one of either positive_chunk_ids or positive_tracking_ids.
#[utoipa::path(
    post,
    path = "/chunk/recommend",
    context_path = "/api",
    tag = "chunk",
    request_body(content = RecommendChunksRequest, description = "JSON request payload to get recommendations of chunks similar to the chunks in the request", content_type = "application/json"),
    responses(

        (status = 200, description = "Chunks with embedding vectors which are similar to positives and dissimilar to negatives", body = Vec<ChunkMetadataWithScore>),
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

    if positive_chunk_ids.is_none() && positive_tracking_ids.is_none() {
        return Err(ServiceError::BadRequest(
            "Either positive_chunk_ids or positive_tracking_ids must be provided".to_string(),
        )
        .into());
    }

    let dataset_id = dataset_org_plan_sub.dataset.id;

    let mut positive_qdrant_ids = vec![];

    let mut timer = Timer::new();

    timer.add("start extending tracking_ids and chunk_ids to qdrant_point_ids");

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

    timer.add("fetched ids from postgres");

    let recommended_qdrant_results = recommend_qdrant_query(
        positive_qdrant_ids,
        negative_qdrant_ids,
        data.strategy.clone(),
        data.recommend_type.clone(),
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

    timer.add("recommend_qdrant_query");

    let recommended_chunk_metadatas = match data.slim_chunks {
        Some(true) => {
            let slim_chunks = get_slim_chunks_from_point_ids_query(
                recommended_qdrant_results
                    .clone()
                    .into_iter()
                    .map(|recommend_qdrant_result| recommend_qdrant_result.point_id)
                    .collect(),
                pool,
            )
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not get recommended slim chunk_metadatas from qdrant_point_ids: {}",
                    err
                ))
            })?;

            slim_chunks.into_iter().map(|chunk| chunk.into()).collect()
        }
        _ => get_chunk_metadatas_from_point_ids(
            recommended_qdrant_results
                .clone()
                .into_iter()
                .map(|recommend_qdrant_result| recommend_qdrant_result.point_id)
                .collect(),
            pool,
        )
        .await
        .map_err(|err| {
            ServiceError::BadRequest(format!(
                "Could not get recommended chunk_metadatas from qdrant_point_ids: {}",
                err
            ))
        })?,
    };

    let recommended_chunk_metadatas_with_score = recommended_chunk_metadatas
        .into_iter()
        .map(|chunk_metadata| {
            let score = recommended_qdrant_results
                .iter()
                .find(|recommend_qdrant_result| {
                    recommend_qdrant_result.point_id
                        == chunk_metadata.qdrant_point_id.unwrap_or_default()
                })
                .map(|recommend_qdrant_result| recommend_qdrant_result.score)
                .unwrap_or(0.0);

            ChunkMetadataWithScore::from((chunk_metadata, score))
        })
        .collect::<Vec<ChunkMetadataWithScore>>();

    let recommended_chunk_metadatas_with_score = recommended_chunk_metadatas_with_score
        .into_iter()
        .sorted_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .collect::<Vec<ChunkMetadataWithScore>>();

    timer.add("fetched metadata from point_ids");

    if data.slim_chunks.unwrap_or(false) {
        let res = recommended_chunk_metadatas_with_score
            .into_iter()
            .map(|chunk| chunk.into())
            .collect::<Vec<SlimChunkMetadataWithScore>>();

        return Ok(HttpResponse::PartialContent().json(res));
    }

    Ok(HttpResponse::Ok()
        .insert_header((Timer::header_key(), timer.header_value()))
        .json(recommended_chunk_metadatas_with_score))
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
    /// The previous messages to be placed into the chat history. The last message in this array will be the prompt for the model to inference on. The length of this array must be at least 1.
    pub prev_messages: Vec<ChatMessageProxy>,
    /// The ids of the chunks to be retrieved and injected into the context window for RAG.
    pub chunk_ids: Vec<uuid::Uuid>,
    /// Prompt for the last message in the prev_messages array. This will be used to generate the next message in the chat. The default is 'Respond to the instruction and include the doc numbers that you used in square brackets at the end of the sentences that you used the docs for:'. You can also specify an empty string to leave the final message alone such that your user's final message can be used as the prompt. See docs.trieve.ai or contact us for more information.
    pub prompt: Option<String>,
    /// Whether or not to stream the response. If this is set to true or not included, the response will be a stream. If this is set to false, the response will be a normal JSON response. Default is true.
    pub stream_response: Option<bool>,
    /// Set highlight_results to false for a slight latency improvement (1-10ms). If not specified, this defaults to true. This will add `<b><mark>`` tags to the chunk_html of the chunks to highlight matching splits.
    pub highlight_results: Option<bool>,
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
        http_client: reqwest::Client::new(),
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
        let content = convert_html_to_text(&(bookmark.chunk_html.clone().unwrap_or_default()));
        let first_2000_words = content
            .split_whitespace()
            .take(2000)
            .collect::<Vec<_>>()
            .join(" ");

        messages.push(ChatMessage {
            role: Role::User,
            content: ChatMessageContent::Text(format!("Doc {}: {}", idx + 1, first_2000_words)),
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
        model: default_model,
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
