use crate::{
    data::models::{Pool, Topic},
    errors::DefaultError,
    handlers::auth_handler::LoggedUser,
    operators::topic_operator::{
        create_topic_query, delete_topic_query, get_all_topics_for_user_query,
        get_topic_for_user_query, update_topic_query,
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTopicData {
    pub resolution: String,
    pub side: bool,
}

pub async fn create_topic(
    data: web::Json<CreateTopicData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data_inner = data.into_inner();
    let resolution = data_inner.resolution;
    let side = data_inner.side;

    if resolution.is_empty() {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Resolution must not be empty".into(),
        }));
    }

    let new_topic = Topic::from_details(resolution, user.id, side);

    let create_topic_result = web::block(move || create_topic_query(new_topic, &pool)).await?;

    match create_topic_result {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteTopicData {
    pub topic_id: uuid::Uuid,
}

pub async fn delete_topic(
    data: web::Json<DeleteTopicData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data_inner = data.into_inner();
    let topic_id = data_inner.topic_id;
    let pool_inner = pool.clone();

    let user_topic =
        web::block(move || get_topic_for_user_query(topic_id, user.id, &pool_inner)).await?;

    match user_topic {
        Ok(topic) => {
            let delete_topic_result =
                web::block(move || delete_topic_query(topic.id, &pool)).await?;

            match delete_topic_result {
                Ok(()) => Ok(HttpResponse::NoContent().finish()),
                Err(e) => Ok(HttpResponse::BadRequest().json(e)),
            }
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateTopicData {
    pub topic_id: uuid::Uuid,
    pub resolution: String,
    pub side: bool,
}

pub async fn update_topic(
    data: web::Json<UpdateTopicData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let data_inner = data.into_inner();
    let topic_id = data_inner.topic_id;
    let resolution = data_inner.resolution;
    let side = data_inner.side;
    let pool_inner = pool.clone();

    if resolution.is_empty() {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Resolution must not be empty".into(),
        }));
    }

    let user_topic =
        web::block(move || get_topic_for_user_query(topic_id, user.id, &pool_inner)).await?;

    match user_topic {
        Ok(topic) => {
            let update_topic_result =
                web::block(move || update_topic_query(topic.id, resolution, side, &pool)).await?;

            match update_topic_result {
                Ok(()) => Ok(HttpResponse::NoContent().finish()),
                Err(e) => Ok(HttpResponse::BadRequest().json(e)),
            }
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

pub async fn get_all_topics(
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topics = web::block(move || get_all_topics_for_user_query(user.id, &pool)).await?;

    match topics {
        Ok(topics) => Ok(HttpResponse::Ok().json(topics)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}
