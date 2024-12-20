use crate::{errors::ServiceError, models::Schema};

pub async fn create_schema_query(
    schema: &Schema,
    clickhouse_client: &clickhouse::Client,
) -> Result<Schema, ServiceError> {
    let mut inserter = clickhouse_client.insert("schemas").map_err(|err| {
        log::error!("Failed to insert schema: {:?}", err);
        ServiceError::InternalServerError("Failed to insert schema".to_string())
    })?;

    inserter.write(schema).await.map_err(|err| {
        log::error!("Failed to write schema: {:?}", err);
        ServiceError::InternalServerError("Failed to write schema".to_string())
    })?;

    inserter.end().await.map_err(|err| {
        log::error!("Failed to end schema insert: {:?}", err);
        ServiceError::InternalServerError("Failed to end schema insert".to_string())
    })?;

    Ok(schema.clone())
}

pub async fn get_schema_query(
    schema_id: &str,
    clickhouse_client: &clickhouse::Client,
) -> Result<Schema, ServiceError> {
    let schema = clickhouse_client
        .query("SELECT ?fields FROM schemas WHERE id = ?")
        .bind(schema_id)
        .fetch_one::<Schema>()
        .await
        .map_err(|err| {
            log::error!("Failed to create schema query: {:?}", err);
            ServiceError::InternalServerError("Failed to create schema query".to_string())
        })?;

    Ok(schema)
}
