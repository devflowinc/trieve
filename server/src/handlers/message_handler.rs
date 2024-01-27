use super::{auth_handler::LoggedUser, chunk_handler::ParsedQuery};
use crate::{
    data::models::{self, DatasetAndOrgWithSubAndPlan, ServerDatasetConfiguration},
    data::models::{ChunkMetadataWithFileData, Dataset, Pool, StripePlan},
    errors::{DefaultError, ServiceError},
    get_env,
    operators::{
        chunk_operator::{
            find_relevant_sentence, get_metadata_and_collided_chunks_from_point_ids_query,
        },
        message_operator::{
            create_message_query, create_topic_message_query, delete_message_query,
            get_message_by_sort_for_topic_query, get_messages_for_topic_query, get_topic_messages,
            user_owns_topic_query,
        },
        model_operator::create_embedding,
        organization_operator::get_message_org_count,
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
    resources::chat::{ChatCompletionParameters, ChatMessage, ChatMessageContent, Role},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_stream::StreamExt;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct CreateMessageData {
    /// The model to use for the assistant's messages. This can be any model from the model list. If no model is provided, the gryphe/mythomax-l2-13b will be used.
    pub model: Option<String>,
    /// The content of the user message to attach to the topic and then generate an assistant message in response to.
    pub new_message_content: String,
    /// The ID of the topic to attach the message to.
    pub topic_id: uuid::Uuid,
}

/// create_message
///
/// Create a message. Messages are attached to topics in order to coordinate memory of gen-AI chat sessions. We are considering refactoring this resource of the API soon. Currently, you can only send user messages. If the topic is a RAG topic then the response will include Chunks first on the stream. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information.
#[utoipa::path(
    post,
    path = "/message",
    context_path = "/api",
    tag = "message",
    request_body(content = CreateMessageData, description = "JSON request payload to create a message completion", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream, check the chat or search UI for an example how to process this"),
        (status = 400, description = "Service error relating to getting a chat completion", body = DefaultError),
    )
)]
pub async fn create_message_completion_handler(
    data: web::Json<CreateMessageData>,
    user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let message_count_pool = pool.clone();
    let message_count_org_id = dataset_org_plan_sub.organization.id;
    let org_message_count =
        web::block(move || get_message_org_count(message_count_org_id, message_count_pool))
            .await?
            .map_err(|err| ServiceError::InternalServerError(err.message.to_string()))?;

    if org_message_count
        >= dataset_org_plan_sub
            .organization
            .plan
            .unwrap_or(StripePlan::default())
            .message_count
    {
        return Ok(HttpResponse::UpgradeRequired().json(json!({
            "message": "To create more message completions, you must upgrade your plan"
        })));
    }

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
        dataset_org_plan_sub.dataset.id,
    );

    let _ = web::block(move || {
        user_owns_topic_query(user.id, topic_id, dataset_org_plan_sub.dataset.id, &pool1)
    })
    .await?
    .map_err(|_e| ServiceError::Unauthorized)?;

    // get the previous messages
    let mut previous_messages =
        web::block(move || get_topic_messages(topic_id, dataset_org_plan_sub.dataset.id, &pool2))
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
    let previous_messages = web::block(move || {
        create_topic_message_query(
            previous_messages,
            new_message,
            user.id,
            dataset_org_plan_sub.dataset.id,
            &pool3,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    stream_response(
        previous_messages,
        user.id,
        topic_id,
        create_message_data.model,
        dataset_org_plan_sub.dataset,
        pool4,
    )
    .await
}

/// get_all_messages
///
/// Get all messages for a given topic. If the topic is a RAG topic then the response will include Chunks first on each message. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information.
#[utoipa::path(
    get,
    path = "/messages/{messages_topic_id}",
    context_path = "/api",
    tag = "message",
    responses(
        (status = 200, description = "All messages relating to the topic with the given ID", body = Vec<Message>),
        (status = 400, description = "Service error relating to getting the messages", body = DefaultError),
    ),
    params(("messages_topic_id" = uuid, description = "The ID of the topic to get messages for."))
)]
pub async fn get_all_topic_messages(
    user: LoggedUser,
    messages_topic_id: web::Path<uuid::Uuid>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let second_pool = pool.clone();
    let topic_id: uuid::Uuid = messages_topic_id.into_inner();
    // check if the user owns the topic
    let _user_topic = web::block(move || {
        user_owns_topic_query(
            user.id,
            topic_id,
            dataset_org_plan_sub.dataset.id,
            &second_pool,
        )
    })
    .await?
    .map_err(|_e| ServiceError::Unauthorized)?;

    let messages = web::block(move || {
        get_messages_for_topic_query(topic_id, dataset_org_plan_sub.dataset.id, &pool)
    })
    .await?;

    match messages {
        Ok(messages) => Ok(HttpResponse::Ok().json(messages)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct RegenerateMessageData {
    /// The model to use for the assistant generative inferences. This can be any model from the model list. If no model is provided, the gryphe/mythomax-l2-13b will be used.~
    model: Option<String>,
    /// The id of the topic to regenerate the last message for.
    topic_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct EditMessageData {
    /// The model to use for the assistant generative inferences. This can be any model from the model list. If no model is provided, the gryphe/mythomax-l2-13b will be used.~
    model: Option<String>,
    /// The id of the topic to edit the message at the given sort order for.
    topic_id: uuid::Uuid,
    /// The sort order of the message to edit.
    message_sort_order: i32,
    /// The new content of the message to replace the old content with.
    new_message_content: String,
}

/// edit_message
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
        (status = 400, description = "Service error relating to getting a chat completion", body = DefaultError),
    )
)]
pub async fn edit_message_handler(
    data: web::Json<EditMessageData>,
    user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = data.topic_id;
    let message_sort_order = data.message_sort_order;
    let new_message_content = &data.new_message_content;
    let second_pool = pool.clone();
    let third_pool = pool.clone();

    let message_from_sort_order_result = web::block(move || {
        get_message_by_sort_for_topic_query(
            topic_id,
            dataset_org_plan_sub.dataset.id,
            message_sort_order,
            &pool,
        )
    })
    .await?;

    let message_id = match message_from_sort_order_result {
        Ok(message) => message.id,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    let _ = web::block(move || {
        delete_message_query(
            &user.id,
            message_id,
            topic_id,
            dataset_org_plan_sub.dataset.id,
            &second_pool,
        )
    })
    .await?;

    create_message_completion_handler(
        actix_web::web::Json(CreateMessageData {
            model: data.model.clone(),
            new_message_content: new_message_content.to_string(),
            topic_id,
        }),
        user,
        dataset_org_plan_sub,
        third_pool,
    )
    .await
}

/// regenerate_message
///
/// Regenerate the assistant response to the last user message of a topic. This will delete the last message and replace it with a new message. The response will include Chunks first on the stream if the topic is using RAG. The structure will look like `[chunks]||mesage`. See docs.trieve.ai for more information.
#[utoipa::path(
    delete,
    path = "/message",
    context_path = "/api",
    tag = "message",
    request_body(content = RegenerateMessageData, description = "JSON request payload to delete an agent message then regenerate it in a strem", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream, check the chat or search UI for an example how to process this"),
        (status = 400, description = "Service error relating to getting a chat completion", body = DefaultError),
    )
)]
pub async fn regenerate_message_handler(
    data: web::Json<RegenerateMessageData>,
    user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = data.topic_id;
    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let pool3 = pool.clone();
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let _ = web::block(move || user_owns_topic_query(user.id, topic_id, dataset_id, &pool1))
        .await?
        .map_err(|_e| ServiceError::Unauthorized)?;

    let previous_messages_result =
        web::block(move || get_topic_messages(topic_id, dataset_id, &pool2)).await?;

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
            previous_messages,
            user.id,
            topic_id,
            data.model.clone(),
            dataset_org_plan_sub.dataset,
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

    let _ =
        web::block(move || delete_message_query(&user.id, message_id, topic_id, dataset_id, &pool))
            .await?;

    stream_response(
        previous_messages_to_regenerate,
        user.id,
        topic_id,
        data.model.clone(),
        dataset_org_plan_sub.dataset,
        pool3,
    )
    .await
}

pub async fn get_topic_string(prompt: String, dataset: &Dataset) -> Result<String, DefaultError> {
    let prompt_topic_message = ChatMessage {
        role: Role::User,
        content: ChatMessageContent::Text(format!(
            "Write a 2-3 word topic name from the following prompt: {}",
            prompt
        )),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    };
    let openai_messages = vec![prompt_topic_message];
    let parameters = ChatCompletionParameters {
        model: "gryphe/mythomax-l2-13b".to_string(),
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
        response_format: None,
        tools: None,
        tool_choice: None,
        logprobs: None,
        top_logprobs: None,
        seed: None,
    };

    let llm_api_key = get_env!("LLM_API_KEY", "LLM_API_KEY should be set").into();
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());
    let base_url = dataset_config
        .LLM_BASE_URL
        .unwrap_or("https://openrouter.ai/v1".into());
    let client = Client {
        api_key: llm_api_key,
        http_client: reqwest::Client::new(),
        base_url,
    };

    let query = client
        .chat()
        .create(parameters)
        .await
        .expect("No OpenAI Completion for topic");
    let topic = match &query
        .choices
        .first()
        .expect("No response for OpenAI completion")
        .message
        .content
    {
        ChatMessageContent::Text(topic) => topic.clone(),
        _ => "".to_string(),
    };

    Ok(topic)
}

#[allow(clippy::too_many_arguments)]
pub async fn stream_response(
    messages: Vec<models::Message>,
    user_id: uuid::Uuid,
    topic_id: uuid::Uuid,
    model: Option<String>,
    dataset: Dataset,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let pool2 = pool.clone();
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone());

    let openai_messages: Vec<ChatMessage> = messages
        .iter()
        .map(|message| ChatMessage::from(message.clone()))
        .collect();

    let llm_api_key = get_env!("LLM_API_KEY", "LLM_API_KEY should be set").into();
    let base_url = dataset_config
        .LLM_BASE_URL
        .clone()
        .unwrap_or("https://openrouter.ai/v1".into());
    let client = Client {
        api_key: llm_api_key,
        http_client: reqwest::Client::new(),
        base_url,
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

    let rag_prompt = dataset_config.RAG_PROMPT.clone().unwrap_or("Write a 1-2 sentence semantic search query along the lines of a hypothetical response to: \n\n".to_string());

    // find evidence for the counter-argument
    let counter_arg_parameters = ChatCompletionParameters {
        model: model
            .clone()
            .unwrap_or("gryphe/mythomax-l2-13b".to_string()),
        messages: vec![ChatMessage {
            role: Role::User,
            content: ChatMessageContent::Text(format!(
                "{}{}",
                rag_prompt,
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

    let evidence_search_query = client
        .chat()
        .create(counter_arg_parameters)
        .await
        .expect("No OpenAI Completion for evidence search");
    let query = match &evidence_search_query
        .choices
        .first()
        .expect("No response for OpenAI completion")
        .message
        .content
    {
        ChatMessageContent::Text(query) => query.clone(),
        _ => "".to_string(),
    };
    let embedding_vector = create_embedding(query.as_str(), dataset_config.clone()).await?;

    let search_chunk_query_results = retrieve_qdrant_points_query(
        Some(embedding_vector),
        1,
        None,
        None,
        None,
        None,
        ParsedQuery {
            query: query.to_string(),
            quote_words: None,
            negated_words: None,
        },
        dataset.id,
        pool.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    let n_retrievals_to_include = dataset_config.N_RETRIEVALS_TO_INCLUDE.unwrap_or(3);

    let retrieval_chunk_ids = search_chunk_query_results
        .search_results
        .iter()
        .take(n_retrievals_to_include)
        .map(|chunk| chunk.point_id)
        .collect::<Vec<uuid::Uuid>>();

    let (metadata_chunks, _collided_chunks) = web::block(move || {
        get_metadata_and_collided_chunks_from_point_ids_query(retrieval_chunk_ids, pool2)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let citation_chunks: Vec<ChunkMetadataWithFileData> = metadata_chunks.to_vec();

    let highlighted_citation_chunks = citation_chunks
        .iter()
        .map(|chunk| {
            find_relevant_sentence(chunk.clone(), query.to_string()).unwrap_or(chunk.clone())
        })
        .collect::<Vec<ChunkMetadataWithFileData>>();

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
            "Here's my prompt. Include the document numbers that you used in square brackets at the end of the sentences that you used the docs for: {} \n\n Pretending you found it, use the following retrieved information as the basis of your response.: {}",
            match &openai_messages.last().expect("There needs to be at least 1 prior message").content {
                ChatMessageContent::Text(text) => text.clone(),
                _ => "".to_string(),
            },
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
        response_format: None,
        tools: None,
        tool_choice: None,
        logprobs: None,
        top_logprobs: None,
        seed: None,
    };

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

        let _ = create_message_query(new_message, user_id, &pool);
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

/// get_suggested_queries
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
        (status = 400, description = "Service error relating to to updating chunk, likely due to conflicting tracking_id", body = DefaultError),
    )
)]
pub async fn create_suggested_queries_handler(
    data: web::Json<SuggestedQueriesRequest>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _required_user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    let llm_api_key = get_env!("LLM_API_KEY", "LLM_API_KEY should be set").into();
    let dataset_config =
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);
    let base_url = dataset_config
        .LLM_BASE_URL
        .unwrap_or("https://openrouter.ai/v1".into());

    let client = Client {
        api_key: llm_api_key,
        http_client: reqwest::Client::new(),
        base_url,
    };
    let query = format!("generate 3 suggested queries based off this query a user made. Your only response should be the 3 queries which are comma seperated and are just text and you do not add any other context or information about the queries.  Here is the query: {}", data.query);
    let message = ChatMessage {
        role: Role::User,
        content: ChatMessageContent::Text(query),
        tool_calls: None,
        name: None,
        tool_call_id: None,
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
        response_format: None,
        tools: None,
        tool_choice: None,
        logprobs: None,
        top_logprobs: None,
        seed: None,
    };

    let mut query = client
        .chat()
        .create(parameters.clone())
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    let mut queries: Vec<String> = match &query
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
