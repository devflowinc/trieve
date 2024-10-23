use actix_web::web;

use crate::{
    data::models::{DatasetConfiguration, Pool, RedisPool, UnifiedId},
    errors::ServiceError,
    handlers::chunk_handler::ChunkReqPayload,
    operators::{chunk_operator::create_chunk_metadata, dataset_operator::get_dataset_by_id_query},
};

use super::chunk_operator::{delete_chunk_metadata_query, get_metadata_from_tracking_id_query};

pub async fn delete_content<T: Into<ChunkReqPayload>>(
    dataset_id: uuid::Uuid,
    value: T,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    let chunk: ChunkReqPayload = value.into();
    let tracking_id_inner = chunk.tracking_id.ok_or(ServiceError::BadRequest(
        "Must provide a tracking_id to delete a chunk".to_string(),
    ))?;

    let full_dataset =
        get_dataset_by_id_query(UnifiedId::TrieveUuid(dataset_id), pool.clone()).await?;

    let dataset_config = DatasetConfiguration::from_json(full_dataset.server_configuration.clone());

    let chunk_metadata =
        get_metadata_from_tracking_id_query(tracking_id_inner, dataset_id, pool.clone()).await?;

    let deleted_at = chrono::Utc::now().naive_utc();

    delete_chunk_metadata_query(
        vec![chunk_metadata.id],
        deleted_at,
        full_dataset,
        pool,
        dataset_config,
    )
    .await?;

    Ok(())
}

pub async fn publish_content<T: Into<ChunkReqPayload>>(
    dataset_id: uuid::Uuid,
    value: T,
    redis_pool: web::Data<RedisPool>,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    let chunk: ChunkReqPayload = value.into();

    let full_dataset =
        get_dataset_by_id_query(UnifiedId::TrieveUuid(dataset_id), pool.clone()).await?;

    let dataset_config = DatasetConfiguration::from_json(full_dataset.server_configuration.clone());

    let (upsert_message, _) = create_chunk_metadata(
        vec![chunk.clone()],
        dataset_id,
        dataset_config,
        pool.clone(),
    )
    .await?;

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let serialized_message: String = serde_json::to_string(&upsert_message).map_err(|_| {
        ServiceError::BadRequest("Failed to Serialize BulkUploadMessage".to_string())
    })?;

    redis::cmd("lpush")
        .arg("ingestion")
        .arg(&serialized_message)
        .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}
