use super::auth_handler::{AdminOnly, LoggedUser, OwnerOnly};
use crate::{
    data::models::{
        CrawlOptions, Dataset, DatasetAndOrgWithSubAndPlan, DatasetConfiguration,
        DatasetConfigurationDTO, DatasetDTO, OrganizationWithSubAndPlan, Pool, RedisPool,
        StripePlan, UnifiedId,
    },
    errors::ServiceError,
    middleware::auth_middleware::{verify_admin, verify_owner},
    operators::{
        crawl_operator::{
            crawl, get_crawl_request_by_dataset_id_query, update_crawl_settings_for_dataset,
            validate_crawl_options,
        },
        dataset_operator::{
            clear_dataset_by_dataset_id_query, create_dataset_query, create_datasets_query,
            get_dataset_by_id_query, get_dataset_usage_query, get_datasets_by_organization_id,
            get_tags_in_dataset_query, soft_delete_dataset_by_id_query, update_dataset_query,
        },
        dittofeed_operator::{
            send_ditto_event, DittoDatasetCreated, DittoTrackProperties, DittoTrackRequest,
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

impl FromRequest for OrganizationWithSubAndPlan {
    type Error = ServiceError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        ready(
            req.extensions()
                .get::<OrganizationWithSubAndPlan>()
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
        "RAG_PROMPT": "Use the following retrieved documents to respond briefly and accurately:",
        "N_RETRIEVALS_TO_INCLUDE": 8,
        "EMBEDDING_SIZE": 1536,
        "DISTANCE_METRIC": "cosine",
        "LLM_DEFAULT_MODEL": "gpt-3.5-turbo-1106",
        "BM25_ENABLED": true,
        "BM25_B": 0.75,
        "BM25_K": 0.75,
        "BM25_AVG_LEN": 256.0,
        "FULLTEXT_ENABLED": true,
        "SEMANTIC_ENABLED": true,
        "QDRANT_ONLY": false,
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
pub struct CreateDatasetReqPayload {
    /// Name of the dataset.
    pub dataset_name: String,
    /// Optional tracking ID for the dataset. Can be used to track the dataset in external systems. Must be unique within the organization. Strongly recommended to not use a valid uuid value as that will not work with the TR-Dataset header.
    pub tracking_id: Option<String>,
    /// The configuration of the dataset. See the example request payload for the potential keys which can be set. It is possible to break your dataset's functionality by erroneously setting this field. We recommend setting through creating a dataset at dashboard.trieve.ai and managing it's settings there.
    pub server_configuration: Option<DatasetConfigurationDTO>,
    /// Optional site to crawl for the dataset. If provided, the dataset will be populated with the contents of the site.
    pub crawl_options: Option<CrawlOptions>,
}

/// Create Dataset
///
/// Dataset will be created in the org specified via the TR-Organization header. Auth'ed user must be an owner of the organization to create a dataset.
#[utoipa::path(
    post,
    path = "/dataset",
    context_path = "/api",
    tag = "Dataset",
    request_body(content = CreateDatasetReqPayload, description = "JSON request payload to create a new dataset", content_type = "application/json"),
    responses(
        (status = 200, description = "Dataset created successfully", body = Dataset),
        (status = 400, description = "Service error relating to creating the dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn create_dataset(
    data: web::Json<CreateDatasetReqPayload>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    org_with_sub_and_plan: OrganizationWithSubAndPlan,
    user: OwnerOnly,
) -> Result<HttpResponse, ServiceError> {
    let org_id = org_with_sub_and_plan.organization.id;

    let organization_sub_plan = get_org_from_id_query(org_id, pool.clone()).await?;

    let unlimited = std::env::var("UNLIMITED").unwrap_or("false".to_string());
    if unlimited == "false" {
        let dataset_count = get_org_dataset_count(org_id, pool.clone()).await?;
        if dataset_count
            >= organization_sub_plan
                .plan
                .unwrap_or(StripePlan::default())
                .dataset_count
        {
            return Ok(HttpResponse::UpgradeRequired().json(
                json!({"message": "Your plan must be upgraded to create additional datasets"}),
            ));
        }
    }

    if let Some(crawl_options) = data.crawl_options.clone() {
        validate_crawl_options(&crawl_options)?;
    };

    let dataset = Dataset::from_details(
        data.dataset_name.clone(),
        org_id,
        data.tracking_id.clone(),
        data.server_configuration
            .clone()
            .map(|c| c.into())
            .unwrap_or_default(),
    );

    let d = create_dataset_query(dataset.clone(), pool.clone()).await?;

    if let Some(crawl_options) = data.crawl_options.clone() {
        crawl(
            crawl_options.clone(),
            pool.clone(),
            redis_pool.clone(),
            dataset.id,
        )
        .await?;
    };

    let dataset_created_event = DittoTrackRequest {
        event: "DATASET_CREATED".to_string(),
        user_id: user.0.id,
        message_id: uuid::Uuid::new_v4(),
        properties: DittoTrackProperties::DittoDatasetCreated(DittoDatasetCreated {
            dataset: DatasetDTO::from(dataset),
        }),
        r#type: None,
    };

    match send_ditto_event(dataset_created_event).await {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error sending ditto event: {}", e);
        }
    };

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
        "RAG_PROMPT": "Use the following retrieved documents to respond briefly and accurately:",
        "N_RETRIEVALS_TO_INCLUDE": 8,
        "EMBEDDING_SIZE": 1536,
        "LLM_DEFAULT_MODEL": "gpt-3.5-turbo-1106",
        "BM25_ENABLED": true,
        "BM25_B": 0.75,
        "BM25_K": 0.75,
        "BM25_AVG_LEN": 256.0,
        "FULLTEXT_ENABLED": true,
        "SEMANTIC_ENABLED": true,
        "QDRANT_ONLY": false,
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
pub struct UpdateDatasetReqPayload {
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
    /// Update crawler settings for the dataset. If provided, the dataset will be populated with the contents of the site.
    pub crawl_options: Option<CrawlOptions>,
}

/// Update Dataset by ID or Tracking ID
///
/// One of id or tracking_id must be provided. The auth'ed user must be an owner of the organization to update a dataset.
#[utoipa::path(
    put,
    path = "/dataset",
    context_path = "/api",
    tag = "Dataset",
    request_body(content = UpdateDatasetReqPayload, description = "JSON request payload to update a dataset", content_type = "application/json"),
    responses(
        (status = 200, description = "Dataset updated successfully", body = Dataset),
        (status = 400, description = "Service error relating to updating the dataset", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn update_dataset(
    data: web::Json<UpdateDatasetReqPayload>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
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

    if let Some(crawl_options) = data.crawl_options.clone() {
        validate_crawl_options(&crawl_options)?;
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

    if let Some(crawl_options) = data.crawl_options.clone() {
        update_crawl_settings_for_dataset(
            crawl_options.clone(),
            curr_dataset.id,
            pool.clone(),
            redis_pool.clone(),
        )
        .await?;
    };

    Ok(HttpResponse::Ok().json(d))
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
#[schema(example = json!({
    "crawl_options": {
        "site_url": "https://example.com",
        "interval": "daily",
        "limit": 1000,
        "exclude_paths": ["https://example.com/exclude"],
        "include_paths": ["https://example.com/include"],
        "include_tags": ["h1", "p", "a", ".main-content"],
        "exclude_tags": ["#ad", "#footer"],
    }
}))]
pub struct GetCrawlOptionsResponse {
    crawl_options: Option<CrawlOptions>,
}

/// Get Dataset Crawl Options
/// Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    get,
    path = "/dataset/crawl_options/{dataset_id}",
    context_path = "/api",
    tag = "Dataset",
    responses(
        (status = 200, description = "Crawl options retrieved successfully", body = GetCrawlOptionsResponse),
        (status = 400, description = "Service error relating to retrieving the crawl options", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("dataset_id" = uuid, Path, description = "The id of the dataset you want to retrieve."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_dataset_crawl_options(
    pool: web::Data<Pool>,
    dataset_id: web::Path<uuid::Uuid>,
    user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let d = get_dataset_by_id_query(UnifiedId::TrieveUuid(dataset_id.into_inner()), pool.clone())
        .await?;
    let crawl_req = get_crawl_request_by_dataset_id_query(d.id, pool).await?;

    if !verify_admin(&user, &d.organization_id) {
        return Err(ServiceError::Forbidden);
    }

    Ok(HttpResponse::Ok().json(GetCrawlOptionsResponse {
        crawl_options: crawl_req.map(|req| req.crawl_options),
    }))
}

/// Delete Dataset
///
/// Auth'ed user must be an owner of the organization to delete a dataset.
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
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("dataset_id" = uuid, Path, description = "The id of the dataset you want to delete."),

    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
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
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
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
/// Auth'ed user must be an owner of the organization to delete a dataset.
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
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("tracking_id" = String, Path, description = "The tracking id of the dataset you want to delete."),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn delete_dataset_by_tracking_id(
    tracking_id: web::Path<String>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    user: OwnerOnly,
) -> Result<HttpResponse, ServiceError> {
    let dataset = get_dataset_by_id_query(
        UnifiedId::TrackingId(tracking_id.into_inner()),
        pool.clone(),
    )
    .await?;

    if !verify_owner(&user, &dataset.organization_id) {
        return Err(ServiceError::Forbidden);
    }

    let config = DatasetConfiguration::from_json(dataset.server_configuration);

    soft_delete_dataset_by_id_query(dataset.id, config, pool, redis_pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Get Dataset By ID
///
/// Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
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
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("dataset_id" = uuid, Path, description = "The id of the dataset you want to retrieve."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_dataset(
    pool: web::Data<Pool>,
    dataset_id: web::Path<uuid::Uuid>,
    user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let mut dataset =
        get_dataset_by_id_query(UnifiedId::TrieveUuid(dataset_id.into_inner()), pool).await?;

    if !verify_admin(&user, &dataset.organization_id) {
        return Err(ServiceError::Forbidden);
    }

    dataset.server_configuration = json!(DatasetConfiguration::from_json(
        dataset.server_configuration
    ));

    Ok(HttpResponse::Ok().json(dataset))
}

/// Get Usage By Dataset ID
///
/// Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
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
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("dataset_id" = uuid, Path, description = "The id of the dataset you want to retrieve usage for."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_usage_by_dataset_id(
    pool: web::Data<Pool>,
    clickhouse_client: web::Data<clickhouse::Client>,
    dataset_id: web::Path<uuid::Uuid>,
    _user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let usage = get_dataset_usage_query(dataset_id.into_inner(), pool, clickhouse_client.get_ref())
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(usage))
}

/// Get Dataset by Tracking ID
///
/// Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
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
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("tracking_id" = String, Path, description = "The tracking id of the dataset you want to retrieve."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_dataset_by_tracking_id(
    tracking_id: web::Path<String>,
    pool: web::Data<Pool>,
    user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let mut dataset =
        get_dataset_by_id_query(UnifiedId::TrackingId(tracking_id.into_inner()), pool)
            .await
            .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    if !verify_admin(&user, &dataset.organization_id) {
        return Err(ServiceError::Forbidden);
    }

    dataset.server_configuration = json!(DatasetConfiguration::from_json(
        dataset.server_configuration
    ));

    Ok(HttpResponse::Ok().json(dataset))
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Default)]
pub struct GetDatasetsPagination {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Get Datasets from Organization
///
/// Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
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
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
        ("organization_id" = uuid, Path, description = "id of the organization you want to retrieve datasets for"),
        ("limit" = Option<i64>, Query, description = "The number of records to return"),
        ("offset" = Option<i64>, Query, description = "The number of records to skip"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
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

    let dataset_and_usages = get_datasets_by_organization_id(organization_id, pagination, pool)
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    Ok(HttpResponse::Ok().json(dataset_and_usages))
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GetAllTagsReqPayload {
    /// Number of items to return per page. Default is 20.
    pub page_size: Option<i64>,
    /// Page number to return, 1-indexed. Default is 1.
    pub page: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Queryable)]
pub struct TagsWithCount {
    /// Content of the tag
    pub tag: String,
    /// Number of chunks in the dataset with that tag
    pub count: i64,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GetAllTagsResponse {
    /// List of tags with the number of chunks in the dataset with that tag.
    pub tags: Vec<TagsWithCount>,
    /// Total number of unique tags in the dataset.
    pub total: i64,
}

/// Get All Tags
///
/// Scroll through all tags in the dataset and get the number of chunks in the dataset with that tag plus the total number of unique tags for the whole datset.
#[utoipa::path(
    post,
    path = "/dataset/get_all_tags",
    context_path = "/api",
    tag = "Dataset",
    request_body(content = GetAllTagsReqPayload, description = "JSON request payload to get items with the tag in the request", content_type = "application/json"),
    responses(
        (status = 200, description = "Page of tags requested with all tags and the number of chunks in the dataset with that tag plus the total number of unique tags for the whole datset", body = GetAllTagsResponse),
        (status = 400, description = "Service error relating to finding items by tag", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_all_tags(
    data: web::Json<GetAllTagsReqPayload>,
    _user: LoggedUser,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let page = data.page.unwrap_or(1);
    if page < 1 {
        return Err(ServiceError::BadRequest(
            "Page must be greater than 0".to_string(),
        ));
    }
    let page_size = data.page_size.unwrap_or(20);
    let items = get_tags_in_dataset_query(dataset_id, page, page_size, pool).await?;

    Ok(HttpResponse::Ok().json(GetAllTagsResponse {
        tags: items.0,
        total: items.1,
    }))
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CreateBatchDataset {
    /// Name of the dataset.
    pub dataset_name: String,
    /// Optional tracking ID for the dataset. Can be used to track the dataset in external systems. Must be unique within the organization. Strongly recommended to not use a valid uuid value as that will not work with the TR-Dataset header.
    pub tracking_id: Option<String>,
    /// The configuration of the dataset. See the example request payload for the potential keys which can be set. It is possible to break your dataset's functionality by erroneously setting this field. We recommend setting through creating a dataset at dashboard.trieve.ai and managing it's settings there.
    pub server_configuration: Option<DatasetConfigurationDTO>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CreateDatasetBatchReqPayload {
    /// List of datasets to create
    pub datasets: Vec<CreateBatchDataset>,
    /// Upsert when a dataset with one of the specified tracking_ids already exists. By default this is false and specified datasets with a tracking_id that already exists in the org will not be ignored. If true, the existing dataset will be updated with the new dataset's details.
    pub upsert: Option<bool>,
}

/// Datasets
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Datasets(Vec<Dataset>);

/// Batch Create Datasets
///
/// Datasets will be created in the org specified via the TR-Organization header. Auth'ed user must be an owner of the organization to create datasets. If a tracking_id is ignored due to it already existing on the org, the response will not contain a dataset with that tracking_id and it can be assumed that a dataset with the missing tracking_id already exists.
#[utoipa::path(
    post,
    path = "/dataset/batch_create_datasets",
    context_path = "/api",
    tag = "Dataset",
    request_body(content = CreateDatasetBatchReqPayload, description = "JSON request payload to bulk create datasets", content_type = "application/json"),
    responses(
        (status = 200, description = "Page of tags requested with all tags and the number of chunks in the dataset with that tag plus the total number of unique tags for the whole datset", body = Datasets),
        (status = 400, description = "Service error relating to finding items by tag", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn batch_create_datasets(
    data: web::Json<CreateDatasetBatchReqPayload>,
    _user: OwnerOnly,
    pool: web::Data<Pool>,
    org_with_sub_and_plan: OrganizationWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let datasets = data
        .datasets
        .iter()
        .map(|d| {
            Dataset::from_details(
                d.dataset_name.clone(),
                org_with_sub_and_plan.organization.id,
                d.tracking_id.clone(),
                d.server_configuration
                    .clone()
                    .map(|c| c.into())
                    .unwrap_or_default(),
            )
        })
        .collect::<Vec<_>>();

    let created_or_upserted_datasets =
        create_datasets_query(datasets, data.upsert, pool.clone()).await?;

    Ok(HttpResponse::Ok().json(created_or_upserted_datasets))
}
