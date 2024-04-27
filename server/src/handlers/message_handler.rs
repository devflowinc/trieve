use super::{auth_handler::LoggedUser, chunk_handler::ParsedQuery};
use crate::{
    data::models::{
        self, ChunkMetadata, Dataset, DatasetAndOrgWithSubAndPlan, Pool, ServerDatasetConfiguration,
    },
    errors::ServiceError,
    get_env,
    operators::{
        chunk_operator::{
            find_relevant_sentence, get_chunk_metadatas_and_collided_chunks_from_point_ids_query,
        },
        message_operator::{
            create_message_query, create_topic_message_query, delete_message_query,
            get_message_by_sort_for_topic_query, get_messages_for_topic_query, get_topic_messages,
            user_owns_topic_query,
        },
        model_operator::create_embedding,
        organization_operator::get_message_org_count,
        qdrant_operator::VectorType,
        search_operator::retrieve_qdrant_points_query,
    },
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
    resources::chat::{
        ChatCompletionChoice, ChatCompletionParameters, ChatMessage, ChatMessageContent, Role,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_stream::StreamExt;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct CreateMessageData {
    /// The content of the user message to attach to the topic and then generate an assistant message in response to.
    pub new_message_content: String,
    /// The ID of the topic to attach the message to.
    pub topic_id: uuid::Uuid,
    /// Whether or not to stream the response. If this is set to true or not included, the response will be a stream. If this is set to false, the response will be a normal JSON response. Default is true.
    pub stream_response: Option<bool>,
    /// Whether or not to highlight the citations in the response. If this is set to true or not included, the citations will be highlighted. If this is set to false, the citations will not be highlighted. Default is true.
    pub highlight_citations: Option<bool>,
    /// The delimiters to use for highlighting the citations. If this is not included, the default delimiters will be used. Default is `[".", "!", "?", "\n", "\t", ","]`.
    pub highlight_delimiters: Option<Vec<String>>,
}

/// Create a message
///
/// Create a message. Messages are attached to topics in order to coordinate memory of gen-AI chat sessions. We are considering refactoring this resource of the API soon. Currently, you can only send user messages. If the topic is a RAG topic then the response will include Chunks first on the stream. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information.
#[utoipa::path(
    post,
    path = "/message",
    context_path = "/api",
    tag = "message",
    request_body(content = CreateMessageData, description = "JSON request payload to create a message completion", content_type = "application/json"),
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
pub async fn create_message_completion_handler(
    data: web::Json<CreateMessageData>,
    user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let message_count_pool = pool.clone();
    let message_count_org_id = dataset_org_plan_sub.organization.organization.id;
    let server_dataset_configuration = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

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
    let user_owns_topic_pool = pool.clone();
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

    let _topic = user_owns_topic_query(
        user.id,
        topic_id,
        dataset_org_plan_sub.dataset.id,
        &user_owns_topic_pool,
    )
    .await
    .map_err(|_e| ServiceError::Unauthorized)?;

    // get the previous messages
    let mut previous_messages = get_topic_messages(
        topic_id,
        dataset_org_plan_sub.dataset.id,
        &get_messages_pool,
    )
    .await?;

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
    let previous_messages = create_topic_message_query(
        previous_messages,
        new_message,
        user.id,
        dataset_org_plan_sub.dataset.id,
        &create_message_pool,
    )
    .await?;

    stream_response(
        previous_messages,
        user.id,
        topic_id,
        create_message_data.stream_response,
        create_message_data.highlight_citations,
        create_message_data.highlight_delimiters,
        dataset_org_plan_sub.dataset,
        stream_response_pool,
        server_dataset_configuration,
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
    let second_pool = pool.clone();
    let topic_id: uuid::Uuid = messages_topic_id.into_inner();
    // check if the user owns the topic
    let _user_topic = user_owns_topic_query(
        user.id,
        topic_id,
        dataset_org_plan_sub.dataset.id,
        &second_pool,
    )
    .await
    .map_err(|_e| ServiceError::Unauthorized)?;

    let messages =
        get_messages_for_topic_query(topic_id, dataset_org_plan_sub.dataset.id, &pool).await?;

    Ok(HttpResponse::Ok().json(messages))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct RegenerateMessageData {
    /// The id of the topic to regenerate the last message for.
    pub topic_id: uuid::Uuid,
    /// Whether or not to stream the response. If this is set to true or not included, the response will be a stream. If this is set to false, the response will be a normal JSON response. Default is true.
    pub stream_response: Option<bool>,
    /// Whether or not to highlight the citations in the response. If this is set to true or not included, the citations will be highlighted. If this is set to false, the citations will not be highlighted. Default is true.
    pub highlight_citations: Option<bool>,
    /// The delimiters to use for highlighting the citations. If this is not included, the default delimiters will be used. Default is `[".", "!", "?", "\n", "\t", ","]`.  
    pub highlight_delimiters: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct EditMessageData {
    /// The id of the topic to edit the message at the given sort order for.
    pub topic_id: uuid::Uuid,
    /// The sort order of the message to edit.
    pub message_sort_order: i32,
    /// The new content of the message to replace the old content with.
    new_message_content: String,
    /// Whether or not to stream the response. If this is set to true or not included, the response will be a stream. If this is set to false, the response will be a normal JSON response. Default is true.
    pub stream_response: Option<bool>,
    /// Whether or not to highlight the citations in the response. If this is set to true or not included, the citations will be highlighted. If this is set to false, the citations will not be highlighted. Default is true.
    pub highlight_citations: Option<bool>,
    /// The delimiters to use for highlighting the citations. If this is not included, the default delimiters will be used. Default is `[".", "!", "?", "\n", "\t", ","]`.
    pub highlight_delimiters: Option<Vec<String>>,
}

/// Edit a message
///
/// Edit a message which exists within the topic's chat history. This will delete the message and replace it with a new message. The new message will be generated by the AI based on the new content provided in the request body. The response will include Chunks first on the stream if the topic is using RAG. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information.
#[utoipa::path(
    put,
    path = "/message",
    context_path = "/api",
    tag = "message",
    request_body(content = EditMessageData, description = "JSON request payload to edit a message and get a new stream", content_type = "application/json"),
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
pub async fn edit_message_handler(
    data: web::Json<EditMessageData>,
    user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id: uuid::Uuid = data.topic_id;
    let stream_response = data.stream_response;
    let message_sort_order = data.message_sort_order;
    let new_message_content = &data.new_message_content;
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

    let _ = delete_message_query(
        &user.id,
        message_id,
        topic_id,
        dataset_org_plan_sub.dataset.id,
        &second_pool,
    )
    .await;

    create_message_completion_handler(
        actix_web::web::Json(CreateMessageData {
            new_message_content: new_message_content.to_string(),
            topic_id,
            stream_response,
            highlight_citations: data.highlight_citations,
            highlight_delimiters: data.highlight_delimiters.clone(),
        }),
        user,
        dataset_org_plan_sub,
        third_pool,
    )
    .await
}

/// Regenerate message
///
/// Regenerate the assistant response to the last user message of a topic. This will delete the last message and replace it with a new message. The response will include Chunks first on the stream if the topic is using RAG. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information.
#[utoipa::path(
    delete,
    path = "/message",
    context_path = "/api",
    tag = "message",
    request_body(content = RegenerateMessageData, description = "JSON request payload to delete an agent message then regenerate it in a strem", content_type = "application/json"),
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
pub async fn regenerate_message_handler(
    data: web::Json<RegenerateMessageData>,
    user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = data.topic_id;
    let should_stream = data.stream_response;
    let server_dataset_configuration = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let user_owns_topic_pool = pool.clone();
    let get_messages_pool = pool.clone();
    let create_message_pool = pool.clone();
    let dataset_id = dataset_org_plan_sub.dataset.id;

    user_owns_topic_query(user.id, topic_id, dataset_id, &user_owns_topic_pool)
        .await
        .map_err(|_e| ServiceError::Unauthorized)?;

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
            user.id,
            topic_id,
            should_stream,
            data.highlight_citations,
            data.highlight_delimiters.clone(),
            dataset_org_plan_sub.dataset,
            create_message_pool,
            server_dataset_configuration,
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

    let _ = delete_message_query(&user.id, message_id, topic_id, dataset_id, &pool).await;

    stream_response(
        previous_messages_to_regenerate,
        user.id,
        topic_id,
        should_stream,
        data.highlight_citations,
        data.highlight_delimiters.clone(),
        dataset_org_plan_sub.dataset,
        create_message_pool,
        server_dataset_configuration,
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
    user_id: uuid::Uuid,
    topic_id: uuid::Uuid,
    should_stream: Option<bool>,
    highlight_citations: Option<bool>,
    highlight_delimiters: Option<Vec<String>>,
    dataset: Dataset,
    pool: web::Data<Pool>,
    config: ServerDatasetConfiguration,
) -> Result<HttpResponse, actix_web::Error> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());

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

    let message_to_query_prompt = dataset_config.MESSAGE_TO_QUERY_PROMPT.clone();
    let rag_prompt = dataset_config.RAG_PROMPT.clone();
    let chosen_model = dataset_config.LLM_DEFAULT_MODEL.clone();

    sentry::configure_scope(|scope| {
        scope.set_tag("LLM_MODEL", chosen_model.clone());
        scope.set_tag("LLM_BASE_URL", base_url.clone());
    });

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

    let mut citation_chunks_stringified;
    let mut citation_chunks_stringified1;

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

    let search_query_from_message_to_query_prompt = client
        .chat()
        .create(gen_inference_parameters)
        .await
        .expect("No OpenAI Completion for chunk search");
    let query = match &search_query_from_message_to_query_prompt
        .choices
        .first()
        .expect("No response for OpenAI completion")
        .message
        .content
    {
        ChatMessageContent::Text(query) => query.clone(),
        _ => "".to_string(),
    };
    let embedding_vector = create_embedding(query.clone(), "query", dataset_config.clone()).await?;

    let n_retrievals_to_include = dataset_config.N_RETRIEVALS_TO_INCLUDE;

    let search_chunk_query_results = retrieve_qdrant_points_query(
        VectorType::Dense(embedding_vector),
        1,
        n_retrievals_to_include.try_into().unwrap(),
        None,
        None,
        ParsedQuery {
            query: query.to_string(),
            quote_words: None,
            negated_words: None,
        },
        dataset.id,
        pool.clone(),
        config,
    )
    .await?;

    let retrieval_chunk_ids = search_chunk_query_results
        .search_results
        .iter()
        .map(|chunk| chunk.point_id)
        .collect::<Vec<uuid::Uuid>>();

    let (metadata_chunks, _) = get_chunk_metadatas_and_collided_chunks_from_point_ids_query(
        retrieval_chunk_ids,
        false,
        pool.clone(),
    )
    .await?;

    let citation_chunks: Vec<ChunkMetadata> = metadata_chunks.to_vec();

    let highlighted_citation_chunks = if highlight_citations.unwrap_or(true) {
        citation_chunks
            .iter()
            .map(|chunk| {
                find_relevant_sentence(
                    chunk.clone(),
                    query.to_string(),
                    highlight_delimiters.clone().unwrap_or(vec![
                        ".".to_string(),
                        "!".to_string(),
                        "?".to_string(),
                        "\n".to_string(),
                        "\t".to_string(),
                        ",".to_string(),
                    ]),
                )
                .unwrap_or(chunk.clone())
            })
            .collect::<Vec<ChunkMetadata>>()
    } else {
        citation_chunks.clone()
    };

    citation_chunks_stringified = serde_json::to_string(&highlighted_citation_chunks)
        .expect("Failed to serialize citation chunks");
    citation_chunks_stringified1 = citation_chunks_stringified.clone();

    let rag_content = citation_chunks
        .iter()
        .enumerate()
        .map(|(idx, chunk)| format!("Doc {}: {}", idx + 1, chunk.content.clone()))
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
        stream: should_stream,
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
                citation_chunks_stringified,
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

        let _ = create_message_query(new_message, user_id, &pool).await;

        return Ok(HttpResponse::Ok().json(completion_content));
    }

    let (s, r) = unbounded::<String>();
    let stream = client.chat().create_stream(parameters).await.unwrap();

    if !citation_chunks_stringified.is_empty() {
        citation_chunks_stringified = format!("{}||", citation_chunks_stringified);
        citation_chunks_stringified1 = citation_chunks_stringified.clone();
    }

    Arbiter::new().spawn(async move {
        let chunk_v: Vec<String> = r.iter().collect();
        let completion = chunk_v.join("");

        let new_message = models::Message::from_details(
            format!("{}{}", citation_chunks_stringified, completion),
            topic_id,
            next_message_order().try_into().unwrap(),
            "assistant".to_string(),
            None,
            Some(chunk_v.len().try_into().unwrap()),
            dataset.id,
        );

        let _ = create_message_query(new_message, user_id, &pool).await;
    });

    let new_stream = stream::iter(vec![Ok(Bytes::from(citation_chunks_stringified1))]);

    Ok(HttpResponse::Ok().streaming(new_stream.chain(stream.map(
        move |response| -> Result<Bytes, actix_web::Error> {
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
        },
    ))))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct SuggestedQueriesRequest {
    /// The query to base the generated suggested queries off of.
    pub query: String,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct SuggestedQueriesResponse {
    pub queries: Vec<String>,
}

/// Generate suggested queries
///
/// This endpoint will generate 3 suggested queries based off the query provided in the request body and return them as a JSON object.
#[utoipa::path(
    post,
    path = "/chunk/gen_suggestions",
    context_path = "/api",
    tag = "chunk",
    request_body(content = SuggestedQueriesRequest, description = "JSON request payload to get alternative suggested queries", content_type = "application/json"),
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
#[tracing::instrument]
pub async fn create_suggested_queries_handler(
    data: web::Json<SuggestedQueriesRequest>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _required_user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);

    let base_url = dataset_config.LLM_BASE_URL;
    let default_model = dataset_config.LLM_DEFAULT_MODEL;

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

    let message = ChatMessage {
        role: Role::User,
        content: ChatMessageContent::Text(format!(
            "Generate 3 suggested queries based off this query a user made. Your only response should be the 3 queries which are comma seperated and are just text and you do not add any other context or information about the queries.  Here is the query: {}",
            data.query
        )),
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
    .split(',')
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
        .split(',')
        .map(|query| query.to_string().trim().trim_matches('\n').to_string())
        .collect();
    }
    Ok(HttpResponse::Ok().json(SuggestedQueriesResponse { queries }))
}
