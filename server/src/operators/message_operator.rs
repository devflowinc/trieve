use crate::data::models::{
    self, ChunkMetadataStringTagSet, ChunkMetadataTypes, Dataset, RagQueryEventClickhouse,
    ServerDatasetConfiguration,
};
use crate::diesel::prelude::*;
use crate::get_env;
use crate::handlers::chunk_handler::{ParsedQuery, SearchChunksReqPayload};
use crate::handlers::message_handler::CreateMessageReqPayload;
use crate::operators::clickhouse_operator::{send_to_clickhouse, ClickHouseEvent};
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
use openai_dive::v1::{
    api::Client,
    resources::{
        chat::{ChatCompletionParameters, ChatMessage, ChatMessageContent, Role},
        shared::StopToken,
    },
};
use serde::{Deserialize, Serialize};
use simple_server_timing_header::Timer;

use super::clickhouse_operator::get_latency_from_header;
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

    let mut conn = pool.get().await.unwrap();

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
pub async fn create_message_query(
    new_message: Message,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::messages::dsl::messages;

    let mut conn = pool.get().await.unwrap();

    diesel::insert_into(messages)
        .values(&new_message)
        .execute(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest("Error creating message, try again".to_string())
        })?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn create_generic_system_message(
    messages_topic_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Message, ServiceError> {
    let topic =
        crate::operators::topic_operator::get_topic_query(messages_topic_id, dataset_id, pool)
            .await?;
    let system_message_content =
        "You are Trieve retrieval augmented chatbot, a large language model trained by Trieve to respond in the same tone as and with the context of retrieved information.";

    let system_message = Message::from_details(
        system_message_content,
        topic.id,
        0,
        "system".into(),
        Some(0),
        Some(0),
        dataset_id,
    );

    Ok(system_message)
}

#[tracing::instrument(skip(pool))]
pub async fn create_topic_message_query(
    previous_messages: Vec<Message>,
    new_message: Message,
    dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<Message>, ServiceError> {
    let mut ret_messages = previous_messages.clone();
    let mut new_message_copy = new_message.clone();
    let mut previous_messages_len = previous_messages.len();

    if previous_messages.is_empty() {
        let system_message =
            create_generic_system_message(new_message.topic_id, dataset_id, pool).await?;
        ret_messages.extend(vec![system_message.clone()]);
        create_message_query(system_message, pool).await?;
        previous_messages_len = 1;
    }

    new_message_copy.sort_order = previous_messages_len as i32;

    create_message_query(new_message_copy.clone(), pool).await?;
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

    let mut conn = pool.get().await.unwrap();

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

    let mut conn = pool.get().await.unwrap();

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

    let mut conn = pool.get().await.unwrap();

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

#[tracing::instrument(skip(pool, clickhouse_client))]
pub async fn stream_response(
    messages: Vec<models::Message>,
    topic_id: uuid::Uuid,
    dataset: Dataset,
    pool: web::Data<Pool>,
    clickhouse_client: web::Data<clickhouse::Client>,
    config: ServerDatasetConfiguration,
    create_message_req_payload: CreateMessageReqPayload,
) -> Result<HttpResponse, actix_web::Error> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());

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

    let llm_api_key = if base_url.contains("openai.com") {
        get_env!("OPENAI_API_KEY", "OPENAI_API_KEY for openai should be set").into()
    } else {
        get_env!(
            "LLM_API_KEY",
            "LLM_API_KEY for openrouter or self-hosted should be set"
        )
        .into()
    };

    let client = Client {
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
        let gen_inference_msgs = vec![ChatMessage {
            role: Role::User,
            content: ChatMessageContent::Text(format!("{}\n{}", message_to_query_prompt, query)),
            tool_calls: None,
            name: None,
            tool_call_id: None,
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
            max_tokens: None,
            logit_bias: None,
            user: None,
            response_format: None,
            tools: None,
            tool_choice: None,
            logprobs: None,
            top_logprobs: None,
            seed: None,
        };

        let search_query_from_message_to_query_prompt = client
            .chat()
            .create(gen_inference_parameters)
            .await
            .expect("No OpenAI Completion for chunk search");

        query = match &search_query_from_message_to_query_prompt
            .choices
            .get(0)
            .expect("No response for OpenAI completion")
            .message
            .content
        {
            ChatMessageContent::Text(query) => query.clone(),
            _ => "".to_string(),
        };
    }

    let n_retrievals_to_include = dataset_config.N_RETRIEVALS_TO_INCLUDE;
    let search_chunk_data = SearchChunksReqPayload {
        search_type: create_message_req_payload
            .search_type
            .unwrap_or("hybrid".to_string()),
        query: query.clone(),
        page_size: Some(
            create_message_req_payload
                .page_size
                .unwrap_or(n_retrievals_to_include.try_into().unwrap_or(8)),
        ),
        highlight_results: create_message_req_payload.highlight_results,
        highlight_delimiters: create_message_req_payload.highlight_delimiters,
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
        dataset.clone(),
        &dataset_config,
        &mut search_timer,
    )
    .await?;

    let clickhouse_search_event = SearchQueryEventClickhouse {
        request_params: serde_json::to_string(&search_chunk_data.clone()).unwrap(),
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
        results: result_chunks.into_response_payload(),
        created_at: time::OffsetDateTime::now_utc(),
    };

    let _ = send_to_clickhouse(
        ClickHouseEvent::SearchQueryEvent(clickhouse_search_event.clone()),
        &clickhouse_client,
    )
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

    let chunk_ids = result_chunks
        .score_chunks
        .iter()
        .filter_map(|score_chunk_dto| {
            score_chunk_dto
                .metadata
                .clone()
                .into_iter()
                .map(|metadata| match metadata {
                    ChunkMetadataTypes::ID(chunk_metadata) => chunk_metadata.id,
                    ChunkMetadataTypes::Metadata(chunk_metadata) => chunk_metadata.id,
                    ChunkMetadataTypes::Content(chunk_metadata) => chunk_metadata.id,
                })
                .next()
        })
        .collect::<Vec<uuid::Uuid>>();

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
            .content
        {
            ChatMessageContent::Text(text) => text.clone(),
            _ => "".to_string(),
        },
        rag_prompt,
        rag_content,
    ));

    // replace the last message with the last message with evidence
    let mut open_ai_messages: Vec<ChatMessage> = openai_messages
        .clone()
        .into_iter()
        .enumerate()
        .map(|(index, message)| {
            if index == openai_messages.len() - 1 {
                ChatMessage {
                    role: message.role,
                    content: last_message.clone(),
                    name: message.name,
                    tool_calls: None,
                    tool_call_id: None,
                }
            } else {
                message
            }
        })
        .collect();

    if dataset_config.SYSTEM_PROMPT.is_some() {
        open_ai_messages.insert(
            0,
            ChatMessage {
                role: Role::System,
                content: ChatMessageContent::Text(dataset_config.SYSTEM_PROMPT.clone().unwrap()),
                tool_calls: None,
                name: None,
                tool_call_id: None,
            },
        )
    }

    let parameters = ChatCompletionParameters {
        model: chosen_model,
        messages: open_ai_messages,
        top_p: None,
        n: None,
        stream: Some(create_message_req_payload.stream_response.unwrap_or(true)),
        temperature: Some(create_message_req_payload.temperature.unwrap_or(0.5)),
        frequency_penalty: Some(create_message_req_payload.frequency_penalty.unwrap_or(0.7)),
        presence_penalty: Some(create_message_req_payload.presence_penalty.unwrap_or(0.7)),
        max_tokens: create_message_req_payload.max_tokens,
        stop: create_message_req_payload.stop_tokens.map(StopToken::Array),
        logit_bias: None,
        user: None,
        response_format: None,
        tools: None,
        tool_choice: None,
        logprobs: None,
        top_logprobs: None,
        seed: None,
    };

    if !chunk_metadatas_stringified.is_empty() {
        chunk_metadatas_stringified =
            if create_message_req_payload.completion_first.unwrap_or(false) {
                format!("||{}", chunk_metadatas_stringified.replace("||", ""))
            } else {
                format!("{}||", chunk_metadatas_stringified.replace("||", ""))
            };
        chunk_metadatas_stringified1.clone_from(&chunk_metadatas_stringified);
    }

    if !create_message_req_payload.stream_response.unwrap_or(true) {
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

        let completion_content = match &assistant_completion.choices[0].message.content {
            ChatMessageContent::Text(text) => text.clone(),
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
        );

        let clickhouse_rag_event = RagQueryEventClickhouse {
            id: uuid::Uuid::new_v4(),
            created_at: time::OffsetDateTime::now_utc(),
            dataset_id: dataset.id,
            search_id: uuid::Uuid::nil(),
            results: chunk_ids
                .clone()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            user_message: query.clone(),
            rag_type: "chosen_chunks".to_string(),
            llm_response: completion_content.clone(),
        };

        let _ = send_to_clickhouse(
            ClickHouseEvent::RagQueryEvent(clickhouse_rag_event),
            &clickhouse_client,
        )
        .await;

        create_message_query(new_message, &pool).await?;

        return Ok(HttpResponse::Ok().json(completion_content));
    }

    let (s, r) = unbounded::<String>();
    let stream = client.chat().create_stream(parameters).await.unwrap();

    Arbiter::new().spawn(async move {
        let chunk_v: Vec<String> = r.iter().collect();
        let completion = chunk_v.join("");

        let new_message = models::Message::from_details(
            format!("{}{}", chunk_metadatas_stringified, completion),
            topic_id,
            next_message_order().try_into().unwrap(),
            "assistant".to_string(),
            None,
            Some(chunk_v.len().try_into().unwrap()),
            dataset.id,
        );

        let clickhouse_rag_event = RagQueryEventClickhouse {
            id: uuid::Uuid::new_v4(),
            created_at: time::OffsetDateTime::now_utc(),
            dataset_id: dataset.id,
            search_id: clickhouse_search_event.id,
            results: chunk_ids
                .clone()
                .into_iter()
                .map(|s| s.to_string())
                .collect(),
            user_message: query.clone(),
            rag_type: "all_chunks".to_string(),
            llm_response: completion.clone(),
        };

        let _ = send_to_clickhouse(
            ClickHouseEvent::RagQueryEvent(clickhouse_rag_event),
            &clickhouse_client,
        )
        .await;

        let _ = create_message_query(new_message, &pool).await;
    });

    let chunk_stream = stream::iter(vec![Ok(Bytes::from(chunk_metadatas_stringified1))]);
    let completion_stream = stream.map(move |response| -> Result<Bytes, actix_web::Error> {
        if let Ok(response) = response {
            let chat_content = response.choices[0].delta.content.clone();
            if let Some(message) = chat_content.clone() {
                s.send(message).unwrap();
            }
            return Ok(Bytes::from(chat_content.unwrap_or("".to_string())));
        }
        Err(ServiceError::InternalServerError(
            "Model Response Error. Please try again later.".into(),
        )
        .into())
    });

    if create_message_req_payload.completion_first.unwrap_or(false) {
        return Ok(HttpResponse::Ok().streaming(completion_stream.chain(chunk_stream)));
    }

    Ok(HttpResponse::Ok().streaming(chunk_stream.chain(completion_stream)))
}

#[tracing::instrument]
pub async fn get_topic_string(
    model: String,
    first_message: String,
    dataset: &Dataset,
) -> Result<String, ServiceError> {
    let prompt_topic_message = ChatMessage {
        role: Role::User,
        content: ChatMessageContent::Text(format!(
            "Write a 2-3 word topic name from the following prompt: {}",
            first_message
        )),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    };
    let parameters = ChatCompletionParameters {
        model,
        messages: vec![prompt_topic_message],
        stream: Some(false),
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        max_tokens: None,
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
    };

    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());
    let base_url = dataset_config.LLM_BASE_URL;

    let base_url = if base_url.is_empty() {
        "https://openrouter.ai/api/v1".into()
    } else {
        base_url
    };

    let llm_api_key = if base_url.contains("openai.com") {
        get_env!("OPENAI_API_KEY", "OPENAI_API_KEY for openai should be set").into()
    } else {
        get_env!(
            "LLM_API_KEY",
            "LLM_API_KEY for openrouter or self-hosted should be set"
        )
        .into()
    };

    let client = Client {
        api_key: llm_api_key,
        http_client: reqwest::Client::new(),
        base_url,
        organization: None,
    };

    let query = client
        .chat()
        .create(parameters)
        .await
        .map_err(|_| ServiceError::BadRequest("No OpenAI Completion for topic".to_string()))?;

    let topic = match &query
        .choices
        .get(0)
        .ok_or(ServiceError::BadRequest(
            "No response for OpenAI completion".to_string(),
        ))?
        .message
        .content
    {
        ChatMessageContent::Text(topic) => topic.clone(),
        _ => "".to_string(),
    };

    Ok(topic)
}
