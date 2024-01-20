use super::auth_handler::{AdminOnly, LoggedUser, OwnerOnly};
use crate::{
    data::models::{Pool, UserOrganization, UserRole},
    errors::ServiceError,
    operators::{
        organization_operator::{
            create_organization_query, get_org_usage_by_id_query, get_org_users_by_id_query,
            get_organization_by_key_query, update_organization_query,
        },
        user_operator::add_user_to_organization,
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// get_organization
/// 
/// Fetch the details of an organization by its id. The auth'ed user must be an admin or owner of the organization to fetch it.
#[utoipa::path(
    get,
    path = "/organization/{organization_id}",
    context_path = "/api",
    tag = "organization",
    responses(
        (status = 200, description = "Organization with the id that was requested", body = [Organization]),
        (status = 400, description = "Service error relating to finding the organization by id", body = [DefaultError]),
    ),
    params(
        ("organization_id" = Option<uuid>, Path, description = "The id of the organization you want to fetch.")
    ),
)]
pub async fn get_organization_by_id(
    organization_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_id = organization_id.into_inner();

    let org_plan_sub = get_organization_by_key_query(organization_id.into(), pool)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(org_plan_sub.with_defaults()))
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateOrganizationData {
    /// The id of the organization to update.
    organization_id: uuid::Uuid,
    /// The new name of the organization. If not provided, the name will not be updated.
    name: Option<String>,
}

/// update_organization
/// 
/// Update an organization. Only the owner of the organization can update it.
#[utoipa::path(
    put,
    path = "/organization",
    context_path = "/api",
    tag = "organization",
    request_body(content = UpdateOrganizationData, description = "The organization data that you want to update", content_type = "application/json"),
    responses(
        (status = 200, description = "Updated organization object", body = [Organization]),
        (status = 400, description = "Service error relating to updating the organization", body = [DefaultError]),
    ),
)]
pub async fn update_organization(
    organization: web::Json<UpdateOrganizationData>,
    pool: web::Data<Pool>,
    _user: OwnerOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_update_data = organization.into_inner();
    let old_organization = get_organization_by_key_query(
        organization_update_data.organization_id.into(),
        pool.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let updated_organization = update_organization_query(
        organization_update_data.organization_id,
        organization_update_data
            .name
            .unwrap_or(old_organization.name)
            .as_str(),
        pool,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(updated_organization))
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateOrganizationData {
    /// The arbitrary name which will be used to identify the organization. This name must be unique.
    name: String,
    /// The configuration for the organization. Should likely default to `{}`. This is a JSON object which can be used to store configuration parameters. See docs.trieve.ai for more information or use the admin panel to set.
    configuration: serde_json::Value,
}

/// create_organization
///
/// Create a new organization. The auth'ed user who creates the organization will be the default owner of the organization.
#[utoipa::path(
    post,
    path = "/organization",
    context_path = "/api",
    tag = "organization",
    request_body(content = CreateOrganizationData, description = "The organization data that you want to create", content_type = "application/json"),
    responses(
        (status = 200, description = "Created organization object", body = [Organization]),
        (status = 400, description = "Service error relating to creating the organization", body = [DefaultError]),
    ),
)]
pub async fn create_organization(
    organization: web::Json<CreateOrganizationData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_create_data = organization.into_inner();

    let created_organization = {
        let pool = pool.clone();
        create_organization_query(organization_create_data.name.as_str(), pool)
            .await
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?
    };

    web::block(move || {
        add_user_to_organization(
            UserOrganization::from_details(user.id, created_organization.id, UserRole::Owner),
            pool,
        )
    })
    .await
    .map_err(|_| ServiceError::BadRequest("Thread pool error".to_owned()))??;

    Ok(HttpResponse::Ok().json(created_organization))
}

/// get_organization_usage
/// 
/// Fetch the current usage specification of an organization by its id. The auth'ed user must be an admin or owner of the organization to fetch it.
#[utoipa::path(
    get,
    path = "/organization/usage/{organization_id}",
    context_path = "/api",
    tag = "organization",
    responses(
        (status = 200, description = "The current usage of the specified organization", body = [OrganizationUsageCount]),
        (status = 400, description = "Service error relating to finding the organization's usage by id", body = [DefaultError]),
    ),
    params(
        ("organization_id" = Option<uuid>, Path, description = "The id of the organization you want to fetch the usage of.")
    ),
)]
pub async fn get_organization_usage(
    organization: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let org_id = organization.into_inner();

    let usage = get_org_usage_by_id_query(org_id, pool)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(usage))
}

/// get_organization_users
/// 
/// Fetch the users of an organization by its id. The auth'ed user must be an admin or owner of the organization to fetch it.
#[utoipa::path(
    get,
    path = "/organization/users/{organization_id}",
    context_path = "/api",
    tag = "organization",
    responses(
        (status = 200, description = "Array of users who belong to the specified by organization", body = [Vec<SlimUser>]),
        (status = 400, description = "Service error relating to finding the organization's users by id", body = [DefaultError]),
    ),
    params(
        ("organization_id" = Option<uuid>, Path, description = "The id of the organization you want to fetch the users of.")
    ),
)]
pub async fn get_organization_users(
    organization: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let org_id = organization.into_inner();

    let usage = get_org_users_by_id_query(org_id, pool)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(usage))
}
