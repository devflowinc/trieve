use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::{Pool, UserDTOWithScore},
    errors::DefaultError,
    operators::user_operator::{
        get_top_users_query, get_total_users_query, get_user_with_votes_and_cards_by_id_query,
        update_user_query,
    },
};

use super::auth_handler::LoggedUser;

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
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_query_id = path_data.user_id;
    let page = path_data.page.clone();

    let user_result =
        web::block(move || get_user_with_votes_and_cards_by_id_query(&user_query_id, &page, &pool))
            .await?;

    match user_result {
        Ok(user_with_votes_and_cards) => Ok(HttpResponse::Ok().json(user_with_votes_and_cards)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
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
        web::block(move || update_user_query(&user.id, &update_user_data, &pool)).await?;

    match user_result {
        Ok(slim_user) => Ok(HttpResponse::Ok().json(slim_user)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}
#[derive(Serialize, Deserialize)]

pub struct TopUserData {
    users: Vec<UserDTOWithScore>,
    total_user_pages: i64,
}
pub async fn get_top_users(
    page: web::Path<i64>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let page = page.into_inner();
    let pool2 = pool.clone();
    let users_result = web::block(move || get_top_users_query(&page, &pool)).await?;
    let total_users = web::block(move || get_total_users_query(&pool2))
        .await?
        .map_err(actix_web::error::ErrorBadRequest)?;
    let total_user_pages = (total_users as f64 / 10.0).ceil() as i64;

    match users_result {
        Ok(users) => Ok(HttpResponse::Ok().json(TopUserData {
            users,
            total_user_pages,
        })),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}
