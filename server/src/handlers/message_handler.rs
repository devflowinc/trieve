use crate::{
    data::models,
    data::models::Pool,
    errors::{DefaultError, ServiceError},
    operators::{
        card_operator::{
            create_embedding, get_metadata_and_collided_cards_from_point_ids_query,
            search_card_query,
        },
        message_operator::{
            create_cut_card, create_message_query, create_topic_message_query,
            delete_message_query, get_message_by_sort_for_topic_query,
            get_messages_for_topic_query, get_topic_messages, user_owns_topic_query,
        },
    },
    AppMutexStore,
};
use actix::Arbiter;
use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};
use crossbeam_channel::unbounded;
use futures_util::stream;
use openai_dive::v1::{
    api::Client,
    resources::chat_completion::{ChatCompletionParameters, ChatMessage, Role},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_stream::StreamExt;

use super::auth_handler::LoggedUser;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateMessageData {
    pub new_message_content: String,
    pub topic_id: uuid::Uuid,
}

pub async fn create_message_completion_handler(
    data: web::Json<CreateMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    mutex_store: web::Data<AppMutexStore>,
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

    let user_owns_topic = web::block(move || user_owns_topic_query(user.id, topic_id, &pool));
    if let Ok(false) = user_owns_topic.await {
        return Ok(HttpResponse::Unauthorized().json("Unauthorized"));
    }

    // get the previous messages
    let mut previous_messages = web::block(move || get_topic_messages(topic_id, &second_pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    // remove citations from the previous messages
    previous_messages = previous_messages
        .into_iter()
        .map(|message| {
            let mut message = message;
            if message.role == "assistant" {
                message.content = message
                    .content
                    .split("||")
                    .last()
                    .unwrap_or("I give up, I can't find a citation")
                    .to_string();
            }
            message
        })
        .collect::<Vec<models::Message>>();

    // call create_topic_message_query with the new_message and previous_messages
    let new_topic_message_result = web::block(move || {
        create_topic_message_query(previous_messages, new_message, user.id, &third_pool)
    })
    .await?;
    let previous_messages = match new_topic_message_result {
        Ok(messages) => messages,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    stream_response(
        previous_messages,
        user.id,
        topic_id,
        fourth_pool,
        mutex_store,
    )
    .await
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
    let second_pool = pool.clone();
    let topic_id: uuid::Uuid = messages_topic_id.into_inner();
    // check if the user owns the topic
    let user_owns_topic =
        web::block(move || user_owns_topic_query(user.id, topic_id, &second_pool));
    if let Ok(false) = user_owns_topic.await {
        return Ok(HttpResponse::Unauthorized().json("Unauthorized"));
    }

    let messages = web::block(move || get_messages_for_topic_query(topic_id, &pool)).await?;

    match messages {
        Ok(messages) => Ok(HttpResponse::Ok().json(messages)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RegenerateMessageData {
    topic_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EditMessageData {
    topic_id: uuid::Uuid,
    message_sort_order: i32,
    new_message_content: String,
}

pub async fn edit_message_handler(
    data: web::Json<EditMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    mutex_store: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = data.topic_id;
    let message_sort_order = data.message_sort_order;
    let new_message_content = &data.new_message_content;
    let second_pool = pool.clone();
    let third_pool = pool.clone();

    let message_from_sort_order_result = web::block(move || {
        get_message_by_sort_for_topic_query(topic_id, message_sort_order, &pool)
    })
    .await?;

    let message_id = match message_from_sort_order_result {
        Ok(message) => message.id,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    let _ = web::block(move || delete_message_query(&user.id, message_id, topic_id, &second_pool))
        .await?;

    create_message_completion_handler(
        actix_web::web::Json(CreateMessageData {
            new_message_content: new_message_content.to_string(),
            topic_id,
        }),
        user,
        third_pool,
        mutex_store,
    )
    .await
}

pub async fn regenerate_message_handler(
    data: web::Json<RegenerateMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    mutex_store: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = data.topic_id;
    let second_pool = pool.clone();
    let third_pool = pool.clone();

    let previous_messages_result =
        web::block(move || get_topic_messages(topic_id, &second_pool)).await?;

    let mut previous_messages = match previous_messages_result {
        Ok(messages) => messages,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    if previous_messages.len() < 3 {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Not enough messages to regenerate",
        }));
    }
    if previous_messages.len() == 3 {
        return stream_response(
            previous_messages,
            user.id,
            topic_id,
            third_pool,
            mutex_store,
        )
        .await;
    }

    // remove citations from the previous messages
    previous_messages = previous_messages
        .into_iter()
        .map(|message| {
            let mut message = message;
            if message.role == "assistant" {
                message.content = message
                    .content
                    .split("||")
                    .last()
                    .unwrap_or("I give up, I can't find a citation")
                    .to_string();
            }
            message
        })
        .collect::<Vec<models::Message>>();

    let mut message_to_regenerate = None;
    for message in previous_messages.iter().rev() {
        if message.role == "assistant" {
            message_to_regenerate = Some(message.clone());
            break;
        }
    }

    let message_id = match message_to_regenerate {
        Some(message) => message.id,
        None => {
            return Ok(HttpResponse::BadRequest().json(DefaultError {
                message: "No message to regenerate",
            }));
        }
    };

    let mut previous_messages_to_regenerate = Vec::new();
    for message in previous_messages.iter() {
        if message.id == message_id {
            break;
        }
        previous_messages_to_regenerate.push(message.clone());
    }

    let _ = web::block(move || delete_message_query(&user.id, message_id, topic_id, &pool)).await?;

    stream_response(
        previous_messages_to_regenerate,
        user.id,
        topic_id,
        third_pool,
        mutex_store,
    )
    .await
}

pub async fn stream_response(
    messages: Vec<models::Message>,
    user_id: uuid::Uuid,
    topic_id: uuid::Uuid,
    pool: web::Data<Pool>,
    mutex_store: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    let pool1 = pool.clone();
    let pool2 = pool.clone();

    let open_ai_messages: Vec<ChatMessage> = messages
        .iter()
        .map(|message| ChatMessage::from(message.clone()))
        .collect();

    let open_ai_api_key = env!("OPENAI_API_KEY","OPENAI_API_KEY is not present.");
    let client = Client::new(open_ai_api_key);
    let next_message_order = move || {
        let messages_len = messages.len();
        if messages_len == 0 {
            return 3;
        }
        messages_len + 1
    };

    // find evidence for the counter-argument
    let counter_arg_parameters = ChatCompletionParameters {
        model: "gpt-3.5-turbo".into(),
        messages: vec![ChatMessage {
            role: Role::User,
            content: format!(
                "Write a 1-2 sentence counterargument for: \"{}\"",
                open_ai_messages
                    .clone()
                    .last()
                    .expect("No messages")
                    .clone()
                    .content
            ),
            name: None,
        }],
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        max_tokens: None,
        presence_penalty: Some(0.8),
        frequency_penalty: Some(0.8),
        logit_bias: None,
        user: None,
    };

    let evidence_search_query = client
        .chat()
        .create(counter_arg_parameters)
        .await
        .expect("No OpenAI Completion for evidence search");
    let embedding_vector = create_embedding(
        evidence_search_query
            .choices
            .first()
            .expect("No response")
            .message
            .content
            .as_str(),
        mutex_store,
    )
    .await?;

    let search_card_query_results =
        search_card_query(embedding_vector, 1, pool1, None, None, None, None)
            .await
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    // get the first and third card from the result
    let first_third_cards = vec![
        search_card_query_results
            .search_results
            .get(0)
            .expect("No first card")
            .point_id,
        search_card_query_results
            .search_results
            .get(2)
            .expect("No third card")
            .point_id,
    ];
    let first_third_cards1 = first_third_cards.clone();

    let (metadata_cards, collided_cards) = web::block(move || {
        get_metadata_and_collided_cards_from_point_ids_query(first_third_cards, None, pool2)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let metadata_card0 = if metadata_cards.get(0).expect("No first card").private {
        let matching_collided_card = collided_cards
            .iter()
            .find(|card| card.qdrant_id == first_third_cards1[0] && !card.metadata.private)
            .expect("No public first card metadata");

        matching_collided_card.metadata.clone()
    } else {
        metadata_cards
            .get(0)
            .expect("No first card metadata")
            .clone()
    };

    let metadata_card2 = if metadata_cards.get(1).expect("No third card").private {
        let matching_collided_card = collided_cards
            .iter()
            .find(|card| card.qdrant_id == first_third_cards1[1] && !card.metadata.private)
            .expect("No public third card metadata");

        matching_collided_card.metadata.clone()
    } else {
        metadata_cards
            .get(1)
            .expect("No third card metadata")
            .clone()
    };

    let citation_cards = vec![metadata_card0.clone(), metadata_card2.clone()];
    let citation_cards_stringified =
        serde_json::to_string(&citation_cards).expect("Failed to serialize citation cards");
    let citation_cards_stringified1 = citation_cards_stringified.clone();

    let last_message_with_evidence = format!(
        "Here's my argument: {} \n\n Pretending you found it, use the following evidence and only the following evidence when constructing your counter-argument : \n\n {},{} \n {},{}",
        open_ai_messages.last().unwrap().content,
        metadata_card0
            .link
            .clone()
            .unwrap_or("".to_string())
            ,
        if metadata_card0.content.len() > 2000 {metadata_card0.content[..2000].to_string()} else {metadata_card0.content.clone()},
        metadata_card2
            .link
            .clone()
            .unwrap_or("".to_string())
            ,
            if metadata_card2.content.len() > 2000 {metadata_card2.content[..2000].to_string()} else {metadata_card2.content.clone()},
    );

    // replace the last message with the last message with evidence
    let open_ai_messages = open_ai_messages
        .clone()
        .into_iter()
        .enumerate()
        .map(|(index, message)| {
            if index == open_ai_messages.len() - 1 {
                ChatMessage {
                    role: message.role,
                    content: last_message_with_evidence.clone(),
                    name: message.name,
                }
            } else {
                message
            }
        })
        .collect();

    let parameters = ChatCompletionParameters {
        model: "gpt-3.5-turbo".into(),
        messages: open_ai_messages,
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        max_tokens: None,
        presence_penalty: Some(0.8),
        frequency_penalty: Some(0.8),
        logit_bias: None,
        user: None,
    };

    let (s, r) = unbounded::<String>();
    let stream = client.chat().create_stream(parameters).await.unwrap();

    Arbiter::new().spawn(async move {
        let chunk_v: Vec<String> = r.iter().collect();
        let completion = chunk_v.join("");

        let new_message = models::Message::from_details(
            format!("{}||{}", citation_cards_stringified, completion),
            topic_id,
            next_message_order().try_into().unwrap(),
            "assistant".to_string(),
            None,
            Some(chunk_v.len().try_into().unwrap()),
        );

        let _ = create_message_query(new_message, user_id, &pool);
    });

    let new_stream = stream::iter(vec![
        Ok(Bytes::from(citation_cards_stringified1)),
        Ok(Bytes::from("||".to_string())),
    ]);

    Ok(HttpResponse::Ok().streaming(new_stream.chain(stream.map(
        move |response| -> Result<Bytes, actix_web::Error> {
            if let Ok(response) = response {
                let chat_content = response.choices[0].delta.content.clone();
                if let Some(message) = chat_content.clone() {
                    s.send(message).unwrap();
                }
                return Ok(Bytes::from(chat_content.unwrap_or("".to_string())));
            }
            Err(ServiceError::InternalServerError.into())
        },
    ))))
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CutCardData {
    pub uncut_card: String,
    pub num_sentences: Option<i32>,
    pub model: Option<String>,
}

pub async fn create_cut_card_handler(
    data: web::Json<CutCardData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let uncut_card_data = data.into_inner();

    let client = reqwest::Client::new();
    let json = json!({
        "input": uncut_card_data.uncut_card,
        "num_sentences": uncut_card_data.num_sentences,
        "model": uncut_card_data.model
    });
    let res = client
        .post("http://3.142.75.154/cut")
        .json(&json)
        .send()
        .await
        .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    match res.error_for_status() {
        Ok(res) => {
            let completion_string = res.text().await.map_err(|e| {
                ServiceError::BadRequest(format!("Failed to get text from response: {}", e))
            })?;
            let completion_string1 = completion_string.clone();

            web::block(move || create_cut_card(user.id, completion_string, pool))
                .await?
                .map_err(|e| ServiceError::BadRequest(e.message.into()))?;

            Ok(HttpResponse::Ok().json(json!({
                "completion": completion_string1,
            })))
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(json!({
            "error": e.to_string(),
        }))),
    }
}
