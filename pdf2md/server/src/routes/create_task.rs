use crate::{
    errors::{ErrorResponseBody, ServiceError},
    middleware::api_key_middleware::ApiKey,
    models::{self, CreateFileTaskResponse, FileTask, FileTaskStatus, Provider, RedisPool},
};
use actix_web::{post, web, HttpResponse};
use s3::creds::time::OffsetDateTime;

/// Create a new File Task
///
/// This endpoint creates a new task to convert a file to markdown. The task is added to a queue in Redis for processing.
#[utoipa::path(
    post,
    path = "/task",
    tag = "Task",
    context_path = "/api",
    request_body(content = models::UploadFileReqPayload, description = "JSON request payload to create a new task", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON response payload containing the created task", body = models::CreateFileTaskResponse),
        (status = 400, description = "Error typically due to deserialization issues", body = ErrorResponseBody),
    ),
    security(
        ("api_key" = [])
    )
)]
#[post("")]
async fn create_task(
    req: web::Json<models::UploadFileReqPayload>,
    redis_pool: web::Data<RedisPool>,
    clickhouse_client: web::Data<clickhouse::Client>,
    _api_key: ApiKey,
) -> Result<HttpResponse, actix_web::Error> {
    let upload_file_data = req.into_inner();
    let provider = upload_file_data.provider.clone().unwrap_or(Provider::LLM);

    let mut clickhouse_task = models::FileTaskClickhouse {
        id: uuid::Uuid::new_v4().to_string(),
        file_name: upload_file_data.file_name.clone(),
        pages: 0,
        pages_processed: 0,
        status: "CREATED".to_string(),
        provider: provider.to_string(),
        created_at: OffsetDateTime::now_utc(),
        chunkr_task_id: "".to_string(),
        chunkr_api_key: upload_file_data.chunkr_api_key.clone(),
    };

    let task: FileTask = FileTask {
        id: clickhouse_task.id.parse().unwrap(),
        file_name: clickhouse_task.file_name.clone(),
        upload_file_data: upload_file_data.clone(),
        attempt_number: 0,
    };

    match provider {
        Provider::LLM => {
            crate::operators::clickhouse::insert_task(clickhouse_task.clone(), &clickhouse_client)
                .await
                .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

            let mut redis_conn = redis_pool
                .get()
                .await
                .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

            let serialized_message: String = serde_json::to_string(&task).map_err(|_| {
                ServiceError::BadRequest("Failed to Serialize FileTask".to_string())
            })?;

            let pos_in_queue = redis::cmd("lpush")
                .arg("files_to_process")
                .arg(&serialized_message)
                .query_async::<String>(&mut *redis_conn)
                .await
                .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

            Ok(HttpResponse::Ok().json(CreateFileTaskResponse {
                id: task.id,
                file_name: task.file_name,
                status: FileTaskStatus::Created,
                pos_in_queue: Some(pos_in_queue),
            }))
        }
        _ => {
            let chunkr_task = crate::operators::chunkr::create_chunkr_task(
                &task.file_name,
                &task.upload_file_data.base64_file,
                task.upload_file_data.chunkr_api_key.as_deref(),
                task.upload_file_data.chunkr_create_task_req_payload.clone(),
            )
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
            clickhouse_task.chunkr_task_id = chunkr_task.task_id.clone();
            crate::operators::clickhouse::insert_task(clickhouse_task.clone(), &clickhouse_client)
                .await
                .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
            Ok(HttpResponse::Ok().json(CreateFileTaskResponse {
                id: task.id,
                file_name: task.file_name,
                status: FileTaskStatus::Created,
                pos_in_queue: None,
            }))
        }
    }
}
