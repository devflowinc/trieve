use actix_web::{get, post, web, HttpResponse};
use serde_json::to_string;
use time::OffsetDateTime;

use crate::{
    errors::ServiceError,
    models::{CreateSchemaRequest, Schema},
    operators::schema::{create_schema_query, get_schema_query},
};

/// Create a Schema
///
/// This endpoint creates a schema in the Batch ETL system. The schema is used to define the structure of the data that will be ingested into the system.
#[utoipa::path(
    post,
    tag = "Schema",
    request_body(content = CreateSchemaRequest, description = "JSON request payload to create a new schema", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON response payload containing the created schema", body = Schema),
        (status = 400, description = "Error typically due to deserialization issues", body = ServiceError),
    ),
)]
#[post("")]
pub async fn create_schema(
    request: web::Json<CreateSchemaRequest>,
    clickhouse_client: web::Data<clickhouse::Client>,
) -> Result<HttpResponse, actix_web::Error> {
    let schema = Schema {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name.clone(),
        schema: to_string(&request.schema.clone()).unwrap(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };

    let schema = create_schema_query(&schema, &clickhouse_client).await?;

    Ok(HttpResponse::Ok().json(schema))
}

/// Get a Schema
///
/// This endpoint retrieves a schema by its id.
#[utoipa::path(
    get,
    path = "/{schema_id}",
    tag = "Schema",
    params(
        ("schema_id" = String, Path, description = "The id of the schema you want to retrieve."),
    ),
    responses(
        (status = 200, description = "JSON response payload containing the schema", body = Schema),
        (status = 400, description = "Error typically due to deserialization issues", body = ServiceError),
    ),
)]
#[get("/{schema_id}")]
pub async fn get_schema(
    schema_id: web::Path<String>,
    clickhouse_client: web::Data<clickhouse::Client>,
) -> Result<HttpResponse, actix_web::Error> {
    let schema_id = schema_id.into_inner();

    let schema = get_schema_query(&schema_id, &clickhouse_client).await?;

    Ok(HttpResponse::Ok().json(schema))
}
