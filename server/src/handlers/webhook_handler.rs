use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
struct WebhookRespose {
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    Publish,
    Archive,
    Delete,
    Unpublish,
    ScheduledStart,
    ScheduledEnd,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebhookRequest {
    model_name: String,
    new_value: Value,
    previous_value: Value,
    operation: Operation,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebhookQueryParams {
    trieve_key: String,
    trieve_dataset: String,
    model_id: String,
}

pub async fn builder_webhook(
    payload: web::Json<WebhookRequest>,
    query: web::Query<WebhookQueryParams>,
) -> Result<HttpResponse, actix_web::Error> {
    let payload = payload.into_inner();
    let query = query.into_inner();

    log::info!("Webhook received: {:?}", payload);
    log::info!("Query params: {:?}", query);

    Ok(HttpResponse::Ok().json(WebhookRespose {
        message: "Webhook received".to_string(),
    }))
}
