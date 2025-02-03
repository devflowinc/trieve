use std::str::FromStr;

use crate::data::models::Pool;
use crate::data::models::UnifiedId;
use crate::middleware::auth_middleware::verify_member;
use crate::operators::dataset_operator::get_dataset_and_organization_from_dataset_id_query;
use crate::operators::user_operator::get_user_from_api_key_query;
use crate::operators::webhook_operator::delete_content;
use crate::FairBroccoliQueue;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use ureq::SerdeMap;

use crate::{errors::ServiceError, operators::webhook_operator::publish_content};

use super::chunk_handler::ChunkReqPayload;

#[derive(Serialize, Deserialize)]
struct WebhookRespose {
    message: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    Publish,
    Archive,
    Delete,
    Unpublish,
    ScheduledStart,
    ScheduledEnd,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentValue {
    id: String,
    name: String,
    model_id: String,
    data: Map<String, Value>,
}

impl From<ContentValue> for ChunkReqPayload {
    fn from(content: ContentValue) -> Self {
        let mut body = String::new();
        body.push_str(&content.name);

        let mut metadata = SerdeMap::new();
        let mut tags = Vec::new();

        for (key, value) in content.data.iter() {
            match value {
                Value::String(val) => {
                    body.push_str(&format!("\n{}", val));
                }
                Value::Array(val) => {
                    for item in val {
                        if let Value::String(val) = item {
                            tags.push(val.clone());
                        }
                    }
                }
                Value::Bool(val) => {
                    metadata.insert(key.clone(), serde_json::Value::Bool(*val));
                }
                Value::Number(val) => {
                    metadata.insert(key.clone(), serde_json::Value::Number(val.clone()));
                }
                Value::Object(val) => {
                    metadata.insert(key.clone(), serde_json::Value::Object(val.clone()));
                }
                _ => {}
            };
        }

        ChunkReqPayload {
            tracking_id: Some(content.id),
            metadata: Some(metadata.into()),
            chunk_html: Some(body),
            tag_set: Some(tags),
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookRequest {
    model_name: String,
    new_value: ContentValue,
    // This is usually always there just being safe
    previous_value: Option<ContentValue>,
    operation: Operation,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookQueryParams {
    trieve_key: String,
    trieve_dataset: String,
    // model_id: String,
}

pub async fn builder_io_webhook(
    payload: web::Json<WebhookRequest>,
    query: web::Query<WebhookQueryParams>,
    broccoli_queue: web::Data<FairBroccoliQueue>,
    pool: web::Data<Pool>,
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

    let dataset_and_org = get_dataset_and_organization_from_dataset_id_query(
        UnifiedId::TrieveUuid(dataset_id),
        None,
        pool.clone(),
    )
    .await?;

    let (user, _) = get_user_from_api_key_query(&query.trieve_key, pool.clone()).await?;

    if !verify_member(&user, &dataset_and_org.organization.organization.id) {
        return Ok(HttpResponse::Forbidden().finish());
    };

    match payload.operation {
        Operation::Publish => {
            publish_content(dataset_id, payload.new_value, broccoli_queue).await?
        }
        Operation::Delete => delete_content(dataset_id, payload.new_value, pool).await?,
        Operation::Unpublish => delete_content(dataset_id, payload.new_value, pool).await?,
        Operation::Archive => delete_content(dataset_id, payload.new_value, pool).await?,

        Operation::ScheduledStart => {
            publish_content(dataset_id, payload.new_value, broccoli_queue).await?
        }
        Operation::ScheduledEnd => delete_content(dataset_id, payload.new_value, pool).await?,
    }

    Ok(HttpResponse::Ok().json(WebhookRespose {
        message: "Webhook received".to_string(),
    }))
}
