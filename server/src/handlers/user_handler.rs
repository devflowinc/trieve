use super::auth_handler::LoggedUser;
use crate::{
    data::models::{DatasetAndOrgWithSubAndPlan, Pool, SlimUser},
    errors::{DefaultError, ServiceError},
    operators::user_operator::{
        delete_user_api_keys_query, get_user_api_keys_query, get_user_by_id_query,
        get_user_with_chunks_by_id_query, set_user_api_key_query, update_user_query,
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UpdateUserData {
    pub user_id: Option<uuid::Uuid>,
    pub username: Option<String>,
    pub name: Option<String>,
    pub website: Option<String>,
    pub visible_email: Option<bool>,
    pub role: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GetUserWithChunksData {
    pub user_id: uuid::Uuid,
    pub page: i64,
}

#[utoipa::path(
    get,
    path = "/user/{user_id}/{page}",
    context_path = "/api",
    tag = "user",
    responses(
        (status = 200, description = "JSON body representing the chunks made by a given user with their chunks", body = [UserDTOWithChunks]),
        (status = 400, description = "Service error relating to getting the chunks for the given user", body = [DefaultError]),
    ),
    params(
        ("user_id" = uuid::Uuid, description = "The id of the user to fetch"),
        ("page" = i64, description = "The page of chunks to fetch"),
    ),
)]
pub async fn get_user_with_chunks_by_id(
    path_data: web::Path<GetUserWithChunksData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _required_user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let user_query_id = path_data.user_id;
    let page = path_data.page;

    let user_result = web::block(move || {
        get_user_with_chunks_by_id_query(
            user_query_id,
            dataset_org_plan_sub.dataset.id,
            &page,
            pool,
        )
    })
    .await?;

    match user_result {
        Ok(user_with_chunks) => Ok(HttpResponse::Ok().json(user_with_chunks)),
        Err(e) => Err(ServiceError::BadRequest(e.message.into()).into()),
    }
}

#[utoipa::path(
    put,
    path = "/user",
    context_path = "/api",
    tag = "user",
    request_body(content = UpdateUserData, description = "JSON request payload to update user information for the auth'ed user", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the updated user information", body = [SlimUser]),
        (status = 400, description = "Service error relating to updating the user", body = [DefaultError]),
    ),
)]
pub async fn update_user(
    data: web::Json<UpdateUserData>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    mut user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let update_user_data = data.into_inner();
    let org_role = user
        .clone()
        .user_orgs
        .into_iter()
        .find(|org| org.organization_id == dataset_org_plan_sub.organization.id)
        .ok_or(ServiceError::BadRequest(
            "You are not a member of this organization".into(),
        ))?
        .role;

    if let Some(user_id) = update_user_data.user_id {
        if org_role < 1 {
            return Ok(HttpResponse::BadRequest().json(DefaultError {
                message: "You must be an admin to update other users",
            }));
        }
        let user_info = get_user_by_id_query(&user_id, pool.clone())
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

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
            return Ok(HttpResponse::BadRequest().json(DefaultError {
                message: "You must be in this organization to update other users",
            }));
        }
    }

    if update_user_data.role.is_some() && update_user_data.role.unwrap() > org_role {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Can not grant a user a higher role than yours",
        }));
    }

    if update_user_data.username.clone().unwrap_or("".to_string()) == ""
        && !update_user_data.visible_email.unwrap_or(user.visible_email)
    {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "You must provide a username or make your email visible",
        }));
    }

    let user_result = web::block(move || {
        update_user_query(
            &user.clone(),
            &update_user_data.username.clone().or(user.username),
            &update_user_data.name.clone().or(user.name),
            &update_user_data.website.or(user.website),
            Some(update_user_data.role.unwrap_or(org_role).into()),
            update_user_data.visible_email.unwrap_or(user.visible_email),
            pool,
        )
    })
    .await?;

    match user_result {
        Ok(slim_user) => Ok(HttpResponse::Ok().json(slim_user)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SetUserApiKeyRequest {
    name: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SetUserApiKeyResponse {
    api_key: String,
}

#[utoipa::path(
    post,
    path = "/user/set_api_key",
    context_path = "/api",
    tag = "user",
    responses(
        (status = 200, description = "JSON body representing the api_key for the user", body = [SetUserApiKeyResponse]),
        (status = 400, description = "Service error relating to creating api_key for the user", body = [DefaultError]),
    ),
)]
pub async fn set_user_api_key(
    user: LoggedUser,
    data: web::Json<SetUserApiKeyRequest>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let new_api_key = web::block(move || set_user_api_key_query(user.id, data.name.clone(), pool))
        .await?
        .map_err(|_err| ServiceError::BadRequest("Failed to set new API key for user".into()))?;

    Ok(HttpResponse::Ok().json(SetUserApiKeyResponse {
        api_key: new_api_key,
    }))
}

#[utoipa::path(
    post,
    path = "/user/get_api_key",
    context_path = "/api",
    tag = "user",
    responses(
        (status = 200, description = "JSON body representing the api_key for the user", body = [Vec<ApiKeyDTO>]),
        (status = 400, description = "Service error relating to creating api_key for the user", body = [DefaultError]),
    ),
)]
pub async fn get_user_api_keys(
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let api_keys = web::block(move || get_user_api_keys_query(user.id, pool))
        .await?
        .map_err(|_err| ServiceError::BadRequest("Failed to get API keys for user".into()))?;

    Ok(HttpResponse::Ok().json(api_keys))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DeleteUserApiKeyRequest {
    pub api_key_id: uuid::Uuid,
}

#[utoipa::path(
    delete,
    path = "/user/delete_api_key",
    context_path = "/api",
    tag = "user",
    request_body(content = DeleteUserApiKeyRequest, description = "JSON request payload to delete a user api key", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the api_key for the user", body = [Vec<ApiKeyDTO>]),
        (status = 400, description = "Service error relating to creating api_key for the user", body = [DefaultError]),
    ),
)]
pub async fn delete_user_api_key(
    user: LoggedUser,
    data: web::Json<DeleteUserApiKeyRequest>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    web::block(move || delete_user_api_keys_query(user.id, data.api_key_id, pool))
        .await?
        .map_err(|_err| ServiceError::BadRequest("Failed to get API keys for user".into()))?;

    Ok(HttpResponse::NoContent().finish())
}
