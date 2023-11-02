use super::auth_handler::LoggedUser;
use crate::{
    data::models::Pool,
    operators::{
        card_operator::get_metadata_from_id_query,
        vote_operator::{create_vote_query, delete_vote_query},
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct CreateVoteData {
    card_metadata_id: uuid::Uuid,
    vote: bool,
}

#[utoipa::path(
    post,
    path = "/vote",
    context_path = "/api",
    tag = "vote",
    request_body(content = CreateVoteData, description = "JSON request payload to create a vote for the auth'ed user", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the vote created for the auth'ed user", body = [CardVote]),
        (status = 400, description = "Service error relating to creating the vote", body = [DefaultError]),
    ),
)]
pub async fn create_vote(
    data: web::Json<CreateVoteData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data_inner = data.into_inner();
    let card_metadata_id = data_inner.card_metadata_id;
    let vote = data_inner.vote;
    let pool1 = pool.clone();
    let card_data = web::block(move || get_metadata_from_id_query(card_metadata_id, pool)).await?;
    match card_data {
        Ok(data) => {
            if data.private {
                return Ok(HttpResponse::BadRequest().json("Votes cannot be cast on private cards"));
            }
        }
        Err(e) => return Ok(HttpResponse::BadRequest().json(e)),
    }
    let create_vote_result =
        web::block(move || create_vote_query(&user.id, &card_metadata_id, &vote, pool1)).await?;

    match create_vote_result {
        Ok(created_vote) => Ok(HttpResponse::Ok().json(created_vote)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[utoipa::path(
    delete,
    path = "/vote/{card_metadata_id}",
    context_path = "/api",
    tag = "vote",
    responses(
        (status = 204, description = "Confirmation that the auth'ed user's vote was deleted for the given card metadata ID"),
        (status = 400, description = "Service error relating to creating the vote", body = [DefaultError]),
    ),
    params(
        ("card_metadata_id" = uuid, description = "The card metadata ID to delete the auth'ed user's vote for"),
    ),
)]
pub async fn delete_vote(
    card_metadata_id: web::Path<uuid::Uuid>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let card_metadata_id_inner = card_metadata_id.into_inner();
    let pool1 = pool.clone();

    let card_data =
        web::block(move || get_metadata_from_id_query(card_metadata_id_inner, pool)).await?;
    match card_data {
        Ok(data) => {
            if data.private {
                return Ok(HttpResponse::BadRequest().json("Votes cannot be cast on private cards"));
            }
        }
        Err(e) => return Ok(HttpResponse::BadRequest().json(e)),
    }
    let delete_vote_result =
        web::block(move || delete_vote_query(&user.id, &card_metadata_id_inner, pool1)).await?;

    match delete_vote_result {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}
