use super::auth_handler::{AdminOnly, LoggedUser, OwnerOnly};
use crate::{
    af_middleware::auth_middleware::{verify_admin, verify_owner},
    data::models::{
        ClientDatasetConfiguration, Dataset, DatasetAndOrgWithSubAndPlan, Pool, RedisPool,
        ServerDatasetConfiguration, StripePlan, UnifiedId,
    },
    errors::ServiceError,
    operators::{
        dataset_operator::{
            create_dataset_query, get_dataset_by_id_query, get_dataset_usage_query,
            get_datasets_by_organization_id, soft_delete_dataset_by_id_query, update_dataset_query,
        },
        organization_operator::{get_org_dataset_count, get_org_from_id_query},
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
#[schema(example = json!({
    "dataset_name": "My Dataset",
    "organization_id": "00000000-0000-0000-0000-000000000000",
    "server_configuration": {},
    "client_configuration": {}
}))]
pub struct CreateDatasetRequest {
    /// Name of the dataset.
    pub dataset_name: String,
    /// Organization ID that the dataset will belong to.
    pub organization_id: uuid::Uuid,
    /// Optional tracking ID for the dataset. Can be used to track the dataset in external systems. Must be unique within the organization.
    pub tracking_id: Option<String>,
    /// Server configuration for the dataset, can be arbitrary JSON. We recommend setting to `{}` to start. See docs.trieve.ai for more information or adjust with the admin dashboard.
    pub server_configuration: serde_json::Value,
    /// Client configuration for the dataset, can be arbitrary JSON. We recommend setting to `{}` to start. See docs.trieve.ai for more information or adjust with the admin dashboard.
    pub client_configuration: serde_json::Value,
}

/// Create dataset
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
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn create_dataset(
    data: web::Json<CreateDatasetRequest>,
    pool: web::Data<Pool>,
    _user: OwnerOnly,
) -> Result<HttpResponse, ServiceError> {
    let org_id = data.organization_id;

    let organization_sub_plan = get_org_from_id_query(org_id, pool.clone()).await?;

    let dataset_count = get_org_dataset_count(org_id, pool.clone()).await?;

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
        data.tracking_id.clone(),
        data.server_configuration.clone(),
        data.client_configuration.clone(),
    );

    let d = create_dataset_query(dataset, pool).await?;
    Ok(HttpResponse::Ok().json(d))
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
#[schema(example = json!({
    "dataset_id": "00000000-0000-0000-0000-000000000000",
    "dataset_name": "My Dataset",
    "server_configuration": {},
    "client_configuration": {}
}))]
pub struct UpdateDatasetRequest {
    /// The id of the dataset you want to update.
    pub dataset_id: Option<uuid::Uuid>,
    /// tracking ID for the dataset. Can be used to track the dataset in external systems.
    pub tracking_id: Option<String>,
    /// The new name of the dataset. Must be unique within the organization. If not provided, the name will not be updated.
    pub dataset_name: Option<String>,
    /// The new server configuration of the dataset, can be arbitrary JSON. See docs.trieve.ai for more information. If not provided, the server configuration will not be updated.
    pub server_configuration: Option<serde_json::Value>,
    /// The new client configuration of the dataset, can be arbitrary JSON. See docs.trieve.ai for more information. If not provided, the client configuration will not be updated.
    pub client_configuration: Option<serde_json::Value>,
}

/// Update Dataset
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
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn update_dataset(
    data: web::Json<UpdateDatasetRequest>,
    pool: web::Data<Pool>,
    user: OwnerOnly,
) -> Result<HttpResponse, ServiceError> {
    let curr_dataset = if let Some(dataset_id) = data.dataset_id {
        get_dataset_by_id_query(UnifiedId::TrieveUuid(dataset_id), pool.clone()).await?
    } else if let Some(tracking_id) = &data.tracking_id {
        get_dataset_by_id_query(UnifiedId::TrackingId(tracking_id.clone()), pool.clone()).await?
    } else {
        return Err(ServiceError::BadRequest(
            "You must provide a dataset_id or tracking_id to update a dataset".to_string(),
        ));
    };

    if !verify_owner(&user, &curr_dataset.organization_id) {
        return Err(ServiceError::Forbidden);
    }

    let d = update_dataset_query(
        curr_dataset.id,
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

    Ok(HttpResponse::Ok().json(d))
}

/// Delete Dataset
///
/// Delete a dataset. The auth'ed user must be an owner of the organization to delete a dataset.
#[utoipa::path(
    delete,
    path = "/dataset/{dataset_id}",
    context_path = "/api",
    tag = "dataset",
    responses(
        (status = 204, description = "Dataset deleted successfully"),
        (status = 400, description = "Service error relating to deleting the dataset", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid, Path, description = "The id of the dataset you want to delete."),

    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
#[tracing::instrument(skip(pool, redis_pool))]
pub async fn delete_dataset(
    data: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    user: OwnerOnly,
) -> Result<HttpResponse, ServiceError> {
    if dataset_org_plan_sub.dataset.id != *data {
        return Err(ServiceError::BadRequest(
            "Dataset header does not match provided dataset ID".to_string(),
        ));
    }
    if !verify_owner(&user, &dataset_org_plan_sub.organization.organization.id) {
        return Err(ServiceError::Forbidden);
    }

    let config =
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);

    soft_delete_dataset_by_id_query(data.into_inner(), config, pool, redis_pool).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Delete Dataset by Tracking ID
///
/// Delete a dataset by its tracking id. The auth'ed user must be an owner of the organization to delete a dataset.
#[utoipa::path(
    delete,
    path = "/dataset/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "dataset",
    responses(
        (status = 204, description = "Dataset deleted successfully"),
        (status = 400, description = "Service error relating to deleting the dataset", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("tracking_id" = String, Path, description = "The tracking id of the dataset you want to delete."),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn delete_dataset_by_tracking_id(
    tracking_id: web::Path<String>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    redis_pool: web::Data<RedisPool>,
    user: OwnerOnly,
) -> Result<HttpResponse, ServiceError> {
    if dataset_org_plan_sub.dataset.tracking_id != Some(tracking_id.clone()) {
        return Err(ServiceError::BadRequest(
            "Dataset header does not match provided dataset ID".to_string(),
        ));
    }

    if !verify_owner(&user, &dataset_org_plan_sub.organization.organization.id) {
        return Err(ServiceError::Forbidden);
    }

    let config =
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);

    soft_delete_dataset_by_id_query(dataset_org_plan_sub.dataset.id, config, pool, redis_pool)
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Get Dataset
///
/// Get a dataset by id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    get,
    path = "/dataset/{dataset_id}",
    context_path = "/api",
    tag = "dataset",
    responses(
        (status = 200, description = "Dataset retrieved successfully", body = Dataset),
        (status = 400, description = "Service error relating to retrieving the dataset", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid, Path, description = "The id of the dataset you want to retrieve."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_dataset(
    pool: web::Data<Pool>,
    dataset_id: web::Path<uuid::Uuid>,
    user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let mut d =
        get_dataset_by_id_query(UnifiedId::TrieveUuid(dataset_id.into_inner()), pool).await?;

    if !verify_admin(&user, &d.organization_id) {
        return Err(ServiceError::Forbidden);
    }

    d.server_configuration = json!(ServerDatasetConfiguration::from_json(
        d.server_configuration
    ));
    d.client_configuration = json!(ClientDatasetConfiguration::from_json(
        d.client_configuration
    ));
    Ok(HttpResponse::Ok().json(d))
}

/// Get Usage By Dataset ID
///
/// Get the usage for a dataset by its id.
#[utoipa::path(
    get,
    path = "/dataset/{dataset_id}/usage",
    context_path = "/api",
    tag = "dataset",
    responses(
        (status = 200, description = "Dataset usage retrieved successfully", body = DatasetUsageCount),
        (status = 400, description = "Service error relating to retrieving the dataset usage", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid, Path, description = "The id of the dataset you want to retrieve usage for."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_usage_by_dataset_id(
    pool: web::Data<Pool>,
    dataset_id: web::Path<uuid::Uuid>,
    user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let usage = get_dataset_usage_query(dataset_id.into_inner(), pool)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(usage))
}

/// Get Dataset by Tracking ID
///
/// Get a dataset by its tracking id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    get,
    path = "/dataset/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "dataset",
    responses(
        (status = 200, description = "Dataset retrieved successfully", body = Dataset),
        (status = 400, description = "Service error relating to retrieving the dataset", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("tracking_id" = String, Path, description = "The tracking id of the dataset you want to retrieve."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_dataset_by_tracking_id(
    tracking_id: web::Path<String>,
    pool: web::Data<Pool>,
    user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let mut d = get_dataset_by_id_query(UnifiedId::TrackingId(tracking_id.into_inner()), pool)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    if !verify_admin(&user, &d.organization_id) {
        return Err(ServiceError::Forbidden);
    }

    d.server_configuration = json!(ServerDatasetConfiguration::from_json(
        d.server_configuration
    ));
    d.client_configuration = json!(ClientDatasetConfiguration::from_json(
        d.client_configuration
    ));
    Ok(HttpResponse::Ok().json(d))
}

/// Get Datasets from Organization
///
/// Get all datasets for an organization. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    get,
    path = "/dataset/organization/{organization_id}",
    context_path = "/api",
    tag = "dataset",
    responses(
        (status = 200, description = "Datasets retrieved successfully", body = Vec<DatasetAndUsage>),
        (status = 400, description = "Service error relating to retrieving the dataset", body = ErrorResponseBody),
        (status = 404, description = "Could not find organization", body = ErrorResponseBody)
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
        ("organization_id" = uuid, Path, description = "id of the organization you want to retrieve datasets for"),
    ),
    security(
        ("ApiKey" = ["admin"]),
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

    let dataset_and_usages = get_datasets_by_organization_id(organization_id.into(), pool)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(dataset_and_usages))
}

/// Get Client Configuration
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
