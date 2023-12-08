use super::auth_handler::LoggedUser;
use crate::{
    data::models::{Dataset, Pool},
    errors::ServiceError,
    operators::{dataset_operator::new_dataset_operation, tantivy_operator::TantivyIndexMap},
};
use actix_web::{web, FromRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use tokio::sync::RwLock;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SlimDataset {
    pub id: uuid::Uuid,
}

impl FromRequest for SlimDataset {
    type Error = ServiceError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        match req.headers().get("AF-Dataset") {
            Some(dataset_header) => match dataset_header.to_str() {
                Ok(dataset) => {
                    // Try to convert to a uuid
                    let id_result = uuid::Uuid::parse_str(dataset).map_err(|err| {
                        ServiceError::BadRequest(format!(
                            "Dataset must be a valid UUID: {:?}",
                            err.to_string()
                        ))
                    });
                    match id_result {
                        Ok(id) => ready(Ok(SlimDataset { id })),
                        Err(err) => ready(Err(err)),
                    }
                }
                Err(_) => ready(Err(ServiceError::BadRequest(
                    "Dataset must be ASCII".to_string(),
                ))),
            },
            None => ready(Err(ServiceError::BadRequest(
                "Dataset must be specified".to_string(),
            ))),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CreateDatasetRequest {
    pub dataset_name: String,
}

#[utoipa::path(
    post,
    path = "/dataset",
    context_path = "/api",
    tag = "dataset",
    request_body(content = CreateDatasetRequest, description = "JSON request payload to create a new dataset", content_type = "application/json"),
    responses(
        (status = 200, description = "Dataset created successfully"),
        (status = 400, description = "Service error relating to creating the dataset", body = [BadRequestBody]),
    ),
)]
pub async fn create_dataset(
    data: web::Json<CreateDatasetRequest>,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    let admin_email = std::env::var("ADMIN_USER_EMAIL").unwrap_or("".to_string());
    if admin_email != user.email {
        return Err(ServiceError::Forbidden);
    }

    let dataset = Dataset::from_details(data.dataset_name.clone(), user.organization_id);

    let d = new_dataset_operation(dataset, tantivy_index_map, pool).await?;
    Ok(HttpResponse::Ok().json(d))
}
