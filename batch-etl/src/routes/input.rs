use actix_web::{get, post, web, HttpResponse};

use crate::{
    errors::ServiceError,
    models::{self, CreateInputRequest},
    operators::input::{create_input_query, get_input_query},
};

/// Create an Input
///
/// This endpoint creates an input in the Batch ETL system. The input is used to define the data that will be ingested into the system.
#[utoipa::path(
    post,
    path = "/input",
    tag = "Input",
    context_path = "/api",
    request_body(content = models::CreateInputRequest, description = "JSON request payload to create a new input", content_type = "application/json"),
    responses(
        (status = 201, description = "JSON response payload containing the created input", body = models::CreateInputResponse),
        (status = 400, description = "Error typically due to deserialization issues", body = ServiceError),
    ),
)]
#[post("")]
pub async fn create_input(
    clickhouse_client: web::Data<clickhouse::Client>,
    input: web::Json<CreateInputRequest>,
) -> Result<HttpResponse, actix_web::Error> {
    let input = input.into_inner();
    let response = create_input_query(&input, &clickhouse_client).await?;

    Ok(HttpResponse::Created().json(response))
}

/// Get an Input
///
/// This endpoint retrieves an s3 url for an input by its id.
#[utoipa::path(
    get,
    path = "/input/{input_id}",
    tag = "Input",
    context_path = "/api",
    params(
        ("input_id" = String, Path, description = "The id of the input you want to retrieve."),
    ),
    responses(
        (status = 200, description = "JSON response payload containing the input", body = models::Input),
        (status = 400, description = "Error typically due to deserialization issues", body = ServiceError),
    ),
)]
#[get("/{input_id}")]
pub async fn get_input(input_id: web::Path<String>) -> Result<HttpResponse, actix_web::Error> {
    let input_id = input_id.into_inner();
    let s3_url = get_input_query(&input_id).await?;

    Ok(HttpResponse::Ok().json(s3_url))
}
