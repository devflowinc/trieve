use trieve_server::{
    errors::ServiceError,
    operators::qdrant_operator::{create_new_qdrant_collection_query, get_qdrant_connection},
};

#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    let qdrant_url = std::env::var("QDRANT_URL").unwrap_or("http://localhost:6333".to_string());
    let qdrant_api_key = std::env::var("QDRANT_API_KEY").unwrap_or("qdrant_api_key".to_string());
    let qdrant_collection =
        std::env::var("QDRANT_COLLECTION").unwrap_or("qdrant_collection".to_string());

    let qdrant_client = get_qdrant_connection(Some(&qdrant_url), Some(&qdrant_api_key)).await?;

    qdrant_client
        .delete_field_index(qdrant_collection.clone(), "content", None)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    create_new_qdrant_collection_query(
        Some(&qdrant_url),
        Some(&qdrant_api_key),
        Some(&qdrant_collection),
        false,
        false,
    )
    .await?;
    Ok(())
}
