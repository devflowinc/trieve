use crate::{
    data::models::{Pool, UserDTOWithScore},
    errors::{DefaultError, ServiceError},
    operators::user_operator::{
        get_top_users_query, get_total_users_query, get_user_with_votes_and_cards_by_id_query,
        set_user_api_key_query, update_user_query,
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

use super::auth_handler::{LoggedUser, RequireAuth};

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserData {
    pub username: Option<String>,
    pub website: Option<String>,
    pub visible_email: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUserWithVotesAndCardsData {
    pub user_id: uuid::Uuid,
    pub page: i64,
}

pub async fn get_user_with_votes_and_cards_by_id(
    path_data: web::Path<GetUserWithVotesAndCardsData>,
    user: Option<LoggedUser>,
    pool: web::Data<Pool>,
    _required_user: RequireAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let user_query_id = path_data.user_id;
    let accessing_user_id = user.map(|user| user.id);
    let page = path_data.page;

    let user_result = web::block(move || {
        get_user_with_votes_and_cards_by_id_query(user_query_id, accessing_user_id, &page, pool)
    })
    .await?;

    match user_result {
        Ok(user_with_votes_and_cards) => Ok(HttpResponse::Ok().json(user_with_votes_and_cards)),
        Err(e) => Err(ServiceError::BadRequest(e.message.into()).into()),
    }
}

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
pub struct TopUserData {
    users: Vec<UserDTOWithScore>,
    total_user_pages: i64,
}

#[utoipa::path(
    get,
    path = "/top_users/{page}",
    context_path = "/api",
    tag = "top_users",
    responses(
        (status = 200, description = "JSON body representing the top users by collected votes", body = [TopUserData]),
        (status = 400, description = "Service error relating to fetching the top users by collected votes", body = [DefaultError]),
    ),
    params(
        ("page" = i64, description = "The page of users to fetch"),
    ),
)]
pub async fn get_top_users(
    page: web::Path<i64>,
    pool: web::Data<Pool>,
    _required_user: RequireAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let page = page.into_inner();

    let pool2 = pool.clone();
    let users_result = web::block(move || get_top_users_query(&page, pool)).await?;
    let total_users = web::block(move || get_total_users_query(pool2))
        .await?
        .map_err(|_err| ServiceError::BadRequest("Failed to get Total users".into()))?;
    let total_user_pages = (total_users as f64 / 10.0).ceil() as i64;

    match users_result {
        Ok(users) => Ok(HttpResponse::Ok().json(TopUserData {
            users,
            total_user_pages,
        })),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

pub async fn set_user_api_key(
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let new_api_key = web::block(move || set_user_api_key_query(user.id, pool))
        .await?
        .map_err(|_err| ServiceError::BadRequest("Failed to set new API key for user".into()))?;

    Ok(HttpResponse::Ok().json(json!({ "api_key": new_api_key })))
}
