use super::auth_handler::{LoggedUser, RequireAuth};
use crate::{
    data::models::{Dataset, Pool},
    errors::{DefaultError, ServiceError},
    operators::user_operator::{
        get_user_with_cards_by_id_query, set_user_api_key_query, update_user_query,
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct UpdateUserData {
    pub username: Option<String>,
    pub website: Option<String>,
    pub visible_email: bool,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GetUserWithCardsData {
    pub user_id: uuid::Uuid,
    pub page: i64,
}

#[utoipa::path(
    get,
    path = "/user/{user_id}/{page}",
    context_path = "/api",
    tag = "user",
    responses(
        (status = 200, description = "JSON body representing the cards made by a given user with their cards", body = [UserDTOWithCards]),
        (status = 400, description = "Service error relating to getting the cards for the given user", body = [DefaultError]),
    ),
    params(
        ("user_id" = uuid::Uuid, description = "The id of the user to fetch"),
        ("page" = i64, description = "The page of cards to fetch"),
    ),
)]
pub async fn get_user_with_cards_by_id(
    path_data: web::Path<GetUserWithCardsData>,
    dataset: Dataset,
    pool: web::Data<Pool>,
    _required_user: RequireAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let user_query_id = path_data.user_id;
    let page = path_data.page;

    let user_result =
        web::block(move || get_user_with_cards_by_id_query(user_query_id, dataset.id, &page, pool))
            .await?;

    match user_result {
        Ok(user_with_cards) => Ok(HttpResponse::Ok().json(user_with_cards)),
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
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let update_user_data = data.into_inner();

    if update_user_data.username.clone().unwrap_or("".to_string()) == ""
        && !update_user_data.visible_email
    {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "You must provide a username or make your email visible",
        }));
    }

    let user_result =
        web::block(move || update_user_query(&user.id, &update_user_data, pool)).await?;

    match user_result {
        Ok(slim_user) => Ok(HttpResponse::Ok().json(slim_user)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SetUserApiKeyResponse {
    api_key: String,
}

#[utoipa::path(
    get,
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
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let new_api_key = web::block(move || set_user_api_key_query(user.id, pool))
        .await?
        .map_err(|_err| ServiceError::BadRequest("Failed to set new API key for user".into()))?;

    Ok(HttpResponse::Ok().json(SetUserApiKeyResponse {
        api_key: new_api_key,
    }))
}
