use super::auth_handler::{AdminOnly, LoggedUser};
use crate::data::models::{
    ChatMessageProxy, ChunkGroup, ChunkMetadata, ChunkMetadataWithFileData,
    DatasetAndOrgWithSubAndPlan, Pool, ServerDatasetConfiguration,
};
use crate::errors::{DefaultError, ServiceError};
use crate::get_env;
use crate::operators::chunk_operator::get_metadata_from_id_query;
use crate::operators::chunk_operator::*;
use crate::operators::group_operator::get_group_by_id_query;
use crate::operators::model_operator::create_embedding;
use crate::operators::qdrant_operator::recommend_qdrant_query;
use crate::operators::qdrant_operator::update_qdrant_point_query;
use crate::operators::search_operator::{
    search_full_text_chunks, search_full_text_groups, search_hybrid_chunks, search_hybrid_groups,
    search_semantic_chunks, search_semantic_groups,
};
use actix_web::web::Bytes;
use actix_web::{web, HttpResponse};
use chrono::NaiveDateTime;
use dateparser::DateTimeUtc;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::chat::{
    ChatCompletionParameters, ChatMessage, ChatMessageContent, Role,
};
use redis::Commands;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Command;
use tokio_stream::StreamExt;
use utoipa::{IntoParams, ToSchema};

pub async fn user_owns_chunk(
    user_id: uuid::Uuid,
    chunk_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, actix_web::Error> {
    let chunks = web::block(move || get_metadata_from_id_query(chunk_id, dataset_id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if chunks.author_id != user_id {
        return Err(ServiceError::Forbidden.into());
    }

    Ok(chunks)
}

pub async fn user_owns_chunk_tracking_id(
    user_id: uuid::Uuid,
    tracking_id: String,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, actix_web::Error> {
    let chunks =
        web::block(move || get_metadata_from_tracking_id_query(tracking_id, dataset_id, pool))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if chunks.author_id != user_id {
        return Err(ServiceError::Forbidden.into());
    }

    Ok(chunks)
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct CreateChunkData {
    /// HTML content of the chunk. This can also be plaintext. The innerText of the HTML will be used to create the embedding vector. The point of using HTML is for convienience, as some users have applications where users submit HTML content.
    pub chunk_html: Option<String>,
    /// Link to the chunk. This can also be any string. Frequently, this is a link to the source of the chunk. The link value will not affect the embedding creation.
    pub link: Option<String>,
    /// Tag set is a comma separated list of tags. This can be used to filter chunks by tag. Unlike with metadata filtering, HNSW indices will exist for each tag such that there is not a performance hit for filtering on them.
    pub tag_set: Option<String>,
    /// File_uuid is the uuid of the file that the chunk is associated with. This is used to associate chunks with files. This is useful for when you want to delete a file and all of its associated chunks.
    pub file_uuid: Option<uuid::Uuid>,
    /// Metadata is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub metadata: Option<serde_json::Value>,
    /// Chunk_vector is a vector of floats which can be used instead of generating a new embedding. This is useful for when you are using a pre-embedded dataset. If this is not provided, the innerText of the chunk_html will be used to create the embedding.
    pub chunk_vector: Option<Vec<f32>>,
    /// Tracking_id is a string which can be used to identify a chunk. This is useful for when you are coordinating with an external system and want to use the tracking_id to identify the chunk.
    pub tracking_id: Option<String>,
    /// Group is the id of the group that the chunk should be placed into. This is useful for when you want to create a chunk and add it to a group in one request.
    pub group_id: Option<uuid::Uuid>,
    /// Time_stamp should be an ISO 8601 combined date and time without timezone. It is used for time window filtering and recency-biasing search results.
    pub time_stamp: Option<String>,
    /// Weight is a float which can be used to bias search results. This is useful for when you want to bias search results for a chunk. The magnitude only matters relative to other chunks in the chunk's dataset dataset.
    pub weight: Option<f64>,
}

pub fn convert_html(html: &str) -> Result<String, DefaultError> {
    let html_parse_result = Command::new("./server-python/html-converter.py")
        .arg(html)
        .output();

    let content = match html_parse_result {
        Ok(result) => {
            if result.status.success() {
                Some(
                    String::from_utf8(result.stdout)
                        .unwrap()
                        .lines()
                        .collect::<Vec<&str>>()
                        .join(" ")
                        .trim_end()
                        .to_string(),
                )
            } else {
                return Err(DefaultError {
                    message: "Could not parse html",
                });
            }
        }
        Err(_) => {
            return Err(DefaultError {
                message: "Could not parse html",
            });
        }
    };

    match content {
        Some(content) => Ok(content),
        None => Err(DefaultError {
            message: "Could not parse html",
        }),
    }
}
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ReturnCreatedChunk {
    pub chunk_metadata: ChunkMetadata,
    pub pos_in_queue: i32,
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct IngestionMessage {
    pub chunk_metadata: ChunkMetadata,
    pub chunk: CreateChunkData,
    pub dataset_config: ServerDatasetConfiguration,
}

/// create_chunk
///
/// Create a new chunk. If the chunk has the same tracking_id as an existing chunk, the request will fail. Once a chunk is created, it can be searched for using the search endpoint.
#[utoipa::path(
    post,
    path = "/chunk",
    context_path = "/api",
    tag = "chunk",
    request_body(content = CreateChunkData, description = "JSON request payload to create a new chunk (chunk)", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON response payload containing the created chunk", body = ReturnCreatedChunk),
        (status = 400, description = "Service error relating to to creating a chunk, likely due to conflicting tracking_id", body = DefaultError),
    )
)]
pub async fn create_chunk(
    chunk: web::Json<CreateChunkData>,
    pool: web::Data<Pool>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    redis_client: web::Data<redis::Client>,
) -> Result<HttpResponse, actix_web::Error> {
    let count_dataset_id = dataset_org_plan_sub.dataset.id;

    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);

    let chunk_count = {
        let pool = pool.clone();
        web::block(move || get_row_count_for_dataset_id_query(count_dataset_id, pool))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?
    };

    if chunk_count
        >= dataset_org_plan_sub
            .organization
            .plan
            .unwrap_or_default()
            .chunk_count
    {
        return Ok(HttpResponse::UpgradeRequired()
            .json(json!({"message": "Must upgrade your plan to add more chunks"})));
    }

    let chunk_tracking_id = chunk
        .tracking_id
        .clone()
        .filter(|chunk_tracking| !chunk_tracking.is_empty());

    let content =
        convert_html(chunk.chunk_html.as_ref().unwrap_or(&"".to_string())).map_err(|err| {
            ServiceError::BadRequest(format!("Could not parse html: {}", err.message))
        })?;

    let chunk_metadata = ChunkMetadata::from_details(
        content,
        &chunk.chunk_html,
        &chunk.link,
        &chunk.tag_set,
        user.0.id,
        None,
        chunk.metadata.clone(),
        chunk_tracking_id,
        chunk
            .time_stamp
            .clone()
            .map(|ts| -> Result<NaiveDateTime, ServiceError> {
                Ok(ts
                    .parse::<DateTimeUtc>()
                    .map_err(|_| ServiceError::BadRequest("Invalid timestamp format".to_string()))?
                    .0
                    .with_timezone(&chrono::Local)
                    .naive_local())
            })
            .transpose()?,
        dataset_org_plan_sub.dataset.id,
        0.0,
    );

    let ingestion_message = IngestionMessage {
        chunk_metadata: chunk_metadata.clone(),
        chunk: chunk.clone(),
        dataset_config,
    };

    let mut pub_client = redis_client
        .get_connection()
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    pub_client
        .lpush("ingestion", serde_json::to_string(&ingestion_message)?)
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let pos_in_queue = pub_client
        .llen("ingestion")
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(HttpResponse::Ok().json(ReturnCreatedChunk {
        chunk_metadata: chunk_metadata.clone(),
        pos_in_queue,
    }))
}

/// bulk_create_chunk
///
/// Create a new chunk from an array of chunks. If the chunk has the same tracking_id as an existing chunk, the request will fail. Once a chunk is created, it can be searched for using the search endpoint.
#[utoipa::path(
    post,
    path = "/chunk",
    context_path = "/api",
    tag = "chunk",
    request_body(content = CreateChunkData, description = "JSON request payload to create a new chunk (chunk)", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON response payload containing the created chunk", body = ReturnCreatedChunk),
        (status = 400, description = "Service error relating to to creating a chunk, likely due to conflicting tracking_id", body = DefaultError),
    )
)]
pub async fn bulk_create_chunk(
    chunks: web::Json<Vec<CreateChunkData>>,
    pool: web::Data<Pool>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    redis_client: web::Data<redis::Client>,
) -> Result<HttpResponse, actix_web::Error> {
    for chunk in chunks.into_inner() {
        create_chunk(
            actix_web::web::Json(chunk),
            pool.clone(),
            user.clone(),
            dataset_org_plan_sub.clone(),
            redis_client.clone(),
        )
        .await?;
    }

    Ok(HttpResponse::NoContent().finish())
}

/// delete_chunk
///
/// Delete a chunk by its id. If deleting a root chunk which has a collision, the most recently created collision will become a new root chunk.
#[utoipa::path(
    delete,
    path = "/chunk/{chunk_id}",
    context_path = "/api",
    tag = "chunk",
    responses(
        (status = 204, description = "Confirmation that the chunk with the id specified was deleted"),
        (status = 400, description = "Service error relating to finding a chunk by tracking_id", body = DefaultError),
    ),
    params(
        ("chunk_id" = Option<uuid>, Path, description = "id of the chunk you want to delete")
    ),
)]
pub async fn delete_chunk(
    chunk_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let chunk_id_inner = chunk_id.into_inner();
    let pool1 = pool.clone();
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let _ = user_owns_chunk(user.0.id, chunk_id_inner, dataset_id, pool).await?;

    delete_chunk_metadata_query(chunk_id_inner, dataset_org_plan_sub.dataset, pool1)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

/// delete_chunk_by_tracking_id
///
/// Delete a chunk by tracking_id. This is useful for when you are coordinating with an external system and want to use the tracking_id to identify the chunk. If deleting a root chunk which has a collision, the most recently created collision will become a new root chunk.
#[utoipa::path(
    delete,
    path = "/chunk/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "chunk",
    responses(
        (status = 204, description = "Confirmation that the chunk with the tracking_id specified was deleted"),
        (status = 400, description = "Service error relating to finding a chunk by tracking_id", body = DefaultError),
    ),
    params(
        ("tracking_id" = Option<String>, Path, description = "tracking_id of the chunk you want to delete")
    ),
)]
pub async fn delete_chunk_by_tracking_id(
    tracking_id: web::Path<String>,
    pool: web::Data<Pool>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let tracking_id_inner = tracking_id.into_inner();
    let pool1 = pool.clone();
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let chunk_metadata =
        user_owns_chunk_tracking_id(user.0.id, tracking_id_inner, dataset_id, pool).await?;

    delete_chunk_metadata_query(chunk_metadata.id, dataset_org_plan_sub.dataset, pool1)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateChunkData {
    /// Id of the chunk you want to update.
    chunk_uuid: uuid::Uuid,
    /// Link of the chunk you want to update. This can also be any string. Frequently, this is a link to the source of the chunk. The link value will not affect the embedding creation. If no link is provided, the existing link will be used.
    link: Option<String>,
    /// HTML content of the chunk you want to update. This can also be plaintext. The innerText of the HTML will be used to create the embedding vector. The point of using HTML is for convienience, as some users have applications where users submit HTML content. If no chunk_html is provided, the existing chunk_html will be used.
    chunk_html: Option<String>,
    /// The metadata is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata. If no metadata is provided, the existing metadata will be used.
    metadata: Option<serde_json::Value>,
    /// Tracking_id is a string which can be used to identify a chunk. This is useful for when you are coordinating with an external system and want to use the tracking_id to identify the chunk. If no tracking_id is provided, the existing tracking_id will be used.
    tracking_id: Option<String>,
    /// Time_stamp should be an ISO 8601 combined date and time without timezone. It is used for time window filtering and recency-biasing search results. If no time_stamp is provided, the existing time_stamp will be used.
    time_stamp: Option<String>,
    /// Weight is a float which can be used to bias search results. This is useful for when you want to bias search results for a chunk. The magnitude only matters relative to other chunks in the chunk's dataset dataset. If no weight is provided, the existing weight will be used.
    weight: Option<f64>,
}
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ChunkHtmlUpdateError {
    pub message: String,
    changed_content: String,
}

/// update_chunk
///
/// Update a chunk. If you try to change the tracking_id of the chunk to have the same tracking_id as an existing chunk, the request will fail.
#[utoipa::path(
    put,
    path = "/chunk/update",
    context_path = "/api",
    tag = "chunk",
    request_body(content = UpdateChunkData, description = "JSON request payload to update a chunk (chunk)", content_type = "application/json"),
    responses(
        (status = 204, description = "No content Ok response indicating the chunk was updated as requested",),
        (status = 400, description = "Service error relating to to updating chunk, likely due to conflicting tracking_id", body = DefaultError),
    )
)]
pub async fn update_chunk(
    chunk: web::Json<UpdateChunkData>,
    pool: web::Data<Pool>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let chunk_metadata = user_owns_chunk(user.0.id, chunk.chunk_uuid, dataset_id, pool).await?;

    let link = chunk
        .link
        .clone()
        .unwrap_or_else(|| chunk_metadata.link.clone().unwrap_or_default());
    let chunk_tracking_id = chunk
        .tracking_id
        .clone()
        .filter(|chunk_tracking| !chunk_tracking.is_empty());

    let new_content = convert_html(chunk.chunk_html.as_ref().unwrap_or(&chunk_metadata.content))
        .map_err(|err| {
            ServiceError::BadRequest(format!("Could not parse html: {}", err.message))
        })?;

    let embedding_vector = create_embedding(
        &new_content,
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration),
    )
    .await?;

    let chunk_html = match chunk.chunk_html.clone() {
        Some(chunk_html) => Some(chunk_html),
        None => chunk_metadata.chunk_html,
    };

    let chunk_id1 = chunk.chunk_uuid;
    let qdrant_point_id = web::block(move || get_qdrant_id_from_chunk_id_query(chunk_id1, pool1))
        .await?
        .map_err(|_| ServiceError::BadRequest("chunk not found".into()))?;

    let metadata = ChunkMetadata::from_details_with_id(
        chunk.chunk_uuid,
        &new_content,
        &chunk_html,
        &Some(link),
        &chunk_metadata.tag_set,
        user.0.id,
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
    let metadata1 = metadata.clone();
    update_chunk_metadata_query(metadata, None, dataset_id, pool2)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    update_qdrant_point_query(
        // If the chunk is a collision, we don't want to update the qdrant point
        if chunk_metadata.qdrant_point_id.is_none() {
            None
        } else {
            Some(metadata1)
        },
        qdrant_point_id,
        Some(user.0.id),
        Some(embedding_vector),
        dataset_id,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
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
}

/// update_chunk_by_tracking_id
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
        (status = 400, description = "Service error relating to to updating chunk", body = DefaultError),
    ),
)]
pub async fn update_chunk_by_tracking_id(
    chunk: web::Json<UpdateChunkByTrackingIdData>,
    pool: web::Data<Pool>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    if chunk.tracking_id.is_empty() {
        return Err(ServiceError::BadRequest(
            "Tracking id must be provided to update by tracking_id".into(),
        )
        .into());
    }
    let tracking_id = chunk.tracking_id.clone();
    let tracking_id1 = tracking_id.clone();

    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let chunk_metadata = user_owns_chunk_tracking_id(
        user.0.id,
        tracking_id,
        dataset_org_plan_sub.dataset.id,
        pool,
    )
    .await?;

    let link = chunk
        .link
        .clone()
        .unwrap_or_else(|| chunk_metadata.link.clone().unwrap_or_default());

    let new_content = convert_html(chunk.chunk_html.as_ref().unwrap_or(&chunk_metadata.content))
        .map_err(|err| {
            ServiceError::BadRequest(format!("Could not parse html: {}", err.message))
        })?;

    let embedding_vector = create_embedding(
        &new_content,
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration),
    )
    .await?;

    let chunk_html = match chunk.chunk_html.clone() {
        Some(chunk_html) => Some(chunk_html),
        None => chunk_metadata.chunk_html,
    };

    let chunk_id1 = chunk_metadata.id;
    let qdrant_point_id = web::block(move || get_qdrant_id_from_chunk_id_query(chunk_id1, pool1))
        .await?
        .map_err(|_| ServiceError::BadRequest("chunk not found".into()))?;

    let metadata = ChunkMetadata::from_details_with_id(
        chunk_metadata.id,
        &new_content,
        &chunk_html,
        &Some(link),
        &chunk_metadata.tag_set,
        user.0.id,
        chunk_metadata.qdrant_point_id,
        <std::option::Option<serde_json::Value> as Clone>::clone(&chunk.metadata)
            .or(chunk_metadata.metadata),
        Some(tracking_id1),
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
    let metadata1 = metadata.clone();
    update_chunk_metadata_query(metadata, None, dataset_org_plan_sub.dataset.id, pool2)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    update_qdrant_point_query(
        // If the chunk is a collision, we don't want to update the qdrant point
        if chunk_metadata.qdrant_point_id.is_none() {
            None
        } else {
            Some(metadata1)
        },
        qdrant_point_id,
        Some(user.0.id),
        Some(embedding_vector),
        dataset_org_plan_sub.dataset.id,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct SearchChunkData {
    /// Can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using reciprocal rank fusion using the specified weights or BAAI/bge-reranker-large. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE.
    pub search_type: String,
    /// Query is the search query. This can be any string. The query will be used to create an embedding vector and/or SPLADE vector which will be used to find the result set.
    pub query: String,
    /// Page of chunks to fetch. Each page is 10 chunks. Support for custom page size is coming soon.
    pub page: Option<u64>,
    /// Link set is a comma separated list of links. This can be used to filter chunks by link. HNSW indices do not exist for links, so there is a performance hit for filtering on them.
    pub link: Option<Vec<String>>,
    /// Tag_set is a comma separated list of tags. This can be used to filter chunks by tag. Unlike with metadata filtering, HNSW indices will exist for each tag such that there is not a performance hit for filtering on them.
    pub tag_set: Option<Vec<String>>,
    /// Time_range is a tuple of two ISO 8601 combined date and time without timezone. The first value is the start of the time range and the second value is the end of the time range. This can be used to filter chunks by time range. HNSW indices do not exist for time range, so there is a performance hit for filtering on them.
    pub time_range: Option<(String, String)>,
    /// Filters is a JSON object which can be used to filter chunks. The values on each key in the object will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<serde_json::Value>,
    /// Set date_bias to true to bias search results towards more recent chunks. This will work best in hybrid search mode.
    pub date_bias: Option<bool>,
    /// Set cross_encoder to true to use the BAAI/bge-reranker-large model to re-rank search results. This will only apply if in hybrid search mode. Reciprocal rank fusion will be used if cross_encoder is set to false or not set.
    pub cross_encoder: Option<bool>,
    /// Weights are a tuple of two floats. The first value is the weight for the semantic search results and the second value is the weight for the full-text search results. This can be used to bias search results towards semantic or full-text results. This will only apply if in hybrid search mode and cross_encoder is set to false.
    pub weights: Option<(f64, f64)>,
    /// Set get_collisions to true to get the collisions for each chunk. This will only apply if
    /// environment variable COLLISIONS_ENABLED is set to true.
    pub get_collisions: Option<bool>,
    /// Set highlight_results to true to highlight the results.
    pub highlight_results: Option<bool>,
    /// Set highlight_delimiters to a list of strings to use as delimiters for highlighting.
    pub highlight_delimiters: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
pub struct ScoreChunkDTO {
    pub metadata: Vec<ChunkMetadataWithFileData>,
    pub score: f64,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct SearchChunkQueryResponseBody {
    pub score_chunks: Vec<ScoreChunkDTO>,
    pub total_chunk_pages: i64,
}

#[derive(Clone)]
pub struct ParsedQuery {
    pub query: String,
    pub quote_words: Option<Vec<String>>,
    pub negated_words: Option<Vec<String>>,
}
fn parse_query(query: String) -> ParsedQuery {
    let re = Regex::new(r#""(.*?)""#).unwrap();
    let quote_words: Vec<String> = re
        .captures_iter(&query.replace('\\', ""))
        .map(|capture| capture[1].to_string())
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

/// search
///
/// This route provides the primary search functionality for the API. It can be used to search for chunks by semantic similarity, full-text similarity, or a combination of both. Results' `chunk_html` values will be modified with `<b>` tags for sub-sentence highlighting.
#[utoipa::path(
    post,
    path = "/chunk/search",
    context_path = "/api",
    tag = "chunk",
    request_body(content = SearchChunkData, description = "JSON request payload to semantically search for chunks (chunks)", content_type = "application/json"),
    responses(
        (status = 200, description = "chunks which are similar to the embedding vector of the search query", body = SearchChunkQueryResponseBody),
        (status = 400, description = "Service error relating to searching", body = DefaultError),
    ),
)]
#[allow(clippy::too_many_arguments)]
pub async fn search_chunk(
    data: web::Json<SearchChunkData>,
    _user: LoggedUser,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let page = data.page.unwrap_or(1);
    let parsed_query = parse_query(data.query.clone());

    let result_chunks = match data.search_type.as_str() {
        "fulltext" => {
            search_full_text_chunks(data, parsed_query, page, pool, dataset_org_plan_sub.dataset)
                .await?
        }
        "hybrid" => {
            search_hybrid_chunks(data, parsed_query, page, pool, dataset_org_plan_sub.dataset)
                .await?
        }
        _ => {
            search_semantic_chunks(data, parsed_query, page, pool, dataset_org_plan_sub.dataset)
                .await?
        }
    };

    Ok(HttpResponse::Ok().json(result_chunks))
}

#[derive(Serialize, Deserialize, Clone, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct SearchGroupsData {
    /// The query is the search query. This can be any string. The query will be used to create an embedding vector and/or SPLADE vector which will be used to find the result set.
    pub query: String,
    /// The page of chunks to fetch. Each page is 10 chunks. Support for custom page size is coming soon.
    pub page: Option<u64>,
    /// The link set is a comma separated list of links. This can be used to filter chunks by link. HNSW indices do not exist for links, so there is a performance hit for filtering on them.
    pub link: Option<Vec<String>>,
    /// The tag set is a comma separated list of tags. This can be used to filter chunks by tag. Unlike with metadata filtering, HNSW indices will exist for each tag such that there is not a performance hit for filtering on them.
    pub tag_set: Option<Vec<String>>,
    /// Filters is a JSON object which can be used to filter chunks. The values on each key in the object will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<serde_json::Value>,
    /// Group specifies the group to search within. Results will only consist of chunks which are bookmarks within the specified group.
    pub group_id: uuid::Uuid,
    #[param(inline)]
    /// Search_type can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using BAAI/bge-reranker-large. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE.
    pub search_type: String,
    /// Set date_bias to true to bias search results towards more recent chunks. This will work best in hybrid search mode.
    pub date_bias: Option<bool>,
    /// Set cross_encoder to true to use the BAAI/bge-reranker-large model to re-rank search results. This will only apply if in hybrid search mode. If no weighs are specified, the re-ranker will be used by default.
    pub cross_encoder: Option<bool>,
    /// Weights are a tuple of two floats. The first value is the weight for the semantic search results and the second value is the weight for the full-text search results. This can be used to bias search results towards semantic or full-text results. This will only apply if in hybrid search mode and cross_encoder is set to false.
    pub weights: Option<(f64, f64)>,
    /// Set highlight_results to true to highlight the results.
    pub highlight_results: Option<bool>,
    /// Set highlight_delimiters to a list of strings to use as delimiters for highlighting.
    pub highlight_delimiters: Option<Vec<String>>,
}

impl From<SearchGroupsData> for SearchChunkData {
    fn from(data: SearchGroupsData) -> Self {
        Self {
            query: data.query,
            page: data.page,
            link: data.link,
            tag_set: data.tag_set,
            time_range: None,
            filters: data.filters,
            cross_encoder: None,
            weights: None,
            search_type: data.search_type,
            date_bias: data.date_bias,
            get_collisions: Some(false),
            highlight_results: data.highlight_results,
            highlight_delimiters: data.highlight_delimiters,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SearchGroupsResult {
    pub bookmarks: Vec<ScoreChunkDTO>,
    pub group: ChunkGroup,
    pub total_pages: i64,
}

/// group_search
///
/// This route allows you to search only within a group. This is useful for when you only want search results to contain chunks which are members of a specific group. Think about this like searching within a playlist or bookmark folder.
#[utoipa::path(
    post,
    path = "/chunk_group/search",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = SearchGroupsData, description = "JSON request payload to semantically search a group", content_type = "application/json"),
    responses(
        (status = 200, description = "Group chunks which are similar to the embedding vector of the search query", body = SearchGroupsResult),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = DefaultError),
    ),
)]
#[allow(clippy::too_many_arguments)]
pub async fn search_groups(
    data: web::Json<SearchGroupsData>,
    pool: web::Data<Pool>,
    _required_user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    //search over the links as well
    let page = data.page.unwrap_or(1);
    let group_id = data.group_id;
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let search_pool = pool.clone();

    let group = {
        web::block(move || get_group_by_id_query(group_id, dataset_id, pool))
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?
    };

    let parsed_query = parse_query(data.query.clone());

    let result_chunks = match data.search_type.as_str() {
        "fulltext" => {
            search_full_text_groups(
                data,
                parsed_query,
                group,
                page,
                search_pool,
                dataset_org_plan_sub.dataset,
            )
            .await?
        }
        "hybrid" => {
            search_hybrid_groups(
                data,
                parsed_query,
                group,
                page,
                search_pool,
                dataset_org_plan_sub.dataset,
            )
            .await?
        }
        _ => {
            search_semantic_groups(
                data,
                parsed_query,
                group,
                page,
                search_pool,
                dataset_org_plan_sub.dataset,
            )
            .await?
        }
    };

    Ok(HttpResponse::Ok().json(result_chunks))
}

/// get_chunk
///
/// Get a singular chunk by id.
#[utoipa::path(
    get,
    path = "/chunk/{chunk_id}",
    context_path = "/api",
    tag = "chunk",
    responses(
        (status = 200, description = "chunk with the id that you were searching for", body = ChunkMetadata),
        (status = 400, description = "Service error relating to fidning a chunk by tracking_id", body = DefaultError),
    ),
    params(
        ("chunk_id" = Option<uuid>, Path, description = "Id of the chunk you want to fetch.")
    ),
)]
pub async fn get_chunk_by_id(
    chunk_id: web::Path<uuid::Uuid>,
    _user: LoggedUser,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let chunk = web::block(move || {
        get_metadata_from_id_query(chunk_id.into_inner(), dataset_org_plan_sub.dataset.id, pool)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(chunk))
}

/// get_chunk_by_tracking_id
///
/// Get a singular chunk by tracking_id. This is useful for when you are coordinating with an external system and want to use your own id as the primary reference for a chunk.
#[utoipa::path(
    get,
    path = "/chunk/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "chunk",
    responses(
        (status = 200, description = "chunk with the tracking_id that you were searching for", body = ChunkMetadata),
        (status = 400, description = "Service error relating to fidning a chunk by tracking_id", body = DefaultError),
    ),
    params(
        ("tracking_id" = Option<String>, Path, description = "tracking_id of the chunk you want to fetch")
    ),
)]
pub async fn get_chunk_by_tracking_id(
    tracking_id: web::Path<String>,
    _user: LoggedUser,
    pool: web::Data<Pool>,
    _required_user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let chunk = web::block(move || {
        get_metadata_from_tracking_id_query(
            tracking_id.into_inner(),
            dataset_org_plan_sub.dataset.id,
            pool,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(chunk))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RecommendChunksRequest {
    /// The ids of the chunks to be used as positive examples for the recommendation. The chunks in this array will be used to find similar chunks.
    pub positive_chunk_ids: Vec<uuid::Uuid>,
}

/// get_recommended_chunks
///
/// Get recommendations of chunks similar to the chunks in the request. Think about this as a feature similar to the "add to playlist" recommendation feature on Spotify. This request pairs especially well with our groups endpoint.
#[utoipa::path(
    post,
    path = "/chunk/recommend",
    context_path = "/api",
    tag = "chunk",
    request_body(content = RecommendChunksRequest, description = "JSON request payload to get recommendations of chunks similar to the chunks in the request", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON response payload containing chunks with scores which are similar to those in the request body", body = Vec<ChunkMetadataWithFileData>),
        (status = 400, description = "Service error relating to to getting similar chunks", body = DefaultError),
    )
)]
pub async fn get_recommended_chunks(
    data: web::Json<RecommendChunksRequest>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let positive_chunk_ids = data.positive_chunk_ids.clone();
    let embed_size =
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration)
            .EMBEDDING_SIZE
            .unwrap_or(1536);

    let recommended_qdrant_point_ids = recommend_qdrant_query(
        positive_chunk_ids,
        dataset_org_plan_sub.dataset.id,
        embed_size,
    )
    .await
    .map_err(|err| {
        ServiceError::BadRequest(format!("Could not get recommended chunks: {}", err))
    })?;

    let recommended_chunk_metadatas =
        web::block(move || get_metadata_from_point_ids(recommended_qdrant_point_ids, pool))
            .await?
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not get recommended chunk_metadas from qdrant_point_ids: {}",
                    err
                ))
            })?;

    Ok(HttpResponse::Ok().json(recommended_chunk_metadatas))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GenerateChunksRequest {
    /// The model to use for the chat. This can be any model from the openrouter model list. If no model is provided, gryphe/mythomax-l2-13b will be used.
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

/// augmented_generation_from_chunks
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
        (status = 400, description = "Service error relating to to updating chunk, likely due to conflicting tracking_id", body = DefaultError),
    ),
)]
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

    let mut chunks = web::block(move || {
        get_metadata_from_ids_query(chunk_ids, dataset_org_plan_sub.dataset.id, pool)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let llm_api_key = get_env!("LLM_API_KEY", "LLM_API_KEY should be set").into();
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);
    let base_url = dataset_config
        .LLM_BASE_URL
        .unwrap_or("https://openrouter.ai/v1".into());

    let client = Client {
        api_key: llm_api_key,
        http_client: reqwest::Client::new(),
        base_url,
    };

    let mut messages: Vec<ChatMessage> = vec![];

    messages.truncate(prev_messages.len() - 1);
    messages.push(ChatMessage {
        role: Role::User,
        content: ChatMessageContent::Text("I am going to provide several pieces of information for you to use in response to a request or question. You will not respond until I ask you to.".to_string()),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    });
    messages.push(ChatMessage {
        role: Role::Assistant,
        content: ChatMessageContent::Text(
            "Understood, I will not reply until I receive a direct request or question."
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

    let prompt = prompt.unwrap_or("Respond to the instruction and include the doc numbers that you used in square brackets at the end of the sentences that you used the docs for:".to_string());

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
        model: data
            .model
            .clone()
            .unwrap_or("gryphe/mythomax-l2-13b".to_string()),
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
