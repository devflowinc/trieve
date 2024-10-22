use actix_web::web;

use crate::{
    data::models::{DatasetConfiguration, Pool, RedisPool, UnifiedId},
    errors::ServiceError,
    handlers::{chunk_handler::ChunkReqPayload, webhook_handler::ContentValue},
    operators::{
        chunk_operator::{create_chunk_metadata, get_row_count_for_organization_id_query},
        dataset_operator::get_dataset_by_id_query,
    },
};

pub async fn publish_content<T: Into<ChunkReqPayload>>(
    dataset_id: uuid::Uuid,
    value: T,
    redis_pool: web::Data<RedisPool>,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    let chunk: ChunkReqPayload = value.into();

    // TODO: Ensure that the chunk count is respected
    // let unlimited = std::env::var("UNLIMITED").unwrap_or("false".to_string());
    // if unlimited == "false" {
    //     let chunk_count = get_row_count_for_organization_id_query(
    //         dataset_org_plan_sub.organization.organization.id,
    //         pool.clone(),
    //     )
    //     .await?;
    //
    //     if chunk_count + chunks.len()
    //         > dataset_org_plan_sub
    //             .organization
    //             .plan
    //             .unwrap_or_default()
    //             .chunk_count as usize
    //     {
    //         return Ok(HttpResponse::UpgradeRequired()
    //             .json(json!({"message": "Must upgrade your plan to add more chunks"})));
    //     }
    //
    //     timer.add("get dataset count");
    // }

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

    let pos_in_queue = redis::cmd("lpush")
        .arg("ingestion")
        .arg(&serialized_message)
        .query_async(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}
