use crate::{
    errors::{ErrorResponseBody, ServiceError},
    middleware::api_key_middleware::ApiKey,
    models::{self, GetTaskRequest},
    operators::s3::{get_aws_bucket, get_signed_url},
};
use actix_web::{get, web, HttpResponse};

/// Retieve a File Task by ID
///
/// This endpoint retrieves a task by its id. The task is returned along with the pages that have been created, if the file chunking has been completed.
#[utoipa::path(
    get,
    path = "/task/{task_id}",
    tag = "Task",
    context_path = "/api",
     params(
        ("task_id" = uuid::Uuid, Path, description = "The id of the task you want to retrieve."),
        ("limit" = Option<u32>, Query, description = "The number of pages to return."),
        ("pagination_token" = Option<String>, Query, description = "The pagination token to use for the next request."),
    ),
    responses(
        (status = 200, description = "JSON response payload containing the created pages", body = models::GetTaskResponse),
        (status = 400, description = "Error typically due to deserialization issues", body = ErrorResponseBody),
    ),
    security(
        ("api_key" = [])
    )
)]
#[get("/{task_id}")]
async fn get_task(
    task_id: web::Path<uuid::Uuid>,
    data: web::Query<GetTaskRequest>,
    clickhouse_client: web::Data<clickhouse::Client>,
    _api_key: ApiKey,
) -> Result<HttpResponse, ServiceError> {
    let task_id = task_id.into_inner();
    let task = crate::operators::clickhouse::get_task(task_id, &clickhouse_client).await?;
    let pages = crate::operators::clickhouse::get_task_pages(
        task.clone(),
        data.limit,
        data.pagination_token,
        &clickhouse_client,
    )
    .await?;
    let bucket = get_aws_bucket()?;
    let file_url = get_signed_url(&bucket, format!("{}.pdf", &task.id).as_str()).await?;

    let result = models::GetTaskResponse::new_with_pages(task, pages, file_url);
    Ok(HttpResponse::Ok().json(result))
}
