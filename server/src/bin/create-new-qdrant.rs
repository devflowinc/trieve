use trieve_server::{errors::ServiceError, operators::qdrant_operator::create_new_qdrant_collection_query};

#[tokio::main]
async fn main() -> Result<(), ServiceError> {

    create_new_qdrant_collection_query(
        Some("http://localhost:6333"),
        Some("qdrant_api_key"),
        Some("qdrant_collection"),
    )
    .await?;
    Ok(())
}
