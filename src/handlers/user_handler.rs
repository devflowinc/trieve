use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::Pool,
    errors::DefaultError,
    operators::user_operator::{get_user_with_votes_and_cards_by_id_query, update_user_query},
};

use super::auth_handler::LoggedUser;

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserData {
    pub username: Option<String>,
    pub website: Option<String>,
    pub visible_email: bool,
}

pub async fn get_user_with_votes_and_cards_by_id(
    user_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_query_id: uuid::Uuid = user_id.into_inner();

    let user_result =
        web::block(move || get_user_with_votes_and_cards_by_id_query(&user_query_id, &pool))
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
