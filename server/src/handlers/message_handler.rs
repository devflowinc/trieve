use super::{
    auth_handler::{AdminOnly, LoggedUser},
    chunk_handler::{ChunkFilter, ParsedQuery, SearchChunksReqPayload},
};
use crate::{
    data::models::{
        self, ChunkMetadataStringTagSet, ChunkMetadataTypes, Dataset, DatasetAndOrgWithSubAndPlan,
        Pool, ServerDatasetConfiguration,
    },
    errors::ServiceError,
    get_env,
    operators::{
        analytics_operator::SearchQueryEvent,
        message_operator::{
            create_message_query, create_topic_message_query, delete_message_query,
            get_message_by_sort_for_topic_query, get_messages_for_topic_query, get_topic_messages,
        },
        organization_operator::get_message_org_count,
        parse_operator::convert_html_to_text,
        search_operator::search_hybrid_chunks,
    },
};
use actix::Arbiter;
use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};
use crossbeam_channel::unbounded;
use futures_util::stream;
use itertools::Itertools;
use openai_dive::v1::{
    api::Client,
    resources::{
        chat::{
            ChatCompletionChoice, ChatCompletionParameters, ChatMessage, ChatMessageContent, Role,
        },
        shared::StopToken,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use simple_server_timing_header::Timer;
use simsearch::SimSearch;
use tokio_stream::StreamExt;
use utoipa::ToSchema;

pub fn check_completion_param_validity(
    temperature: Option<f32>,
    frequency_penalty: Option<f32>,
    presence_penalty: Option<f32>,
    stop_tokens: Option<Vec<String>>,
) -> Result<(), ServiceError> {
    if let Some(temperature) = temperature {
        if !(0.0..=2.0).contains(&temperature) {
            return Err(ServiceError::BadRequest(
                "Temperature must be between 0 and 2".to_string(),
            ));
        }
    }

    if let Some(frequency_penalty) = frequency_penalty {
        if !(-2.0..=2.0).contains(&frequency_penalty) {
            return Err(ServiceError::BadRequest(
                "Frequency penalty must be between -2.0 and 2.0".to_string(),
            ));
        }
    }

    if let Some(presence_penalty) = presence_penalty {
        if !(-2.0..=2.0).contains(&presence_penalty) {
            return Err(ServiceError::BadRequest(
                "Presence penalty must be between -2.0 and 2.0".to_string(),
            ));
        }
    }

    if let Some(stop_tokens) = stop_tokens {
        if stop_tokens.len() > 4 {
            return Err(ServiceError::BadRequest(
                "Stop tokens must be less than or equal to 4".to_string(),
            ));
        }
    }

    Ok(())
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct CreateMessageReqPayload {
    /// The content of the user message to attach to the topic and then generate an assistant message in response to.
    pub new_message_content: String,
    /// The ID of the topic to attach the message to.
    pub topic_id: uuid::Uuid,
    /// Set highlight_results to false for a slight latency improvement (1-10ms). If not specified, this defaults to true. This will add `<b><mark>` tags to the chunk_html of the chunks to highlight matching splits and return the highlights on each scored chunk in the response.
    pub highlight_results: Option<bool>,
    /// The delimiters to use for highlighting the citations. If this is not included, the default delimiters will be used. Default is `[".", "!", "?", "\n", "\t", ","]`.
    pub highlight_delimiters: Option<Vec<String>>,
    /// Search_type can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using BAAI/bge-reranker-large. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE. Default is "hybrid".
    pub search_type: Option<String>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Completion first decides whether the stream should contain the stream of the completion response or the chunks first. Default is false. Keep in mind that || is used to separate the chunks from the completion response. If || is in the completion then you may want to split on ||{ instead.
    pub completion_first: Option<bool>,
    /// Whether or not to stream the response. If this is set to true or not included, the response will be a stream. If this is set to false, the response will be a normal JSON response. Default is true.
    pub stream_response: Option<bool>,
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. Default is 0.5.
    pub temperature: Option<f32>,
    /// Frequency penalty is a number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim. Default is 0.7.
    pub frequency_penalty: Option<f32>,
    /// Presence penalty is a number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics. Default is 0.7.
    pub presence_penalty: Option<f32>,
    /// The maximum number of tokens to generate in the chat completion. Default is None.
    pub max_tokens: Option<u32>,
    /// Stop tokens are up to 4 sequences where the API will stop generating further tokens. Default is None.
    pub stop_tokens: Option<Vec<String>>,
}

/// Create message
///
/// Create message. Messages are attached to topics in order to coordinate memory of gen-AI chat sessions. We are considering refactoring this resource of the API soon. Currently, you can only send user messages. If the topic is a RAG topic then the response will include Chunks first on the stream. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/message",
    context_path = "/api",
    tag = "message",
    request_body(content = CreateMessageReqPayload, description = "JSON request payload to create a message completion", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream of a string, check the chat or search UI for an example how to process this. Response if streaming.",),
        (status = 200, description = "This will be a JSON response of a string containing the LLM's generated inference. Response if not streaming.", body = String),
        (status = 400, description = "Service error relating to getting a chat completion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn create_message(
    data: web::Json<CreateMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let message_count_pool = pool.clone();
    let message_count_org_id = dataset_org_plan_sub.organization.organization.id;
    let server_dataset_configuration = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    check_completion_param_validity(
        data.temperature,
        data.frequency_penalty,
        data.presence_penalty,
        data.stop_tokens.clone(),
    )?;

    let org_message_count = get_message_org_count(message_count_org_id, message_count_pool).await?;

    if org_message_count
        >= dataset_org_plan_sub
            .organization
            .plan
            .unwrap_or_default()
            .message_count
    {
        return Ok(HttpResponse::UpgradeRequired().json(json!({
            "message": "To create more message completions, you must upgrade your plan"
        })));
    }

    let create_message_data = data.into_inner();
    let get_messages_pool = pool.clone();
    let create_message_pool = pool.clone();
    let stream_response_pool = pool.clone();
    let topic_id = create_message_data.topic_id;

    let new_message = models::Message::from_details(
        create_message_data.new_message_content,
        topic_id,
        0,
        "user".to_string(),
        None,
        None,
        dataset_org_plan_sub.dataset.id,
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
                message.content = message
                    .content
                    .split("||")
                    .last()
                    .unwrap_or("I give up, I can't find chunks for this message")
                    .to_string();
            }
            message
        })
        .collect::<Vec<models::Message>>();

    // call create_topic_message_query with the new_message and previous_messages
    let previous_messages = create_topic_message_query(
        previous_messages,
        new_message,
        dataset_org_plan_sub.dataset.id,
        &create_message_pool,
    )
    .await?;

    stream_response(
        previous_messages,
        topic_id,
        create_message_data.stream_response,
        create_message_data.highlight_results,
        create_message_data.highlight_delimiters,
        create_message_data.search_type,
        create_message_data.filters,
        dataset_org_plan_sub.dataset,
        stream_response_pool,
        server_dataset_configuration,
        create_message_data.completion_first,
        create_message_data.temperature,
        create_message_data.frequency_penalty,
        create_message_data.presence_penalty,
        create_message_data.max_tokens,
        create_message_data.stop_tokens,
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
    tag = "message",
    responses(
        (status = 200, description = "All messages relating to the topic with the given ID", body = Vec<Message>),
        (status = 400, description = "Service error relating to getting the messages", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
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

    let messages =
        get_messages_for_topic_query(topic_id, dataset_org_plan_sub.dataset.id, &pool).await?;

    Ok(HttpResponse::Ok().json(messages))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct RegenerateMessageReqPayload {
    /// The id of the topic to regenerate the last message for.
    pub topic_id: uuid::Uuid,
    /// Whether or not to highlight the citations in the response. If this is set to true or not included, the citations will be highlighted. If this is set to false, the citations will not be highlighted. Default is true.
    pub highlight_citations: Option<bool>,
    /// The delimiters to use for highlighting the citations. If this is not included, the default delimiters will be used. Default is `[".", "!", "?", "\n", "\t", ","]`.  
    pub highlight_delimiters: Option<Vec<String>>,
    /// Search_type can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using BAAI/bge-reranker-large. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE.
    pub search_type: Option<String>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Completion first decides whether the stream should contain the stream of the completion response or the chunks first. Default is false. Keep in mind that || is used to separate the chunks from the completion response. If || is in the completion then you may want to split on ||{ instead.
    pub completion_first: Option<bool>,
    /// Whether or not to stream the response. If this is set to true or not included, the response will be a stream. If this is set to false, the response will be a normal JSON response. Default is true.
    pub stream_response: Option<bool>,
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. Default is 0.7.
    pub temperature: Option<f32>,
    /// Frequency penalty is a number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim. Default is 0.7.
    pub frequency_penalty: Option<f32>,
    /// Presence penalty is a number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
    pub presence_penalty: Option<f32>,
    /// The maximum number of tokens to generate in the chat completion.
    pub max_tokens: Option<u32>,
    /// Stop tokens are up to 4 sequences where the API will stop generating further tokens.
    pub stop_tokens: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct EditMessageReqPayload {
    /// The id of the topic to edit the message at the given sort order for.
    pub topic_id: uuid::Uuid,
    /// The sort order of the message to edit.
    pub message_sort_order: i32,
    /// The new content of the message to replace the old content with.
    pub new_message_content: String,
    /// Whether or not to highlight the citations in the response. If this is set to true or not included, the citations will be highlighted. If this is set to false, the citations will not be highlighted. Default is true.
    pub highlight_citations: Option<bool>,
    /// The delimiters to use for highlighting the citations. If this is not included, the default delimiters will be used. Default is `[".", "!", "?", "\n", "\t", ","]`.
    pub highlight_delimiters: Option<Vec<String>>,
    /// Search_type can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using BAAI/bge-reranker-large. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE.
    pub search_type: Option<String>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Completion first decides whether the stream should contain the stream of the completion response or the chunks first. Default is false. Keep in mind that || is used to separate the chunks from the completion response. If || is in the completion then you may want to split on ||{ instead.
    pub completion_first: Option<bool>,
    /// Whether or not to stream the response. If this is set to true or not included, the response will be a stream. If this is set to false, the response will be a normal JSON response. Default is true.
    pub stream_response: Option<bool>,
    /// What sampling temperature to use, between 0 and 2. Higher values like 0.8 will make the output more random, while lower values like 0.2 will make it more focused and deterministic. Default is 0.7.
    pub temperature: Option<f32>,
    /// Frequency penalty is a number between -2.0 and 2.0. Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim. Default is 0.7.
    pub frequency_penalty: Option<f32>,
    /// Presence penalty is a number between -2.0 and 2.0. Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.
    pub presence_penalty: Option<f32>,
    /// The maximum number of tokens to generate in the chat completion.
    pub max_tokens: Option<u32>,
    /// Stop tokens are up to 4 sequences where the API will stop generating further tokens.
    pub stop_tokens: Option<Vec<String>>,
}

/// Edit message
///
/// Edit message which exists within the topic's chat history. This will delete the message and replace it with a new message. The new message will be generated by the AI based on the new content provided in the request body. The response will include Chunks first on the stream if the topic is using RAG. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    put,
    path = "/message",
    context_path = "/api",
    tag = "message",
    request_body(content = EditMessageReqPayload, description = "JSON request payload to edit a message and get a new stream", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream, check the chat or search UI for an example how to process this"),
        (status = 400, description = "Service error relating to getting a chat completion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn edit_message(
    data: web::Json<EditMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id: uuid::Uuid = data.topic_id;
    let stream_response = data.stream_response;
    let message_sort_order = data.message_sort_order;
    let new_message_content = &data.new_message_content;

    check_completion_param_validity(
        data.temperature,
        data.frequency_penalty,
        data.presence_penalty,
        data.stop_tokens.clone(),
    )?;

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
        actix_web::web::Json(CreateMessageReqPayload {
            new_message_content: new_message_content.to_string(),
            topic_id,
            stream_response,
            highlight_results: data.highlight_citations,
            highlight_delimiters: data.highlight_delimiters.clone(),
            search_type: data.search_type.clone(),
            filters: data.filters.clone(),
            completion_first: data.completion_first,
            temperature: data.temperature,
            frequency_penalty: data.frequency_penalty,
            presence_penalty: data.presence_penalty,
            max_tokens: data.max_tokens,
            stop_tokens: data.stop_tokens.clone(),
        }),
        user,
        dataset_org_plan_sub,
        third_pool,
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
    tag = "message",
    request_body(content = RegenerateMessageReqPayload, description = "JSON request payload to delete an agent message then regenerate it in a strem", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream of a string, check the chat or search UI for an example how to process this. Response if streaming.",),
        (status = 200, description = "This will be a JSON response of a string containing the LLM's generated inference. Response if not streaming.", body = String),
        (status = 400, description = "Service error relating to getting a chat completion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn regenerate_message(
    data: web::Json<RegenerateMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = data.topic_id;
    let should_stream = data.stream_response;
    let server_dataset_configuration = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    check_completion_param_validity(
        data.temperature,
        data.frequency_penalty,
        data.presence_penalty,
        data.stop_tokens.clone(),
    )?;

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
            should_stream,
            data.highlight_citations,
            data.highlight_delimiters.clone(),
            data.search_type.clone(),
            data.filters.clone(),
            dataset_org_plan_sub.dataset,
            create_message_pool,
            server_dataset_configuration,
            data.completion_first,
            data.temperature,
            data.frequency_penalty,
            data.presence_penalty,
            data.max_tokens,
            data.stop_tokens.clone(),
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

    let _ = delete_message_query(message_id, topic_id, dataset_id, &pool).await;

    stream_response(
        previous_messages_to_regenerate,
        topic_id,
        should_stream,
        data.highlight_citations,
        data.highlight_delimiters.clone(),
        data.search_type.clone(),
        data.filters.clone(),
        dataset_org_plan_sub.dataset,
        create_message_pool,
        server_dataset_configuration,
        data.completion_first,
        data.temperature,
        data.frequency_penalty,
        data.presence_penalty,
        data.max_tokens,
        data.stop_tokens.clone(),
    )
    .await
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
        .first()
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

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool))]
pub async fn stream_response(
    messages: Vec<models::Message>,
    topic_id: uuid::Uuid,
    should_stream: Option<bool>,
    highlight_results: Option<bool>,
    highlight_delimiters: Option<Vec<String>>,
    search_type: Option<String>,
    filters: Option<ChunkFilter>,
    dataset: Dataset,
    pool: web::Data<Pool>,
    config: ServerDatasetConfiguration,
    completion_first: Option<bool>,
    temperature: Option<f32>,
    frequency_penalty: Option<f32>,
    presence_penalty: Option<f32>,
    max_tokens: Option<u32>,
    stop_tokens: Option<Vec<String>>,
) -> Result<HttpResponse, actix_web::Error> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());

    let user_message = match messages.last() {
        Some(message) => message.clone().content,
        None => {
            return Err(
                ServiceError::BadRequest("No messages found for the topic".to_string()).into(),
            );
        }
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

    let mut query = user_message;
    let use_message_to_query_prompt = dataset_config.USE_MESSAGE_TO_QUERY_PROMPT;
    if use_message_to_query_prompt {
        let message_to_query_prompt = dataset_config.MESSAGE_TO_QUERY_PROMPT.clone();

        let gen_inference_parameters = ChatCompletionParameters {
            model: chosen_model.clone(),
            messages: vec![ChatMessage {
                role: Role::User,
                content: ChatMessageContent::Text(format!(
                    "{}{}",
                    message_to_query_prompt,
                    match openai_messages
                        .clone()
                        .last()
                        .expect("No messages")
                        .clone()
                        .content
                    {
                        ChatMessageContent::Text(text) => text,
                        _ => "".to_string(),
                    }
                )),
                tool_calls: None,
                name: None,
                tool_call_id: None,
            }],
            stream: Some(false),
            temperature: None,
            frequency_penalty: Some(0.8),
            presence_penalty: Some(0.8),
            stop: None,
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
            .first()
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
        search_type: search_type.unwrap_or("hybrid".to_string()),
        query: query.clone(),
        page_size: Some(n_retrievals_to_include.try_into().unwrap_or(8)),
        highlight_results,
        highlight_delimiters,
        filters,
        ..Default::default()
    };
    let parsed_query = ParsedQuery {
        query: query.clone(),
        quote_words: None,
        negated_words: None,
    };
    let mut search_timer = Timer::new();
    //TODO: actually do the clickhouse event here
    let result_chunks = search_hybrid_chunks(
        search_chunk_data,
        parsed_query,
        &mut SearchQueryEvent {
            id: uuid::Uuid::new_v4(),
            search_type: String::from(""),
            query: String::from(""),
            request_params: String::from(""),
            query_vector: vec![],
            latency: 0.0,
            results: vec![],
            dataset_id: uuid::Uuid::new_v4(),
            created_at: time::OffsetDateTime::now_utc(),
        },
        pool.clone(),
        dataset.clone(),
        &dataset_config,
        &mut search_timer,
    )
    .await?;

    let chunk_metadatas = result_chunks
        .score_chunks
        .iter()
        .map(
            |score_chunk| match score_chunk.metadata.first().expect("No metadata found") {
                ChunkMetadataTypes::Metadata(chunk_metadata) => chunk_metadata.clone(),
                _ => unreachable!("The operator should never return slim chunks for this"),
            },
        )
        .collect::<Vec<ChunkMetadataStringTagSet>>();

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
                    tool_calls: None,
                    tool_call_id: None,
                }
            } else {
                message
            }
        })
        .collect();

    let parameters = ChatCompletionParameters {
        model: chosen_model,
        messages: open_ai_messages,
        top_p: None,
        n: None,
        stream: Some(should_stream.unwrap_or(true)),
        temperature: Some(temperature.unwrap_or(0.5)),
        frequency_penalty: Some(frequency_penalty.unwrap_or(0.7)),
        presence_penalty: Some(presence_penalty.unwrap_or(0.7)),
        max_tokens,
        stop: stop_tokens.map(StopToken::Array),
        logit_bias: None,
        user: None,
        response_format: None,
        tools: None,
        tool_choice: None,
        logprobs: None,
        top_logprobs: None,
        seed: None,
    };

    if !should_stream.unwrap_or(true) {
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

        let _ = create_message_query(new_message, &pool).await;

        return Ok(HttpResponse::Ok().json(completion_content));
    }

    let (s, r) = unbounded::<String>();
    let stream = client.chat().create_stream(parameters).await.unwrap();

    if !chunk_metadatas_stringified.is_empty() {
        chunk_metadatas_stringified = if completion_first.unwrap_or(false) {
            format!("||{}", chunk_metadatas_stringified.replace("||", ""))
        } else {
            format!("{}||", chunk_metadatas_stringified.replace("||", ""))
        };
        chunk_metadatas_stringified1.clone_from(&chunk_metadatas_stringified);
    }

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

    if completion_first.unwrap_or(false) {
        return Ok(HttpResponse::Ok().streaming(completion_stream.chain(chunk_stream)));
    }

    Ok(HttpResponse::Ok().streaming(chunk_stream.chain(completion_stream)))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct SuggestedQueriesReqPayload {
    /// The query to base the generated suggested queries off of using RAG. A hybrid search for 10 chunks from your dataset using this query will be performed and the context of the chunks will be used to generate the suggested queries.
    pub query: String,
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
    path = "/chunk/gen_suggestions",
    context_path = "/api",
    tag = "chunk",
    request_body(content = SuggestedQueriesReqPayload, description = "JSON request payload to get alternative suggested queries", content_type = "application/json"),
    responses(
        (status = 200, description = "A JSON object containing a list of alternative suggested queries", body = SuggestedQueriesResponse),
        (status = 400, description = "Service error relating to to updating chunk, likely due to conflicting tracking_id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
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
    _required_user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    let dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.clone().server_configuration,
    );

    let base_url = dataset_config.LLM_BASE_URL.clone();
    let default_model = dataset_config.LLM_DEFAULT_MODEL.clone();

    let base_url = if base_url.is_empty() {
        "https://api.openai.com/api/v1".into()
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

    let chunk_metadatas = search_hybrid_chunks(
        SearchChunksReqPayload {
            search_type: "hybrid".to_string(),
            query: data.query.clone(),
            page_size: Some(10),
            ..Default::default()
        },
        ParsedQuery {
            query: data.query.clone(),
            quote_words: None,
            negated_words: None,
        },
        &mut SearchQueryEvent {
            id: uuid::Uuid::new_v4(),
            search_type: String::from(""),
            query: String::from(""),
            request_params: String::from(""),
            query_vector: vec![],
            latency: 0.0,
            results: vec![],
            dataset_id: uuid::Uuid::new_v4(),
            created_at: time::OffsetDateTime::now_utc(),
        },
        pool,
        dataset_org_plan_sub.dataset.clone(),
        &dataset_config,
        &mut Timer::new(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.to_string()))?
    .score_chunks;

    let rag_content = chunk_metadatas
        .iter()
        .enumerate()
        .map(|(idx, chunk)| {
            let chunk = match chunk.metadata.first().unwrap() {
                ChunkMetadataTypes::Metadata(chunk_metadata) => chunk_metadata,
                _ => unreachable!("The operator should never return slim chunks for this"),
            };

            format!(
                "Doc {}: {}",
                idx + 1,
                convert_html_to_text(&(chunk.chunk_html.clone().unwrap_or_default()))
            )
        })
        .collect::<Vec<String>>()
        .join("\n\n");

    let content = ChatMessageContent::Text(format!(
        "Here is some context for the dataset for which the user is querying for {}. Generate 10 suggested followup keyword searches based off the domain of this dataset. Your only response should be the 10 followup keyword searches which are seperated by new lines and are just text and you do not add any other context or information about the followup keyword searches. This should not be a list, so do not number each keyword search. These followup keyword searches should be related to the domain of the dataset.",
        rag_content
    ));

    let message = ChatMessage {
        role: Role::User,
        content,
        tool_calls: None,
        name: None,
        tool_call_id: None,
    };

    let parameters = ChatCompletionParameters {
        model: default_model,
        messages: vec![message],
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

    let client = Client {
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
            index: None,
            message: ChatMessage {
                role: Role::User,
                content: ChatMessageContent::Text("".to_string()),
                tool_calls: None,
                name: None,
                tool_call_id: None,
            },
            finish_reason: None,
        })
        .message
        .content
    {
        ChatMessageContent::Text(content) => content.clone(),
        _ => "".to_string(),
    }
    .split('\n')
    .map(|query| query.to_string().trim().trim_matches('\n').to_string())
    .collect();

    while queries.len() < 3 {
        query = client
            .chat()
            .create(parameters.clone())
            .await
            .expect("No OpenAI Completion for topic");
        queries = match &query
            .choices
            .first()
            .expect("No response for OpenAI completion")
            .message
            .content
        {
            ChatMessageContent::Text(content) => content.clone(),
            _ => "".to_string(),
        }
        .split('\n')
        .map(|query| query.to_string().trim().trim_matches('\n').to_string())
        .collect();
    }

    let mut engine: SimSearch<String> = SimSearch::new();

    chunk_metadatas.iter().for_each(|chunk| {
        let chunk = match chunk.metadata.first().unwrap() {
            ChunkMetadataTypes::Metadata(chunk_metadata) => chunk_metadata,
            _ => unreachable!("The operator should never return slim chunks for this"),
        };
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
