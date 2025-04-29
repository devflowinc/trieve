use super::auth_handler::LoggedUser;
use crate::{
    data::models::{OrganizationWithSubAndPlan, Pool, RedisPool, UserRole},
    errors::ServiceError,
    operators::user_operator::{
        delete_user_api_keys_query, get_user_api_keys_query, get_user_by_id_query,
        update_user_query,
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UpdateUserReqPayload {
    /// The id of the user to update, if not provided, the auth'ed user will be updated. If provided, the role of the auth'ed user or api key must be an admin (1) or owner (2) of the organization.
    pub user_id: Option<uuid::Uuid>,
    /// Either 0 (user), 1 (admin), or 2 (owner). If not provided, the current role will be used. The auth'ed user must have a role greater than or equal to the role being assigned.
    pub role: Option<i32>,
    /// The scopes the user will have in the organization.
    pub scopes: Option<Vec<String>>,
}

/// Update User Org Role
///
/// Update a user's information for the org specified via header. If the user_id is not provided, the auth'ed user will be updated. If the user_id is provided, the role of the auth'ed user or api key must be an admin (1) or owner (2) of the organization.
#[utoipa::path(
    put,
    path = "/user",
    context_path = "/api",
    tag = "User",
    request_body(content = UpdateUserReqPayload, description = "JSON request payload to update user information for the auth'ed user", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the user's role was updated"),
        (status = 400, description = "Service error relating to updating the user", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn update_user(
    data: web::Json<UpdateUserReqPayload>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    org_with_plan_and_sub: OrganizationWithSubAndPlan,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, ServiceError> {
    let update_user_data = data.into_inner();
    let org_role = user
        .clone()
        .user_orgs
        .into_iter()
        .find(|org| org.organization_id == org_with_plan_and_sub.organization.id)
        .ok_or(ServiceError::BadRequest(
            "You are not a member of this organization".into(),
        ))?
        .role;

    if let Some(user_id) = update_user_data.user_id {
        if org_role < 1 {
            return Err(ServiceError::BadRequest(
                "You must have an admin or owner role to update other users".to_string(),
            ));
        }

        let user_info = get_user_by_id_query(&user_id, pool.clone()).await?;

        let already_in_org = user_info
            .1
            .iter()
            .any(|org| org.organization_id == org_with_plan_and_sub.organization.id);

        if !already_in_org {
            return Err(ServiceError::BadRequest(
                "The user who you would like to update the role of must be added to the specified org first before their role can be updated".to_string(),
            ));
        }
    }

    let user_role = update_user_data.role.map(UserRole::from);

    if let Some(ref user_role) = user_role {
        if user_role > &(org_role.into()) {
            return Err(ServiceError::BadRequest(
                "Can not grant a user a higher role than that of the requesting user's".to_string(),
            ));
        }
    }

    update_user_query(
        update_user_data.user_id.unwrap_or(user.id),
        org_with_plan_and_sub.organization.id,
        user_role,
        update_user_data.scopes,
        pool,
        redis_pool,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Get User Api Keys
///
/// Get the api keys which belong to the auth'ed user. The actual api key values are not returned, only the ids, names, and creation dates.
#[utoipa::path(
    get,
    path = "/user/api_key",
    context_path = "/api",
    tag = "User",
    responses(
        (status = 200, description = "JSON body representing the api_key for the user", body = Vec<ApiKeyRespBody>),
        (status = 400, description = "Service error relating to creating api_key for the user", body = ErrorResponseBody),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_user_api_keys(
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let api_keys = get_user_api_keys_query(user.id, pool)
        .await
        .map_err(|_err| ServiceError::BadRequest("Failed to get API keys for user".into()))?;

    Ok(HttpResponse::Ok().json(api_keys))
}

/// Delete User Api Key
///
/// Delete an api key for the auth'ed user.
#[utoipa::path(
    delete,
    path = "/user/api_key/{api_key_id}",
    context_path = "/api",
    tag = "User",
    responses(
        (status = 204, description = "Confirmation that the api key was deleted"),
        (status = 400, description = "Service error relating to creating api_key for the user", body = ErrorResponseBody),
    ),
    params(
        ("api_key_id" = uuid::Uuid, Path, description = "The id of the api key to delete"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn delete_user_api_key(
    user: LoggedUser,
    data: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    delete_user_api_keys_query(user.id, data.into_inner(), pool)
        .await
        .map_err(|_err| ServiceError::BadRequest("Failed to get API keys for user".into()))?;

    Ok(HttpResponse::NoContent().finish())
}
