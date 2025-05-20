use super::auth_handler::{AdminOnly, LoggedUser, OwnerOnly};
use crate::data::models::DateRange;
use crate::operators::organization_operator::{
    get_extended_org_usage_by_id_query, get_org_usage_by_id_query,
};
use crate::{
    data::models::{
        ApiKeyRequestParams, ApiKeyRespBody, OrganizationWithSubAndPlan, Pool, RedisPool,
        UserOrganization, UserRole,
    },
    errors::ServiceError,
    middleware::auth_middleware::{get_role_for_org, verify_admin, verify_owner},
    operators::{
        organization_operator::{
            create_organization_api_key_query, create_organization_query,
            delete_organization_api_keys_query, delete_organization_query, get_org_from_id_query,
            get_org_users_by_id_query, get_organization_api_keys_query,
            update_all_org_dataset_configs_query, update_organization_query,
        },
        user_operator::{add_user_to_organization, remove_user_from_org_query},
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use sanitize_html::rules;
use sanitize_html::sanitize_str;
use serde::{Deserialize, Serialize};
use simple_server_timing_header::Timer;
use utoipa::ToSchema;

/// Get Organization
///
/// Fetch the details of an organization by its id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    get,
    path = "/organization/{organization_id}",
    context_path = "/api",
    tag = "Organization",
    responses(
        (status = 200, description = "Organization with the id that was requested", body = OrganizationWithSubAndPlan),
        (status = 400, description = "Service error relating to finding the organization by id", body = ErrorResponseBody),
        (status = 404, description = "Organization not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
        ("organization_id" = Option<uuid::Uuid>, Path, description = "The id of the organization you want to fetch."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_organization(
    organization_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    if !verify_admin(&user, &organization_id) {
        return Ok(HttpResponse::Forbidden().finish());
    };
    let organization_id = organization_id.into_inner();

    let org_plan_sub = get_org_from_id_query(organization_id, pool).await?;

    Ok(HttpResponse::Ok().json(org_plan_sub.with_defaults()))
}

/// Delete Organization
///
/// Delete an organization by its id. The auth'ed user must be an owner of the organization to delete it.
#[utoipa::path(
    delete,
    path = "/organization/{organization_id}",
    context_path = "/api",
    tag = "Organization",
    responses(
        (status = 204, description = "Confirmation that the organization was deleted"),
        (status = 400, description = "Service error relating to deleting the organization by id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
        ("organization_id" = Option<uuid::Uuid>, Path, description = "The id of the organization you want to fetch."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn delete_organization(
    req: HttpRequest,
    organization_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    user: OwnerOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_id = organization_id.into_inner();

    if !verify_owner(&user, &organization_id) {
        return Ok(HttpResponse::Forbidden().finish());
    }

    delete_organization_query(
        Some(&req),
        Some(user.0.id),
        organization_id,
        pool,
        redis_pool,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct UpdateOrganizationReqPayload {
    /// The new name of the organization. If not provided, the name will not be updated.
    name: Option<String>,
    /// New details for the partnership configuration. If not provided, the partnership configuration will not be updated.
    partner_configuration: Option<serde_json::Value>,
}

/// Update Organization
///
/// Update an organization. Only the owner of the organization can update it.
#[utoipa::path(
    put,
    path = "/organization",
    context_path = "/api",
    tag = "Organization",
    request_body(content = UpdateOrganizationReqPayload, description = "The organization data that you want to update", content_type = "application/json"),
    responses(
        (status = 200, description = "Updated organization object", body = Organization),
        (status = 400, description = "Service error relating to updating the organization", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn update_organization(
    organization: web::Json<UpdateOrganizationReqPayload>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    org_with_plan_and_sub: OrganizationWithSubAndPlan,
    user: OwnerOnly,
) -> Result<HttpResponse, actix_web::Error> {
    if !verify_owner(&user, &org_with_plan_and_sub.organization.id) {
        return Ok(HttpResponse::Forbidden().finish());
    }
    let organization_update_data = organization.into_inner();
    let old_organization =
        get_org_from_id_query(org_with_plan_and_sub.organization.id, pool.clone()).await?;

    let updated_organization = update_organization_query(
        org_with_plan_and_sub.organization.id,
        &sanitize_str(
            &rules::predefined::DEFAULT,
            organization_update_data
                .name
                .unwrap_or(old_organization.organization.name)
                .as_str(),
        )
        .map_err(|_| {
            ServiceError::BadRequest("Failed to sanitize organization name".to_string())
        })?,
        organization_update_data
            .partner_configuration
            .unwrap_or(old_organization.organization.partner_configuration),
        pool,
        redis_pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(updated_organization))
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct CreateOrganizationReqPayload {
    /// The arbitrary name which will be used to identify the organization. This name must be unique.
    name: String,
}

/// Create Organization
///
/// Create a new organization. The auth'ed user who creates the organization will be the default owner of the organization.
#[utoipa::path(
    post,
    path = "/organization",
    context_path = "/api",
    tag = "Organization",
    request_body(content = CreateOrganizationReqPayload, description = "The organization data that you want to create", content_type = "application/json"),
    responses(
        (status = 200, description = "Created organization object", body = Organization),
        (status = 400, description = "Service error relating to creating the organization", body = ErrorResponseBody),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn create_organization(
    req: HttpRequest,
    organization: web::Json<CreateOrganizationReqPayload>,
    pool: web::Data<Pool>,
    user: LoggedUser,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_create_data = organization.into_inner();

    let created_organization =
        create_organization_query(organization_create_data.name.as_str(), pool.clone()).await?;

    add_user_to_organization(
        Some(&req),
        Some(user.id),
        UserOrganization::from_details(user.id, created_organization.id, UserRole::Owner, None),
        pool,
        redis_pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(created_organization))
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct GetOrganizationUsageReqPayload {
    date_range: Option<DateRange>,
    v1_usage: Option<bool>,
}

/// Get Organization Usage
///
/// Fetch the current usage specification of an organization by its id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/organization/usage/{organization_id}",
    context_path = "/api",
    tag = "Organization",
    request_body(content = GetOrganizationUsageReqPayload, description = "The organization usage timeframe that you want to fetch", content_type = "application/json"),
    responses(
        (status = 200, description = "The current usage of the specified organization", body = ExtendedOrganizationUsageCount),
        (status = 400, description = "Service error relating to finding the organization's usage by id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
        ("organization_id" = Option<uuid::Uuid>, Path, description = "The id of the organization you want to fetch the usage of."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_organization_usage(
    organization: web::Path<uuid::Uuid>,
    data: web::Json<GetOrganizationUsageReqPayload>,
    pool: web::Data<Pool>,
    clickhouse_client: web::Data<clickhouse::Client>,
    _: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let org_id = organization.into_inner();

    if data.v1_usage.unwrap_or(false) {
        let usage = get_org_usage_by_id_query(org_id, pool).await?;
        Ok(HttpResponse::Ok().json({
            ExtendedOrganizationUsageCount {
                dataset_count: usage.dataset_count,
                user_count: usage.user_count,
                file_storage: usage.file_storage,
                message_count: usage.message_count as u64,
                chunk_count: usage.chunk_count,
                search_tokens: 0,
                message_tokens: 0,
                search_count: 0,
                bytes_ingested: 0,
                tokens_ingested: 0,
                ocr_pages_ingested: 0,
                website_pages_scraped: 0,
                events_ingested: 0,
                current_months_message_count: 0,
            }
        }))
    } else {
        let mut usage_timer = Some(Timer::new());

        let extended_usage = get_extended_org_usage_by_id_query(
            org_id,
            data.date_range.clone(),
            clickhouse_client.get_ref(),
            pool,
            &mut usage_timer,
        )
        .await?;

        Ok(HttpResponse::Ok()
            .insert_header((
                Timer::header_key(),
                usage_timer.expect("Timer should exist").header_value(),
            ))
            .json(extended_usage))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct ExtendedOrganizationUsageCount {
    pub search_tokens: u64,
    pub message_tokens: u64,
    pub dataset_count: i32,
    pub user_count: i32,
    pub file_storage: i64,
    pub message_count: u64,
    pub search_count: u64,
    pub chunk_count: i32,
    pub bytes_ingested: u64,  // For dataset size
    pub tokens_ingested: u64, // For ingest charge
    pub ocr_pages_ingested: u64,
    // website pages scraped
    pub website_pages_scraped: u64,
    pub events_ingested: u64,
    pub current_months_message_count: u64,
}

/// Get Organization Users
///
/// Fetch the users of an organization by its id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    get,
    path = "/organization/users/{organization_id}",
    context_path = "/api",
    tag = "Organization",
    responses(
        (status = 200, description = "Array of users who belong to the specified by organization", body = Vec<SlimUser>),
        (status = 400, description = "Service error relating to finding the organization's users by id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
        ("organization_id" = Option<uuid::Uuid>, Path, description = "The id of the organization you want to fetch the users of."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_organization_users(
    organization_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    if !verify_admin(&user, &organization_id) {
        return Ok(HttpResponse::Forbidden().finish());
    };

    let org_id = organization_id.into_inner();

    let usage = get_org_users_by_id_query(org_id, pool).await?;

    Ok(HttpResponse::Ok().json(usage))
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RemoveUserFromOrgPathParams {
    /// The id of the user to remove from the organization.
    user_id: uuid::Uuid,
}

/// Remove User From Organization
///
/// Remove a user from an organization. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization..
#[utoipa::path(
    delete,
    path = "/organization/{organization_id}/user/{user_id}",
    context_path = "/api",
    tag = "Organization",
    responses(
        (status = 204, description = "Confirmation that the user was removed from the organization"),
        (status = 400, description = "Service error relating to removing the user from the organization", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
        ("organization_id" = uuid::Uuid, Path, description = "The id of the organization you want to remove the user from"),
        ("user_id" = uuid::Uuid, Path, description = "The id of the user you want to remove from the organization"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn remove_user_from_org(
    data: web::Path<RemoveUserFromOrgPathParams>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    org_with_plan_and_sub: OrganizationWithSubAndPlan,
    user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    if !verify_admin(&user, &org_with_plan_and_sub.organization.id) {
        return Ok(HttpResponse::Forbidden().finish());
    };

    let org_id = org_with_plan_and_sub.organization.id;
    let user_role = match get_role_for_org(&user.0, &org_id.clone()) {
        Some(role) => role,
        None => return Err(ServiceError::Forbidden.into()),
    };

    remove_user_from_org_query(data.user_id, user_role, org_id, pool, redis_pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "organization_id": "00000000-0000-0000-0000-000000000000",
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
        "MAX_LIMIT": 10000,
        "AIMON_RERANKER_TASK_DEFINITION":"Your task is to grade the relevance of context document(s) against the specified user query."
    }
}))]
pub struct UpdateAllOrgDatasetConfigsReqPayload {
    /// The configuration to provide a filter on what datasets to update.
    pub match_configuration: Option<serde_json::Value>,
    /// The new configuration for all datasets in the organization. Only the specified keys in the configuration object will be changed per dataset such that you can preserve dataset unique values.
    pub to_configuration: serde_json::Value,
}

/// Update All Dataset Configurations
///
/// Update the configurations for all datasets in an organization. Only the specified keys in the configuration object will be changed per dataset such that you can preserve dataset unique values. Auth'ed user or api key must have an owner role for the specified organization.
#[utoipa::path(
    post,
    path = "/organization/update_dataset_configs",
    context_path = "/api",
    tag = "Organization",
    request_body(content = UpdateAllOrgDatasetConfigsReqPayload, description = "The organization data that you want to create", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the dataset ServerConfigurations were updated successfully"),
        (status = 400, description = "Service error relating to updating the dataset ServerConfigurations", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn update_all_org_dataset_configs(
    req_payload: web::Json<UpdateAllOrgDatasetConfigsReqPayload>,
    pool: web::Data<Pool>,
    org_with_plan_and_sub: OrganizationWithSubAndPlan,
    user: OwnerOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_id = org_with_plan_and_sub.organization.id;
    if !verify_owner(&user, &organization_id) {
        return Ok(HttpResponse::Forbidden().finish());
    };

    let req_payload = req_payload.into_inner();

    let new_dataset_config = req_payload.to_configuration.clone();
    let match_configuration = req_payload.match_configuration.clone();

    update_all_org_dataset_configs_query(
        organization_id,
        new_dataset_config,
        match_configuration,
        pool,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Default)]
pub struct CreateApiKeyReqPayload {
    /// The name which will be assigned to the new api key.
    pub name: String,
    /// The role which will be assigned to the new api key. Either 0 (read), 1 (Admin) or 2 (Owner). The auth'ed user must have a role greater than or equal to the role being assigned.
    pub role: i32,
    /// The dataset ids which the api key will have access to. If not provided or empty, the api key will have access to all datasets in the dataset.
    pub dataset_ids: Option<Vec<uuid::Uuid>>,
    /// The routes which the api key will have access to. If not provided or empty, the api key will have access to all routes. Specify the routes as a list of strings. For example, ["GET /api/dataset", "POST /api/dataset"].
    pub scopes: Option<Vec<String>>,
    /// The expiration date of the api key. If not provided, the api key will not expire. This should be provided in UTC time.
    pub expires_at: Option<String>,
    /// The default parameters which will be forcibly used when the api key is given on a request. If not provided, the api key will not have default parameters.
    pub default_params: Option<ApiKeyRequestParams>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CreateApiKeyResponse {
    /// The api key which was created. This is the value which should be used in the Authorization header.
    api_key: String,
}

/// Create Organization Api Key
///
/// Create a new api key for the organization. Successful response will contain the newly created api key.
#[utoipa::path(
    post,
    path = "/organization/api_key",
    context_path = "/api",
    tag = "Organization",
    request_body(content = CreateApiKeyReqPayload, description = "JSON request payload to create a new organization api key", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the api_key for the organization", body = CreateApiKeyResponse),
        (status = 400, description = "Service error relating to creating api_key for the organization", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn create_organization_api_key(
    _user: LoggedUser,
    data: web::Json<CreateApiKeyReqPayload>,
    organization: OrganizationWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let new_api_key =
        create_organization_api_key_query(organization.organization.id, data.into_inner(), pool)
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Failed to set new API key for organization {}",
                    err
                ))
            })?;

    Ok(HttpResponse::Ok().json(CreateApiKeyResponse {
        api_key: new_api_key,
    }))
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct GetOrganizationApiKeysQuery {
    /// The cursor to start the pagination from.
    cursor: Option<uuid::Uuid>,
    /// The number of items to return per page.
    limit: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct GetOrganizationApiKeysResponse {
    /// The api keys which belong to the organization.
    pub api_keys: Vec<ApiKeyRespBody>,
    /// The cursor to start the pagination from.
    pub cursor: Option<uuid::Uuid>,
}

/// Get Organization Api Keys
///
/// Get the api keys which belong to the organization. The actual api key values are not returned, only the ids, names, and creation dates.
#[utoipa::path(
    get,
    path = "/organization/api_key",
    context_path = "/api",
    tag = "Organization",
    responses(
        (status = 200, description = "JSON body representing the api_key for the organization", body = Vec<ApiKeyRespBody>),
        (status = 400, description = "Service error relating to creating api_key for the organization", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request."),
        ("cursor" = Option<String>, Query, description = "The cursor to start the pagination from."),
        ("limit" = Option<i32>, Query, description = "The number of items to return per page."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_organization_api_keys(
    query: web::Query<GetOrganizationApiKeysQuery>,
    _user: AdminOnly,
    organization: OrganizationWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data = query.into_inner();
    let cursor = data.cursor.unwrap_or(uuid::Uuid::nil());
    let limit = data.limit.unwrap_or(10);
    let api_keys =
        get_organization_api_keys_query(organization.organization.id, cursor, limit, pool)
            .await
            .map_err(|_err| ServiceError::BadRequest("Failed to get API keys for user".into()))?;

    Ok(HttpResponse::Ok().json(api_keys))
}

/// Delete Organization Api Key
///
/// Delete an api key for the auth'ed organization.
#[utoipa::path(
    delete,
    path = "/organization/api_key/{api_key_id}",
    context_path = "/api",
    tag = "Organization",
    responses(
        (status = 204, description = "Confirmation that the api key was deleted"),
        (status = 400, description = "Service error relating to creating api_key for the organization", body = ErrorResponseBody),
    ),
    params(
        ("api_key_id" = uuid::Uuid, Path, description = "The id of the api key to delete"),
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn delete_organization_api_key(
    _user: AdminOnly,
    organization: OrganizationWithSubAndPlan,
    data: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    delete_organization_api_keys_query(organization.organization.id, data.into_inner(), pool)
        .await
        .map_err(|_err| ServiceError::BadRequest("Failed to get API keys for user".into()))?;

    Ok(HttpResponse::NoContent().finish())
}
