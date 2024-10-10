use super::{
    auth_handler::{AdminOnly, LoggedUser},
    chunk_handler::{ChunkFilter, ParsedQuery, ParsedQueryTypes, SearchChunksReqPayload},
};
use crate::{
    data::models::{
        self, ChunkMetadata, DatasetAndOrgWithSubAndPlan, DatasetConfiguration, HighlightOptions,
        LLMOptions, Pool, RedisPool, SearchMethod, SuggestType,
    },
    errors::ServiceError,
    get_env,
    operators::{
        chunk_operator::{get_chunk_metadatas_from_point_ids, get_random_chunk_metadatas_query},
        clickhouse_operator::EventQueue,
        message_operator::{
            create_topic_message_query, delete_message_query, get_message_by_sort_for_topic_query,
            get_messages_for_topic_query, get_topic_messages, stream_response,
        },
        organization_operator::get_message_org_count,
        parse_operator::convert_html_to_text,
        qdrant_operator::scroll_dataset_points,
        search_operator::{assemble_qdrant_filter, search_chunks_query, search_hybrid_chunks},
    },
};
use actix_web::{web, HttpResponse};

use itertools::Itertools;
use openai_dive::v1::{
    api::Client,
    resources::chat::{
        ChatCompletionChoice, ChatCompletionParameters, ChatMessage, ChatMessageContent,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use simple_server_timing_header::Timer;
use simsearch::SimSearch;
use utoipa::ToSchema;

pub fn check_completion_param_validity(
    llm_options: Option<LLMOptions>,
) -> Result<(), ServiceError> {
    if let Some(llm_options) = llm_options {
        if let Some(temperature) = llm_options.temperature {
            if !(0.0..=2.0).contains(&temperature) {
                return Err(ServiceError::BadRequest(
                    "Temperature must be between 0 and 2".to_string(),
                ));
            }
        }

        if let Some(frequency_penalty) = llm_options.frequency_penalty {
            if !(-2.0..=2.0).contains(&frequency_penalty) {
                return Err(ServiceError::BadRequest(
                    "Frequency penalty must be between -2.0 and 2.0".to_string(),
                ));
            }
        }

        if let Some(presence_penalty) = llm_options.presence_penalty {
            if !(-2.0..=2.0).contains(&presence_penalty) {
                return Err(ServiceError::BadRequest(
                    "Presence penalty must be between -2.0 and 2.0".to_string(),
                ));
            }
        }

        if let Some(stop_tokens) = llm_options.stop_tokens {
            if stop_tokens.len() > 4 {
                return Err(ServiceError::BadRequest(
                    "Stop tokens must be less than or equal to 4".to_string(),
                ));
            }
        }
    }

    Ok(())
}

#[derive(Serialize, Debug, ToSchema)]
pub struct CreateMessageReqPayload {
    /// The content of the user message to attach to the topic and then generate an assistant message in response to.
    pub new_message_content: String,
    /// The ID of the topic to attach the message to.
    pub topic_id: uuid::Uuid,
    /// The user_id is the id of the user who is making the request. This is used to track user interactions with the RAG results.
    pub user_id: Option<String>,
    /// Highlight Options lets you specify different methods to highlight the chunks in the result set. If not specified, this defaults to the score of the chunks.
    pub highlight_options: Option<HighlightOptions>,
    /// Search_type can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using scores from a cross encoder model. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE. Default is "hybrid".
    pub search_type: Option<SearchMethod>,
    /// If concat user messages query is set to true, all of the user messages in the topic will be concatenated together and used as the search query. If not specified, this defaults to false. Default is false.
    pub concat_user_messages_query: Option<bool>,
    /// Query is the search query. This can be any string. The search_query will be used to create a dense embedding vector and/or sparse vector which will be used to find the result set. If not specified, will default to the last user message or HyDE if HyDE is enabled in the dataset configuration. Default is None.
    pub search_query: Option<String>,
    /// Page size is the number of chunks to fetch during RAG. If 0, then no search will be performed. If specified, this will override the N retrievals to include in the dataset configuration. Default is None.
    pub page_size: Option<u64>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold. This threshold applies before weight and bias modifications. If not specified, this defaults to 0.0.
    pub score_threshold: Option<f32>,
    /// LLM options to use for the completion. If not specified, this defaults to the dataset's LLM options.
    pub llm_options: Option<LLMOptions>,
}

/// Create message
///
/// Create message. Messages are attached to topics in order to coordinate memory of gen-AI chat sessions.Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/message",
    context_path = "/api",
    tag = "Message",
    request_body(content = CreateMessageReqPayload, description = "JSON request payload to create a message completion", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream of a string, check the chat or search UI for an example how to process this. Response if streaming.",
            headers(
                ("TR-QueryID" = uuid::Uuid, description = "Query ID that is used for tracking analytics")
            )
        ),
        (status = 200, description = "This will be a JSON response of a string containing the LLM's generated inference. Response if not streaming.", body = String,
            headers(
                ("TR-QueryID" = uuid::Uuid, description = "Query ID that is used for tracking analytics")
            )
        ),
        (status = 400, description = "Service error relating to getting a chat completion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool, event_queue))]
pub async fn create_message(
    data: web::Json<CreateMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    event_queue: web::Data<EventQueue>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let message_count_pool = pool.clone();
    let message_count_org_id = dataset_org_plan_sub.organization.organization.id;
    let mut dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());

    check_completion_param_validity(data.llm_options.clone())?;

    let org_message_count = get_message_org_count(message_count_org_id, message_count_pool).await?;

    if org_message_count
        >= dataset_org_plan_sub
            .organization
            .plan
            .unwrap_or_default()
            .message_count
    {
        return Ok(HttpResponse::UpgradeRequired().json(json!({
            "message": "To create more message completions, you must upgrade your plan" })));
    }

    let create_message_data = data.into_inner();
    let get_messages_pool = pool.clone();
    let create_message_pool = pool.clone();
    let stream_response_pool = pool.clone();
    let topic_id = create_message_data.topic_id;
    if let Some(llm_options) = &create_message_data.llm_options {
        if let Some(data_system_prompt) = &llm_options.system_prompt {
            dataset_config.SYSTEM_PROMPT.clone_from(data_system_prompt);
        }
    }

    let new_message = models::Message::from_details(
        create_message_data.new_message_content.clone(),
        topic_id,
        0,
        "user".to_string(),
        None,
        None,
        dataset_org_plan_sub.dataset.id,
        uuid::Uuid::new_v4(),
    );

    // get the previous messages
    let mut previous_messages = get_topic_messages(
        topic_id,
        dataset_org_plan_sub.dataset.id,
        &get_messages_pool,
    )
    .await?;

    // remove chunks from the previous messages
    previous_messages = previous_messages
        .into_iter()
        .map(|message| {
            let mut message = message;
            if message.role == "assistant" {
                if message.content.starts_with("[{") {
                    // This is (chunks, content)
                    message.content = message
                        .content
                        .split("||")
                        .last()
                        .unwrap_or("I give up, I can't find a citation")
                        .to_string();
                } else {
                    // This is (content, chunks)
                    message.content = message
                        .content
                        .rsplit("||")
                        .last()
                        .unwrap_or("I give up, I can't find a citation")
                        .to_string();
                }
            }
            message
        })
        .collect::<Vec<models::Message>>();

    // call create_topic_message_query with the new_message and previous_messages
    let previous_messages = create_topic_message_query(
        &dataset_config,
        previous_messages,
        new_message,
        dataset_org_plan_sub.dataset.id,
        &create_message_pool,
    )
    .await?;

    stream_response(
        previous_messages,
        topic_id,
        dataset_org_plan_sub.dataset,
        stream_response_pool,
        event_queue,
        redis_pool,
        dataset_config,
        create_message_data,
    )
    .await
}

/// Get all messages for a given topic
///
/// Get all messages for a given topic. If the topic is a RAG topic then the response will include Chunks first on each message. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information.
#[utoipa::path(
    get,
    path = "/messages/{messages_topic_id}",
    context_path = "/api",
    tag = "Message",
    responses(
        (status = 200, description = "All messages relating to the topic with the given ID", body = Vec<Message>),
        (status = 400, description = "Service error relating to getting the messages", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("messages_topic_id" = uuid, description = "The ID of the topic to get messages for."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_all_topic_messages(
    user: LoggedUser,
    messages_topic_id: web::Path<uuid::Uuid>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id: uuid::Uuid = messages_topic_id.into_inner();

    let messages: Vec<models::Message> =
        get_messages_for_topic_query(topic_id, dataset_org_plan_sub.dataset.id, &pool)
            .await?
            .into_iter()
            .filter_map(|mut message| {
                if message.content.starts_with("||[{") {
                    match message.content.rsplit_once("}]") {
                        Some((chunks, ai_message)) => {
                            message.content = format!("{}{}}}]", ai_message, chunks);
                        }
                        _ => return None,
                    }
                }

                Some(message)
            })
            .collect();

    Ok(HttpResponse::Ok().json(messages))
}

#[derive(Serialize, Debug, ToSchema)]
pub struct RegenerateMessageReqPayload {
    /// The id of the topic to regenerate the last message for.
    pub topic_id: uuid::Uuid,
    /// Highlight Options lets you specify different methods to highlight the chunks in the result set. If not specified, this defaults to the score of the chunks.
    pub highlight_options: Option<HighlightOptions>,
    /// Search_type can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using scores from a cross encoder model. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE.
    pub search_type: Option<SearchMethod>,
    /// If concat user messages query is set to true, all of the user messages in the topic will be concatenated together and used as the search query. If not specified, this defaults to false. Default is false.
    pub concat_user_messages_query: Option<bool>,
    /// Query is the search query. This can be any string. The search_query will be used to create a dense embedding vector and/or sparse vector which will be used to find the result set. If not specified, will default to the last user message or HyDE if HyDE is enabled in the dataset configuration. Default is None.
    pub search_query: Option<String>,
    /// Page size is the number of chunks to fetch during RAG. If 0, then no search will be performed. If specified, this will override the N retrievals to include in the dataset configuration. Default is None.
    pub page_size: Option<u64>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold. This threshold applies before weight and bias modifications. If not specified, this defaults to 0.0.
    pub score_threshold: Option<f32>,
    /// LLM options to use for the completion. If not specified, this defaults to the dataset's LLM options.
    pub llm_options: Option<LLMOptions>,
    /// The user_id is the id of the user who is making the request. This is used to track user interactions with the RAG results.
    pub user_id: Option<String>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct EditMessageReqPayload {
    /// The id of the topic to edit the message at the given sort order for.
    pub topic_id: uuid::Uuid,
    /// The sort order of the message to edit.
    pub message_sort_order: i32,
    /// The new content of the message to replace the old content with.
    pub new_message_content: String,
    /// Highlight Options lets you specify different methods to highlight the chunks in the result set. If not specified, this defaults to the score of the chunks.
    pub highlight_options: Option<HighlightOptions>,
    /// Search_type can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using scores from a cross encoder model. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE.
    pub search_type: Option<SearchMethod>,
    /// If concat user messages query is set to true, all of the user messages in the topic will be concatenated together and used as the search query. If not specified, this defaults to false. Default is false.
    pub concat_user_messages_query: Option<bool>,
    /// Query is the search query. This can be any string. The search_query will be used to create a dense embedding vector and/or sparse vector which will be used to find the result set. If not specified, will default to the last user message or HyDE if HyDE is enabled in the dataset configuration. Default is None.
    pub search_query: Option<String>,
    /// Page size is the number of chunks to fetch during RAG. If 0, then no search will be performed. If specified, this will override the N retrievals to include in the dataset configuration. Default is None.
    pub page_size: Option<u64>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold. This threshold applies before weight and bias modifications. If not specified, this defaults to 0.0.
    pub score_threshold: Option<f32>,
    /// LLM options to use for the completion. If not specified, this defaults to the dataset's LLM options.
    pub llm_options: Option<LLMOptions>,
    /// The user_id is the id of the user who is making the request. This is used to track user interactions with the RAG results.
    pub user_id: Option<String>,
}

impl From<EditMessageReqPayload> for CreateMessageReqPayload {
    fn from(data: EditMessageReqPayload) -> Self {
        CreateMessageReqPayload {
            new_message_content: data.new_message_content,
            topic_id: data.topic_id,
            highlight_options: data.highlight_options,
            search_type: data.search_type,
            concat_user_messages_query: data.concat_user_messages_query,
            search_query: data.search_query,
            page_size: data.page_size,
            filters: data.filters,
            score_threshold: data.score_threshold,
            llm_options: data.llm_options,
            user_id: data.user_id,
        }
    }
}

impl From<RegenerateMessageReqPayload> for CreateMessageReqPayload {
    fn from(data: RegenerateMessageReqPayload) -> Self {
        CreateMessageReqPayload {
            new_message_content: "".to_string(),
            topic_id: data.topic_id,
            highlight_options: data.highlight_options,
            search_type: data.search_type,
            concat_user_messages_query: data.concat_user_messages_query,
            search_query: data.search_query,
            page_size: data.page_size,
            filters: data.filters,
            score_threshold: data.score_threshold,
            llm_options: data.llm_options,
            user_id: data.user_id,
        }
    }
}

/// Edit message
///
/// Edit message which exists within the topic's chat history. This will delete the message and replace it with a new message. The new message will be generated by the AI based on the new content provided in the request body. The response will include Chunks first on the stream if the topic is using RAG. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    put,
    path = "/message",
    context_path = "/api",
    tag = "Message",
    request_body(content = EditMessageReqPayload, description = "JSON request payload to edit a message and get a new stream", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream, check the chat or search UI for an example how to process this",
            headers(
                ("TR-QueryID" = uuid::Uuid, description = "Query ID that is used for tracking analytics")
            )
        ),
        (status = 400, description = "Service error relating to getting a chat completion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool, event_queue, redis_pool))]
pub async fn edit_message(
    data: web::Json<EditMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    event_queue: web::Data<EventQueue>,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id: uuid::Uuid = data.topic_id;
    let message_sort_order = data.message_sort_order;

    check_completion_param_validity(data.llm_options.clone())?;

    let second_pool = pool.clone();
    let third_pool = pool.clone();

    let message_id = get_message_by_sort_for_topic_query(
        topic_id,
        dataset_org_plan_sub.dataset.id,
        message_sort_order,
        &pool,
    )
    .await?
    .id;

    delete_message_query(
        message_id,
        topic_id,
        dataset_org_plan_sub.dataset.id,
        &second_pool,
    )
    .await?;

    create_message(
        actix_web::web::Json(data.into_inner().into()),
        user,
        dataset_org_plan_sub,
        event_queue,
        third_pool,
        redis_pool,
    )
    .await
}

/// Regenerate message
///
/// Regenerate the assistant response to the last user message of a topic. This will delete the last message and replace it with a new message. The response will include Chunks first on the stream if the topic is using RAG. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    patch,
    path = "/message",
    context_path = "/api",
    tag = "Message",
    request_body(content = RegenerateMessageReqPayload, description = "JSON request payload to delete an agent message then regenerate it in a strem", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream of a string, check the chat or search UI for an example how to process this. Response if streaming.",
            headers(
                ("TR-QueryID" = uuid::Uuid, description = "Query ID that is used for tracking analytics")
            )
        ),
        (status = 200, description = "This will be a JSON response of a string containing the LLM's generated inference. Response if not streaming.", body = String,
            headers(
                ("TR-QueryID" = uuid::Uuid, description = "Query ID that is used for tracking analytics")
            )
        ),
        (status = 400, description = "Service error relating to getting a chat completion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool, event_queue, redis_pool))]
pub async fn regenerate_message_patch(
    data: web::Json<RegenerateMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    event_queue: web::Data<EventQueue>,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = data.topic_id;
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());

    check_completion_param_validity(data.llm_options.clone())?;

    let get_messages_pool = pool.clone();
    let create_message_pool = pool.clone();
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let mut previous_messages =
        get_topic_messages(topic_id, dataset_id, &get_messages_pool).await?;

    if previous_messages.len() < 2 {
        return Err(
            ServiceError::BadRequest("Not enough messages to regenerate".to_string()).into(),
        );
    }

    if previous_messages.len() == 2 {
        return stream_response(
            previous_messages,
            topic_id,
            dataset_org_plan_sub.dataset,
            create_message_pool,
            event_queue,
            redis_pool.clone(),
            dataset_config,
            data.into_inner().into(),
        )
        .await;
    }

    // remove citations from the previous messages
    previous_messages = previous_messages
        .into_iter()
        .map(|message| {
            let mut message = message;
            if message.role == "assistant" {
                if message.content.starts_with("||[{") {
                    match message.content.rsplit_once("}]") {
                        Some((_, ai_message)) => {
                            message.content = ai_message.to_string();
                        }
                        _ => return message,
                    }
                } else if message.content.starts_with("[{") {
                    // This is (chunks, content)
                    message.content = message
                        .content
                        .split("||")
                        .last()
                        .unwrap_or("I give up, I can't find a citation")
                        .to_string();
                } else {
                    // This is (content, chunks)
                    message.content = message
                        .content
                        .rsplit("||")
                        .last()
                        .unwrap_or("I give up, I can't find a citation")
                        .to_string();
                }
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
            return Err(ServiceError::BadRequest("No message to regenerate".to_string()).into());
        }
    };

    let mut previous_messages_to_regenerate = Vec::new();
    for message in previous_messages.iter() {
        if message.id == message_id {
            break;
        }
        previous_messages_to_regenerate.push(message.clone());
    }

    delete_message_query(message_id, topic_id, dataset_id, &pool).await?;

    stream_response(
        previous_messages_to_regenerate,
        topic_id,
        dataset_org_plan_sub.dataset,
        create_message_pool,
        event_queue,
        redis_pool.clone(),
        dataset_config,
        data.into_inner().into(),
    )
    .await
}

/// Regenerate message
///
/// Regenerate the assistant response to the last user message of a topic. This will delete the last message and replace it with a new message. The response will include Chunks first on the stream if the topic is using RAG. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    delete,
    path = "/message",
    context_path = "/api",
    tag = "Message",
    request_body(content = RegenerateMessageReqPayload, description = "JSON request payload to delete an agent message then regenerate it in a strem", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream of a string, check the chat or search UI for an example how to process this. Response if streaming.",
            headers(
                ("TR-QueryID" = uuid::Uuid, description = "Query ID that is used for tracking analytics")
            )
        ),
        (status = 200, description = "This will be a JSON response of a string containing the LLM's generated inference. Response if not streaming.", body = String,
            headers(
                ("TR-QueryID" = uuid::Uuid, description = "Query ID that is used for tracking analytics")
            )
        ),
        (status = 400, description = "Service error relating to getting a chat completion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[deprecated]
#[tracing::instrument(skip(pool, event_queue, redis_pool))]
pub async fn regenerate_message(
    data: web::Json<RegenerateMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    event_queue: web::Data<EventQueue>,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    regenerate_message_patch(
        data,
        user,
        dataset_org_plan_sub,
        pool,
        event_queue,
        redis_pool,
    )
    .await
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct SuggestedQueriesReqPayload {
    /// The query to base the generated suggested queries off of using RAG. A hybrid search for 10 chunks from your dataset using this query will be performed and the context of the chunks will be used to generate the suggested queries.
    pub query: Option<String>,
    /// Can be either "semantic", "fulltext", "hybrid, or "bm25". If specified as "hybrid", it will pull in one page of both semantic and full-text results then re-rank them using scores from a cross encoder model. "semantic" will pull in one page of the nearest cosine distant vectors. "fulltext" will pull in one page of full-text results based on SPLADE. "bm25" will get one page of results scored using BM25 with the terms OR'd together.
    pub search_type: Option<SearchMethod>,
    /// Type of suggestions. Can be "question", "keyword", or "semantic". If not specified, this defaults to "keyword".
    pub suggestion_type: Option<SuggestType>,
    /// Context is the context of the query. This can be any string under 15 words and 200 characters. The context will be used to generate the suggested queries. Defaults to None.
    pub context: Option<String>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct SuggestedQueriesResponse {
    pub queries: Vec<String>,
}

/// Generate suggested queries
///
/// This endpoint will generate 3 suggested queries based off a hybrid search using RAG with the query provided in the request body and return them as a JSON object.
#[utoipa::path(
    post,
    path = "/chunk/suggestions",
    context_path = "/api",
    tag = "Chunk",
    request_body(content = SuggestedQueriesReqPayload, description = "JSON request payload to get alternative suggested queries", content_type = "application/json"),
    responses(
        (status = 200, description = "A JSON object containing a list of alternative suggested queries", body = SuggestedQueriesResponse),
        (status = 400, description = "Service error relating to to updating chunk, likely due to conflicting tracking_id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )

)]
#[tracing::instrument(skip(pool))]
pub async fn get_suggested_queries(
    data: web::Json<SuggestedQueriesReqPayload>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    _required_user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.clone().server_configuration);

    let base_url = dataset_config.LLM_BASE_URL.clone();
    let default_model = dataset_config.LLM_DEFAULT_MODEL.clone();

    let base_url = if base_url.is_empty() {
        "https://api.openai.com/api/v1".into()
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
    let search_type = data.search_type.clone().unwrap_or(SearchMethod::Hybrid);
    let filters = data.filters.clone();

    let chunk_metadatas = match data.query.clone() {
        Some(query) => {
            let search_req_payload = SearchChunksReqPayload {
                search_type: search_type.clone(),
                query: models::QueryTypes::Single(query.clone()),
                page_size: Some(10),
                filters,
                ..Default::default()
            };
            let parsed_query = ParsedQuery {
                query,
                quote_words: None,
                negated_words: None,
            };
            match search_type {
                SearchMethod::Hybrid => search_hybrid_chunks(
                    search_req_payload,
                    parsed_query,
                    pool,
                    redis_pool,
                    dataset_org_plan_sub.dataset.clone(),
                    &dataset_config,
                    &mut Timer::new(),
                )
                .await
                .map_err(|err| ServiceError::BadRequest(err.to_string()))?,
                _ => search_chunks_query(
                    search_req_payload,
                    ParsedQueryTypes::Single(parsed_query),
                    pool,
                    redis_pool,
                    dataset_org_plan_sub.dataset.clone(),
                    &dataset_config,
                    &mut Timer::new(),
                )
                .await
                .map_err(|err| ServiceError::BadRequest(err.to_string()))?,
            }
            .score_chunks
            .into_iter()
            .filter_map(|chunk| chunk.metadata.clone().first().cloned())
            .map(ChunkMetadata::from)
            .collect::<Vec<ChunkMetadata>>()
        }
        None => {
            let random_chunk = get_random_chunk_metadatas_query(dataset_id, 1, pool.clone())
                .await?
                .clone()
                .first()
                .cloned();
            match random_chunk {
                Some(chunk) => {
                    let filter =
                        assemble_qdrant_filter(filters, None, None, dataset_id, pool.clone())
                            .await?;

                    let qdrant_point_ids = scroll_dataset_points(
                        10,
                        Some(chunk.qdrant_point_id),
                        None,
                        dataset_config,
                        filter,
                    )
                    .await?;

                    get_chunk_metadatas_from_point_ids(qdrant_point_ids.clone(), pool)
                        .await?
                        .into_iter()
                        .map(ChunkMetadata::from)
                        .collect()
                }
                None => vec![],
            }
        }
    };

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

    let query_style = match data.suggestion_type.clone().unwrap_or(SuggestType::Keyword) {
        SuggestType::Question => "question",
        SuggestType::Keyword => "keyword",
        SuggestType::Semantic => "semantic while not question",
    };
    let context_sentence = match data.context.clone() {
        Some(context) => {
            if context.split_whitespace().count() > 15 || context.len() > 200 {
                return Err(ServiceError::BadRequest(
                    "Context must be under 15 words and 200 characters".to_string(),
                ));
            }

            format!(
                "\n\nSuggest things with the following context in mind: {}.\n\n",
                context
            )
        }
        None => "".to_string(),
    };

    let content = ChatMessageContent::Text(format!(
        "Here is some context for the dataset for which the user is querying for {}{}. Generate 10 suggested followup {} style queries based off the domain of this dataset. Your only response should be the 10 followup {} style queries which are separated by new lines and are just text and you do not add any other context or information about the followup {} style queries. This should not be a list, so do not number each {} style queries. These followup {} style queries should be related to the domain of the dataset.",
        rag_content,
        context_sentence,
        query_style,
        query_style,
        query_style,
        query_style,
        query_style
    ));

    let message = ChatMessage::User {
        content,
        name: None,
    };

    let parameters = ChatCompletionParameters {
        model: default_model,
        messages: vec![message],
        stream: Some(false),
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        max_completion_tokens: None,
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

    let client = Client {
        headers: None,
        project: None,
        api_key: llm_api_key,
        http_client: reqwest::Client::new(),
        base_url,
        organization: None,
    };

    let mut query = client
        .chat()
        .create(parameters.clone())
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let mut queries: Vec<String> = match &query
        .choices
        .first()
        .unwrap_or(&ChatCompletionChoice {
            logprobs: None,
            index: 0,
            message: ChatMessage::User {
                content: ChatMessageContent::Text("".to_string()),
                name: None,
            },
            finish_reason: None,
        })
        .message
    {
        ChatMessage::User {
            content: ChatMessageContent::Text(content),
            ..
        }
        | ChatMessage::System {
            content: ChatMessageContent::Text(content),
            ..
        }
        | ChatMessage::Assistant {
            content: Some(ChatMessageContent::Text(content)),
            ..
        } => content.clone(),
        _ => "".to_string(),
    }
    .split('\n')
    .filter_map(|query| {
        let cleaned_query = query.to_string().trim().trim_matches('\n').to_string();
        if cleaned_query.is_empty() {
            None
        } else {
            Some(cleaned_query)
        }
    })
    .map(|query| query.to_string().trim().trim_matches('\n').to_string())
    .collect();

    while queries.len() < 3 {
        query = client
            .chat()
            .create(parameters.clone())
            .await
            .expect("No LLM Completion for topic");
        queries = match &query
            .choices
            .first()
            .expect("No response for LLM completion")
            .message
        {
            ChatMessage::User {
                content: ChatMessageContent::Text(content),
                ..
            }
            | ChatMessage::System {
                content: ChatMessageContent::Text(content),
                ..
            }
            | ChatMessage::Assistant {
                content: Some(ChatMessageContent::Text(content)),
                ..
            } => content.clone(),
            _ => "".to_string(),
        }
        .split('\n')
        .map(|query| query.to_string().trim().trim_matches('\n').to_string())
        .collect();
    }

    let mut engine: SimSearch<String> = SimSearch::new();

    chunk_metadatas.iter().for_each(|chunk| {
        let content = convert_html_to_text(&chunk.chunk_html.clone().unwrap_or_default());

        engine.insert(content.clone(), &content);
    });

    let sortable_queries = queries
        .iter()
        .map(|query| (query, engine.search(query).len()))
        .collect_vec();

    //search for the query
    queries = sortable_queries
        .iter()
        .sorted_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .map(|(content, _length)| content)
        .cloned()
        .cloned()
        .collect_vec();

    Ok(HttpResponse::Ok().json(SuggestedQueriesResponse { queries }))
}
