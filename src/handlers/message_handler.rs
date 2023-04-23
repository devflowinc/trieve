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
        create_message_query, get_openai_completion, get_topic_messages,
    },
};

use super::auth_handler::LoggedUser;

pub type StreamItem = Result<Bytes, actix_web::Error>;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateMessageData {
    pub new_message: Message,
    pub topic_id: uuid::Uuid,
}

pub async fn create_message_completion_handler(
    data: web::Json<CreateMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let create_message_data = data.into_inner();
    let new_message = create_message_data.new_message;
    let topic_id = create_message_data.topic_id;

    let previous_messages_result = web::block(move || get_topic_messages(topic_id, &pool)).await?;

    let previous_messages = match previous_messages_result {
        Ok(messages) => messages,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    let (tx, rx) = mpsc::channel::<StreamItem>(1000);
    let receiver_stream: ReceiverStream<StreamItem> = ReceiverStream::new(rx);

    let _ = get_openai_completion(previous_messages, tx).then(|chat_completion| match chat_completion {
        Ok(chat_completion) => {
            let _ = create_message_query(new_message, &pool);
            futures_util::future::ok::<(), actix_web::Error>(())
        }
        _ => futures_util::future::ok::<(), actix_web::Error>(()),
    });

    Ok(HttpResponse::Ok().streaming(receiver_stream))
}
