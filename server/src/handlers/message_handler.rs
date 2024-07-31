use super::{
    auth_handler::{AdminOnly, LoggedUser},
    chunk_handler::{ChunkFilter, ParsedQuery, SearchChunksReqPayload},
};
use crate::{
    data::models::{
        self, ChunkMetadataTypes, DatasetAndOrgWithSubAndPlan, DatasetConfiguration,
        HighlightOptions, LLMOptions, Pool, SearchMethod,
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
/// Messages are attached to topics in order to coordinate memory of gen-AI chat sessions.Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/message",
    context_path = "/api",
    tag = "Message",
    request_body(content = CreateMessageReqPayload, description = "JSON request payload to create a message completion", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream of a string, check the chat or search UI for an example how to process this. Response if streaming.",),
        (status = 200, description = "This will be a JSON response of a string containing the LLM's generated inference. Response if not streaming.", body = String),
        (status = 400, description = "Service error relating to getting a chat completion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
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
        clickhouse_client,
        dataset_config,
        create_message_data,
    )
    .await
}

/// Get all messages for a given topic
///
/// If the topic is a RAG topic then the response will include Chunks first on each message. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information.
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
        ("TR-Dataset" = String, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
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
        (status = 200, description = "This will be a HTTP stream, check the chat or search UI for an example how to process this"),
        (status = 400, description = "Service error relating to getting a chat completion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
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
        clickhouse_client,
        third_pool,
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
        (status = 200, description = "This will be a HTTP stream of a string, check the chat or search UI for an example how to process this. Response if streaming.",),
        (status = 200, description = "This will be a JSON response of a string containing the LLM's generated inference. Response if not streaming.", body = String),
        (status = 400, description = "Service error relating to getting a chat completion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool, clickhouse_client))]
pub async fn regenerate_message_patch(
    data: web::Json<RegenerateMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    clickhouse_client: web::Data<clickhouse::Client>,
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
            clickhouse_client,
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
        (status = 200, description = "This will be a HTTP stream of a string, check the chat or search UI for an example how to process this. Response if streaming.",),
        (status = 200, description = "This will be a JSON response of a string containing the LLM's generated inference. Response if not streaming.", body = String),
        (status = 400, description = "Service error relating to getting a chat completion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[deprecated]
#[tracing::instrument(skip(pool, clickhouse_client))]
pub async fn regenerate_message(
    data: web::Json<RegenerateMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    clickhouse_client: web::Data<clickhouse::Client>,
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
            clickhouse_client,
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
        dataset_config,
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
    path = "/chunk/suggestions",
    context_path = "/api",
    tag = "Chunk",
    request_body(content = SuggestedQueriesReqPayload, description = "JSON request payload to get alternative suggested queries", content_type = "application/json"),
    responses(
        (status = 200, description = "A JSON object containing a list of alternative suggested queries", body = SuggestedQueriesResponse),
        (status = 400, description = "Service error relating to to updating chunk, likely due to conflicting tracking_id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
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

    let chunk_metadatas = search_hybrid_chunks(
        SearchChunksReqPayload {
            search_type: SearchMethod::Hybrid,
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
