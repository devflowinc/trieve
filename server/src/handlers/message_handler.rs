use super::{
    auth_handler::{AdminOnly, LoggedUser},
    chunk_handler::{ChunkFilter, ParsedQuery, SearchChunksReqPayload},
};
use crate::{
    data::models::{
        self, ChunkMetadataTypes, DatasetAndOrgWithSubAndPlan, Pool, ServerDatasetConfiguration,
    },
    errors::ServiceError,
    get_env,
    operators::{
        message_operator::{
            create_topic_message_query, delete_message_query, get_message_by_sort_for_topic_query,
            get_messages_for_topic_query, get_topic_messages, stream_response,
        },
        organization_operator::get_message_org_count,
        parse_operator::convert_html_to_text,
        search_operator::search_hybrid_chunks,
    },
};
use actix_web::{web, HttpResponse};

use itertools::Itertools;
use openai_dive::v1::{
    api::Client,
    resources::chat::{
        ChatCompletionChoice, ChatCompletionParameters, ChatMessage, ChatMessageContent, Role,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use simple_server_timing_header::Timer;
use simsearch::SimSearch;
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
    /// If concat user messages query is set to true, all of the user messages in the topic will be concatenated together and used as the search query. If not specified, this defaults to false. Default is false.
    pub concat_user_messages_query: Option<bool>,
    /// Query is the search query. This can be any string. The search_query will be used to create a dense embedding vector and/or sparse vector which will be used to find the result set. If not specified, will default to the last user message or HyDE if HyDE is enabled in the dataset configuration. Default is None.
    pub search_query: Option<String>,
    /// Page size is the number of chunks to fetch during RAG. If 0, then no search will be performed. If specified, this will override the N retrievals to include in the dataset configuration. Default is None.
    pub page_size: Option<u64>,
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
/// Create message. Messages are attached to topics in order to coordinate memory of gen-AI chat sessions.Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
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
#[tracing::instrument(skip(pool, clickhouse_client))]
pub async fn create_message(
    data: web::Json<CreateMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    clickhouse_client: web::Data<clickhouse::Client>,
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
        create_message_data.new_message_content.clone(),
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
        dataset_org_plan_sub.dataset,
        stream_response_pool,
        clickhouse_client,
        server_dataset_configuration,
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
    /// If concat user messages query is set to true, all of the user messages in the topic will be concatenated together and used as the search query. If not specified, this defaults to false. Default is false.
    pub concat_user_messages_query: Option<bool>,
    /// Query is the search query. This can be any string. The search_query will be used to create a dense embedding vector and/or sparse vector which will be used to find the result set. If not specified, will default to the last user message or HyDE if HyDE is enabled in the dataset configuration. Default is None.
    pub search_query: Option<String>,
    /// Page size is the number of chunks to fetch during RAG. If 0, then no search will be performed. If specified, this will override the N retrievals to include in the dataset configuration. Default is None.
    pub page_size: Option<u64>,
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
    /// If concat user messages query is set to true, all of the user messages in the topic will be concatenated together and used as the search query. If not specified, this defaults to false. Default is false.
    pub concat_user_messages_query: Option<bool>,
    /// Query is the search query. This can be any string. The search_query will be used to create a dense embedding vector and/or sparse vector which will be used to find the result set. If not specified, will default to the last user message or HyDE if HyDE is enabled in the dataset configuration. Default is None.
    pub search_query: Option<String>,
    /// Page size is the number of chunks to fetch during RAG. If 0, then no search will be performed. If specified, this will override the N retrievals to include in the dataset configuration. Default is None.
    pub page_size: Option<u64>,
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

impl From<EditMessageReqPayload> for CreateMessageReqPayload {
    fn from(data: EditMessageReqPayload) -> Self {
        CreateMessageReqPayload {
            new_message_content: data.new_message_content,
            topic_id: data.topic_id,
            highlight_results: data.highlight_citations,
            highlight_delimiters: data.highlight_delimiters,
            search_type: data.search_type,
            concat_user_messages_query: data.concat_user_messages_query,
            search_query: data.search_query,
            page_size: data.page_size,
            filters: data.filters,
            completion_first: data.completion_first,
            stream_response: data.stream_response,
            temperature: data.temperature,
            frequency_penalty: data.frequency_penalty,
            presence_penalty: data.presence_penalty,
            max_tokens: data.max_tokens,
            stop_tokens: data.stop_tokens,
        }
    }
}

impl From<RegenerateMessageReqPayload> for CreateMessageReqPayload {
    fn from(data: RegenerateMessageReqPayload) -> Self {
        CreateMessageReqPayload {
            new_message_content: "".to_string(),
            topic_id: data.topic_id,
            highlight_results: data.highlight_citations,
            highlight_delimiters: data.highlight_delimiters,
            search_type: data.search_type,
            concat_user_messages_query: data.concat_user_messages_query,
            search_query: data.search_query,
            page_size: data.page_size,
            filters: data.filters,
            completion_first: data.completion_first,
            stream_response: data.stream_response,
            temperature: data.temperature,
            frequency_penalty: data.frequency_penalty,
            presence_penalty: data.presence_penalty,
            max_tokens: data.max_tokens,
            stop_tokens: data.stop_tokens,
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
#[tracing::instrument(skip(pool, clickhouse_client))]
pub async fn edit_message(
    data: web::Json<EditMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    clickhouse_client: web::Data<clickhouse::Client>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id: uuid::Uuid = data.topic_id;
    let message_sort_order = data.message_sort_order;

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
        actix_web::web::Json(data.into_inner().into()),
        user,
        dataset_org_plan_sub,
        clickhouse_client,
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
#[tracing::instrument(skip(pool, clickhouse_client))]
pub async fn regenerate_message(
    data: web::Json<RegenerateMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    clickhouse_client: web::Data<clickhouse::Client>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = data.topic_id;
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
            dataset_org_plan_sub.dataset,
            create_message_pool,
            clickhouse_client,
            server_dataset_configuration,
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

    delete_message_query(message_id, topic_id, dataset_id, &pool).await?;

    stream_response(
        previous_messages_to_regenerate,
        topic_id,
        dataset_org_plan_sub.dataset,
        create_message_pool,
        clickhouse_client,
        server_dataset_configuration,
        data.into_inner().into(),
    )
    .await
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
