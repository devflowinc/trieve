use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};
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
    data: web::Json<CreateMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
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

    stream_response(previous_messages, fourth_pool).await
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
    data: web::Json<RegenerateMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
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

    stream_response(previous_messages, fourth_pool).await
}

pub async fn stream_response(messages: Vec<models::Message>, pool: web::Data<Pool>) -> Result<HttpResponse, actix_web::Error> {

    let (tx, rx) = mpsc::channel::<StreamItem>(1000);
    let receiver_stream: ReceiverStream<StreamItem> = ReceiverStream::new(rx);

    let open_ai_messages: Vec<ChatMessage> = messages
        .iter()
        .map(|message| ChatMessage::from(message.clone()))
        .collect();

    let open_ai_api_key = std::env::var("OPENAI_API_KEY").expect("OPEN_AI_API_KEY must be set");
    let client = Client::new(open_ai_api_key);

    let parameters = ChatCompletionParameters {
        model: "gpt-3.5-turbo".into(),
        messages: open_ai_messages,
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        max_tokens: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
    };

    
    {
        // while let Some(response) = stream.next().await {
        //
        //     completion_tokens += 1;
        //     let chat_response = response.map_err(|e| {
        //         log::error!("Error: {}", e);
        //     }).unwrap();
        //
        //     log::info!("Got chat completion: {:?}", chat_response);
        //
        //     let chat_content = chat_response.choices[0].delta.content.clone();
        //     if chat_content.is_none() {
        //         log::error!("Chat content is none");
        //         continue;
        //     }
        //     let chat_content = chat_content.unwrap();
        //
        //     let multi_use_chat_content = chat_content.clone();
        //     let _ = tx.send(Ok(chat_content.into())).await;
            // yield Ok(chat_content.into());
            // response_content.push_str(multi_use_chat_content.clone().as_str());
        // }
        // let completion_message = Message::from_details(
        //     response_content,
        //     messages[0].topic_id,
        //     (messages.len() + 1).try_into().unwrap(),
        //     "assistant".into(),
        //     Some(0),
        //     Some(completion_tokens),
        // );


        // Since we're on a different thread already no need to block
        // let _ = create_message_query(completion_message, &pool);
    };

    let streamer = StreamingBoi {
        messages,
    };
    let addr =streamer.start();


    Ok(HttpResponse::Ok().streaming(receiver_stream))
}

#[derive(Message)]
#[rtype(result = "()")]
struct StreamNow {
    sender: mpsc::Sender<Result<Bytes, actix_web::Error>>,
}

#[derive(Debug, Clone)]
struct StreamingBoi {
    messages: Vec<models::Message>,
}

impl Actor for StreamingBoi {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut actix::prelude::Context<Self>) {
        log::info!("Starting streaming boi");
    }

    fn stopped(&mut self, ctx: &mut actix::prelude::Context<Self>) {
        log::info!("Stopped streaming boi");
    }
}

impl actix::prelude::Handler<StreamNow> for StreamingBoi {
    type Result = ();

    fn handle(&mut self, msg: StreamNow, _ctx: &mut Self::Context) -> Self::Result {
        let messages = self.messages.clone();
        actix_web::rt::spawn(async move {

            let open_ai_messages: Vec<ChatMessage> = messages
                .iter()
                .map(|message| ChatMessage::from(message.clone()))
                .collect();

            let parameters = ChatCompletionParameters {
                model: "gpt-3.5-turbo".into(),
                messages: open_ai_messages,
                temperature: None,
                top_p: None,
                n: None,
                stop: None,
                max_tokens: None,
                presence_penalty: None,
                frequency_penalty: None,
                logit_bias: None,
            };

            let open_ai_api_key = std::env::var("OPENAI_API_KEY").expect("OPEN_AI_API_KEY must be set");
            let client = Client::new(open_ai_api_key);

            let mut response_content = String::new();
            let mut completion_tokens = 0;
            let mut stream = client.chat().create_stream(parameters).await.unwrap();

            while let Some(response) = stream.next().await {

                completion_tokens += 1;
                let chat_response = response.map_err(|e| {
                    log::error!("Error: {}", e);
                }).unwrap();

                log::info!("Got chat completion: {:?}", chat_response);

                let chat_content = chat_response.choices[0].delta.content.clone();
                if chat_content.is_none() {
                    log::error!("Chat content is none");
                    continue;
                }
                let chat_content = chat_content.unwrap();

                let multi_use_chat_content = chat_content.clone();
                let _ = msg.sender.send(Ok(chat_content.into())).await;
                // response_content.push_str(multi_use_chat_content.clone().as_str());
            }
        });
    }
}
