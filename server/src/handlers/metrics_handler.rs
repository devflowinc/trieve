use actix_web::{web, HttpResponse};

use crate::{data::models::RedisPool, metrics::Metrics};

fn check_x_api_access(req: &actix_web::HttpRequest) -> bool {
    let admin_key = std::env::var("ADMIN_API_KEY");
    let api_key = req.headers().get("X-API-KEY");

    if let Some(api_key) = api_key {
        if let Ok(admin_key) = admin_key {
            return api_key.to_str().unwrap_or_default() == admin_key.as_str();
        }
    }
    false
}

/// Get Prometheus Metrics
///
/// This route allows you to view the number of items in each queue in the Prometheus format.
#[utoipa::path(
    post,
    path = "/metrics",
    tag = "metrics",
    responses(
        (status = 200, description = "Prometheus metrics for the server", body = String),
        (status = 500, description = "Internal Server Error", body = ErrorResponseBody),
    ),
    security(
        ("X-API-KEY" = []),
    )
)]
#[tracing::instrument(skip(redis_pool))]
pub async fn get_metrics(
    req: actix_web::HttpRequest,
    metrics: web::Data<Metrics>,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let authed = check_x_api_access(&req);
    if !authed {
        return Ok(HttpResponse::Unauthorized().finish());
    }

    let _ = metrics.update_queue_gauges(redis_pool).await;
    let reponse = metrics.get_response();
    Ok(HttpResponse::Ok().content_type("text/plain").body(reponse))
}
