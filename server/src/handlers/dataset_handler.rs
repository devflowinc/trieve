use super::auth_handler::{AdminOnly, OwnerOnly};
use crate::{
    data::models::{
        Dataset, DatasetAndOrgWithSubAndPlan, DatasetConfiguration, DatasetConfigurationDTO, Pool,
        RedisPool, StripePlan, UnifiedId,
    },
    errors::ServiceError,
    middleware::auth_middleware::{verify_admin, verify_owner},
    operators::{
        dataset_operator::{
            clear_dataset_by_dataset_id_query, create_dataset_query, get_dataset_by_id_query,
            get_dataset_usage_query, get_datasets_by_organization_id,
            soft_delete_dataset_by_id_query, update_dataset_query,
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
    "server_configuration": {
        "LLM_BASE_URL": "https://api.openai.com/v1",
        "EMBEDDING_BASE_URL": "https://api.openai.com/v1",
        "EMBEDDING_MODEL_NAME": "text-embedding-3-small",
        "MESSAGE_TO_QUERY_PROMPT": "Write a 1-2 sentence semantic search query along the lines of a hypothetical response to: \n\n",
        "RAG_PROMPT": "Use the following retrieved documents in your response. Include footnotes in the format of the document number that you used for a sentence in square brackets at the end of the sentences like [^n] where n is the doc number. These are the docs:",
        "N_RETRIEVALS_TO_INCLUDE": 8,
        "EMBEDDING_SIZE": 1536,
        "LLM_DEFAULT_MODEL": "gpt-3.5-turbo-1106",
        "BM25_ENABLED": true,
        "BM25_B": 0.75,
        "BM25_K": 0.75,
        "BM25_AVG_LEN": 256.0,
        "FULLTEXT_ENABLED": true,
        "SEMANTIC_ENABLED": true,
        "EMBEDDING_QUERY_PREFIX": "",
        "USE_MESSAGE_TO_QUERY_PROMPT": false,
        "FREQUENCY_PENALTY": 0.0,
        "TEMPERATURE": 0.5,
        "PRESENCE_PENALTY": 0.0,
        "STOP_TOKENS": ["\n\n", "\n"],
        "INDEXED_ONLY": false,
        "LOCKED": false,
        "SYSTEM_PROMPT": "You are a helpful assistant",
        "MAX_LIMIT": 10000
    }
}))]
pub struct CreateDatasetRequest {
    /// Name of the dataset.
    pub dataset_name: String,
    /// Organization ID that the dataset will belong to.
    pub organization_id: uuid::Uuid,
    /// Optional tracking ID for the dataset. Can be used to track the dataset in external systems. Must be unique within the organization. Strongly recommended to not use a valid uuid value as that will not work with the TR-Dataset header.
    pub tracking_id: Option<String>,
    /// The configuration of the dataset. See the example request payload for the potential keys which can be set. It is possible to break your dataset's functionality by erroneously setting this field. We recommend setting through creating a dataset at dashboard.trieve.ai and managing it's settings there.
    pub server_configuration: Option<DatasetConfigurationDTO>,
}

/// Create dataset
///
/// Create a new dataset. The auth'ed user must be an owner of the organization to create a dataset.
#[utoipa::path(
    post,
    path = "/dataset",
    context_path = "/api",
    tag = "Dataset",
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

    let unlimited = std::env::var("UNLIMITED").unwrap_or("false".to_string());
    if unlimited == "false"
        && dataset_count
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
        data.server_configuration
            .clone()
            .map(|c| c.into())
            .unwrap_or_default(),
    );

    let d = create_dataset_query(dataset, pool).await?;
    Ok(HttpResponse::Ok().json(d))
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
#[schema(example = json!({
    "dataset_id": "00000000-0000-0000-0000-000000000000",
    "dataset_name": "My Dataset",
    "server_configuration": {
        "LLM_BASE_URL": "https://api.openai.com/v1",
        "EMBEDDING_BASE_URL": "https://api.openai.com/v1",
        "EMBEDDING_MODEL_NAME": "text-embedding-3-small",
        "MESSAGE_TO_QUERY_PROMPT": "Write a 1-2 sentence semantic search query along the lines of a hypothetical response to: \n\n",
        "RAG_PROMPT": "Use the following retrieved documents in your response. Include footnotes in the format of the document number that you used for a sentence in square brackets at the end of the sentences like [^n] where n is the doc number. These are the docs:",
        "N_RETRIEVALS_TO_INCLUDE": 8,
        "EMBEDDING_SIZE": 1536,
        "LLM_DEFAULT_MODEL": "gpt-3.5-turbo-1106",
        "BM25_ENABLED": true,
        "BM25_B": 0.75,
        "BM25_K": 0.75,
        "BM25_AVG_LEN": 256.0,
        "FULLTEXT_ENABLED": true,
        "SEMANTIC_ENABLED": true,
        "EMBEDDING_QUERY_PREFIX": "",
        "USE_MESSAGE_TO_QUERY_PROMPT": false,
        "FREQUENCY_PENALTY": 0.0,
        "TEMPERATURE": 0.5,
        "PRESENCE_PENALTY": 0.0,
        "STOP_TOKENS": ["\n\n", "\n"],
        "INDEXED_ONLY": false,
        "LOCKED": false,
        "SYSTEM_PROMPT": "You are a helpful assistant",
        "MAX_LIMIT": 10000
    }
}))]
pub struct UpdateDatasetRequest {
    /// The id of the dataset you want to update.
    pub dataset_id: Option<uuid::Uuid>,
    /// The tracking ID of the dataset you want to update.
    pub tracking_id: Option<String>,
    /// The new name of the dataset. Must be unique within the organization. If not provided, the name will not be updated.
    pub dataset_name: Option<String>,
    /// The configuration of the dataset. See the example request payload for the potential keys which can be set. It is possible to break your dataset's functionality by erroneously updating this field. We recommend updating through the settings panel for your dataset at dashboard.trieve.ai.
    pub server_configuration: Option<DatasetConfigurationDTO>,
    /// Optional new tracking ID for the dataset. Can be used to track the dataset in external systems. Must be unique within the organization. If not provided, the tracking ID will not be updated. Strongly recommended to not use a valid uuid value as that will not work with the TR-Dataset header.
    pub new_tracking_id: Option<String>,
}

/// Update Dataset
///
/// Update a dataset by id or tracking_id. One of id or tracking_id must be provided. The auth'ed user must be an owner of the organization to update a dataset.
#[utoipa::path(
    put,
    path = "/dataset",
    context_path = "/api",
    tag = "Dataset",
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

    let curr_dataset_config = DatasetConfiguration::from_json(curr_dataset.server_configuration);

    let d = update_dataset_query(
        curr_dataset.id,
        data.dataset_name.clone().unwrap_or(curr_dataset.name),
        data.server_configuration
            .clone()
            .map(|c| c.from_curr_dataset(curr_dataset_config.clone()))
            .unwrap_or(curr_dataset_config),
        data.new_tracking_id.clone(),
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
    tag = "Dataset",
    responses(
        (status = 204, description = "Dataset deleted successfully"),
        (status = 400, description = "Service error relating to deleting the dataset", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
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

    let config = DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);

    soft_delete_dataset_by_id_query(data.into_inner(), config, pool, redis_pool).await?;
    Ok(HttpResponse::NoContent().finish())
}

/// Clear Dataset
///
/// Removes all chunks, files, and groups from the dataset while retaining the analytics and dataset itself. The auth'ed user must be an owner of the organization to clear a dataset.
#[utoipa::path(
    put,
    path = "/dataset/clear/{dataset_id}",
    context_path = "/api",
    tag = "Dataset",
    responses(
        (status = 204, description = "Dataset cleared successfully"),
        (status = 400, description = "Service error relating to deleting the dataset", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("dataset_id" = uuid, Path, description = "The id of the dataset you want to clear."),

    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn clear_dataset(
    data: web::Path<uuid::Uuid>,
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

    let config = DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);

    clear_dataset_by_dataset_id_query(data.into_inner(), config, redis_pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Delete Dataset by Tracking ID
///
/// Delete a dataset by its tracking id. The auth'ed user must be an owner of the organization to delete a dataset.
#[utoipa::path(
    delete,
    path = "/dataset/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "Dataset",
    responses(
        (status = 204, description = "Dataset deleted successfully"),
        (status = 400, description = "Service error relating to deleting the dataset", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
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

    let config = DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);

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
    tag = "Dataset",
    responses(
        (status = 200, description = "Dataset retrieved successfully", body = Dataset),
        (status = 400, description = "Service error relating to retrieving the dataset", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
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

    d.server_configuration = json!(DatasetConfiguration::from_json(d.server_configuration));

    Ok(HttpResponse::Ok().json(d))
}

/// Get Usage By Dataset ID
///
/// Get the usage for a dataset by its id.
#[utoipa::path(
    get,
    path = "/dataset/usage/{dataset_id}",
    context_path = "/api",
    tag = "Dataset",
    responses(
        (status = 200, description = "Dataset usage retrieved successfully", body = DatasetUsageCount),
        (status = 400, description = "Service error relating to retrieving the dataset usage", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
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
    tag = "Dataset",
    responses(
        (status = 200, description = "Dataset retrieved successfully", body = Dataset),
        (status = 400, description = "Service error relating to retrieving the dataset", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
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

    d.server_configuration = json!(DatasetConfiguration::from_json(d.server_configuration));

    Ok(HttpResponse::Ok().json(d))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GetDatasetsPagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Get Datasets from Organization
///
/// Get all datasets for an organization. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    get,
    path = "/dataset/organization/{organization_id}",
    context_path = "/api",
    tag = "Dataset",
    responses(
        (status = 200, description = "Datasets retrieved successfully", body = Vec<DatasetAndUsage>),
        (status = 400, description = "Service error relating to retrieving the dataset", body = ErrorResponseBody),
        (status = 404, description = "Could not find organization", body = ErrorResponseBody)
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
        ("organization_id" = uuid, Path, description = "id of the organization you want to retrieve datasets for"),
        ("limit" = Option<i64>, Query, description = "The number of records to return"),
        ("offset" = Option<i64>, Query, description = "The number of records to skip"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_datasets_from_organization(
    organization_id: web::Path<uuid::Uuid>,
    pagination: web::Query<GetDatasetsPagination>,
    user: AdminOnly,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let pagination = pagination.into_inner();
    let organization_id = organization_id.into_inner();
    user.0
        .user_orgs
        .iter()
        .find(|org| org.organization_id == organization_id)
        .ok_or(ServiceError::Forbidden)?;

    // If offset is set, limit must also be set and vice versa
    if pagination.offset.is_some() != pagination.limit.is_some() {
        return Err(ServiceError::BadRequest(
            "Pagination requires both offset and limit.".to_string(),
        )
        .into());
    }

    let dataset_and_usages =
        get_datasets_by_organization_id(organization_id.into(), pagination, pool)
            .await
            .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(dataset_and_usages))
}
