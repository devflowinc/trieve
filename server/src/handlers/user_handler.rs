use super::auth_handler::LoggedUser;
use crate::{
    data::models::{Pool, SlimUser},
    errors::ServiceError,
    operators::user_operator::{
        delete_user_api_keys_query, get_user_api_keys_query, get_user_by_id_query,
        set_user_api_key_query, update_user_query,
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UpdateUserData {
    /// The id of the organization to update the user for.
    pub organization_id: uuid::Uuid,
    /// The id of the user to update, if not provided, the auth'ed user will be updated. If provided, the auth'ed user must be an admin (1) or owner (2) of the organization.
    pub user_id: Option<uuid::Uuid>,
    /// In the sense of a legal name, not a username. The new name to assign to the user, if not provided, the current name will be used.
    pub name: Option<String>,
    /// Either 0 (user), 1 (admin), or 2 (owner). If not provided, the current role will be used. The auth'ed user must have a role greater than or equal to the role being assigned.
    pub role: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUserWithChunksData {
    /// The id of the user to fetch the chunks for.
    pub user_id: uuid::Uuid,
    /// The page of chunks to fetch. Each page is 10 chunks. Support for custom page size is coming soon.
    pub page: i64,
}

/// Update User
///
/// Update a user's information. If the user_id is not provided, the auth'ed user will be updated. If the user_id is provided, the auth'ed user must be an admin (1) or owner (2) of the organization.
#[utoipa::path(
    put,
    path = "/user",
    context_path = "/api",
    tag = "user",
    request_body(content = UpdateUserData, description = "JSON request payload to update user information for the auth'ed user", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the updated user information", body = SlimUser),
        (status = 400, description = "Service error relating to updating the user", body = ErrorResponseBody),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn update_user(
    data: web::Json<UpdateUserData>,
    mut user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let update_user_data = data.into_inner();
    let org_role = user
        .clone()
        .user_orgs
        .into_iter()
        .find(|org| org.organization_id == update_user_data.organization_id)
        .ok_or(ServiceError::BadRequest(
            "You are not a member of this organization".into(),
        ))?
        .role;

    if let Some(user_id) = update_user_data.user_id {
        if org_role < 1 {
            return Err(ServiceError::BadRequest(
                "You must be an admin to update other users".to_string(),
            ));
        }
        let user_info = get_user_by_id_query(&user_id, pool.clone()).await?;

        let authorized = user_info
            .1
            .iter()
            .zip(user.user_orgs.iter())
            .any(|(org, user_org)| {
                org.organization_id == user_org.organization_id && user_org.role >= 1
            });
        if authorized {
            user = SlimUser::from_details(user_info.0, user_info.1, user_info.2);
        } else {
            return Err(ServiceError::BadRequest(
                "You must be in this organization to update other users".to_string(),
            ));
        }
    }

    if update_user_data.role.is_some()
        && update_user_data
            .role
            .expect("Role must not be null after the &&")
            > org_role
    {
        return Err(ServiceError::BadRequest(
            "Can not grant a user a higher role than yours".to_string(),
        ));
    }

    let new_role = update_user_data.role.map(|role| role.into());

    let slim_user = update_user_query(
        &user.clone(),
        &update_user_data.name.clone().or(user.name),
        new_role,
        pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(slim_user))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SetUserApiKeyRequest {
    /// The name which will be assigned to the new api key.
    name: String,
    /// The role which will be assigned to the new api key. Either 0 (read), 1 (read and write at the level of the currently auth'ed user). The auth'ed user must have a role greater than or equal to the role being assigned which means they must be an admin (1) or owner (2) of the organization to assign write permissions with a role of 1.
    role: i32,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SetUserApiKeyResponse {
    /// The api key which was created. This is the value which should be used in the Authorization header.
    api_key: String,
}

/// Set User Api Key
///
/// Create a new api key for the auth'ed user. Successful response will contain the newly created api key. If a write role is assigned the api key will have permission level of the auth'ed user who calls this endpoint.
#[utoipa::path(
    post,
    path = "/user/set_api_key",
    context_path = "/api",
    tag = "user",
    request_body(content = SetUserApiKeyRequest, description = "JSON request payload to create a new user api key", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the api_key for the user", body = SetUserApiKeyResponse),
        (status = 400, description = "Service error relating to creating api_key for the user", body = ErrorResponseBody),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn set_user_api_key(
    user: LoggedUser,
    data: web::Json<SetUserApiKeyRequest>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let role = data.role;

    let new_api_key = set_user_api_key_query(user.id, data.name.clone(), role.into(), pool)
        .await
        .map_err(|_err| ServiceError::BadRequest("Failed to set new API key for user".into()))?;

    Ok(HttpResponse::Ok().json(SetUserApiKeyResponse {
        api_key: new_api_key,
    }))
}

/// Get User Api Keys
///
/// Get the api keys which belong to the auth'ed user. The actual api key values are not returned, only the ids, names, and creation dates.
#[utoipa::path(
    post,
    path = "/user/get_api_key",
    context_path = "/api",
    tag = "user",
    responses(
        (status = 200, description = "JSON body representing the api_key for the user", body = Vec<ApiKeyDTO>),
        (status = 400, description = "Service error relating to creating api_key for the user", body = ErrorResponseBody),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_user_api_keys(
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let api_keys = get_user_api_keys_query(user.id, pool)
        .await
        .map_err(|_err| ServiceError::BadRequest("Failed to get API keys for user".into()))?;

    Ok(HttpResponse::Ok().json(api_keys))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeleteUserApiKeyRequest {
    /// The id of the api key to delete.
    pub api_key_id: uuid::Uuid,
}

/// Delete User Api Key
///
/// Delete an api key for the auth'ed user.
#[utoipa::path(
    delete,
    path = "/user/delete_api_key/{api_key_id}",
    context_path = "/api",
    tag = "user",
    responses(
        (status = 200, description = "JSON body representing the api_key for the user", body = Vec<ApiKeyDTO>),
        (status = 400, description = "Service error relating to creating api_key for the user", body = ErrorResponseBody),
    ),
    params(
        ("api_key_id" = uuid::Uuid, Path, description = "The id of the api key to delete"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
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
