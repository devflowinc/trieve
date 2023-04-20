use crate::{
    data::models::{Pool, Topic},
    errors::DefaultError,
    handlers::auth_handler::LoggedUser,
    operators::topic_operator::create_topic_query,
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

    if user.id.is_nil() {
        return Ok(HttpResponse::Unauthorized().json(DefaultError {
            message: "You must be logged in to create a topic".into(),
        }));
    }

    let new_topic = Topic::from_details(resolution, user.id, side);

    let create_topic_result = web::block(move || create_topic_query(new_topic, &pool)).await?;

    match create_topic_result {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}
