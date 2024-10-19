use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct WebhookRespose {
    message: String,
}

// #[tracing::instrument(skip(pool))]
pub async fn builder_webhook(
    req: HttpRequest,
    payload: web::Bytes,
) -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().json(WebhookRespose {
        message: "Webhook received".to_string(),
    }))
}

