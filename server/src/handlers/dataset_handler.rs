use std::future::{Ready, ready};

use actix_web::{HttpResponse, web, FromRequest};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use crate::{operators::qdrant_operator, errors::ServiceError};

use super::auth_handler::LoggedUser;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct Dataset {
    pub name: Option<String>
}

impl FromRequest for Dataset {
    type Error = ServiceError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut actix_web::dev::Payload) -> Self::Future {
        match req.headers().get("AF-Dataset") {
            Some(dataset_header) => match dataset_header.to_str() {
                Ok(dataset) => ready(Ok(Dataset { name: Some(dataset.to_string()) })),
                Err(_) => ready(Err(ServiceError::BadRequest("Dataset must be ASCII".to_string()))),
            },
            None => ready(Ok(Dataset { name: None })),
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
    user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    let admin_email = std::env::var("ADMIN_USER_EMAIL").unwrap_or("".to_string());
    if admin_email != user.email {
        return Err(ServiceError::Forbidden);
    }

    qdrant_operator::create_new_qdrant_collection_query(data.dataset.clone()).await?;
    Ok(HttpResponse::NoContent().finish())
}
