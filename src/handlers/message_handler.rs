use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};
use openai_dive::v1::{
    api::Client,
    resources::chat_completion::{ChatCompletionParameters, ChatMessage},
};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

use crate::{
    data::models::{Message, Pool},
    operators::message_operator::{
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

    log::info!("Getting openai completion");

    let open_ai_messages: Vec<ChatMessage> = previous_messages
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

    let mut response_content = String::new();
    let mut completion_tokens = 0;
    let mut stream = client.chat().create_stream(parameters).await.unwrap();

    log::info!("Getting chat completion");
    while let Some(response) = stream.next().await {
        match response {
            Ok(chat_response) => {
                completion_tokens += 1;

                log::info!("Got chat completion: {:?}", chat_response);

                let chat_content = chat_response.choices[0].delta.content.clone();
                if chat_content.is_none() {
                    log::error!("Chat content is none");
                    continue;
                }
                let chat_content = chat_content.unwrap();

                let multi_use_chat_content = chat_content.clone();
                let _ = tx.send(Ok(chat_content.into())).await;
                response_content.push_str(multi_use_chat_content.clone().as_str());
            }
            Err(e) => log::error!("Error getting chat completion: {}", e),
        }
    }

    let completion_message = Message::from_details(
        response_content,
        previous_messages[0].topic_id,
        (previous_messages.len() + 1).try_into().unwrap(),
        "assistant".into(),
        Some(0),
        Some(completion_tokens),
    );

    let _ = web::block(move || create_message_query(completion_message, &fourth_pool)).await?;

    Ok(HttpResponse::Ok().streaming(receiver_stream))
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
    let topic_id = messages_topic_id.into_inner();

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
