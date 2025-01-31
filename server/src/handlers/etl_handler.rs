use actix_web::{web, HttpResponse};
use broccoli_queue::queue::BroccoliQueue;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{data::models::DatasetAndOrgWithSubAndPlan, errors::ServiceError};

use super::auth_handler::AdminOnly;

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema)]
pub struct CreateSchemaReqPayload {
    pub prompt: String,
    pub model: Option<String>,
    pub tag_enum: Option<Vec<String>>,
    pub include_images: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum EtlJobMessage {
    CreateJob(EtlJobRequest),
    WebhookResponse(EtlWebhookResponse),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EtlJobRequest {
    pub payload: CreateSchemaReqPayload,
    pub dataset_id: uuid::Uuid,
}

/// Create ETL Job
///
/// This endpoint is used to create a new ETL job for a dataset.
#[utoipa::path(
    post,
    path = "/etl/create_job",
    context_path = "/api",
    tag = "Dataset",
    request_body(content = CreateSchemaReqPayload, description = "JSON request payload to create a new ETL Job", content_type = "application/json"),
    responses(
        (status = 204, description = "ETL Job created successfully"),
        (status = 400, description = "Service error relating to creating the dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn create_etl_job(
    data: web::Json<CreateSchemaReqPayload>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    broccoli_queue: web::Data<BroccoliQueue>,
) -> Result<HttpResponse, actix_web::Error> {
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let payload = EtlJobRequest {
        payload: data.clone(),
        dataset_id,
    };

    broccoli_queue
        .publish("etl_queue", None, &payload, None)
        .await
        .map_err(|e| {
            log::error!("Error publishing to queue: {:?}", e);
            ServiceError::InternalServerError("Error publishing to queue".to_string())
        })?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EtlWebhookResponse {
    pub job_id: String,
    pub batch_url: String,
}

pub async fn webhook_response(
    data: web::Json<EtlWebhookResponse>,
    broccoli_queue: web::Data<BroccoliQueue>,
) -> Result<HttpResponse, actix_web::Error> {
    broccoli_queue
        .publish("etl_queue", None, &data.clone(), None)
        .await
        .map_err(|e| {
            log::error!("Error publishing to queue: {:?}", e);
            ServiceError::InternalServerError("Error publishing to queue".to_string())
        })?;

    Ok(HttpResponse::NoContent().finish())
}
