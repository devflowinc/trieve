use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::Pool,
    operators::vote_operator::{create_vote_query, delete_vote_query},
};

use super::auth_handler::LoggedUser;

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateVoteData {
    card_metadata_id: uuid::Uuid,
    vote: bool,
}

pub async fn create_vote(
    data: web::Json<CreateVoteData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data_inner = data.into_inner();
    let card_metadata_id = data_inner.card_metadata_id;
    let vote = data_inner.vote;

    let create_vote_result =
        web::block(move || create_vote_query(&user.id, &card_metadata_id, &vote, &pool)).await?;

    match create_vote_result {
        Ok(created_vote) => Ok(HttpResponse::Ok().json(created_vote)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

pub async fn delete_vote(
    card_metadata_id: web::Path<uuid::Uuid>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let card_metadata_id_inner = card_metadata_id.into_inner();
    let pool_inner = pool.clone();

    let delete_vote_result =
        web::block(move || delete_vote_query(&user.id, &card_metadata_id_inner, &pool_inner))
            .await?;

    match delete_vote_result {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}
