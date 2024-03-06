use super::auth_handler::{AdminOnly, LoggedUser, OwnerOnly};
use crate::{
    data::models::{
        ClientDatasetConfiguration, Dataset, DatasetAndOrgWithSubAndPlan, Pool,
        ServerDatasetConfiguration, StripePlan,
    },
    errors::ServiceError,
    operators::{
        dataset_operator::{
            create_dataset_query, delete_dataset_by_id_query, get_dataset_by_id_query,
            get_datasets_by_organization_id, update_dataset_query,
        },
        organization_operator::{get_org_dataset_count, get_organization_by_key_query},
        stripe_operator::refresh_redis_org_plan_sub,
    },
};
use actix_web::{web, FromRequest, HttpMessage, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::future::{ready, Ready};
use utoipa::ToSchema;

impl FromRequest for DatasetAndOrgWithSubAndPlan {
    type Error = ServiceError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(
            req.extensions()
                .get::<DatasetAndOrgWithSubAndPlan>()
                .cloned()
                .ok_or(ServiceError::InternalServerError(
                    "Could not get dataset from request".to_string(),
                )),
        )
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
pub struct CreateDatasetRequest {
    /// Name of the dataset. Must be unique within the organization.
    pub dataset_name: String,
    /// Organization ID that the dataset will belong to.
    pub organization_id: uuid::Uuid,
    /// Server configuration for the dataset, can be arbitrary JSON. We recommend setting to `{}` to start. See docs.trieve.ai for more information or adjust with the admin dashboard.
    pub server_configuration: serde_json::Value,
    /// Client configuration for the dataset, can be arbitrary JSON. We recommend setting to `{}` to start. See docs.trieve.ai for more information or adjust with the admin dashboard.
    pub client_configuration: serde_json::Value,
}

/// create_dataset
///
/// Create a new dataset. The auth'ed user must be an owner of the organization to create a dataset.
#[utoipa::path(
    post,
    path = "/dataset",
    context_path = "/api",
    tag = "dataset",
    request_body(content = CreateDatasetRequest, description = "JSON request payload to create a new dataset", content_type = "application/json"),
    responses(
        (status = 200, description = "Dataset created successfully", body = Dataset),
        (status = 400, description = "Service error relating to creating the dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
        ("Cookie" = ["owner"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn create_dataset(
    data: web::Json<CreateDatasetRequest>,
    pool: web::Data<Pool>,
    _user: OwnerOnly,
) -> Result<HttpResponse, ServiceError> {
    let org_pool = pool.clone();
    let org_id = data.organization_id;

    let organization_sub_plan = get_organization_by_key_query(org_id.into(), org_pool.clone())
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let dataset_count = web::block(move || get_org_dataset_count(org_id, org_pool))
        .await
        .map_err(|_| {
            ServiceError::BadRequest("Blocking error getting org dataset count".to_string())
        })?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if dataset_count
        >= organization_sub_plan
            .plan
            .unwrap_or(StripePlan::default())
            .dataset_count
    {
        return Ok(HttpResponse::UpgradeRequired()
            .json(json!({"message": "Your plan must be upgraded to create additional datasets"})));
    }

    let dataset = Dataset::from_details(
        data.dataset_name.clone(),
        data.organization_id,
        data.server_configuration.clone(),
        data.client_configuration.clone(),
    );

    let d = create_dataset_query(dataset, pool).await?;
    Ok(HttpResponse::Ok().json(d))
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
pub struct UpdateDatasetRequest {
    /// The id of the dataset you want to update.
    pub dataset_id: uuid::Uuid,
    /// The new name of the dataset. Must be unique within the organization. If not provided, the name will not be updated.
    pub dataset_name: Option<String>,
    /// The new server configuration of the dataset, can be arbitrary JSON. See docs.trieve.ai for more information. If not provided, the server configuration will not be updated.
    pub server_configuration: Option<serde_json::Value>,
    /// The new client configuration of the dataset, can be arbitrary JSON. See docs.trieve.ai for more information. If not provided, the client configuration will not be updated.
    pub client_configuration: Option<serde_json::Value>,
}

/// update_dataset
///
/// Update a dataset. The auth'ed user must be an owner of the organization to update a dataset.
#[utoipa::path(
    put,
    path = "/dataset",
    context_path = "/api",
    tag = "dataset",
    request_body(content = UpdateDatasetRequest, description = "JSON request payload to update a dataset", content_type = "application/json"),
    responses(
        (status = 200, description = "Dataset updated successfully", body = Dataset),
        (status = 400, description = "Service error relating to updating the dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
        ("Cookie" = ["owner"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn update_dataset(
    data: web::Json<UpdateDatasetRequest>,
    pool: web::Data<Pool>,
    _user: OwnerOnly,
) -> Result<HttpResponse, ServiceError> {
    let curr_dataset = get_dataset_by_id_query(data.dataset_id, pool.clone()).await?;
    let d = update_dataset_query(
        data.dataset_id,
        data.dataset_name.clone().unwrap_or(curr_dataset.name),
        data.server_configuration
            .clone()
            .unwrap_or(curr_dataset.server_configuration),
        data.client_configuration
            .clone()
            .unwrap_or(curr_dataset.client_configuration),
        pool.clone(),
    )
    .await?;
    let _ = refresh_redis_org_plan_sub(d.organization_id, pool.clone())
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!(
                "Error refreshing redis org plan sub: {}",
                err
            ))
        });
    Ok(HttpResponse::Ok().json(d))
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
pub struct DeleteDatasetRequest {
    /// The id of the dataset you want to delete.
    pub dataset_id: uuid::Uuid,
}

/// delete_dataset
///
/// Delete a dataset. The auth'ed user must be an owner of the organization to delete a dataset.
#[utoipa::path(
    delete,
    path = "/dataset",
    context_path = "/api",
    tag = "dataset",
    request_body(content = DeleteDatasetRequest, description = "JSON request payload to delete a dataset", content_type = "application/json"),
    responses(
        (status = 204, description = "Dataset deleted successfully"),
        (status = 400, description = "Service error relating to deleting the dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
        ("Cookie" = ["owner"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn delete_dataset(
    data: web::Json<DeleteDatasetRequest>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: OwnerOnly,
) -> Result<HttpResponse, ServiceError> {
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );
    delete_dataset_by_id_query(data.dataset_id, pool, server_dataset_config).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// get_dataset
///
/// Get a dataset by id. The auth'ed user must be an admin or owner of the organization to get a dataset.
#[utoipa::path(
    get,
    path = "/dataset/{dataset_id}",
    context_path = "/api",
    tag = "dataset",
    responses(
        (status = 200, description = "Dataset retrieved successfully", body = Dataset),
        (status = 400, description = "Service error relating to retrieving the dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid, Path, description = "The id of the dataset you want to retrieve."),
    ),
    security(
        ("ApiKey" = ["admin"]),
        ("Cookie" = ["admin"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_dataset(
    pool: web::Data<Pool>,
    dataset_id: web::Path<uuid::Uuid>,
    _user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let mut d = get_dataset_by_id_query(dataset_id.into_inner(), pool).await?;
    d.server_configuration = json!(ServerDatasetConfiguration::from_json(
        d.server_configuration
    ));
    d.client_configuration = json!(ClientDatasetConfiguration::from_json(
        d.client_configuration
    ));
    Ok(HttpResponse::Ok().json(d))
}

/// get_organization_datasets
///
/// Get all datasets for an organization. The auth'ed user must be an admin or owner of the organization to get its datasets.
#[utoipa::path(
    get,
    path = "/dataset/organization/{organization_id}",
    context_path = "/api",
    tag = "dataset",
    responses(
        (status = 200, description = "Datasets retrieved successfully", body = Vec<DatasetAndUsage>),
        (status = 400, description = "Service error relating to retrieving the dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
        ("organization_id" = uuid, Path, description = "id of the organization you want to retrieve datasets for"),
    ),
    security(
        ("ApiKey" = ["admin"]),
        ("Cookie" = ["admin"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_datasets_from_organization(
    organization_id: web::Path<uuid::Uuid>,
    user: AdminOnly,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_id = organization_id.into_inner();
    user.0
        .user_orgs
        .iter()
        .find(|org| org.organization_id == organization_id)
        .ok_or(ServiceError::Forbidden)?;

    let dataset_and_usages =
        web::block(move || get_datasets_by_organization_id(organization_id.into(), pool))
            .await
            .map_err(|e| ServiceError::InternalServerError(e.to_string()))??;

    Ok(HttpResponse::Ok().json(dataset_and_usages))
}

/// get_client_dataset_config
///
/// Get the client configuration for a dataset. Will use the TR-D
#[utoipa::path(
    get,
    path = "/dataset/envs",
    context_path = "/api",
    tag = "dataset",
    responses(
        (status = 200, description = "Dataset environment variables", body = ClientDatasetConfiguration),
        (status = 400, description = "Service error relating to retrieving the dataset. Typically this only happens when your auth credentials are invalid.", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
#[tracing::instrument]
pub async fn get_client_dataset_config(
    dataset: DatasetAndOrgWithSubAndPlan,
    _logged_user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    Ok(
        HttpResponse::Ok().json(ClientDatasetConfiguration::from_json(
            dataset.dataset.client_configuration,
        )),
    )
}
