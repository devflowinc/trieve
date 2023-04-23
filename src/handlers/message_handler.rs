use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};
use futures_util::FutureExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::{
    data::models::{Message, Pool},
    operators::message_operator::{
        create_message_query, create_topic_message_query, get_openai_completion, get_topic_messages,
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
    let new_message = Message::from_details(
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

    let (tx, rx) = mpsc::channel::<StreamItem>(1000);
    let receiver_stream: ReceiverStream<StreamItem> = ReceiverStream::new(rx);

    let _ =
        get_openai_completion(previous_messages, tx).then(
            |chat_completion| match chat_completion {
                Ok(chat_completion) => {
                    let create_message_result= create_message_query(chat_completion.completion_message, &fourth_pool);
                    match create_message_result {
                        Ok(_) => (),
                        Err(e) => {
                            log::error!("Error creating message: {:?}", e)
                        },
                    }
                    futures_util::future::ok::<(), actix_web::Error>(())
                }
                _ => futures_util::future::ok::<(), actix_web::Error>(()),
            },
        );

    Ok(HttpResponse::Ok().streaming(receiver_stream))
}
