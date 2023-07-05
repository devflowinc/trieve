use std::sync::{Arc, Mutex};

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::Pool,
    operators::{
        card_operator::get_metadata_from_id_query,
        vote_operator::{create_vote_query, delete_vote_query},
    },
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
    let thread_safe_pool = Arc::new(Mutex::new(pool));

    let data_inner = data.into_inner();
    let card_metadata_id = data_inner.card_metadata_id;
    let vote = data_inner.vote;
    let pool1 = thread_safe_pool.clone();
    let card_data = web::block(move || {
        get_metadata_from_id_query(card_metadata_id, thread_safe_pool.lock().unwrap())
    })
    .await?;
    match card_data {
        Ok(data) => {
            if data.private {
                return Ok(HttpResponse::BadRequest().json("Votes cannot be cast on private cards"));
            }
        }
        Err(e) => return Ok(HttpResponse::BadRequest().json(e)),
    }
    let create_vote_result = web::block(move || {
        create_vote_query(&user.id, &card_metadata_id, &vote, pool1.lock().unwrap())
    })
    .await?;

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
    let thread_safe_pool = Arc::new(Mutex::new(pool));

    let card_metadata_id_inner = card_metadata_id.into_inner();
    let pool_inner = thread_safe_pool.clone();
    let card_data = web::block(move || {
        get_metadata_from_id_query(card_metadata_id_inner, thread_safe_pool.lock().unwrap())
    })
    .await?;
    match card_data {
        Ok(data) => {
            if data.private {
                return Ok(HttpResponse::BadRequest().json("Votes cannot be cast on private cards"));
            }
        }
        Err(e) => return Ok(HttpResponse::BadRequest().json(e)),
    }
    let delete_vote_result = web::block(move || {
        delete_vote_query(
            &user.id,
            &card_metadata_id_inner,
            &pool_inner.lock().unwrap(),
        )
    })
    .await?;

    match delete_vote_result {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}
