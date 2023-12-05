use super::auth_handler::LoggedUser;
use crate::{
    errors::ServiceError,
    operators::{qdrant_operator, tantivy_operator::TantivyIndexMap},
};
use actix_web::{web, FromRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use tokio::sync::RwLock;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct Dataset {
    pub name: String,
}

impl FromRequest for Dataset {
    type Error = ServiceError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        log::error!(
            "Dataset from request, headers: {:?}",
            req.headers().get("AF-Dataset")
        );
        match req.headers().get("AF-Dataset") {
            Some(dataset_header) => match dataset_header.to_str() {
                Ok(dataset) if !dataset.is_empty() && dataset != "undefined" => {
                    log::error!("Dataset is {}", dataset);
                    ready(Ok(Dataset {
                        name: dataset.to_string(),
                    }))
                }
                Ok(dataset) if dataset.eq("") || dataset.eq("undefined") => ready(Ok(Dataset {
                    name: "DEFAULT".to_string(),
                })),
                Ok(dataset) => ready(Ok(Dataset {
                    name: dataset.to_string(),
                })),
                Err(_) => ready(Err(ServiceError::BadRequest(
                    "Dataset must be ASCII".to_string(),
                ))),
            },
            None => ready(Ok(Dataset {
                name: "DEFAULT".to_string(),
            })),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CreateDatasetRequest {
    pub dataset: String,
}

#[utoipa::path(
    post,
    path = "/dataset",
    context_path = "/api",
    tag = "dataset",
    request_body(content = CreateDatasetRequest, description = "JSON request payload to create a new dataset", content_type = "application/json"),
    responses(
        (status = 204, description = "Dataset created successfully"),
        (status = 400, description = "Service error relating to creating the dataset", body = [BadRequestBody]),
    ),
)]
pub async fn create_dataset(
    data: web::Json<CreateDatasetRequest>,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    log::info!("Creating dataset {:?}", data.dataset);
    let admin_email = std::env::var("ADMIN_USER_EMAIL").unwrap_or("".to_string());
    if admin_email != user.email {
        return Err(ServiceError::Forbidden);
    }

    tantivy_index_map
        .write()
        .await
        .create_index(Some(&data.dataset))
        .map_err(|err| {
            ServiceError::BadRequest(format!(
                "Failed to create tantivy index: {:?}",
                err.to_string()
            ))
        })?;

    log::info!("Creating dataset {:?}", data.dataset);
    qdrant_operator::create_new_qdrant_collection_query(data.dataset.clone()).await?;
    Ok(HttpResponse::NoContent().finish())
}
