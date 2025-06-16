use super::{
    auth_handler::{AdminOnly, LoggedUser},
    chunk_handler::ChunkFilter,
};
use crate::{
    data::models::{
        self, ContextOptions, DatasetAndOrgWithSubAndPlan, DatasetConfiguration, HighlightOptions,
        LLMOptions, Pool, RedisPool, SearchMethod, SortOptions, SuggestType, TypoOptions,
    },
    errors::ServiceError,
    get_env,
    operators::{
        clickhouse_operator::EventQueue,
        file_operator::put_file_in_s3_get_signed_url,
        message_operator::{
            create_topic_message_query, delete_message_query, get_llm_api_key,
            get_message_by_id_query, get_message_by_sort_for_topic_query,
            get_messages_for_topic_query, get_text_from_audio, get_topic_messages_query,
            stream_response, stream_response_with_agentic_search, suggested_followp_questions,
            suggested_new_queries,
        },
        organization_operator::get_message_org_count,
    },
};
use actix_web::{web, HttpResponse};
use base64::Engine;
#[cfg(feature = "hallucination-detection")]
use hallucination_detection::HallucinationDetector;
use openai_dive::v1::{
    api::Client,
    models::WhisperModel,
    resources::{
        audio::{AudioOutputFormat, AudioTranscriptionParametersBuilder},
        chat::{
            ChatCompletionFunction, ChatCompletionParametersBuilder, ChatCompletionTool,
            ChatCompletionToolType, ChatMessage, ChatMessageContent, ChatMessageContentPart,
            ChatMessageImageContentPart, ChatMessageTextContentPart, ImageUrlType,
        },
        image::{EditImageParametersBuilder, ImageData, ImageQuality, ImageSize},
        shared::{FileUpload, FileUploadBytes},
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
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

#[derive(Serialize, Debug, ToSchema, Clone)]
pub struct CreateMessageReqPayload {
    /// The content of the user message to attach to the topic and then generate an assistant message in response to.
    pub new_message_content: Option<String>,
    /// The URL of the image(s) to attach to the message.
    pub image_urls: Option<Vec<String>>,
    /// The base64 encoded audio input of the user message to attach to the topic and then generate an assistant message in response to.
    pub audio_input: Option<String>,
    /// The ID of the topic to attach the message to.
    pub topic_id: uuid::Uuid,
    /// The user_id is the id of the user who is making the request. This is used to track user interactions with the RAG results.
    pub user_id: Option<String>,
    /// If use_group_search is set to true, the search will be conducted using the `search_over_groups` api. If not specified, this defaults to false.
    pub use_group_search: Option<bool>,
    /// If concat user messages query is set to true, all of the user messages in the topic will be concatenated together and used as the search query. If not specified, this defaults to false. Default is false.
    pub concat_user_messages_query: Option<bool>,
    /// LLM options to use for the completion. If not specified, this defaults to the dataset's LLM options.
    pub llm_options: Option<LLMOptions>,
    /// Context options to use for the completion. If not specified, all options will default to false.
    pub context_options: Option<ContextOptions>,
    /// No result message for when there are no chunks found above the score threshold.
    pub no_result_message: Option<String>,
    /// Only include docs used is a boolean that indicates whether or not to only include the docs that were used in the completion. If true, the completion will only include the docs that were used in the completion. If false, the completion will include all of the docs.
    pub only_include_docs_used: Option<bool>,
    /// The currency to use for the completion. If not specified, this defaults to "USD".
    pub currency: Option<String>,
    /// Search_type can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using scores from a cross encoder model. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE. Default is "hybrid".
    pub search_type: Option<SearchMethod>,
    /// Query is the search query. This can be any string. The search_query will be used to create a dense embedding vector and/or sparse vector which will be used to find the result set. If not specified, will default to the last user message or HyDE if HyDE is enabled in the dataset configuration. Default is None.
    pub search_query: Option<String>,
    /// Page size is the number of chunks to fetch during RAG. If 0, then no search will be performed. If specified, this will override the N retrievals to include in the dataset configuration. Default is None.
    pub page_size: Option<u64>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Sort Options lets you specify different methods to rerank the chunks in the result set. If not specified, this defaults to the score of the chunks.
    pub sort_options: Option<SortOptions>,
    /// Highlight Options lets you specify different methods to highlight the chunks in the result set. If not specified, this defaults to the score of the chunks.
    pub highlight_options: Option<HighlightOptions>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold. This threshold applies before weight and bias modifications. If not specified, this defaults to 0.0.
    pub score_threshold: Option<f32>,
    /// If true, quoted and - prefixed words will be parsed from the queries and used as required and negated words respectively. Default is false.
    pub use_quote_negated_terms: Option<bool>,
    /// If true, stop words (specified in server/src/stop-words.txt in the git repo) will be removed. Queries that are entirely stop words will be preserved.
    pub remove_stop_words: Option<bool>,
    /// Typo options lets you specify different methods to handle typos in the search query. If not specified, this defaults to no typo handling.
    pub typo_options: Option<TypoOptions>,
    /// Metadata is any metadata you want to associate w/ the event that is created from this request
    pub metadata: Option<serde_json::Value>,
    /// Overrides what the way chunks are placed into the context window
    pub rag_context: Option<String>,
    /// If true, the search will be conducted using llm tool calling. If not specified, this defaults to false.
    pub use_agentic_search: Option<bool>,
    /// Number of messages to include in the context window. If not specified, this defaults to 10.
    pub number_of_messages_to_include: Option<u64>,
    /// Model name to use for the completion. If not specified, this defaults to the dataset's model.
    pub model: Option<String>,
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
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn create_message(
    data: web::Json<CreateMessageReqPayload>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    event_queue: web::Data<EventQueue>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    #[cfg(feature = "hallucination-detection")] hallucination_detector: web::Data<
        HallucinationDetector,
    >,
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
            .message_count()
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

    let message_content = if let Some(ref audio_input) = create_message_data.audio_input {
        get_text_from_audio(audio_input).await?
    } else {
        create_message_data
            .new_message_content
            .clone()
            .ok_or(ServiceError::BadRequest(
                "No message content provided. Must provide either a audio or text input"
                    .to_string(),
            ))?
    };

    let new_message = models::Message::from_details(
        message_content.clone(),
        topic_id,
        0,
        "user".to_string(),
        None,
        None,
        dataset_org_plan_sub.dataset.id,
        uuid::Uuid::new_v4(),
    );

    // get the previous messages
    let mut previous_messages = get_topic_messages_query(
        topic_id,
        dataset_org_plan_sub.dataset.id,
        &get_messages_pool,
    )
    .await?;

    // remove chunks from the previous messages
    previous_messages = previous_messages
        .into_iter()
        .take(
            create_message_data
                .number_of_messages_to_include
                .unwrap_or(10) as usize,
        )
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
        create_message_data.use_agentic_search.unwrap_or(false),
        create_message_data.only_include_docs_used.unwrap_or(false),
        new_message,
        dataset_org_plan_sub.dataset.id,
        &create_message_pool,
    )
    .await?;

    if create_message_data.use_agentic_search.unwrap_or(false) {
        stream_response_with_agentic_search(
            previous_messages,
            topic_id,
            dataset_org_plan_sub.dataset,
            pool,
            redis_pool,
            dataset_config,
            create_message_data,
            event_queue,
            #[cfg(feature = "hallucination-detection")]
            hallucination_detector,
        )
        .await
    } else {
        stream_response(
            previous_messages,
            topic_id,
            dataset_org_plan_sub.dataset,
            stream_response_pool,
            event_queue,
            redis_pool,
            dataset_config,
            create_message_data,
            #[cfg(feature = "hallucination-detection")]
            hallucination_detector,
        )
        .await
    }
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
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("messages_topic_id" = uuid, Path, description = "The ID of the topic to get messages for."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_all_topic_messages(
    _user: LoggedUser,
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

/// Get a message by its ID
///
/// Quickly get the full object for a given message. From the message, you can get the topic and all messages which exist on that topic.
#[utoipa::path(
    get,
    path = "/message/{message_id}",
    context_path = "/api",
    tag = "Message",
    responses(
        (status = 200, description = "Message with the given ID", body = Message),
        (status = 400, description = "Service error relating to getting the message", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("message_id" = uuid, Path, description = "The ID of the message to get."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_message_by_id(
    _user: AdminOnly,
    message_id: web::Path<uuid::Uuid>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let message: models::Message = get_message_by_id_query(
        message_id.into_inner(),
        dataset_org_plan_sub.dataset.id,
        &pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(message))
}

#[derive(Serialize, Debug, ToSchema)]
pub struct RegenerateMessageReqPayload {
    /// The id of the topic to regenerate the last message for.
    pub topic_id: uuid::Uuid,
    /// If use_group_search is set to true, the search will be conducted using the `search_over_groups` api. If not specified, this defaults to false.
    pub use_group_search: Option<bool>,
    /// If concat user messages query is set to true, all of the user messages in the topic will be concatenated together and used as the search query. If not specified, this defaults to false. Default is false.
    pub concat_user_messages_query: Option<bool>,
    /// LLM options to use for the completion. If not specified, this defaults to the dataset's LLM options.
    pub llm_options: Option<LLMOptions>,
    /// The user_id is the id of the user who is making the request. This is used to track user interactions with the RAG results.
    pub user_id: Option<String>,
    /// Context options to use for the completion. If not specified, all options will default to false.
    pub context_options: Option<ContextOptions>,
    /// No result message for when there are no chunks found above the score threshold.
    pub no_result_message: Option<String>,
    /// Only include docs used is a boolean that indicates whether or not to only include the docs that were used in the completion. If true, the completion will only include the docs that were used in the completion. If false, the completion will include all of the docs.
    pub only_include_docs_used: Option<bool>,
    /// The currency symbol to use for the completion. If not specified, this defaults to "$".
    pub currency: Option<String>,
    /// Search_type can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using scores from a cross encoder model. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE. Default is "hybrid".
    pub search_type: Option<SearchMethod>,
    /// Query is the search query. This can be any string. The search_query will be used to create a dense embedding vector and/or sparse vector which will be used to find the result set. If not specified, will default to the last user message or HyDE if HyDE is enabled in the dataset configuration. Default is None.
    pub search_query: Option<String>,
    /// Page size is the number of chunks to fetch during RAG. If 0, then no search will be performed. If specified, this will override the N retrievals to include in the dataset configuration. Default is None.
    pub page_size: Option<u64>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Sort Options lets you specify different methods to rerank the chunks in the result set. If not specified, this defaults to the score of the chunks.
    pub sort_options: Option<SortOptions>,
    /// Highlight Options lets you specify different methods to highlight the chunks in the result set. If not specified, this defaults to the score of the chunks.
    pub highlight_options: Option<HighlightOptions>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold. This threshold applies before weight and bias modifications. If not specified, this defaults to 0.0.
    pub score_threshold: Option<f32>,
    /// If true, quoted and - prefixed words will be parsed from the queries and used as required and negated words respectively. Default is false.
    pub use_quote_negated_terms: Option<bool>,
    /// If true, stop words (specified in server/src/stop-words.txt in the git repo) will be removed. Queries that are entirely stop words will be preserved.
    pub remove_stop_words: Option<bool>,
    /// Typo options lets you specify different methods to handle typos in the search query. If not specified, this defaults to no typo handling.
    pub typo_options: Option<TypoOptions>,
    /// Metadata is any metadata you want to associate w/ the event that is created from this request
    pub metadata: Option<serde_json::Value>,
    /// Overrides what the way chunks are placed into the context window
    pub rag_context: Option<String>,
    /// If true, the search will be conducted using llm tool calling. If not specified, this defaults to false.
    pub use_agentic_search: Option<bool>,
    /// Number of messages to include in the context window. If not specified, this defaults to 10.
    pub number_of_messages_to_include: Option<u64>,
    /// Model name to use for the completion. If not specified, this defaults to the dataset's model.
    pub model: Option<String>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct EditMessageReqPayload {
    /// The id of the topic to edit the message at the given sort order for.
    pub topic_id: uuid::Uuid,
    /// The sort order of the message to edit.
    pub message_sort_order: i32,
    /// The new content of the message to replace the old content with.
    pub new_message_content: Option<String>,
    /// The base64 encoded audio input of the user message to attach to the topic and then generate an assistant message in response to.
    pub audio_input: Option<String>,
    /// The URL of the image(s) to attach to the message.
    pub image_urls: Option<Vec<String>>,
    // If use_group_search is set to true, the search will be conducted using the `search_over_groups` api. If not specified, this defaults to false.
    pub use_group_search: Option<bool>,
    /// If concat user messages query is set to true, all of the user messages in the topic will be concatenated together and used as the search query. If not specified, this defaults to false. Default is false.
    pub concat_user_messages_query: Option<bool>,
    /// LLM options to use for the completion. If not specified, this defaults to the dataset's LLM options.
    pub llm_options: Option<LLMOptions>,
    /// The user_id is the id of the user who is making the request. This is used to track user interactions with the RAG results.
    pub user_id: Option<String>,
    /// Context options to use for the completion. If not specified, all options will default to false.
    pub context_options: Option<ContextOptions>,
    /// No result message for when there are no chunks found above the score threshold.
    pub no_result_message: Option<String>,
    /// Only include docs used is a boolean that indicates whether or not to only include the docs that were used in the completion. If true, the completion will only include the docs that were used in the completion. If false, the completion will include all of the docs.
    pub only_include_docs_used: Option<bool>,
    /// The currency symbol to use for the completion. If not specified, this defaults to "$".
    pub currency: Option<String>,
    /// Search_type can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using scores from a cross encoder model. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE. Default is "hybrid".
    pub search_type: Option<SearchMethod>,
    /// Query is the search query. This can be any string. The search_query will be used to create a dense embedding vector and/or sparse vector which will be used to find the result set. If not specified, will default to the last user message or HyDE if HyDE is enabled in the dataset configuration. Default is None.
    pub search_query: Option<String>,
    /// Page size is the number of chunks to fetch during RAG. If 0, then no search will be performed. If specified, this will override the N retrievals to include in the dataset configuration. Default is None.
    pub page_size: Option<u64>,
    /// Filters is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Sort Options lets you specify different methods to rerank the chunks in the result set. If not specified, this defaults to the score of the chunks.
    pub sort_options: Option<SortOptions>,
    /// Highlight Options lets you specify different methods to highlight the chunks in the result set. If not specified, this defaults to the score of the chunks.
    pub highlight_options: Option<HighlightOptions>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold. This threshold applies before weight and bias modifications. If not specified, this defaults to 0.0.
    pub score_threshold: Option<f32>,
    /// If true, quoted and - prefixed words will be parsed from the queries and used as required and negated words respectively. Default is false.
    pub use_quote_negated_terms: Option<bool>,
    /// If true, stop words (specified in server/src/stop-words.txt in the git repo) will be removed. Queries that are entirely stop words will be preserved.
    pub remove_stop_words: Option<bool>,
    /// Typo options lets you specify different methods to handle typos in the search query. If not specified, this defaults to no typo handling.
    pub typo_options: Option<TypoOptions>,
    /// Metadata is any metadata you want to associate w/ the event that is created from this request
    pub metadata: Option<serde_json::Value>,
    /// Overrides what the way chunks are placed into the context window
    pub rag_context: Option<String>,
    /// If true, the search will be conducted using llm tool calling. If not specified, this defaults to false.
    pub use_agentic_search: Option<bool>,
    /// Number of messages to include in the context window. If not specified, this defaults to 10.
    pub number_of_messages_to_include: Option<u64>,
    /// Model name to use for the completion. If not specified, this defaults to the dataset's model.
    pub model: Option<String>,
}

impl From<EditMessageReqPayload> for CreateMessageReqPayload {
    fn from(data: EditMessageReqPayload) -> Self {
        CreateMessageReqPayload {
            new_message_content: data.new_message_content,
            image_urls: data.image_urls,
            audio_input: data.audio_input,
            topic_id: data.topic_id,
            highlight_options: data.highlight_options,
            search_type: data.search_type,
            sort_options: data.sort_options,
            use_group_search: data.use_group_search,
            concat_user_messages_query: data.concat_user_messages_query,
            search_query: data.search_query,
            page_size: data.page_size,
            filters: data.filters,
            currency: data.currency,
            score_threshold: data.score_threshold,
            llm_options: data.llm_options,
            user_id: data.user_id,
            context_options: data.context_options,
            no_result_message: data.no_result_message,
            use_quote_negated_terms: data.use_quote_negated_terms,
            remove_stop_words: data.remove_stop_words,
            typo_options: data.typo_options,
            metadata: data.metadata,
            rag_context: data.rag_context,
            use_agentic_search: data.use_agentic_search,
            only_include_docs_used: data.only_include_docs_used,
            number_of_messages_to_include: data.number_of_messages_to_include,
            model: data.model,
        }
    }
}

impl From<RegenerateMessageReqPayload> for CreateMessageReqPayload {
    fn from(data: RegenerateMessageReqPayload) -> Self {
        CreateMessageReqPayload {
            new_message_content: None,
            image_urls: None,
            audio_input: None,
            topic_id: data.topic_id,
            highlight_options: data.highlight_options,
            search_type: data.search_type,
            use_group_search: data.use_group_search,
            sort_options: data.sort_options,
            concat_user_messages_query: data.concat_user_messages_query,
            search_query: data.search_query,
            page_size: data.page_size,
            filters: data.filters,
            currency: data.currency,
            score_threshold: data.score_threshold,
            llm_options: data.llm_options,
            user_id: data.user_id,
            context_options: data.context_options,
            no_result_message: data.no_result_message,
            use_quote_negated_terms: data.use_quote_negated_terms,
            remove_stop_words: data.remove_stop_words,
            typo_options: data.typo_options,
            metadata: data.metadata,
            rag_context: data.rag_context,
            use_agentic_search: data.use_agentic_search,
            only_include_docs_used: data.only_include_docs_used,
            number_of_messages_to_include: data.number_of_messages_to_include,
            model: data.model,
        }
    }
}

/// Edit message
///
/// This will delete the specified message and replace it with a new message. All messages after the message being edited in the sort order will be deleted. The new message will be generated by the AI based on the new content provided in the request body. The response will include Chunks first on the stream if the topic is using RAG. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
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
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn edit_message(
    data: web::Json<EditMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    event_queue: web::Data<EventQueue>,
    redis_pool: web::Data<RedisPool>,
    #[cfg(feature = "hallucination-detection")] hallucination_detector: web::Data<
        HallucinationDetector,
    >,
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
        #[cfg(feature = "hallucination-detection")]
        hallucination_detector,
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
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn regenerate_message_patch(
    data: web::Json<RegenerateMessageReqPayload>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    event_queue: web::Data<EventQueue>,
    redis_pool: web::Data<RedisPool>,
    #[cfg(feature = "hallucination-detection")] hallucination_detector: web::Data<
        HallucinationDetector,
    >,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = data.topic_id;
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());

    check_completion_param_validity(data.llm_options.clone())?;

    let get_messages_pool = pool.clone();
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let mut previous_messages =
        get_topic_messages_query(topic_id, dataset_id, &get_messages_pool).await?;

    if previous_messages.len() < 2 {
        return Err(
            ServiceError::BadRequest("Not enough messages to regenerate".to_string()).into(),
        );
    }

    let create_message_data: CreateMessageReqPayload = data.into_inner().into();

    if previous_messages.len() == 2 {
        if create_message_data.use_agentic_search.unwrap_or(false) {
            return stream_response_with_agentic_search(
                previous_messages,
                topic_id,
                dataset_org_plan_sub.dataset,
                pool,
                redis_pool,
                dataset_config,
                create_message_data,
                event_queue,
                #[cfg(feature = "hallucination-detection")]
                hallucination_detector,
            )
            .await;
        } else {
            return stream_response(
                previous_messages,
                topic_id,
                dataset_org_plan_sub.dataset,
                pool,
                event_queue,
                redis_pool,
                dataset_config,
                create_message_data,
                #[cfg(feature = "hallucination-detection")]
                hallucination_detector,
            )
            .await;
        }
    }

    // remove citations from the previous messages
    previous_messages = previous_messages
        .into_iter()
        .take(
            create_message_data
                .number_of_messages_to_include
                .unwrap_or(10) as usize,
        )
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

    if create_message_data.use_agentic_search.unwrap_or(false) {
        stream_response_with_agentic_search(
            previous_messages,
            topic_id,
            dataset_org_plan_sub.dataset,
            pool,
            redis_pool,
            dataset_config,
            create_message_data,
            event_queue,
            #[cfg(feature = "hallucination-detection")]
            hallucination_detector,
        )
        .await
    } else {
        stream_response(
            previous_messages,
            topic_id,
            dataset_org_plan_sub.dataset,
            pool,
            event_queue,
            redis_pool,
            dataset_config,
            create_message_data,
            #[cfg(feature = "hallucination-detection")]
            hallucination_detector,
        )
        .await
    }
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
        ("ApiKey" = ["admin"]),
    )
)]
#[deprecated]
pub async fn regenerate_message(
    data: web::Json<RegenerateMessageReqPayload>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    event_queue: web::Data<EventQueue>,
    redis_pool: web::Data<RedisPool>,
    #[cfg(feature = "hallucination-detection")] hallucination_detector: web::Data<
        HallucinationDetector,
    >,
) -> Result<HttpResponse, actix_web::Error> {
    regenerate_message_patch(
        data,
        user,
        dataset_org_plan_sub,
        pool,
        event_queue,
        redis_pool,
        #[cfg(feature = "hallucination-detection")]
        hallucination_detector,
    )
    .await
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct SuggestedQueriesReqPayload {
    /// The number of suggested queries to create, defaults to 10
    pub suggestions_to_create: Option<usize>,
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
    /// Whether or not the suggested queries are being generated for a followup question. If true, the suggested queries will be generated for a followup question. If false, the suggested queries will be generated for a new question.
    pub is_followup: Option<bool>,
    /// Whether of not the suggested queries are being generated for ecommerce.
    pub is_ecommerce: Option<bool>,
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
pub async fn get_suggested_queries(
    data: web::Json<SuggestedQueriesReqPayload>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    _required_user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    let queries = if data.is_followup.unwrap_or(false) {
        let dataset_config = DatasetConfiguration::from_json(
            dataset_org_plan_sub.dataset.clone().server_configuration,
        );

        suggested_followp_questions(data.into_inner(), dataset_config).await?
    } else {
        suggested_new_queries(data.into_inner(), dataset_org_plan_sub, pool, redis_pool).await?
    };

    Ok(HttpResponse::Ok().json(SuggestedQueriesResponse { queries }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
/// Type of a given parameter for a LLM tool call
pub enum ToolFunctionParameterType {
    #[serde(rename = "number")]
    Number,
    #[serde(rename = "boolean")]
    Boolean,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
/// Function parameter for a LLM tool call
#[schema(example = json!({
    "name": "jackets",
    "parameter_type": "boolean",
    "description": "Whether or not the user is looking for jackets."
}))]
pub struct ToolFunctionParameter {
    /// Name of the parameter.
    pub name: String,
    /// Type of the parameter.
    pub parameter_type: ToolFunctionParameterType,
    /// The description of the tag.
    pub description: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
/// Function for a LLM tool call
#[schema(example = json!({
    "name": "get_filters",
    "description": "Decide on which filters to apply to available catalog being used within the knowledge base to respond. Always get filters.",
    "parameters": [
        {
            "name": "jackets",
            "parameter_type": "boolean",
            "description": "Whether or not the user is looking for jackets."
        },
        {
            "name": "shirts",
            "parameter_type": "boolean",
            "description": "Whether or not the user is looking for shirts."
        }
    ]
}))]
pub struct ToolFunction {
    /// Name of the function.
    pub name: String,
    /// Description of the function.
    pub description: String,
    /// Parameters of the function.
    pub parameters: Vec<ToolFunctionParameter>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
/// Request payload for getting the parameters of a tool function
#[schema(example = json!({
    "user_message_text": "Get filters for the following message: \n\nI am looking for a jacket.",
    "image_url": "https://example.com/jacket.jpg",
    "tool_function": {
        "name": "get_filters",
        "description": "Decide on which filters to apply to available catalog being used within the knowledge base to respond. Always get filters.",
        "parameters": [
            {
                "name": "jackets",
                "parameter_type": "boolean",
                "description": "Whether or not the user is looking for jackets."
            },
            {
                "name": "shirts",
                "parameter_type": "boolean",
                "description": "Whether or not the user is looking for shirts."
            }
        ]
    }
}))]
pub struct GetToolFunctionParamsReqPayload {
    /// Text of the user's message to the assistant which will be used to generate the parameters for the tool function.
    pub user_message_text: Option<String>,
    /// Image URL to attach to the message to generate the parameters for the tool function.
    #[deprecated(note = "Use image_urls instead")]
    pub image_url: Option<String>,
    /// Image URLs to attach to the message to generate the parameters for the tool function.
    pub image_urls: Option<Vec<String>>,
    /// The base64 encoded audio input of the user message to attach to the topic and then generate an assistant message in response to.
    pub audio_input: Option<String>,
    /// Function to get the parameters for.
    pub tool_function: ToolFunction,
    /// Model name to use for the completion. If not specified, this defaults to the dataset's model.
    pub model: Option<String>,
    /// Temperature to use for the completion. If not specified, this defaults to the dataset's temperature.
    pub temperature: Option<f64>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
/// Response body for getting the parameters of a tool function
#[schema(example = json!({
    "parameters": {
        "jackets": true,
        "shirts": false
    }
}))]
pub struct GetToolFunctionParamsRespBody {
    /// Parameters for the tool function.
    pub parameters: Option<serde_json::Value>,
}

/// Get tool function parameters
///
/// This endpoint will generate the parameters for a tool function based on the user's message and image URL provided in the request body. The response will include the parameters for the tool function as a JSON object.
#[utoipa::path(
    post,
    path = "/message/get_tool_function_params",
    context_path = "/api",
    tag = "Message",
    request_body(content = GetToolFunctionParamsReqPayload, description = "JSON request payload to get the parameters for a tool function", content_type = "application/json"),
    responses(
        (status = 200, description = "A JSON object containing the parameters for the tool function", body = GetToolFunctionParamsRespBody),
        (status = 400, description = "Service error relating to to updating chunk, likely due to conflicting tracking_id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn get_tool_function_params(
    data: web::Json<GetToolFunctionParamsReqPayload>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _required_user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    // check that there are less than 50 parameters
    if data.tool_function.parameters.len() > 50 {
        return Err(ServiceError::BadRequest(
            "The number of parameters for the tool function must be less than 50".to_string(),
        ));
    }

    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());

    let base_url = dataset_config.LLM_BASE_URL.clone();
    let chosen_model = data
        .model
        .clone()
        .unwrap_or(dataset_config.LLM_DEFAULT_MODEL.clone());

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

    let message_content = if let Some(ref audio_input) = data.audio_input {
        get_text_from_audio(audio_input).await?
    } else {
        data.user_message_text
            .clone()
            .ok_or(ServiceError::BadRequest(
                "No message content provided. Must provide either a audio or text input"
                    .to_string(),
            ))?
    };

    let mut message_content_parts =
        vec![ChatMessageContentPart::Text(ChatMessageTextContentPart {
            r#type: "text".to_string(),
            text: message_content.clone(),
        })];
    if let Some(image_url) = data.image_url.clone() {
        message_content_parts.insert(
            0,
            ChatMessageContentPart::Image(ChatMessageImageContentPart {
                r#type: "image_url".to_string(),
                image_url: ImageUrlType {
                    url: image_url,
                    detail: None,
                },
            }),
        );
    }
    if let Some(image_urls) = data.image_urls.clone() {
        for image_url in image_urls.iter() {
            message_content_parts.insert(
                0,
                ChatMessageContentPart::Image(ChatMessageImageContentPart {
                    r#type: "image_url".to_string(),
                    image_url: ImageUrlType {
                        url: image_url.clone(),
                        detail: None,
                    },
                }),
            );
        }
    }

    let client = Client {
        headers: None,
        project: None,
        api_key: llm_api_key,
        http_client: reqwest::Client::new(),
        base_url,
        organization: None,
    };

    let temperature = data.temperature.or(dataset_config.TEMPERATURE);

    let mut parameters_builder = ChatCompletionParametersBuilder::default();

    parameters_builder
        .model(chosen_model)
        .messages(vec![ChatMessage::User {
            content: ChatMessageContent::ContentPart(message_content_parts),
            name: None,
        }])
        .tools(vec![ChatCompletionTool {
            r#type: ChatCompletionToolType::Function,
            function: ChatCompletionFunction {
                name: data.tool_function.name.clone(),
                description: Some(data.tool_function.description.clone()),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": data
                        .tool_function
                        .parameters
                        .iter()
                        .map(|parameter| {
                            (
                                parameter.name.clone(),
                                serde_json::json!({
                                    "type": parameter.parameter_type,
                                    "description": parameter.description.clone(),
                                }),
                            )
                        })
                        .collect::<serde_json::Map<String, serde_json::Value>>(),
                    "required": data
                        .tool_function
                        .parameters
                        .iter()
                        .map(|parameter| parameter.name.clone())
                        .collect::<Vec<String>>(),
                }),
            },
        }]);

    if let Some(temperature) = temperature {
        parameters_builder.temperature(temperature as f32);
    }

    let parameters = parameters_builder.build().map_err(|err| {
        ServiceError::BadRequest(format!(
            "Failed to build tool call parameters completion API call params because: {err}"
        ))
    })?;

    let result = client.chat().create(parameters).await.map_err(|err| {
        ServiceError::BadRequest(format!(
            "Failed to get tool function parameters completion from openai API host because: {err}"
        ))
    })?;

    let first_message = match result.choices.first() {
        Some(first_message) => first_message.message.clone(),
        None => {
            return Err(ServiceError::BadRequest(
                "No first message in choices on call to LLM".to_string(),
            ));
        }
    };

    let tool_call = match first_message {
        ChatMessage::Assistant {
            tool_calls: Some(tool_calls),
            ..
        } => tool_calls.first().cloned(),
        _ => None,
    };

    let resp_body = GetToolFunctionParamsRespBody {
        parameters: tool_call
            .and_then(|tool_call| serde_json::from_str(&tool_call.function.arguments).ok()),
    };

    if data.audio_input.is_some() {
        return Ok(HttpResponse::Ok()
            .insert_header((
                "X-TR-Query",
                message_content
                    .to_string()
                    .replace(|c: char| c.is_ascii_control(), ""),
            ))
            .json(resp_body));
    }

    Ok(HttpResponse::Ok().json(resp_body))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EditImageReqPayload {
    /// The images to edit
    pub input_images: Vec<ImageUpload>,
    /// The prompt describing the desired edit
    pub prompt: String,
    /// The number of images to generate (default: 1)
    pub n: Option<u32>,
    /// The size of the generated image as a string (e.g. "256x256", "512x512", "1024x1024", "1792x1024", "1024x1792")
    pub size: Option<InputImageSize>,
    // /// The quality of the generated image. (e.g. "low", "medium", "high")
    pub quality: Option<InputImageQuality>,
    /// The mime type of the uploaded image(s)
    pub mime_type: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, ToSchema, Copy, Clone)]
pub enum InputImageQuality {
    #[default]
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "medium")]
    Medium,
    #[serde(rename = "high")]
    High,
}

impl From<InputImageQuality> for ImageQuality {
    fn from(quality: InputImageQuality) -> Self {
        match quality {
            InputImageQuality::Low => ImageQuality::Low,
            InputImageQuality::Medium => ImageQuality::Medium,
            InputImageQuality::High => ImageQuality::High,
        }
    }
}

#[derive(Default, Debug, Serialize, Deserialize, ToSchema, Copy, Clone)]
pub enum InputImageSize {
    #[default]
    #[serde(rename = "1024x1024")]
    Size1024X1024,
    #[serde(rename = "1024x1536")]
    Size1024x1536,
    #[serde(rename = "1536x1024")]
    Size1536x1024,
}

impl From<InputImageSize> for ImageSize {
    fn from(size: InputImageSize) -> Self {
        match size {
            InputImageSize::Size1024X1024 => ImageSize::Size1024X1024,
            InputImageSize::Size1024x1536 => ImageSize::Size1024X1536,
            InputImageSize::Size1536x1024 => ImageSize::Size1536X1024,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub enum ImageSourceType {
    #[serde(rename = "base64")]
    #[schema(title = "Base64")]
    /// Base64 encoded image data
    Base64(String),
    #[serde(rename = "url")]
    #[schema(title = "URL")]
    /// URL of the image
    Url(String),
}

// impl a to base64 function for the ImageDTO enum
impl ImageSourceType {
    pub async fn to_image_bytes(&self) -> Result<Vec<u8>, ServiceError> {
        match self {
            ImageSourceType::Base64(base64_string) => {
                let decoded_bytes = base64::prelude::BASE64_STANDARD
                    .decode(base64_string)
                    .map_err(|_| ServiceError::BadRequest("Invalid base64 string".to_string()))?;
                Ok(decoded_bytes)
            }
            ImageSourceType::Url(url) => {
                let response = reqwest::get(url)
                    .await
                    .map_err(|_| ServiceError::BadRequest("Invalid URL".to_string()))?;
                let bytes = response.bytes().await.map_err(|_| {
                    ServiceError::BadRequest("Failed to read bytes from URL".to_string())
                })?;
                Ok(bytes.to_vec())
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImageUpload {
    /// The image base64 encoded
    /// The image data - either a base64-encoded string or a URL
    pub image_src: ImageSourceType,
    /// The file name of the image
    pub file_name: String,
}

impl ImageUpload {
    pub async fn to_file_upload_bytes(&self) -> Result<FileUploadBytes, ServiceError> {
        let image_bytes = self.image_src.to_image_bytes().await?;
        let file_upload_bytes = FileUploadBytes {
            bytes: image_bytes.into(),
            filename: self.file_name.clone(),
        };
        Ok(file_upload_bytes)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImageEditResponse {
    /// The URL of the generated image
    pub image_urls: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ImageResponseData {
    /// The base64-encoded JSON of the generated image.
    b64_json: String,
}

/// Edit Image
///
/// Uses `gpt-image-1` to edit an images based on a given prompt. Note that the images must be
/// base64 encoded and all must have the same mime type.
#[utoipa::path(
    post,
    path = "/message/edit_image",
    context_path = "/api",
    tag = "Message",
    request_body(content = EditImageReqPayload, description = "JSON request payload to edit an image", content_type = "application/json"),
    responses(
        (status = 200, description = "A list of base64 encoded images", body = ImageEditResponse,
            headers(
                ("TR-QueryID" = uuid::Uuid, description = "Query ID that is used for tracking analytics")
            )
        ),
        (status = 400, description = "Service error relating to editing the image", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn edit_image(
    data: web::Json<EditImageReqPayload>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.clone().server_configuration);

    let llm_api_key = get_llm_api_key(&dataset_config);
    let mut base_url = dataset_config.LLM_BASE_URL.clone();

    if !base_url.contains("openai.com") {
        base_url = "https://api.openai.com/v1".to_string();
    }

    let client = Client {
        headers: None,
        api_key: llm_api_key,
        project: None,
        http_client: reqwest::Client::new(),
        base_url,
        organization: None,
    };

    let mut file_upload_bytes_futures = Vec::new();
    for input_image in &data.input_images {
        let fut = input_image.to_file_upload_bytes();
        file_upload_bytes_futures.push(fut);
    }
    let file_upload_bytes = futures::future::try_join_all(file_upload_bytes_futures).await?;

    let parameters = EditImageParametersBuilder::default()
        .image(FileUpload::BytesArray(file_upload_bytes))
        .model("gpt-image-1")
        .quality::<ImageQuality>(data.quality.unwrap_or_default().into())
        .prompt(data.prompt.clone())
        .mime_type(data.mime_type.clone().unwrap_or("image/png".to_string()))
        .n(data.n.unwrap_or(1))
        .size::<ImageSize>(data.size.unwrap_or_default().into())
        .build()
        .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    let result = client
        .images()
        .edit(parameters)
        .await
        .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    let images: Vec<ImageResponseData> = result
        .data
        .iter()
        .filter_map(|image| {
            if let ImageData::B64Json { b64_json, .. } = &image {
                Some(ImageResponseData {
                    b64_json: b64_json.clone(),
                })
            } else {
                None
            }
        })
        .collect();

    let image_signed_url_futures = images.iter().map(|image| {
        let file_id = uuid::Uuid::new_v4();
        let b64_json = image.b64_json.clone();
        let decoded_bytes = base64::prelude::BASE64_STANDARD
            .decode(b64_json)
            .unwrap_or_default();
        put_file_in_s3_get_signed_url(file_id, decoded_bytes)
    });

    let image_urls = futures::future::join_all(image_signed_url_futures)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(HttpResponse::Ok().json(ImageEditResponse { image_urls }))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TranscribeAudioReqPayload {
    /// The base64 encoded audio input of the user's input message.
    pub base64_audio: String,
}

/// Transcribe Audio
///
/// Uses `whisper-1` to transcribe an audio file passed in as a base64 encoded string.
#[utoipa::path(
    post,
    path = "/message/transcribe_audio",
    context_path = "/api",
    tag = "Message",
    request_body(content = TranscribeAudioReqPayload, description = "JSON request payload to transcribe an audio file", content_type = "application/json"),
    responses(
        (status = 200, description = "The transcribed text", body = String,
            headers(
                ("TR-QueryID" = uuid::Uuid, description = "Query ID that is used for tracking analytics")
            )
        ),
        (status = 400, description = "Service error relating to transcribing the audio", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn transcribe_audio(
    data: web::Json<TranscribeAudioReqPayload>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _required_user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let audio_bytes = base64::decode(data.base64_audio.clone()).map_err(|err| {
        log::error!("Failed to decode base64 audio: {:?}", err);
        ServiceError::BadRequest(format!("Error decoding audio base64: {:?}", err))
    })?;

    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.clone().server_configuration);

    let llm_api_key = get_llm_api_key(&dataset_config);
    let mut base_url = dataset_config.LLM_BASE_URL.clone();

    if !base_url.contains("openai.com") {
        base_url = "https://api.openai.com/v1".to_string();
    }

    let client = Client {
        headers: None,
        api_key: llm_api_key,
        project: None,
        http_client: reqwest::Client::new(),
        base_url,
        organization: None,
    };

    let file_upload = FileUpload::Bytes(FileUploadBytes {
        filename: "audio.mp3".to_string(),
        bytes: audio_bytes.into(),
    });

    let parameters = AudioTranscriptionParametersBuilder::default()
        .file(file_upload)
        .model(WhisperModel::Whisper1.to_string())
        .response_format(AudioOutputFormat::Text)
        .language("en".to_string())
        .build()
        .map_err(|err| {
            log::error!("Failed to build transcription parameters: {:?}", err);
            ServiceError::InternalServerError(format!(
                "Failed to build transcription parameters: {:?}",
                err
            ))
        })?;

    let transcribed_text = client
        .audio()
        .create_transcription(parameters)
        .await
        .map_err(|err| {
            log::error!("Failed to transcribe audio: {:?}", err);
            ServiceError::InternalServerError(format!("Failed to transcribe audio: {:?}", err))
        })?;

    Ok(HttpResponse::Ok().json(transcribed_text.replace("\n", "")))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GenerateMessageCompletionsReqPayload {
    /// The system message to use for the message completion.
    pub system_message: String,
    /// The user message to use for the message completion.
    pub user_message: String,
}

/// Generate Message Completions
///
/// Uses `openai` to generate a message completion for a given prompt.
#[utoipa::path(
    post,
    path = "/message/generate_message_completions",
    context_path = "/api",
    tag = "Message",
    request_body(content = GenerateMessageCompletionsReqPayload, description = "JSON request payload to generate a message completion", content_type = "application/json"),
    responses(
        (status = 200, description = "The generated message completion", body = String,
            headers(
                ("TR-QueryID" = uuid::Uuid, description = "Query ID that is used for tracking analytics")
            )
        ),
        (status = 400, description = "Service error relating to generating a message completion", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn generate_message_completions(
    data: web::Json<GenerateMessageCompletionsReqPayload>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _required_user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.clone().server_configuration);

    let llm_api_key = get_llm_api_key(&dataset_config);
    let mut base_url = dataset_config.LLM_BASE_URL.clone();

    if !base_url.contains("openai.com") {
        base_url = "https://api.openai.com/v1".to_string();
    }

    let client = Client {
        headers: None,
        api_key: llm_api_key,
        project: None,
        http_client: reqwest::Client::new(),
        base_url,
        organization: None,
    };

    let system_message = ChatMessage::System {
        content: ChatMessageContent::Text(data.system_message.clone()),
        name: None,
    };

    let user_message = ChatMessage::User {
        content: ChatMessageContent::Text(data.user_message.clone()),
        name: None,
    };

    let parameters = ChatCompletionParametersBuilder::default()
        .model("gpt-4o-mini".to_string())
        .messages(vec![system_message, user_message])
        .build()
        .map_err(|err| {
            log::error!("Failed to build message completion parameters: {:?}", err);
            ServiceError::InternalServerError(format!(
                "Failed to build message completion parameters: {:?}",
                err
            ))
        })?;

    let message_completion = client.chat().create(parameters).await.map_err(|err| {
        log::error!("Failed to generate message completion: {:?}", err);
        ServiceError::InternalServerError(format!(
            "Failed to generate message completion: {:?}",
            err
        ))
    })?;

    let first_message = match message_completion.choices.first() {
        Some(first_message) => first_message.message.clone(),
        None => {
            return Err(ServiceError::BadRequest(
                "No first message in choices on call to LLM".to_string(),
            ));
        }
    };

    let response_content = match first_message {
        ChatMessage::Assistant {
            content: Some(ChatMessageContent::Text(content)),
            ..
        } => content,
        _ => {
            log::error!(
                "ChatMessage of first choice did not have text or was either Tool or Function {:?}",
                first_message
            );
            "ChatMessage of first did not have text or was either Tool or Function".to_string()
        }
    };

    Ok(HttpResponse::Ok().json(response_content))
}
