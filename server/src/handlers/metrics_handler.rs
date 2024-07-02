use actix_web::{web, HttpResponse};

use crate::{data::models::RedisPool, metrics::Metrics};

#[utoipa::path(
    post,
    path = "/metrics",
    tag = "metrics",
    responses(
        (status = 200, description = "Prometheus metrics for the server", body = String),
        (status = 500, description = "Internal Server Error", body = ErrorResponseBody),
    ),
)]
#[tracing::instrument(skip(redis_pool))]
pub async fn get_metrics(
    metrics: web::Data<Metrics>,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let _ = metrics.update_queue_gauges(redis_pool).await?;

    let reponse = metrics.get_response();
    Ok(HttpResponse::Ok().content_type("text/plain").body(reponse))
}
