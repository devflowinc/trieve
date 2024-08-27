use qdrant_client::{
    qdrant::{PointStruct, UpsertPointsBuilder},
    Qdrant,
};
use trieve_server::{
    errors::ServiceError, operators::qdrant_operator::scroll_qdrant_collection_ids_custom_url,
};
#[allow(clippy::print_stdout)]
#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    dotenvy::dotenv().ok();

    let origin_qdrant_url =
        std::env::var("ORIGIN_QDRANT_URL").expect("ORIGIN_QDRANT_URL is not set");
    let new_qdrant_url = std::env::var("NEW_QDRANT_URL").expect("NEW_QDRANT_URL is not set");
    let qdrant_api_key = std::env::var("QDRANT_API_KEY").expect("QDRANT_API_KEY is not set");
    let collection_to_clone =
        std::env::var("COLLECTION_TO_CLONE").expect("COLLECTION_TO_CLONE is not set");

    let mut offset = Some(uuid::Uuid::nil().to_string());

    while let Some(cur_offset) = offset {
        let original_qdrant_connection = Qdrant::from_url(&origin_qdrant_url)
            .api_key(qdrant_api_key.clone())
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|_err| ServiceError::BadRequest("Failed to connect to Qdrant".to_string()))?;
        let new_qdrant_connection = Qdrant::from_url(&new_qdrant_url)
            .api_key(qdrant_api_key.clone())
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|_err| ServiceError::BadRequest("Failed to connect to Qdrant".to_string()))?;

        let (origin_qdrant_points, new_offset) = scroll_qdrant_collection_ids_custom_url(
            collection_to_clone.clone(),
            Some(cur_offset.to_string()),
            Some(1000),
            original_qdrant_connection,
        )
        .await?;

        let point_structs_to_upsert = origin_qdrant_points
            .iter()
            .filter_map(|retrieved_point| {
                let id = retrieved_point.id.clone();
                let payload = retrieved_point.payload.clone();
                let vectors = retrieved_point.vectors.clone();
                if let (Some(id), payload, Some(vectors)) = (id, payload, vectors) {
                    Some(PointStruct {
                        id: Some(id),
                        payload,
                        vectors: Some(vectors),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<PointStruct>>();

        new_qdrant_connection
            .upsert_points(UpsertPointsBuilder::new(
                collection_to_clone.clone(),
                point_structs_to_upsert,
            ))
            .await
            .map_err(|err| {
                log::error!("Failed inserting chunk to qdrant {:?}", err);
                ServiceError::BadRequest(format!("Failed inserting chunk to qdrant {:?}", err))
            })?;

        offset = new_offset;
    }

    Ok(())
}
