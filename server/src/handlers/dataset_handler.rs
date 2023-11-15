use actix_web::{HttpResponse, web};
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

use crate::{operators::qdrant_operator, errors::ServiceError};

use super::auth_handler::LoggedUser;


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
) -> Result<HttpResponse, actix_web::Error> {

    let admin_email = std::env::var("ADMIN_USER_EMAIL").unwrap_or("".to_string());
    if admin_email != user.email {
        return Err(ServiceError::Forbidden.into());
    }

    qdrant_operator::create_new_qdrant_collection_query(data.dataset.clone()).await?;
    Ok(HttpResponse::NotFound().finish())
}
