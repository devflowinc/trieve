use actix_web::{get, web, HttpResponse};

use crate::errors::ServiceError;

#[get("/task/{task_id}")]
async fn get_task(
    task_id: web::Path<uuid::Uuid>,
    clickhouse_client: web::Data<clickhouse::Client>,
) -> Result<HttpResponse, ServiceError> {
    let task_id = task_id.into_inner();

    let task = crate::operators::clickhouse::get_task(task_id, &clickhouse_client).await?;

    Ok(HttpResponse::Ok().json(task))
}
