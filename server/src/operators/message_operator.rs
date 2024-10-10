use crate::data::models::{
    self, ChunkMetadataStringTagSet, ChunkMetadataTypes, Dataset, DatasetConfiguration, LLMOptions,
    QueryTypes, RagQueryEventClickhouse, RedisPool, SearchMethod,
};
use crate::diesel::prelude::*;
use crate::get_env;
use crate::handlers::chunk_handler::{ParsedQuery, SearchChunksReqPayload};
use crate::handlers::message_handler::CreateMessageReqPayload;
use crate::operators::clickhouse_operator::ClickHouseEvent;
use crate::operators::parse_operator::convert_html_to_text;
use crate::{
    data::models::{Message, Pool, SearchQueryEventClickhouse},
    errors::ServiceError,
};
use actix::Arbiter;
use actix_web::web::Bytes;
use actix_web::{web, HttpResponse};
use crossbeam_channel::unbounded;
use diesel_async::RunQueryDsl;
use futures::StreamExt;
use futures_util::stream;
use openai_dive::v1::resources::chat::{DeltaChatMessage, ImageUrl, ImageUrlType};
use openai_dive::v1::{
    api::Client,
    resources::{
        chat::{ChatCompletionParameters, ChatMessage, ChatMessageContent},
        shared::StopToken,
    },
};
use serde::{Deserialize, Serialize};
use simple_server_timing_header::Timer;

use super::clickhouse_operator::{get_latency_from_header, EventQueue};
use super::search_operator::search_hybrid_chunks;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionDTO {
    pub completion_message: Message,
    pub completion_tokens: i32,
}

#[tracing::instrument(skip(pool))]
pub async fn get_topic_messages(
    messages_topic_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<Message>, ServiceError> {
    use crate::data::schema::messages::dsl::*;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let topic_messages = messages
        .filter(topic_id.eq(messages_topic_id))
        .filter(dataset_id.eq(given_dataset_id))
        .filter(deleted.eq(false))
        .order(sort_order.asc())
        .load::<Message>(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest("Error getting topic messages".to_string())
        })?;

    Ok(topic_messages)
}

#[tracing::instrument(skip(pool))]
pub async fn create_messages_query(
    new_messages: Vec<Message>,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::messages::dsl::messages;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    diesel::insert_into(messages)
        .values(&new_messages)
        .execute(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest("Error creating message, try again".to_string())
        })?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn create_generic_system_message(
    system_prompt: String,
    messages_topic_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Message, ServiceError> {
    let topic =
        crate::operators::topic_operator::get_topic_query(messages_topic_id, dataset_id, pool)
            .await?;

    let system_message = Message::from_details(
        system_prompt,
        topic.id,
        0,
        "system".into(),
        Some(0),
        Some(0),
        dataset_id,
        uuid::Uuid::new_v4(),
    );

    Ok(system_message)
}

#[tracing::instrument(skip(pool))]
pub async fn create_topic_message_query(
    config: &DatasetConfiguration,
    previous_messages: Vec<Message>,
    new_message: Message,
    dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<Message>, ServiceError> {
    let mut ret_messages = previous_messages.clone();
    let mut new_message_copy = new_message.clone();
    let mut previous_messages_len = previous_messages.len();

    if previous_messages.is_empty() {
        let system_message = create_generic_system_message(
            config.SYSTEM_PROMPT.clone(),
            new_message.topic_id,
            dataset_id,
            pool,
        )
        .await?;
        ret_messages.extend(vec![system_message.clone()]);
        create_messages_query(vec![system_message], pool).await?;
        previous_messages_len = 1;
    }

    new_message_copy.sort_order = previous_messages_len as i32;

    create_messages_query(vec![new_message_copy.clone()], pool).await?;
    ret_messages.push(new_message_copy);

    Ok(ret_messages)
}

#[tracing::instrument(skip(pool))]
pub async fn get_message_by_sort_for_topic_query(
    message_topic_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    message_sort_order: i32,
    pool: &web::Data<Pool>,
) -> Result<Message, ServiceError> {
    use crate::data::schema::messages::dsl::*;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    messages
        .filter(deleted.eq(false))
        .filter(topic_id.eq(message_topic_id))
        .filter(sort_order.eq(message_sort_order))
        .filter(dataset_id.eq(given_dataset_id))
        .first::<Message>(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest(
                "This message does not exist for the authenticated user".to_string(),
            )
        })
}

#[tracing::instrument(skip(pool))]
pub async fn get_messages_for_topic_query(
    message_topic_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<Message>, ServiceError> {
    use crate::data::schema::messages::dsl::*;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    messages
        .filter(topic_id.eq(message_topic_id))
        .filter(deleted.eq(false))
        .filter(dataset_id.eq(given_dataset_id))
        .order_by(sort_order.asc())
        .load::<Message>(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest(
                "This topic does not exist for the authenticated user".to_string(),
            )
        })
}

#[tracing::instrument(skip(pool))]
pub async fn delete_message_query(
    given_message_id: uuid::Uuid,
    given_topic_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::messages::dsl::*;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let target_message: Message = messages
        .find(given_message_id)
        .first::<Message>(&mut conn)
        .await
        .map_err(|_db_error| ServiceError::BadRequest("Error finding message".to_string()))?;

    diesel::update(
        messages
            .filter(topic_id.eq(given_topic_id))
            .filter(dataset_id.eq(given_dataset_id))
            .filter(sort_order.ge(target_message.sort_order)),
    )
    .set(deleted.eq(true))
    .execute(&mut conn)
    .await
    .map_err(|_| ServiceError::BadRequest("Error deleting message".to_string()))?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool, redis_pool, event_queue))]
pub async fn stream_response(
    messages: Vec<models::Message>,
    topic_id: uuid::Uuid,
    dataset: Dataset,
    pool: web::Data<Pool>,
    event_queue: web::Data<EventQueue>,
    redis_pool: web::Data<RedisPool>,
    dataset_config: DatasetConfiguration,
    create_message_req_payload: CreateMessageReqPayload,
) -> Result<HttpResponse, actix_web::Error> {
    let dataset_config = DatasetConfiguration::from_json(dataset.server_configuration.clone());

    let user_message_query = match create_message_req_payload.concat_user_messages_query {
        Some(true) => messages
            .iter()
            .filter(|message| message.role == "user")
            .map(|message| message.content.clone())
            .collect::<Vec<String>>()
            .join("\n\n"),
        _ => match messages.last() {
            Some(message) => message.clone().content,
            None => {
                return Err(ServiceError::BadRequest(
                    "No messages found for the topic".to_string(),
                )
                .into());
            }
        },
    };

    let openai_messages: Vec<ChatMessage> = messages
        .iter()
        .map(|message| ChatMessage::from(message.clone()))
        .collect();

    let base_url = dataset_config.LLM_BASE_URL.clone();

    let llm_api_key = if !dataset_config.LLM_API_KEY.is_empty() {
        dataset_config.LLM_API_KEY.clone()
    } else if base_url.contains("openai.com") {
        get_env!("OPENAI_API_KEY", "OPENAI_API_KEY for openai should be set").into()
    } else {
        get_env!(
            "LLM_API_KEY",
            "LLM_API_KEY for openrouter or self-hosted should be set"
        )
        .into()
    };

    let client = Client {
        headers: None,
        project: None,
        api_key: llm_api_key,
        http_client: reqwest::Client::new(),
        base_url,
        organization: None,
    };

    let next_message_order = move || {
        let messages_len = messages.len();
        if messages_len == 0 {
            return 2;
        }
        messages_len
    };

    let rag_prompt = dataset_config.RAG_PROMPT.clone();
    let chosen_model = dataset_config.LLM_DEFAULT_MODEL.clone();

    let mut query =
        if let Some(create_message_query) = create_message_req_payload.search_query.clone() {
            create_message_query
        } else {
            user_message_query
        };

    let use_message_to_query_prompt = dataset_config.USE_MESSAGE_TO_QUERY_PROMPT;
    if create_message_req_payload.search_query.is_none() && use_message_to_query_prompt {
        let message_to_query_prompt = dataset_config.MESSAGE_TO_QUERY_PROMPT.clone();
        let gen_inference_msgs = vec![ChatMessage::User {
            content: ChatMessageContent::Text(format!("{}\n{}", message_to_query_prompt, query)),
            name: None,
        }];

        let gen_inference_parameters = ChatCompletionParameters {
            model: chosen_model.clone(),
            messages: gen_inference_msgs,
            stream: Some(false),
            temperature: dataset_config.TEMPERATURE.map(|temp| temp as f32),
            frequency_penalty: Some(dataset_config.FREQUENCY_PENALTY.unwrap_or(0.8) as f32),
            presence_penalty: Some(dataset_config.PRESENCE_PENALTY.unwrap_or(0.8) as f32),
            stop: dataset_config.STOP_TOKENS.clone().map(StopToken::Array),
            top_p: None,
            n: None,
            max_completion_tokens: dataset_config.MAX_TOKENS.map(|max| max as u32),
            logit_bias: None,
            user: None,
            response_format: None,
            tools: None,
            tool_choice: None,
            logprobs: None,
            top_logprobs: None,
            seed: None,
            ..Default::default()
        };

        let search_query_from_message_to_query_prompt = client
            .chat()
            .create(gen_inference_parameters)
            .await
            .expect("No LLM Completion for chunk search");

        query = match &search_query_from_message_to_query_prompt
            .choices
            .get(0)
            .expect("No response for LLM completion")
            .message
        {
            ChatMessage::User {
                content: ChatMessageContent::Text(query),
                ..
            }
            | ChatMessage::System {
                content: ChatMessageContent::Text(query),
                ..
            }
            | ChatMessage::Assistant {
                content: Some(ChatMessageContent::Text(query)),
                ..
            } => query.clone(),
            _ => "".to_string(),
        };
    }

    let n_retrievals_to_include = dataset_config.N_RETRIEVALS_TO_INCLUDE;
    let search_chunk_data = SearchChunksReqPayload {
        search_type: create_message_req_payload
            .search_type
            .unwrap_or(SearchMethod::Hybrid),
        query: QueryTypes::Single(query.clone()),
        score_threshold: create_message_req_payload.score_threshold,
        page_size: Some(
            create_message_req_payload
                .page_size
                .unwrap_or(n_retrievals_to_include.try_into().unwrap_or(8)),
        ),
        highlight_options: create_message_req_payload.highlight_options,
        filters: create_message_req_payload.filters,
        ..Default::default()
    };
    let parsed_query = ParsedQuery {
        query: query.clone(),
        quote_words: None,
        negated_words: None,
    };
    let mut search_timer = Timer::new();

    let result_chunks = search_hybrid_chunks(
        search_chunk_data.clone(),
        parsed_query,
        pool.clone(),
        redis_pool,
        dataset.clone(),
        &dataset_config,
        &mut search_timer,
    )
    .await?;

    let clickhouse_search_event = SearchQueryEventClickhouse {
        request_params: serde_json::to_string(&search_chunk_data.clone()).unwrap_or_default(),
        id: uuid::Uuid::new_v4(),
        search_type: "rag".to_string(),
        query: query.clone(),
        dataset_id: dataset.id,
        top_score: result_chunks
            .score_chunks
            .get(0)
            .map(|x| x.score as f32)
            .unwrap_or(0.0),

        latency: get_latency_from_header(search_timer.header_value()),
        results: result_chunks
            .score_chunks
            .clone()
            .into_iter()
            .map(|x| serde_json::to_string(&x).unwrap_or_default())
            .collect(),
        created_at: time::OffsetDateTime::now_utc(),
        query_rating: String::from(""),
        user_id: create_message_req_payload
            .user_id
            .clone()
            .unwrap_or_default(),
    };

    event_queue
        .send(ClickHouseEvent::SearchQueryEvent(
            clickhouse_search_event.clone(),
        ))
        .await;

    let chunk_metadatas = result_chunks
        .score_chunks
        .iter()
        .map(
            |score_chunk| match score_chunk.metadata.get(0).expect("No metadata found") {
                ChunkMetadataTypes::Metadata(chunk_metadata) => chunk_metadata.clone(),
                _ => unreachable!("The operator should never return slim chunks for this"),
            },
        )
        .collect::<Vec<ChunkMetadataStringTagSet>>();

    let chunk_data = result_chunks
        .score_chunks
        .clone()
        .into_iter()
        .map(|x| serde_json::to_string(&x).unwrap_or_default())
        .collect();

    let mut chunk_metadatas_stringified =
        serde_json::to_string(&chunk_metadatas).expect("Failed to serialize citation chunks");
    let mut chunk_metadatas_stringified1 = chunk_metadatas_stringified.clone();

    let rag_content = chunk_metadatas
        .iter()
        .enumerate()
        .map(|(idx, chunk)| {
            format!(
                "Doc {}: {}",
                idx + 1,
                convert_html_to_text(&(chunk.chunk_html.clone().unwrap_or_default()))
            )
        })
        .collect::<Vec<String>>()
        .join("\n\n");

    let last_message = ChatMessageContent::Text(format!(
        "Here's my prompt: {} \n\n {} {}",
        match &openai_messages
            .last()
            .expect("There needs to be at least 1 prior message")
        {
            ChatMessage::User {
                content: ChatMessageContent::Text(text),
                ..
            }
            | ChatMessage::System {
                content: ChatMessageContent::Text(text),
                ..
            }
            | ChatMessage::Assistant {
                content: Some(ChatMessageContent::Text(text)),
                ..
            } => text.clone(),
            _ => "".to_string(),
        },
        rag_prompt,
        rag_content,
    ));

    let images: Vec<String> = chunk_metadatas
        .iter()
        .filter_map(|chunk| chunk.image_urls.clone())
        .flat_map(|image_urls| {
            image_urls
                .iter()
                .filter_map(|image| image.clone())
                .collect::<Vec<_>>()
        })
        .collect();

    // replace the last message with the last message with evidence
    let mut open_ai_messages: Vec<ChatMessage> = openai_messages
        .clone()
        .into_iter()
        .enumerate()
        .map(|(index, message)| {
            if index == openai_messages.len() - 1 {
                match message {
                    ChatMessage::Assistant { name, .. } => ChatMessage::Assistant {
                        content: Some(last_message.clone()),
                        name,
                        tool_calls: None,
                        refusal: None,
                    },
                    ChatMessage::System { name, .. } => ChatMessage::System {
                        content: last_message.clone(),
                        name,
                    },
                    ChatMessage::User { name, .. } => ChatMessage::User {
                        content: last_message.clone(),
                        name,
                    },
                    _ => message,
                }
            } else {
                message
            }
        })
        .collect();

    if !images.is_empty() {
        if let Some(LLMOptions {
            image_config: Some(ref image_config),
            ..
        }) = create_message_req_payload.llm_options
        {
            if image_config.use_images.unwrap_or(false) {
                open_ai_messages.push(ChatMessage::User {
                    name: None,
                    content: ChatMessageContent::ImageUrl(
                        images
                            .iter()
                            .take(image_config.images_per_chunk.unwrap_or(5))
                            .map(|url| ImageUrl {
                                r#type: "image_url".to_string(),
                                text: None,
                                image_url: ImageUrlType {
                                    url: url.to_string(),
                                    detail: None,
                                },
                            })
                            .collect(),
                    ),
                })
            }
        }
    }

    let mut parameters = ChatCompletionParameters {
        model: chosen_model,
        messages: open_ai_messages,
        ..Default::default()
    };

    if let Some(llm_options) = create_message_req_payload.llm_options.clone() {
        parameters.stream = llm_options.stream_response;
        parameters.temperature = dataset_config
            .TEMPERATURE
            .map(|x| x as f32)
            .or(llm_options.temperature);
        parameters.frequency_penalty = dataset_config
            .FREQUENCY_PENALTY
            .map(|x| x as f32)
            .or(llm_options.frequency_penalty);
        parameters.presence_penalty = dataset_config
            .PRESENCE_PENALTY
            .map(|x| x as f32)
            .or(llm_options.presence_penalty);
        parameters.max_completion_tokens = dataset_config
            .MAX_TOKENS
            .map(|x| x as u32)
            .or(llm_options.max_tokens);
        parameters.stop = dataset_config
            .STOP_TOKENS
            .clone()
            .map(StopToken::Array)
            .or(llm_options.stop_tokens.map(StopToken::Array));
    }

    if !chunk_metadatas_stringified.is_empty() {
        chunk_metadatas_stringified = if create_message_req_payload
            .llm_options
            .as_ref()
            .map(|x| x.completion_first)
            .unwrap_or(Some(false))
            .unwrap_or(false)
        {
            format!("||{}", chunk_metadatas_stringified.replace("||", ""))
        } else {
            format!("{}||", chunk_metadatas_stringified.replace("||", ""))
        };
        chunk_metadatas_stringified1.clone_from(&chunk_metadatas_stringified);
    }

    let query_id = uuid::Uuid::new_v4();

    if !create_message_req_payload
        .llm_options
        .as_ref()
        .map(|x| x.stream_response)
        .unwrap_or(Some(true))
        .unwrap_or(true)
    {
        let assistant_completion =
            client
                .chat()
                .create(parameters.clone())
                .await
                .map_err(|err| {
                    ServiceError::BadRequest(format!(
                        "Bad response from LLM server provider: {}",
                        err
                    ))
                })?;

        let completion_content = match &assistant_completion
            .choices
            .get(0)
            .map(|chat_completion_choice| chat_completion_choice.message.clone())
        {
            Some(ChatMessage::User {
                content: ChatMessageContent::Text(text),
                ..
            })
            | Some(ChatMessage::System {
                content: ChatMessageContent::Text(text),
                ..
            })
            | Some(ChatMessage::Assistant {
                content: Some(ChatMessageContent::Text(text)),
                ..
            }) => text.clone(),
            _ => "".to_string(),
        };

        let new_message = models::Message::from_details(
            format!(
                "{}{}",
                chunk_metadatas_stringified,
                completion_content.clone()
            ),
            topic_id,
            next_message_order()
                .try_into()
                .expect("usize to i32 conversion should always succeed"),
            "assistant".to_string(),
            None,
            Some(
                completion_content
                    .len()
                    .try_into()
                    .expect("usize to i32 conversion should always succeed"),
            ),
            dataset.id,
            query_id,
        );

        let clickhouse_rag_event = RagQueryEventClickhouse {
            id: query_id,
            created_at: time::OffsetDateTime::now_utc(),
            dataset_id: dataset.id,
            search_id: clickhouse_search_event.id,
            results: vec![],
            json_results: chunk_data,
            user_message: query.clone(),
            query_rating: String::new(),
            rag_type: "chosen_chunks".to_string(),
            llm_response: completion_content.clone(),
            user_id: create_message_req_payload
                .user_id
                .clone()
                .unwrap_or_default(),
        };

        event_queue
            .send(ClickHouseEvent::RagQueryEvent(clickhouse_rag_event.clone()))
            .await;

        create_messages_query(vec![new_message], &pool).await?;

        return Ok(HttpResponse::Ok()
            .insert_header(("TR-QueryID", query_id.to_string()))
            .json(completion_content));
    }

    let (s, r) = unbounded::<String>();
    let stream = client.chat().create_stream(parameters).await.unwrap();

    let completion_first = create_message_req_payload
        .llm_options
        .as_ref()
        .map(|x| x.completion_first)
        .unwrap_or(Some(false))
        .unwrap_or(false);

    let query_id_arb = query_id;

    Arbiter::new().spawn(async move {
        let chunk_v: Vec<String> = r.iter().collect();
        let completion = chunk_v.join("");

        let message_to_be_stored = if completion_first {
            format!("{}{}", completion, chunk_metadatas_stringified)
        } else {
            format!("{}{}", chunk_metadatas_stringified, completion)
        };

        let new_message = models::Message::from_details(
            message_to_be_stored,
            topic_id,
            next_message_order().try_into().unwrap(),
            "assistant".to_string(),
            None,
            Some(chunk_v.len().try_into().unwrap()),
            dataset.id,
            query_id_arb,
        );

        let clickhouse_rag_event = RagQueryEventClickhouse {
            id: query_id_arb,
            created_at: time::OffsetDateTime::now_utc(),
            dataset_id: dataset.id,
            search_id: clickhouse_search_event.id,
            results: vec![],
            json_results: chunk_data,
            user_message: query.clone(),
            query_rating: String::new(),
            rag_type: "all_chunks".to_string(),
            llm_response: completion.clone(),
            user_id: create_message_req_payload
                .user_id
                .clone()
                .unwrap_or_default(),
        };

        event_queue
            .send(ClickHouseEvent::RagQueryEvent(clickhouse_rag_event.clone()))
            .await;

        let _ = create_messages_query(vec![new_message], &pool).await;
    });

    let chunk_stream = stream::iter(vec![Ok(Bytes::from(chunk_metadatas_stringified1))]);
    let completion_stream = stream.map(move |response| -> Result<Bytes, actix_web::Error> {
        if let Ok(response) = response {
            let chat_content = response
                .choices
                .get(0)
                .map(
                    |chat_completion_content| match &chat_completion_content.delta {
                        DeltaChatMessage::User {
                            content: ChatMessageContent::Text(topic),
                            ..
                        }
                        | DeltaChatMessage::System {
                            content: ChatMessageContent::Text(topic),
                            ..
                        }
                        | DeltaChatMessage::Assistant {
                            content: Some(ChatMessageContent::Text(topic)),
                            ..
                        } => Some(topic.clone()),
                        _ => None,
                    },
                )
                .unwrap_or(None);

            if let Some(message) = chat_content.clone() {
                s.send(message).unwrap();
            }
            return Ok(Bytes::from(chat_content.unwrap_or("".to_string())));
        }
        Err(ServiceError::InternalServerError(format!(
            "Model Response Error. Please try again later. {:?}",
            response
        ))
        .into())
    });

    if create_message_req_payload
        .llm_options
        .as_ref()
        .map(|x| x.completion_first)
        .unwrap_or(Some(false))
        .unwrap_or(false)
    {
        return Ok(HttpResponse::Ok()
            .insert_header(("TR-QueryID", query_id.to_string()))
            .streaming(completion_stream.chain(chunk_stream)));
    }

    Ok(HttpResponse::Ok()
        .insert_header(("TR-QueryID", query_id.to_string()))
        .streaming(chunk_stream.chain(completion_stream)))
}

#[tracing::instrument]
pub async fn get_topic_string(
    model: String,
    first_message: String,
    dataset: &Dataset,
) -> Result<String, ServiceError> {
    let prompt_topic_message = ChatMessage::User {
        content: ChatMessageContent::Text(format!(
            "Write a 2-3 word topic name from the following prompt: {}",
            first_message
        )),
        name: None,
    };
    let parameters = ChatCompletionParameters {
        model,
        messages: vec![prompt_topic_message],
        stream: Some(false),
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        presence_penalty: Some(0.8),
        frequency_penalty: Some(0.8),
        logit_bias: None,
        user: None,
        response_format: None,
        tools: None,
        tool_choice: None,
        logprobs: None,
        top_logprobs: None,
        seed: None,
        ..Default::default()
    };

    let dataset_config = DatasetConfiguration::from_json(dataset.server_configuration.clone());
    let base_url = dataset_config.LLM_BASE_URL;

    let base_url = if base_url.is_empty() {
        "https://openrouter.ai/api/v1".into()
    } else {
        base_url
    };

    let llm_api_key = if !dataset_config.LLM_API_KEY.is_empty() {
        dataset_config.LLM_API_KEY.clone()
    } else if base_url.contains("openai.com") {
        get_env!("OPENAI_API_KEY", "OPENAI_API_KEY for openai should be set").into()
    } else {
        get_env!(
            "LLM_API_KEY",
            "LLM_API_KEY for openrouter or self-hosted should be set"
        )
        .into()
    };

    let client = Client {
        headers: None,
        api_key: llm_api_key,
        project: None,
        http_client: reqwest::Client::new(),
        base_url,
        organization: None,
    };

    let query = client
        .chat()
        .create(parameters)
        .await
        .map_err(|_| ServiceError::BadRequest("No LLM Completion for topic".to_string()))?;

    let topic = match &query
        .choices
        .get(0)
        .ok_or(ServiceError::BadRequest(
            "No response for LLM completion".to_string(),
        ))?
        .message
    {
        ChatMessage::User {
            content: ChatMessageContent::Text(topic),
            ..
        }
        | ChatMessage::System {
            content: ChatMessageContent::Text(topic),
            ..
        }
        | ChatMessage::Assistant {
            content: Some(ChatMessageContent::Text(topic)),
            ..
        } => topic.clone(),
        _ => "".to_string(),
    };

    Ok(topic)
}
