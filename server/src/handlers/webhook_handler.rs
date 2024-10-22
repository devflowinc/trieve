use std::str::FromStr;

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{errors::ServiceError, operators::webhook_operator::publish_content};

use super::chunk_handler::ChunkReqPayload;

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
pub struct ContentValue {
    id: String,
    name: String,
    model_id: String,
    data: Value,
}

impl Into<ChunkReqPayload> for ContentValue {
    fn into(self) -> ChunkReqPayload {
        todo!();
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WebhookRequest {
    model_name: String,
    new_value: ContentValue,
    // This is usually always there just being safe
    previous_value: Option<ContentValue>,
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
    // Ensure that the trieve_key and trieve_dataset are valid
    if query.trieve_key.is_empty() || query.trieve_dataset.is_empty() {
        return Err(ServiceError::BadRequest(
            "trieve_key and trieve_dataset are required".to_string(),
        )
        .into());
    }

    let payload = payload.into_inner();
    let query = query.into_inner();
    let dataset_id = uuid::Uuid::from_str(query.trieve_dataset.as_str()).map_err(|_| {
        ServiceError::BadRequest(format!("Invalid dataset id: {}", query.trieve_dataset))
    })?;

    // TODO: Ensure user has proper perms

    log::info!("Webhook received: {:?}", payload);
    log::info!("Query params: {:?}", query);

    match payload.operation {
        Operation::Publish => {
            publish_content(dataset_id, payload.new_value).await?;
        }

        _ => {
            return Err(ServiceError::BadRequest("Operation not supported".to_string()).into());
        }
    }

    Ok(HttpResponse::Ok().json(WebhookRespose {
        message: "Webhook received".to_string(),
    }))
}
