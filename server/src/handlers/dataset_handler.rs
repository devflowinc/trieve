use super::auth_handler::LoggedUser;
use crate::{
    data::models::{Dataset, Pool},
    errors::ServiceError,
    operators::{
        dataset_operator::{
            delete_dataset_by_id_query, get_dataset_by_id_query, new_dataset_operation,
            update_dataset_query,
        },
        tantivy_operator::TantivyIndexMap,
    },
};
use actix_web::{web, FromRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use tokio::sync::RwLock;
use utoipa::ToSchema;

impl FromRequest for Dataset {
    type Error = ServiceError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        match req.headers().get("AF-Dataset") {
            Some(dataset_header) => match dataset_header.to_str() {
                Ok(dataset_id) => {
                    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

                    let client = match redis::Client::open(redis_url) {
                        Ok(client) => client,
                        Err(_) => {
                            return ready(Err(ServiceError::BadRequest(
                                "Could not create redis client".to_string(),
                            )))
                        }
                    };

                    let mut redis_conn = match client.get_connection() {
                        Ok(redis_conn) => redis_conn,
                        Err(_) => {
                            return ready(Err(ServiceError::BadRequest(
                                "Could not get redis connection".to_string(),
                            )))
                        }
                    };

                    let dataset: String = match redis::cmd("GET")
                        .arg(format!("dataset:{}", dataset_id))
                        .query(&mut redis_conn)
                    {
                        Ok(dataset) => dataset,
                        Err(_) => {
                            return ready(Err(ServiceError::BadRequest(
                                "Could not get dataset from redis".to_string(),
                            )))
                        }
                    };

                    let dataset: Dataset = match serde_json::from_str(&dataset) {
                        Ok(dataset) => dataset,
                        Err(_) => {
                            return ready(Err(ServiceError::BadRequest(
                                "Could not parse dataset from redis".to_string(),
                            )))
                        }
                    };

                    let user = match LoggedUser::from_request(req, payload).into_inner() {
                        Ok(user) => user,
                        Err(_) => {
                            return ready(Err(ServiceError::BadRequest(
                                "Could not get user from request".to_string(),
                            )))
                        }
                    };

                    if dataset.organization_id != user.organization_id {
                        return ready(Err(ServiceError::Forbidden));
                    }

                    ready(Ok(dataset))
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

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
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
        (status = 200, description = "Dataset created successfully", body = [Dataset]),
        (status = 400, description = "Service error relating to creating the dataset", body = [DefaultError]),
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

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
pub struct UpdateDatasetRequest {
    pub dataset_id: String,
    pub dataset_name: String,
}

#[utoipa::path(
    put,
    path = "/dataset",
    context_path = "/api",
    tag = "dataset",
    request_body(content = UpdateDatasetRequest, description = "JSON request payload to update a dataset", content_type = "application/json"),
    responses(
        (status = 200, description = "Dataset updated successfully", body = [Dataset]),
        (status = 400, description = "Service error relating to updating the dataset", body = [DefaultError]),
    ),
)]
pub async fn update_dataset(
    data: web::Json<UpdateDatasetRequest>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    let admin_email = std::env::var("ADMIN_USER_EMAIL").unwrap_or("".to_string());
    if admin_email != user.email {
        return Err(ServiceError::Forbidden);
    }

    let dataset_id = data
        .dataset_id
        .clone()
        .parse::<uuid::Uuid>()
        .map_err(|_| ServiceError::BadRequest("Dataset ID must be a valid UUID".to_string()))?;
    let d = update_dataset_query(dataset_id, data.dataset_name.clone(), pool).await?;
    Ok(HttpResponse::Ok().json(d))
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
pub struct DeleteDatasetRequest {
    pub dataset_id: String,
}

#[utoipa::path(
    delete,
    path = "/dataset",
    context_path = "/api",
    tag = "dataset",
    request_body(content = DeleteDatasetRequest, description = "JSON request payload to delete a dataset", content_type = "application/json"),
    responses(
        (status = 204, description = "Dataset deleted successfully"),
        (status = 400, description = "Service error relating to deleting the dataset", body = [DefaultError]),
    ),
)]
pub async fn delete_dataset(
    data: web::Json<DeleteDatasetRequest>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    let admin_email = std::env::var("ADMIN_USER_EMAIL").unwrap_or("".to_string());
    if admin_email != user.email {
        return Err(ServiceError::Forbidden);
    }

    let dataset_id = data
        .dataset_id
        .clone()
        .parse::<uuid::Uuid>()
        .map_err(|_| ServiceError::BadRequest("Dataset ID must be a valid UUID".to_string()))?;
    delete_dataset_by_id_query(dataset_id, pool).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    path = "/dataset",
    context_path = "/api",
    tag = "dataset",
    request_body(content = GetDatasetRequest, description = "JSON request payload to get a dataset", content_type = "application/json"),
    responses(
        (status = 200, description = "Dataset retrieved successfully", body = [Dataset]),
        (status = 400, description = "Service error relating to retrieving the dataset", body = [DefaultError]),
    ),
)]

pub async fn get_dataset(
    pool: web::Data<Pool>,
    dataset_id: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, ServiceError> {
    let d = get_dataset_by_id_query(dataset_id.into_inner(), pool).await?;
    Ok(HttpResponse::Ok().json(d))
}
