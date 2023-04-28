use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};
use actix_web_actors::ws;
use actix_web::HttpRequest;
use async_stream::{try_stream, __private::AsyncStream};
use futures_util::Future;
use openai_dive::v1::{
    api::Client,
    resources::chat_completion::{ChatCompletionParameters, ChatMessage},
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};
use actix::Actor;
use actix::prelude::*;
use crate::{
    data::models as models,
    data::models::Pool,
    operators::message_operator::{
        delete_message_query,
        create_message_query, create_topic_message_query, get_messages_for_topic_query,
        get_topic_messages,
    },
};

use super::auth_handler::LoggedUser;

pub type StreamItem = Result<Bytes, actix_web::Error>;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateMessageData {
    pub new_message_content: String,
    pub topic_id: uuid::Uuid,
}

pub async fn create_message_completion_handler(
    req: HttpRequest,
    data: web::Json<CreateMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let create_message_data = data.into_inner();
    let new_message = models::Message::from_details(
        create_message_data.new_message_content,
        create_message_data.topic_id,
        0,
        "user".to_string(),
        None,
        None,
    );
    let topic_id = create_message_data.topic_id;
    let second_pool = pool.clone();
    let third_pool = pool.clone();
    let fourth_pool = pool.clone();

    // check if the user owns the topic
    let topic_result = crate::operators::topic_operator::get_topic_query(topic_id, &pool);
    match topic_result {
        Ok(topic) if topic.user_id != user.id => {
            return Ok(HttpResponse::Unauthorized().json("Unauthorized"));
        }
        Ok(topic) => topic,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    // get the previous messages
    let previous_messages_result =
        web::block(move || get_topic_messages(topic_id, &second_pool)).await?;
    let previous_messages = match previous_messages_result {
        Ok(messages) => messages,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    // call create_topic_message_query with the new_message and previous_messages
    let previous_messages_result =
        web::block(move || create_topic_message_query(previous_messages, new_message, &third_pool))
            .await?;
    let previous_messages = match previous_messages_result {
        Ok(messages) => messages,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    stream_response(req, previous_messages, fourth_pool, stream).await
}

// get_all_topic_messages_handler
// verify that the user owns the topic for the topic_id they are requesting
// get all the messages for the topic_id
// filter out deleted messages
// return the messages
pub async fn get_all_topic_messages(
    user: LoggedUser,
    messages_topic_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id: uuid::Uuid = messages_topic_id.into_inner();
    // check if the user owns the topic
    let topic_result = crate::operators::topic_operator::get_topic_query(topic_id, &pool);
    match topic_result {
        Ok(topic) if topic.user_id != user.id => {
            return Ok(HttpResponse::Unauthorized().json("Unauthorized"));
        }
        Ok(topic) => topic,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    let messages = web::block(move || get_messages_for_topic_query(topic_id, &pool)).await?;

    match messages {
        Ok(messages) => Ok(HttpResponse::Ok().json(messages)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RegenerateMessageData {
    message_id: uuid::Uuid,
    topic_id: uuid::Uuid,
}

pub async fn regenerate_message_handler(
    req: HttpRequest,
    data: web::Json<RegenerateMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    // TODO: check if the user owns the message
    // Get message
    let message_id = data.message_id.clone();
    let topic_id = data.topic_id.clone();
    let second_pool = pool.clone();
    let fourth_pool = pool.clone();

    let _ = web::block(move || delete_message_query(&user.id, message_id, topic_id, &pool)).await?;

    // Recreate
    let previous_messages_result =
        web::block(move || get_topic_messages(topic_id, &second_pool)).await?;
    let previous_messages = match previous_messages_result {
        Ok(messages) => messages,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    stream_response(req, previous_messages, fourth_pool, stream).await
}

pub async fn stream_response(req: HttpRequest, messages: Vec<models::Message>, pool: web::Data<Pool>, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {

    let resp = ws::start(
        StreamingBoi {
            messages,
        }, &req, stream);

    resp
}

#[derive(Debug, Clone)]
struct StreamingBoi {
    messages: Vec<models::Message>,
}

impl Actor for StreamingBoi {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for StreamingBoi {

    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg)
            },
            Ok(ws::Message::Text(text)) => {
                ctx.text(text)
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}
