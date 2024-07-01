use actix_web::{web, HttpResponse};

use crate::{data::models::RedisPool, metrics::Metrics};

pub async fn get_metrics(
    metrics: web::Data<Metrics>,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let _ = metrics.update_queue_gauges(redis_pool).await?;

    let reponse = metrics.get_response();
    Ok(HttpResponse::Ok().content_type("text/plain").body(reponse))
}
