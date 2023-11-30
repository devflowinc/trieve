use super::{
    auth_handler::{LoggedUser, RequireAuth},
    card_handler::ParsedQuery,
};
use crate::{
    data::models,
    data::models::{CardMetadataWithVotesWithScore, Pool},
    errors::{DefaultError, ServiceError},
    get_env,
    operators::{
        card_operator::{
            find_relevant_sentence, get_metadata_and_collided_cards_from_point_ids_query,
        },
        message_operator::{
            create_message_query, create_topic_message_query, delete_message_query,
            get_message_by_sort_for_topic_query, get_messages_for_topic_query, get_topic_messages,
            user_owns_topic_query,
        },
        qdrant_operator::create_embedding,
        search_operator::retrieve_qdrant_points_query,
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
use tokio_stream::StreamExt;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct CreateMessageData {
    pub new_message_content: String,
    pub topic_id: uuid::Uuid,
}

#[utoipa::path(
    post,
    path = "/message",
    context_path = "/api",
    tag = "message",
    request_body(content = CreateMessageData, description = "JSON request payload to create a message completion", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream, check the chat or search UI for an example how to process this"),
        (status = 400, description = "Service error relating to getting a chat completion", body = [DefaultError]),
    )
)]
pub async fn create_message_completion_handler(
    data: web::Json<CreateMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    app_mutex: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    let create_message_data = data.into_inner();
    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let pool3 = pool.clone();
    let pool4 = pool.clone();
    let topic_id = create_message_data.topic_id;

    let new_message = models::Message::from_details(
        create_message_data.new_message_content,
        topic_id,
        0,
        "user".to_string(),
        None,
        None,
    );

    let user_topic = web::block(move || user_owns_topic_query(user.id, topic_id, &pool1))
        .await?
        .map_err(|_e| ServiceError::Unauthorized)?;

    // get the previous messages
    let mut previous_messages = web::block(move || get_topic_messages(topic_id, &pool2))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if !user_topic.normal_chat {
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
    }

    // call create_topic_message_query with the new_message and previous_messages
    let previous_messages = web::block(move || {
        create_topic_message_query(
            user_topic.normal_chat,
            previous_messages,
            new_message,
            user.id,
            &pool3,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    stream_response(
        user_topic.normal_chat,
        previous_messages,
        user.id,
        topic_id,
        app_mutex,
        pool4,
    )
    .await
}

#[utoipa::path(
    get,
    path = "/messages/{messages_topic_id}",
    context_path = "/api",
    tag = "message",
    responses(
        (status = 200, description = "All messages relating to the topic with the given ID", body = [Vec<Message>]),
        (status = 400, description = "Service error relating to getting the messages", body = [DefaultError]),
    ),
    params(("messages_topic_id" = uuid, description = "The ID of the topic to get messages for"))
)]
pub async fn get_all_topic_messages(
    user: LoggedUser,
    messages_topic_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let second_pool = pool.clone();
    let topic_id: uuid::Uuid = messages_topic_id.into_inner();
    // check if the user owns the topic
    let _user_topic = web::block(move || user_owns_topic_query(user.id, topic_id, &second_pool))
        .await?
        .map_err(|_e| ServiceError::Unauthorized)?;

    let messages = web::block(move || get_messages_for_topic_query(topic_id, &pool)).await?;

    match messages {
        Ok(messages) => Ok(HttpResponse::Ok().json(messages)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct RegenerateMessageData {
    topic_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct EditMessageData {
    topic_id: uuid::Uuid,
    message_sort_order: i32,
    new_message_content: String,
}

#[utoipa::path(
    put,
    path = "/message",
    context_path = "/api",
    tag = "message",
    request_body(content = EditMessageData, description = "JSON request payload to edit a message and get a new stream", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream, check the chat or search UI for an example how to process this"),
        (status = 400, description = "Service error relating to getting a chat completion", body = [DefaultError]),
    )
)]
pub async fn edit_message_handler(
    data: web::Json<EditMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    app_mutex: web::Data<AppMutexStore>,
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
        app_mutex,
    )
    .await
}

#[utoipa::path(
    delete,
    path = "/message",
    context_path = "/api",
    tag = "message",
    request_body(content = RegenerateMessageData, description = "JSON request payload to delete an agent message then regenerate it in a strem", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream, check the chat or search UI for an example how to process this"),
        (status = 400, description = "Service error relating to getting a chat completion", body = [DefaultError]),
    )
)]
pub async fn regenerate_message_handler(
    data: web::Json<RegenerateMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    app_mutex: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = data.topic_id;
    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let pool3 = pool.clone();

    let user_topic = web::block(move || user_owns_topic_query(user.id, topic_id, &pool1))
        .await?
        .map_err(|_e| ServiceError::Unauthorized)?;

    let previous_messages_result = web::block(move || get_topic_messages(topic_id, &pool2)).await?;

    let mut previous_messages = match previous_messages_result {
        Ok(messages) => messages,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    if previous_messages.len() < 2 {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Not enough messages to regenerate",
        }));
    }

    if previous_messages.len() == 2 {
        return stream_response(
            user_topic.normal_chat,
            previous_messages,
            user.id,
            topic_id,
            app_mutex,
            pool3,
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
        user_topic.normal_chat,
        previous_messages_to_regenerate,
        user.id,
        topic_id,
        app_mutex,
        pool3,
    )
    .await
}

pub async fn get_topic_string(prompt: String) -> Result<String, DefaultError> {
    let prompt_topic_message = ChatMessage {
        role: Role::User,
        content: format!(
            "Write a 2-3 word topic name from the following prompt: {}",
            prompt
        ),
        name: None,
    };
    let openai_messages = vec![prompt_topic_message];
    let parameters = ChatCompletionParameters {
        model: "gpt-3.5-turbo".into(),
        messages: openai_messages,
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

    let openai_api_key = get_env!("OPENAI_API_KEY", "OPENAI_API_KEY should be set").into();
    let client = Client {
        api_key: openai_api_key,
        http_client: reqwest::Client::new(),
        base_url: std::env::var("OPENAI_BASE_URL")
            .map(|url| {
                if url.is_empty() {
                    "https://api.openai.com/v1".to_string()
                } else {
                    url
                }
            })
            .unwrap_or("https://api.openai.com/v1".into()),
    };

    let query = client
        .chat()
        .create(parameters)
        .await
        .expect("No OpenAI Completion for topic");
    let topic = query
        .choices
        .first()
        .expect("No response for OpenAI completion")
        .message
        .content
        .to_string();

    Ok(topic)
}

pub async fn stream_response(
    normal_chat: bool,
    messages: Vec<models::Message>,
    user_id: uuid::Uuid,
    topic_id: uuid::Uuid,
    app_mutex: web::Data<AppMutexStore>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let pool2 = pool.clone();

    let openai_messages: Vec<ChatMessage> = messages
        .iter()
        .map(|message| ChatMessage::from(message.clone()))
        .collect();

    let openai_api_key = get_env!("OPENAI_API_KEY", "OPENAI_API_KEY should be set").into();
    let client = Client {
        api_key: openai_api_key,
        http_client: reqwest::Client::new(),
        base_url: std::env::var("OPENAI_BASE_URL")
            .map(|url| {
                if url.is_empty() {
                    "https://api.openai.com/v1".to_string()
                } else {
                    url
                }
            })
            .unwrap_or("https://api.openai.com/v1".into()),
    };
    let next_message_order = move || {
        let messages_len = messages.len();
        if messages_len == 0 {
            return 2;
        }
        messages_len
    };

    let mut last_message = openai_messages
        .last()
        .expect("There needs to be at least 1 prior message")
        .content
        .clone();
    let mut citation_cards_stringified = "".to_string();
    let mut citation_cards_stringified1 = citation_cards_stringified.clone();

    if !normal_chat {
        let rag_prompt = std::env::var("RAG_PROMPT").unwrap_or("Write a 1-2 sentence semantic search query along the lines of a hypothetical response to: \n\n".to_string());

        // find evidence for the counter-argument
        let counter_arg_parameters = ChatCompletionParameters {
            model: "gpt-3.5-turbo".into(),
            messages: vec![ChatMessage {
                role: Role::User,
                content: format!(
                    "{}{}",
                    rag_prompt,
                    openai_messages
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
        let query = &evidence_search_query
            .choices
            .first()
            .expect("No response for OpenAI completion")
            .message
            .content;
        let embedding_vector = create_embedding(query.as_str(), app_mutex).await?;

        let search_card_query_results = retrieve_qdrant_points_query(
            embedding_vector,
            1,
            None,
            None,
            None,
            None,
            Some(user_id),
            ParsedQuery {
                quote_words: None,
                negated_words: None,
            },
        )
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
        let n_retrievals_to_include = std::env::var("N_RETRIEVALS_TO_INCLUDE")
            .unwrap_or("3".to_string())
            .parse::<usize>()
            .expect("N_RETRIEVALS_TO_INCLUDE must be a number");

        let retrieval_card_ids = search_card_query_results
            .search_results
            .iter()
            .take(n_retrievals_to_include)
            .map(|card| card.point_id)
            .collect::<Vec<uuid::Uuid>>();

        let (metadata_cards, collided_cards) = web::block(move || {
            get_metadata_and_collided_cards_from_point_ids_query(retrieval_card_ids, None, pool2)
        })
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

        let citation_cards: Vec<CardMetadataWithVotesWithScore> = metadata_cards
            .iter()
            .map(|card| {
                if card.private
                    && card
                        .author
                        .as_ref()
                        .is_some_and(|author| author.id != user_id)
                {
                    let matching_collided_card = collided_cards
                        .iter()
                        .find(|card| !card.metadata.private)
                        .expect("No public card metadata");

                    matching_collided_card.metadata.clone()
                } else {
                    card.clone()
                }
            })
            .collect();

        let highlighted_citation_cards = citation_cards
            .iter()
            .map(|card| {
                find_relevant_sentence(card.clone(), query.to_string()).unwrap_or(card.clone())
            })
            .collect::<Vec<CardMetadataWithVotesWithScore>>();

        citation_cards_stringified = serde_json::to_string(&highlighted_citation_cards)
            .expect("Failed to serialize citation cards");
        citation_cards_stringified1 = citation_cards_stringified.clone();

        let rag_content = citation_cards
            .iter()
            .enumerate()
            .map(|(idx, card)| format!("Doc {}: {}", idx + 1, card.content.clone()))
            .collect::<Vec<String>>()
            .join("\n\n");

        last_message = format!(
            "Here's my prompt. Include the document numbers that you used in square brackets at the end of the sentences that you used the docs for: {} \n\n Pretending you found it, base your tone on and use the following retrieved information as the basis of your response.: {}",
            openai_messages.last().expect("There needs to be at least 1 prior message").content,
            rag_content,
        );
    }

    // replace the last message with the last message with evidence
    let open_ai_messages = openai_messages
        .clone()
        .into_iter()
        .enumerate()
        .map(|(index, message)| {
            if index == openai_messages.len() - 1 {
                ChatMessage {
                    role: message.role,
                    content: last_message.clone(),
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

    if !citation_cards_stringified.is_empty() {
        citation_cards_stringified = format!("{}||", citation_cards_stringified);
        citation_cards_stringified1 = citation_cards_stringified.clone();
    }

    Arbiter::new().spawn(async move {
        let chunk_v: Vec<String> = r.iter().collect();
        let completion = chunk_v.join("");

        let new_message = models::Message::from_details(
            format!("{}{}", citation_cards_stringified, completion),
            topic_id,
            next_message_order().try_into().unwrap(),
            "assistant".to_string(),
            None,
            Some(chunk_v.len().try_into().unwrap()),
        );

        let _ = create_message_query(new_message, user_id, &pool);
    });

    let new_stream = stream::iter(vec![Ok(Bytes::from(citation_cards_stringified1))]);

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

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct SuggestedQueriesRequest {
    pub query: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct SuggestedQueriesResponse {
    pub queries: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/card/fulltextsearch/{page}",
    context_path = "/api",
    tag = "card",
    request_body(content = SuggestedQueriesRequest, description = "JSON request payload to get alternative suggested queries", content_type = "application/json"),
    responses(
        (status = 200, description = "A JSON object containing a list of alternative suggested queries", body = [SuggestedQueriesResponse]),
        (status = 400, description = "Service error relating to to updating card, likely due to conflicting tracking_id", body = [DefaultError]),
    )
)]
pub async fn create_suggested_queries_handler(
    data: web::Json<SuggestedQueriesRequest>,
    _required_user: RequireAuth,
) -> Result<HttpResponse, ServiceError> {
    let openai_api_key = get_env!("OPENAI_API_KEY", "OPENAI_API_KEY should be set").into();
    let client = Client {
        api_key: openai_api_key,
        http_client: reqwest::Client::new(),
        base_url: std::env::var("OPENAI_BASE_URL")
            .map(|url| {
                if url.is_empty() {
                    "https://api.openai.com/v1".to_string()
                } else {
                    url
                }
            })
            .unwrap_or("https://api.openai.com/v1".into()),
    };
    let query = format!("generate 3 suggested queries based off this query a user made. Your only response should be the 3 queries which are comma seperated and are just text and you do not add any other context or information about the queries.  Here is the query: {}", data.query);
    let message = ChatMessage {
        role: Role::User,
        content: query,
        name: None,
    };
    let parameters = ChatCompletionParameters {
        model: "gpt-3.5-turbo".into(),
        messages: vec![message],
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

    let mut query = client
        .chat()
        .create(parameters.clone())
        .await
        .expect("No OpenAI Completion for topic");
    let mut queries: Vec<String> = query.choices[0]
        .message
        .content
        .split(',')
        .map(|query| query.to_string().trim().trim_matches('\n').to_string())
        .collect();
    while queries.len() < 3 {
        query = client
            .chat()
            .create(parameters.clone())
            .await
            .expect("No OpenAI Completion for topic");
        queries = query.choices[0]
            .message
            .content
            .split(',')
            .map(|query| query.to_string().trim().trim_matches('\n').to_string())
            .collect();
    }
    Ok(HttpResponse::Ok().json(SuggestedQueriesResponse { queries }))
}
