use super::auth_handler::LoggedUser;
use crate::{
    data::models::{OrganizationWithSubAndPlan, Pool, RedisPool, UserRole},
    errors::ServiceError,
    operators::user_operator::{get_user_by_id_query, update_user_org_role_query},
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UpdateUserOrgRoleReqPayload {
    /// The id of the user to update, if not provided, the auth'ed user will be updated. If provided, the role of the auth'ed user or api key must be an admin (1) or owner (2) of the organization.
    pub user_id: Option<uuid::Uuid>,
    /// Either 0 (user), 1 (admin), or 2 (owner). If not provided, the current role will be used. The auth'ed user must have a role greater than or equal to the role being assigned.
    pub role: i32,
}

/// Update User Org Role
///
/// Update a user's information for the org specified via header. If the user_id is not provided, the auth'ed user will be updated. If the user_id is provided, the role of the auth'ed user or api key must be an admin (1) or owner (2) of the organization.
#[utoipa::path(
    put,
    path = "/user",
    context_path = "/api",
    tag = "User",
    request_body(content = UpdateUserOrgRoleReqPayload, description = "JSON request payload to update user information for the auth'ed user", content_type = "application/json"),
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
    data: web::Json<UpdateUserOrgRoleReqPayload>,
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

    if update_user_data.role > org_role {
        return Err(ServiceError::BadRequest(
            "Can not grant a user a higher role than that of the requesting user's".to_string(),
        ));
    }

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

    let user_role = UserRole::from(update_user_data.role);

    update_user_org_role_query(
        update_user_data.user_id.unwrap_or(user.id),
        org_with_plan_and_sub.organization.id,
        user_role,
        pool,
        redis_pool,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}
