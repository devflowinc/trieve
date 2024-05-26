use super::auth_handler::{AdminOnly, LoggedUser, OwnerOnly};
use crate::{
    af_middleware::auth_middleware::{get_role_for_org, verify_admin, verify_owner},
    data::models::{Pool, RedisPool, UserOrganization, UserRole},
    errors::ServiceError,
    operators::{
        organization_operator::{
            create_organization_query, delete_organization_query, get_org_from_id_query,
            get_org_usage_by_id_query, get_org_users_by_id_query, update_organization_query,
        },
        user_operator::{add_user_to_organization, remove_user_from_org_query},
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Get Organization
///
/// Fetch the details of an organization by its id. The auth'ed user must be an admin or owner of the organization to fetch it.
#[utoipa::path(
    get,
    path = "/organization/{organization_id}",
    context_path = "/api",
    tag = "organization",
    responses(
        (status = 200, description = "Organization with the id that was requested", body = Organization),
        (status = 400, description = "Service error relating to finding the organization by id", body = ErrorResponseBody),
        (status = 404, description = "Organization not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
        ("organization_id" = Option<uuid::Uuid>, Path, description = "The id of the organization you want to fetch."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
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
    tag = "organization",
    responses(
        (status = 200, description = "Confirmation that the organization was deleted", body = Organization),
        (status = 400, description = "Service error relating to deleting the organization by id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
        ("organization_id" = Option<uuid::Uuid>, Path, description = "The id of the organization you want to fetch."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
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

    let org = delete_organization_query(
        Some(&req),
        Some(user.0.id),
        organization_id,
        pool,
        redis_pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(org))
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct UpdateOrganizationData {
    /// The id of the organization to update.
    organization_id: uuid::Uuid,
    /// The new name of the organization. If not provided, the name will not be updated.
    name: Option<String>,
}

/// Update Organization
///
/// Update an organization. Only the owner of the organization can update it.
#[utoipa::path(
    put,
    path = "/organization",
    context_path = "/api",
    tag = "organization",
    request_body(content = UpdateOrganizationData, description = "The organization data that you want to update", content_type = "application/json"),
    responses(
        (status = 200, description = "Updated organization object", body = Organization),
        (status = 400, description = "Service error relating to updating the organization", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn update_organization(
    organization: web::Json<UpdateOrganizationData>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    user: OwnerOnly,
) -> Result<HttpResponse, actix_web::Error> {
    if !verify_owner(&user, &organization.organization_id) {
        return Ok(HttpResponse::Forbidden().finish());
    }
    let organization_update_data = organization.into_inner();
    let old_organization =
        get_org_from_id_query(organization_update_data.organization_id, pool.clone()).await?;

    let updated_organization = update_organization_query(
        organization_update_data.organization_id,
        organization_update_data
            .name
            .unwrap_or(old_organization.organization.name)
            .as_str(),
        pool,
        redis_pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(updated_organization))
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct CreateOrganizationData {
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
    tag = "organization",
    request_body(content = CreateOrganizationData, description = "The organization data that you want to create", content_type = "application/json"),
    responses(
        (status = 200, description = "Created organization object", body = Organization),
        (status = 400, description = "Service error relating to creating the organization", body = ErrorResponseBody),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn create_organization(
    req: HttpRequest,
    organization: web::Json<CreateOrganizationData>,
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
        UserOrganization::from_details(user.id, created_organization.id, UserRole::Owner),
        pool,
        redis_pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(created_organization))
}

/// Get Organization Usage
///
/// Fetch the current usage specification of an organization by its id. The auth'ed user must be an admin or owner of the organization to fetch it.
#[utoipa::path(
    get,
    path = "/organization/usage/{organization_id}",
    context_path = "/api",
    tag = "organization",
    responses(
        (status = 200, description = "The current usage of the specified organization", body = OrganizationUsageCount),
        (status = 400, description = "Service error relating to finding the organization's usage by id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
        ("organization_id" = Option<uuid::Uuid>, Path, description = "The id of the organization you want to fetch the usage of."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_organization_usage(
    organization: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    if !verify_admin(&user, &organization) {
        return Ok(HttpResponse::Forbidden().finish());
    };

    let org_id = organization.into_inner();

    let usage = get_org_usage_by_id_query(org_id, pool).await?;

    Ok(HttpResponse::Ok().json(usage))
}

/// Get Organization Users
///
/// Fetch the users of an organization by its id. The auth'ed user must be an admin or owner of the organization to fetch it.
#[utoipa::path(
    get,
    path = "/organization/users/{organization_id}",
    context_path = "/api",
    tag = "organization",
    responses(
        (status = 200, description = "Array of users who belong to the specified by organization", body = Vec<SlimUser>),
        (status = 400, description = "Service error relating to finding the organization's users by id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
        ("organization_id" = Option<uuid::Uuid>, Path, description = "The id of the organization you want to fetch the users of."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_organization_users(
    organization: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    if !verify_admin(&user, &organization) {
        return Ok(HttpResponse::Forbidden().finish());
    };

    let org_id = organization.into_inner();

    let usage = get_org_users_by_id_query(org_id, pool).await?;

    Ok(HttpResponse::Ok().json(usage))
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct RemoveUserFromOrgData {
    /// The id of the organization to remove the user from.
    organization_id: uuid::Uuid,
    /// The id of the user to remove from the organization.
    user_id: uuid::Uuid,
}

/// Remove User From Organization
///
/// Remove a user from an organization. The auth'ed user must be an admin or owner of the organization to remove a user.
#[utoipa::path(
    delete,
    path = "/organization/{organization_id}/user/{user_id}",
    context_path = "/api",
    tag = "organization",
    responses(
        (status = 204, description = "Confirmation that the user was removed from the organization"),
        (status = 400, description = "Service error relating to removing the user from the organization", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
        ("user_id" = Option<uuid::Uuid>, Path, description = "The id of the user you want to remove from the organization."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn remove_user_from_org(
    data: web::Path<RemoveUserFromOrgData>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    admin: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    if !verify_admin(&admin, &data.organization_id) {
        return Ok(HttpResponse::Forbidden().finish());
    };

    let org_id = data.organization_id;
    let user_role = match get_role_for_org(&admin.0, &org_id.clone()) {
        Some(role) => role,
        None => return Err(ServiceError::Forbidden.into()),
    };

    remove_user_from_org_query(data.user_id, user_role, org_id, pool, redis_pool).await?;

    Ok(HttpResponse::NoContent().finish())
}
