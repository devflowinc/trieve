use crate::operators::chunk_operator::get_random_chunk_qdrant_point_id_query;
use crate::operators::message_operator::models::DatasetAndOrgWithSubAndPlan;
use itertools::Itertools;
use openai_dive::v1::models::WhisperModel;
use simple_server_timing_header::Timer;
use simsearch::SimSearch;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Arc, Mutex};

#[cfg(not(feature = "hallucination-detection"))]
use crate::data::models::DummyHallucinationScore;
use crate::data::models::{
    self, escape_quotes, ChunkMetadata, ChunkMetadataStringTagSet,
    ChunkMetadataStringTagSetWithHighlightsScore, ChunkMetadataTypes, ConditionType, Dataset,
    DatasetConfiguration, FieldCondition, LLMOptions, MultiQuery, QdrantChunkMetadata, QueryTypes,
    RagQueryEventClickhouse, Range, RangeCondition, RedisPool, ScoreChunk, SearchMethod,
    SearchModalities, SuggestType,
};
use crate::diesel::prelude::*;
use crate::get_env;
use crate::handlers::chunk_handler::SearchChunksReqPayload;
use crate::handlers::group_handler::SearchOverGroupsReqPayload;
use crate::handlers::message_handler::{CreateMessageReqPayload, SuggestedQueriesReqPayload};
use crate::operators::clickhouse_operator::ClickHouseEvent;
use crate::operators::parse_operator::{convert_html_to_text, parse_streaming_completetion};
use crate::operators::qdrant_operator::scroll_dataset_points;
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
#[cfg(feature = "hallucination-detection")]
use hallucination_detection::{HallucinationDetector, HallucinationScore};
use openai_dive::v1::endpoints::chat::RoleTrackingStream;
use openai_dive::v1::resources::audio::{AudioOutputFormat, AudioTranscriptionParametersBuilder};
use openai_dive::v1::resources::chat::{
    ChatCompletionChoice, ChatCompletionFunction, ChatCompletionTool, ChatCompletionToolType,
    ChatMessageContentPart, ChatMessageImageContentPart, ChatMessageTextContentPart,
    DeltaChatMessage, DeltaFunction, Function, ImageUrlType, ToolCall,
};
use openai_dive::v1::resources::shared::{FileUpload, FileUploadBytes, FinishReason};
use openai_dive::v1::{
    api::Client,
    resources::{
        chat::{ChatCompletionParameters, ChatMessage, ChatMessageContent},
        shared::StopToken,
    },
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use ureq::json;

use super::chunk_operator::get_chunk_metadatas_from_point_ids;
use super::clickhouse_operator::{get_latency_from_header, EventQueue};
use super::model_operator::{count_message_tokens, count_tokens};
use super::search_operator::{
    assemble_qdrant_filter, hybrid_search_over_groups, search_chunks_query, search_hybrid_chunks,
    search_over_groups_query, ParsedQuery, ParsedQueryTypes,
};

pub fn parse_text_into_docs_message(
    text: &str,
    score_chunks: Vec<ScoreChunk>,
) -> Result<(String, Vec<ScoreChunk>), ServiceError> {
    let parsed: serde_json::Value = serde_json::from_str(text).map_err(|_| {
        log::error!("Invalid JSON response when trying to fetch used documents array");
        ServiceError::BadRequest(
            "Invalid JSON response when trying to fetch used documents array".to_string(),
        )
    })?;

    let used_docs = parsed["documents"].as_array().ok_or_else(|| {
        log::error!("Missing documents array");
        ServiceError::BadRequest("Missing documents array".to_string())
    })?;

    let rag_message = parsed["message"].as_str().ok_or_else(|| {
        log::error!("Missing message");
        ServiceError::BadRequest("Missing message".to_string())
    })?;

    // Filter chunk_metadatas to only include used documents
    let filtered_chunks: Vec<_> = used_docs
        .iter()
        .filter_map(|doc_idx| {
            doc_idx
                .as_u64()
                .and_then(|idx| score_chunks.get(idx as usize - 1))
                .cloned()
        })
        .collect();

    Ok((rag_message.to_string(), filtered_chunks))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionDTO {
    pub completion_message: Message,
    pub completion_tokens: i32,
}

pub async fn get_topic_messages_query(
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

pub async fn get_message_by_id_query(
    message_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Message, ServiceError> {
    use crate::data::schema::messages::dsl as messages_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let message = messages_columns::messages
        .filter(messages_columns::id.eq(message_id))
        .filter(messages_columns::dataset_id.eq(given_dataset_id))
        .filter(messages_columns::deleted.eq(false))
        .first::<Message>(&mut conn)
        .await
        .map_err(|db_error| {
            log::error!("Error getting message by id {:?}", db_error);
            ServiceError::BadRequest("Error getting message by id".to_string())
        })?;

    Ok(message)
}

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

const AGENTIC_SEARCH_SYSTEM_PROMPT: &str = r#"
You are a smart assistant with access to a search tool that can query a knowledge base for accurate and relevant information. Use this tool when you need to find facts, product details, or pricing information to answer the user's question correctly.
How to Use the Search Tool:
When using the search tool, you must provide:
    query: A string describing what you're looking for (e.g. "wireless headphones with noise cancellation").
    price_filter (optional): An object that limits results based on price range.

Example with price filter:
If a user asks:
“Find laptops under $1000”

Your query should be:
{
  "query": "laptops",
  "price_filter": {
    "min": null,
    "max": 1000
  }
}

You can also set both:
{
  "query": "4K TVs",
  "price_filter": {
    "min": 500,
    "max": 1500
  }
}

If price isn't relevant, you can leave the filter empty or set both min and max to null.
Search Process:
Start with a simple query, with or without a price filter.
Refine your query using the results if you don’t get enough useful info.
Repeat until you're confident you can answer well.

Important:
If you use the search tool, you must use the chunks_used tool to select the relevant content before writing your reply.
Your response must be based only on the selected chunks.
You can use the search tool multiple times to gather more context.
Your job is to be accurate, helpful, and complete—based only on what you find. Be smart, be curious, and use price filters when needed to meet the user's needs.
"#;

const STRUCTURE_SYSTEM_PROMPT: &str = r#"
Before you start generating respond with the documents that you plan to use to generate your response, YOU MUST INCLUDE AT LEAST 1.
YOU MUST DO THIS BEFORE YOU CONTINUE TO GENERATE A RESPONSE.
After responding with the documents, YOU MUST RESPOND TO THE USERS PROMPT.
```
Example:
User: 
Here's my prompt: what about for spreadsheets \n\n Use the following retrieved documents to respond briefly and accurately: {"doc": 1, "text": "chunk text..", "link": "chunk link.." }\n\n{"doc": 2, "text": "chunk text..", "link": "chunk link.." }... etc
Assistant:
documents: [1,2]
...continue with model response
```
After you have done these things now follow:
"#;

pub async fn create_generic_system_message(
    system_prompt: String,
    use_agentic_search: bool,
    only_include_docs_used: bool,
    messages_topic_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Message, ServiceError> {
    let topic =
        crate::operators::topic_operator::get_topic_query(messages_topic_id, dataset_id, pool)
            .await?;

    let mut system_prompt = if use_agentic_search {
        format!("{}\n\n{}", AGENTIC_SEARCH_SYSTEM_PROMPT, system_prompt)
    } else {
        system_prompt
    };

    if only_include_docs_used {
        system_prompt = format!("{}\n\n{}", STRUCTURE_SYSTEM_PROMPT, system_prompt);
    }

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

pub async fn create_topic_message_query(
    config: &DatasetConfiguration,
    previous_messages: Vec<Message>,
    use_agentic_search: bool,
    only_include_docs_used: bool,
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
            use_agentic_search,
            only_include_docs_used,
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
pub async fn get_rag_chunks_query(
    create_message_req_payload: CreateMessageReqPayload,
    dataset_config: DatasetConfiguration,
    dataset: Dataset,
    user_message_query: String,
    chosen_model: String,
    client: &Client,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    event_queue: web::Data<EventQueue>,
) -> Result<(SearchQueryEventClickhouse, Vec<ScoreChunk>), actix_web::Error> {
    let mut query =
        if let Some(create_message_query) = create_message_req_payload.search_query.clone() {
            create_message_query
        } else {
            user_message_query
        };

    let use_message_to_query_prompt = dataset_config.USE_MESSAGE_TO_QUERY_PROMPT;
    if create_message_req_payload.search_query.is_none() && use_message_to_query_prompt {
        let message_to_query_prompt = dataset_config.MESSAGE_TO_QUERY_PROMPT.clone();
        let mut gen_inference_msgs = vec![ChatMessage::User {
            content: ChatMessageContent::Text(format!("{}\n{}", message_to_query_prompt, query)),
            name: None,
        }];

        if let Some(ref image_urls) = create_message_req_payload.image_urls {
            if !image_urls.is_empty() {
                gen_inference_msgs.push(ChatMessage::User {
                    name: None,
                    content: ChatMessageContent::ContentPart(
                        image_urls
                            .iter()
                            .map(|url| {
                                ChatMessageContentPart::Image(ChatMessageImageContentPart {
                                    r#type: "image_url".to_string(),
                                    image_url: ImageUrlType {
                                        url: url.to_string(),
                                        detail: None,
                                    },
                                })
                            })
                            .collect::<Vec<_>>(),
                    ),
                });
                gen_inference_msgs.push(ChatMessage::User {
                    name: None,
                    content: ChatMessageContent::Text(
                        "These are the images that the user provided with their query. Use them to create your search query".to_string(),
                    ),
                });
            }
        }

        let gen_inference_parameters = ChatCompletionParameters {
            model: chosen_model.clone(),
            messages: gen_inference_msgs,
            stream: Some(false),
            temperature: dataset_config.TEMPERATURE.map(|temp| temp as f32),
            frequency_penalty: dataset_config.FREQUENCY_PENALTY.map(|pen| pen as f32),
            presence_penalty: dataset_config.PRESENCE_PENALTY.map(|pen| pen as f32),
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

        let search_query_from_message_to_query_prompt =
            match client.chat().create(gen_inference_parameters).await {
                Ok(query) => query,
                Err(err) => {
                    log::error!(
                        "Error getting LLM completion for message to query prompt {:?}",
                        err
                    );
                    return Err(actix_web::error::ErrorInternalServerError(
                        "Error getting LLM completion for message to query prompt",
                    ));
                }
            };

        query = match &search_query_from_message_to_query_prompt
            .choices
            .get(0)
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
            _ => query,
        };
    }

    let n_retrievals_to_include = dataset_config.N_RETRIEVALS_TO_INCLUDE;
    let search_type = create_message_req_payload
        .search_type
        .unwrap_or(SearchMethod::Hybrid);

    let query_type =
        if let Some(ref image_urls) = create_message_req_payload.image_urls.and_then(|x| {
            if x.is_empty() {
                None
            } else {
                Some(x)
            }
        }) {
            let image_queries = image_urls.iter().map(|url| MultiQuery {
                query: SearchModalities::Image {
                    image_url: url.clone(),
                    llm_prompt: None,
                },
                weight: 0.5 / image_urls.len() as f32,
            });

            QueryTypes::Multi(
                vec![MultiQuery {
                    query: SearchModalities::Text(query.clone()),
                    weight: 0.5,
                }]
                .into_iter()
                .chain(image_queries)
                .collect(),
            )
        } else {
            QueryTypes::Single(SearchModalities::Text(query.clone()))
        };

    if create_message_req_payload
        .use_group_search
        .is_some_and(|x| x)
    {
        let search_groups_data = SearchOverGroupsReqPayload {
            search_type: search_type.clone(),
            query: query_type.clone(),
            score_threshold: create_message_req_payload.score_threshold,
            page_size: Some(
                create_message_req_payload
                    .page_size
                    .unwrap_or(n_retrievals_to_include.try_into().unwrap_or(8)),
            ),
            sort_options: create_message_req_payload.sort_options,
            highlight_options: create_message_req_payload.highlight_options,
            filters: create_message_req_payload.filters,
            group_size: Some(1),
            use_quote_negated_terms: create_message_req_payload.use_quote_negated_terms,
            remove_stop_words: create_message_req_payload.remove_stop_words,
            typo_options: create_message_req_payload.typo_options,
            ..Default::default()
        };

        let parsed_query = ParsedQuery {
            query: query.clone(),
            quote_words: None,
            negated_words: None,
        };

        let mut search_timer = Timer::new();

        let result_groups = match search_type {
            SearchMethod::Hybrid => {
                hybrid_search_over_groups(
                    search_groups_data.clone(),
                    parsed_query,
                    pool.clone(),
                    redis_pool,
                    dataset.clone(),
                    &dataset_config,
                    &mut search_timer,
                )
                .await?
            }
            _ => {
                search_over_groups_query(
                    search_groups_data.clone(),
                    ParsedQueryTypes::Single(parsed_query),
                    pool.clone(),
                    redis_pool,
                    dataset.clone(),
                    &dataset_config,
                    &mut search_timer,
                )
                .await?
            }
        };

        let clickhouse_search_event = SearchQueryEventClickhouse {
            request_params: serde_json::to_string(&search_groups_data.clone()).unwrap_or_default(),
            id: uuid::Uuid::new_v4(),
            search_type: "rag_groups".to_string(),
            tokens: count_tokens(&query),
            query: query.clone(),
            dataset_id: dataset.id,
            organization_id: dataset.organization_id,
            metadata: serde_json::to_string(&create_message_req_payload.metadata.clone())
                .unwrap_or_default(),
            top_score: result_groups
                .group_chunks
                .get(0)
                .map(|x| x.metadata.get(0).map(|y| y.score as f32).unwrap_or(0.0))
                .unwrap_or(0.0),
            latency: get_latency_from_header(search_timer.header_value()),
            results: result_groups
                .group_chunks
                .clone()
                .into_iter()
                .map(|x| {
                    let mut json = serde_json::to_value(&x).unwrap_or_default();
                    escape_quotes(&mut json);
                    json.to_string()
                })
                .collect(),
            created_at: time::OffsetDateTime::now_utc(),
            query_rating: String::from(""),
            user_id: create_message_req_payload
                .user_id
                .clone()
                .unwrap_or_default(),
        };
        if !dataset_config.DISABLE_ANALYTICS {
            event_queue
                .send(ClickHouseEvent::SearchQueryEvent(
                    clickhouse_search_event.clone(),
                ))
                .await;
        }
        Ok((
            clickhouse_search_event,
            result_groups
                .group_chunks
                .into_iter()
                .flat_map(|group_score_chunk| {
                    group_score_chunk
                        .metadata
                        .into_iter()
                        .map(ScoreChunk::from)
                        .collect::<Vec<ScoreChunk>>()
                })
                .collect::<Vec<ScoreChunk>>(),
        ))
    } else {
        let search_chunk_data = SearchChunksReqPayload {
            search_type: search_type.clone(),
            query: query_type.clone(),
            score_threshold: create_message_req_payload.score_threshold,
            sort_options: create_message_req_payload.sort_options,
            page_size: Some(
                create_message_req_payload
                    .page_size
                    .unwrap_or(n_retrievals_to_include.try_into().unwrap_or(8)),
            ),
            highlight_options: create_message_req_payload.highlight_options,
            filters: create_message_req_payload.filters,
            use_quote_negated_terms: create_message_req_payload.use_quote_negated_terms,
            remove_stop_words: create_message_req_payload.remove_stop_words,
            typo_options: create_message_req_payload.typo_options,
            ..Default::default()
        };
        let parsed_query = ParsedQuery {
            query: query.clone(),
            quote_words: None,
            negated_words: None,
        };
        let mut search_timer = Timer::new();

        let result_chunks = match search_type {
            SearchMethod::Hybrid => {
                search_hybrid_chunks(
                    search_chunk_data.clone(),
                    parsed_query,
                    pool.clone(),
                    redis_pool,
                    dataset.clone(),
                    &dataset_config,
                    &mut search_timer,
                )
                .await?
            }
            _ => {
                search_chunks_query(
                    search_chunk_data.clone(),
                    ParsedQueryTypes::Single(parsed_query),
                    pool.clone(),
                    redis_pool,
                    dataset.clone(),
                    &dataset_config,
                    &mut search_timer,
                )
                .await?
            }
        };

        let clickhouse_search_event = SearchQueryEventClickhouse {
            request_params: serde_json::to_string(&search_chunk_data.clone()).unwrap_or_default(),
            id: uuid::Uuid::new_v4(),
            search_type: "rag_chunks".to_string(),
            tokens: count_tokens(&query),
            query: query.clone(),
            dataset_id: dataset.id,
            organization_id: dataset.organization_id,
            top_score: result_chunks
                .score_chunks
                .get(0)
                .map(|x| x.score as f32)
                .unwrap_or(0.0),
            latency: get_latency_from_header(search_timer.header_value()),
            metadata: serde_json::to_string(&create_message_req_payload.metadata.clone())
                .unwrap_or_default(),
            results: result_chunks
                .score_chunks
                .clone()
                .into_iter()
                .map(|x| {
                    let mut json = serde_json::to_value(&x).unwrap_or_default();
                    escape_quotes(&mut json);
                    json.to_string()
                })
                .collect(),
            created_at: time::OffsetDateTime::now_utc(),
            query_rating: String::from(""),
            user_id: create_message_req_payload
                .user_id
                .clone()
                .unwrap_or_default(),
        };
        if !dataset_config.DISABLE_ANALYTICS {
            event_queue
                .send(ClickHouseEvent::SearchQueryEvent(
                    clickhouse_search_event.clone(),
                ))
                .await;
        }
        Ok((
            clickhouse_search_event,
            result_chunks
                .score_chunks
                .into_iter()
                .map(ScoreChunk::from)
                .collect::<Vec<ScoreChunk>>(),
        ))
    }
}

pub fn clean_markdown(markdown_text: &str) -> String {
    let mut text = markdown_text.to_string();

    let patterns = [
        // Code blocks (both ``` and indented)
        (r"```[\s\S]*?```", ""),
        (r"(?m)^( {4,}|\t+)[^\n]+", ""),
        // Headers
        (r"(?m)^#{1,6}\s+", ""),
        // Emphasis (bold, italic)
        (r"\*\*(.+?)\*\*", "$1"), // Bold
        (r"__(.+?)__", "$1"),     // Bold
        (r"\*(.+?)\*", "$1"),     // Italic
        (r"_(.+?)_", "$1"),       // Italic
        // Inline code
        (r"`([^`]+)`", "$1"),
        // Blockquotes
        (r"(?m)^\s*>\s+", ""),
        // Horizontal rules
        (r"\n[\*\-_]{3,}\n", "\n"),
        // Lists
        (r"(?m)^\s*[\*\-+]\s+", ""), // Unordered lists
        (r"(?m)^\s*\d+\.\s+", ""),   // Ordered lists
        // Links and images
        (r"\[([^\]]+)\]\([^\)]+\)", "$1"), // [text](url)
        (r"\[([^\]]+)\]\[[^\]]*\]", "$1"), // [text][reference]
        (r"!\[([^\]]*)\]\([^\)]+\)", ""),  // Images
        // Reference-style links
        (r"(?m)^\s*\[[^\]]+\]:\s+[^\s]+\s*$", ""),
        // Clean up whitespace
        (r"\n\s*\n", "\n\n"),
    ];

    // Apply all patterns
    for (pattern, replacement) in patterns.iter() {
        if let Ok(regex) = Regex::new(pattern) {
            text = regex.replace_all(&text, *replacement).to_string();
        }
    }

    // Final cleanup
    text.trim().to_string()
}

#[allow(clippy::too_many_arguments)]
pub async fn stream_response(
    messages: Vec<models::Message>,
    topic_id: uuid::Uuid,
    dataset: Dataset,
    pool: web::Data<Pool>,
    event_queue: web::Data<EventQueue>,
    redis_pool: web::Data<RedisPool>,
    dataset_config: DatasetConfiguration,
    create_message_req_payload: CreateMessageReqPayload,
    #[cfg(feature = "hallucination-detection")] hallucination_detector: web::Data<
        HallucinationDetector,
    >,
) -> Result<HttpResponse, actix_web::Error> {
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

    let llm_api_key = get_llm_api_key(&dataset_config);

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
    let chosen_model = create_message_req_payload
        .model
        .clone()
        .unwrap_or(dataset_config.LLM_DEFAULT_MODEL.clone());

    let (search_event, score_chunks) = get_rag_chunks_query(
        create_message_req_payload.clone(),
        dataset_config.clone(),
        dataset.clone(),
        user_message_query.clone(),
        chosen_model.clone(),
        &client,
        pool.clone(),
        redis_pool.clone(),
        event_queue.clone(),
    )
    .await?;

    if score_chunks.is_empty() {
        let response_stream = stream::iter(vec![Ok::<actix_web::web::Bytes, actix_web::Error>(
            Bytes::from(format!(
                "[]||{}",
                create_message_req_payload.no_result_message.unwrap_or(
                    "I was not able to find any relevant information to answer your query."
                        .to_string(),
                )
            )),
        )]);
        return Ok(HttpResponse::Ok()
            .insert_header(("TR-QueryID", search_event.id.to_string()))
            .streaming(response_stream));
    }

    let rag_content = match create_message_req_payload.rag_context {
        Some(rag_context) => rag_context,
        None => {
            score_chunks
            .iter()
            .enumerate()
            .map(|(idx, score_chunk)| {
                json!({
                    "doc": idx + 1,
                    "text": convert_html_to_text(&(ChunkMetadata::from(score_chunk.chunk.clone()).chunk_html.clone().unwrap_or_default())),
                    "num_value": ChunkMetadata::from(score_chunk.chunk.clone()).num_value.map(|x| format!("{} {}", create_message_req_payload.currency.clone().unwrap_or("".to_string()), x)).unwrap_or("".to_string()),
                    "link": ChunkMetadata::from(score_chunk.chunk.clone()).link.clone().unwrap_or_default()
                })
                .to_string()
            })
            .collect::<Vec<String>>()
            .join("\n\n")
        }
    };

    let user_message = match &openai_messages
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
    };

    let last_message = ChatMessageContent::Text(format!(
        "Here's my prompt: {} \n\n {} {}",
        user_message.clone(),
        rag_prompt,
        rag_content,
    ));

    let images: Vec<String> = score_chunks
        .iter()
        .filter_map(|score_chunk| {
            ChunkMetadata::from(score_chunk.chunk.clone())
                .image_urls
                .clone()
        })
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
                        audio: None,
                        reasoning_content: None,
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

    if let Some(image_urls) = create_message_req_payload.image_urls.clone() {
        if !image_urls.is_empty() {
            open_ai_messages.push(ChatMessage::User {
                name: None,
                content: ChatMessageContent::ContentPart(
                    image_urls
                        .iter()
                        .map(|url| {
                            ChatMessageContentPart::Image(ChatMessageImageContentPart {
                                r#type: "image_url".to_string(),
                                image_url: ImageUrlType {
                                    url: url.to_string(),
                                    detail: None,
                                },
                            })
                        })
                        .chain(std::iter::once(ChatMessageContentPart::Text(
                            ChatMessageTextContentPart {
                                r#type: "text".to_string(),
                                text:
                                    "These are the images that the user provided with their query."
                                        .to_string(),
                            },
                        )))
                        .collect::<Vec<_>>(),
                ),
            });
        }
    }

    if !images.is_empty() {
        if let Some(LLMOptions {
            image_config: Some(ref image_config),
            ..
        }) = create_message_req_payload.llm_options
        {
            if image_config.use_images.unwrap_or(false) {
                open_ai_messages.push(ChatMessage::User {
                    name: None,
                    content: ChatMessageContent::ContentPart(
                        images
                            .iter()
                            .take(image_config.images_per_chunk.unwrap_or(5))
                            .map(|url| {
                                ChatMessageContentPart::Image(ChatMessageImageContentPart {
                                    r#type: "image_url".to_string(),
                                    image_url: ImageUrlType {
                                        url: url.to_string(),
                                        detail: None,
                                    },
                                })
                            })
                            .collect::<Vec<_>>(),
                    ),
                })
            }
        }
    }

    let mut parameters = ChatCompletionParameters {
        model: chosen_model,
        messages: open_ai_messages.clone(),
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

    let query_id = uuid::Uuid::new_v4();

    if create_message_req_payload
        .llm_options
        .as_ref()
        .is_some_and(|llm_options| !llm_options.stream_response.unwrap_or(true))
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
            Some(ChatMessage::Assistant {
                content: Some(ChatMessageContent::Text(text)),
                ..
            }) => {
                if create_message_req_payload
                    .only_include_docs_used
                    .unwrap_or(false)
                {
                    match parse_text_into_docs_message(text, score_chunks.clone()) {
                        Ok((response_text, filtered_chunks)) => (response_text, filtered_chunks),
                        Err(_) => (text.clone(), vec![]),
                    }
                } else {
                    (text.clone(), score_chunks.clone())
                }
            }
            _ => {
                return Err(ServiceError::BadRequest("Invalid response format, did not receive text on the assistant message from the LLM provider".to_string()).into())
            }
        };

        let (response_text, filtered_chunks) = completion_content;

        let filtered_chunks_stringified = serde_json::to_string(&filtered_chunks)
            .expect("Failed to serialize filtered citation chunks");

        let final_response = if create_message_req_payload
            .llm_options
            .as_ref()
            .map(|x| x.completion_first)
            .unwrap_or(Some(false))
            .unwrap_or(false)
        {
            format!(
                "{}||{}",
                response_text,
                filtered_chunks_stringified.replace("||", "")
            )
        } else {
            format!(
                "{}||{}",
                filtered_chunks_stringified.replace("||", ""),
                response_text
            )
        };

        let new_message = models::Message::from_details(
            final_response.clone(),
            topic_id,
            next_message_order()
                .try_into()
                .expect("usize to i32 conversion should always succeed"),
            "assistant".to_string(),
            assistant_completion
                .usage
                .as_ref()
                .map(|usg| usg.prompt_tokens.unwrap_or(0) as i32),
            assistant_completion
                .usage
                .map(|usg| usg.completion_tokens.unwrap_or(0) as i32),
            dataset.id,
            query_id,
        );

        #[cfg(feature = "hallucination-detection")]
        let score = {
            let docs = filtered_chunks
                .iter()
                .map(|score_chunk| {
                    ChunkMetadata::from(score_chunk.chunk.clone())
                        .chunk_html
                        .clone()
                        .unwrap_or_default()
                })
                .collect::<Vec<String>>();
            hallucination_detector
                .detect_hallucinations(&clean_markdown(&response_text), &docs)
                .await
                .map_err(|err| {
                    ServiceError::BadRequest(format!("Failed to detect hallucinations: {}", err))
                })?
        };

        #[cfg(not(feature = "hallucination-detection"))]
        let score = DummyHallucinationScore {
            total_score: 0.0,
            detected_hallucinations: vec![],
        };

        let filtered_chunks_data = filtered_chunks
            .iter()
            .map(|x| {
                let mut json = serde_json::to_value(x).unwrap_or_default();
                escape_quotes(&mut json);
                json.to_string()
            })
            .collect::<Vec<String>>();

        let clickhouse_rag_event = RagQueryEventClickhouse {
            id: query_id,
            created_at: time::OffsetDateTime::now_utc(),
            dataset_id: dataset.id,
            organization_id: dataset.organization_id,
            search_id: search_event.id,
            top_score: search_event.top_score,
            results: vec![],
            topic_id,
            metadata: search_event.metadata,
            json_results: filtered_chunks_data,
            user_message: user_message_query.clone(),
            query_rating: String::new(),
            rag_type: "all_chunks".to_string(),
            llm_response: response_text.clone(),
            user_id: create_message_req_payload
                .user_id
                .clone()
                .unwrap_or_default(),
            hallucination_score: score.total_score,
            detected_hallucinations: score.detected_hallucinations,
            tokens: count_message_tokens(open_ai_messages) + count_tokens(&response_text),
        };

        if !dataset_config.DISABLE_ANALYTICS {
            event_queue
                .send(ClickHouseEvent::RagQueryEvent(clickhouse_rag_event.clone()))
                .await;
        }
        create_messages_query(vec![new_message], &pool).await?;
        if create_message_req_payload.audio_input.is_some() {
            return Ok(HttpResponse::Ok()
                .insert_header((
                    "X-TR-Query",
                    user_message
                        .to_string()
                        .replace(|c: char| c.is_ascii_control(), ""),
                ))
                .insert_header(("TR-QueryID", query_id.to_string().replace("\n", "")))
                .json(final_response));
        } else {
            return Ok(HttpResponse::Ok()
                .insert_header(("TR-QueryID", query_id.to_string().replace("\n", "")))
                .json(final_response));
        }
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

        let mut split_completion = completion.split("||");

        #[allow(unused_variables)]
        let (response, chunks) = if completion_first {
            let response = split_completion.next().unwrap_or_default().to_string();
            let chunk_data: Vec<ChunkMetadataStringTagSet> =
                serde_json::from_str(split_completion.next().unwrap_or_default())
                    .unwrap_or_default();

            (response, chunk_data)
        } else {
            let chunk_data: Vec<ChunkMetadataStringTagSet> =
                serde_json::from_str(split_completion.next().unwrap_or_default())
                    .unwrap_or_default();

            let response = split_completion.next().unwrap_or_default().to_string();

            (response, chunk_data)
        };

        let chunk_data: Vec<String> = chunks
            .iter()
            .map(|x| {
                let mut json = serde_json::to_value(x).unwrap_or_default();
                escape_quotes(&mut json);
                json.to_string()
            })
            .collect();

        let new_message = models::Message::from_details(
            completion.clone(),
            topic_id,
            next_message_order().try_into().unwrap(),
            "assistant".to_string(),
            None,
            Some(chunk_v.len().try_into().unwrap()),
            dataset.id,
            query_id_arb,
        );

        if !dataset_config.DISABLE_ANALYTICS {
            #[cfg(feature = "hallucination-detection")]
            let score = {
                let docs = chunks
                    .iter()
                    .map(|x| x.chunk_html.clone().unwrap_or_default())
                    .collect::<Vec<String>>();

                hallucination_detector
                    .detect_hallucinations(&clean_markdown(&response), &docs)
                    .await
                    .unwrap_or(HallucinationScore {
                        total_score: 0.0,
                        proper_noun_score: 0.0,
                        number_mismatch_score: 0.0,
                        unknown_word_score: 0.0,
                        detected_hallucinations: vec![],
                    })
            };

            #[cfg(not(feature = "hallucination-detection"))]
            let score = DummyHallucinationScore {
                total_score: 0.0,
                detected_hallucinations: vec![],
            };

            let clickhouse_rag_event = RagQueryEventClickhouse {
                id: query_id_arb,
                created_at: time::OffsetDateTime::now_utc(),
                search_id: search_event.id,
                top_score: search_event.top_score,
                dataset_id: dataset.id,
                topic_id,
                results: vec![],
                json_results: chunk_data,
                user_message: user_message_query.clone(),
                metadata: search_event.metadata,
                query_rating: String::new(),
                rag_type: "all_chunks".to_string(),
                llm_response: completion.clone(),
                user_id: create_message_req_payload
                    .user_id
                    .clone()
                    .unwrap_or_default(),
                hallucination_score: score.total_score,
                detected_hallucinations: score.detected_hallucinations,
                tokens: count_message_tokens(open_ai_messages) + count_tokens(&completion),
                organization_id: dataset.organization_id,
            };

            event_queue
                .send(ClickHouseEvent::RagQueryEvent(clickhouse_rag_event.clone()))
                .await;
        }
        let _ = create_messages_query(vec![new_message], &pool).await;
    });

    let chat_completion_timeout = std::env::var("CHAT_COMPLETION_TIMEOUT_SECS")
        .unwrap_or("120".to_string())
        .parse::<u64>()
        .unwrap_or(120);

    let state = Arc::new(AtomicU16::new(0));
    let documents = if create_message_req_payload
        .only_include_docs_used
        .unwrap_or(false)
    {
        Arc::new(Mutex::new(vec![]))
    } else {
        Arc::new(Mutex::new((0..score_chunks.len() as u32).collect()))
    };
    let started_parsing_completion = AtomicBool::new(false);
    let mut bail_on_parsing = AtomicBool::new(false);

    let completion_stream = stream
        .take_until(tokio::time::sleep(std::time::Duration::from_secs(chat_completion_timeout)))
        .map(move |response| -> Result<Bytes, actix_web::Error> {
        if let Ok(response) = response {
            let chat_content = response
                .choices
                .get(0)
                .map(|choice| {
                    if choice.finish_reason.is_some() {
                        let docs = documents.lock().unwrap();
                        if !docs.is_empty() && completion_first {
                            let filtered_chunks = score_chunks.iter().enumerate()
                                    .filter_map(|(idx, score_chunk)| {
                                        if docs.contains(&(idx as u32)) {
                                            Some(ChunkMetadataStringTagSetWithHighlightsScore::from(score_chunk.clone()))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<ChunkMetadataStringTagSetWithHighlightsScore>>();
                                Some(format!("||{}", serde_json::to_string(&filtered_chunks).unwrap_or_default().replace("||", "")))
                        } else {
                            Some("".to_string())
                        }
                    } else {
                        match &choice.delta {
                            DeltaChatMessage::Assistant {
                                content: Some(ChatMessageContent::Text(text)),
                                ..
                            }
                            | DeltaChatMessage::Untagged {
                                content: Some(ChatMessageContent::Text(text)),
                                ..
                            } => {
                                if create_message_req_payload
                                    .only_include_docs_used
                                    .unwrap_or(false) {
                                        let bailed_on_iter = bail_on_parsing.get_mut();
                                        let (text, docs) = if !*bailed_on_iter {
                                            let (parsed_text, docs, bail) = parse_streaming_completetion(text, state.clone(), documents.clone());
                                            if bail {
                                                *bailed_on_iter = true;
                                                documents.lock().unwrap().extend(0..score_chunks.len() as u32);
                                                (Some(text.clone()), Some((0..score_chunks.len() as u32).collect()))
                                            } else {
                                                (parsed_text, docs)
                                            }
                                        } else {
                                            (Some(text.clone()), None)
                                        };

                                        if let Some(docs) = docs {
                                            if !completion_first {
                                                let filtered_chunks = score_chunks.iter().enumerate().filter_map(|(idx, score_chunk)| {
                                                    if docs.contains(&(idx as u32)) {
                                                        Some(ChunkMetadataStringTagSetWithHighlightsScore::from(score_chunk.clone()))
                                                    } else {
                                                        None
                                                    }
                                                }).collect::<Vec<ChunkMetadataStringTagSetWithHighlightsScore>>();
                                                if *bailed_on_iter {
                                                    return Some(format!("{}||{}", serde_json::to_string(&filtered_chunks).unwrap_or_default().replace("||", ""), text.unwrap_or("".to_string())));
                                                } else {
                                                    return Some(format!("{}||", serde_json::to_string(&filtered_chunks).unwrap_or_default().replace("||", "")));
                                                }
                                            }
                                        }
                                        text.clone()
                                    } else if !completion_first && !started_parsing_completion.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |_| Some(true)).unwrap_or(true) {
                                        let returned_chunks = score_chunks.iter().map(|score_chunk| {
                                            ChunkMetadataStringTagSetWithHighlightsScore::from(score_chunk.clone())
                                        }).collect::<Vec<ChunkMetadataStringTagSetWithHighlightsScore>>();
                                        return Some(format!("{}||", serde_json::to_string(&returned_chunks).unwrap_or_default().replace("||", "")));
                                    } else {
                                        Some(text.clone())
                                    }
                            },
                            _ => {
                                log::error!("Delta of first choice did not have text or was either Tool or Function {:?}", choice);
                                None
                            },
                        }
                    }
                })
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

    if create_message_req_payload.audio_input.is_some() {
        Ok(HttpResponse::Ok()
            .insert_header((
                "X-TR-Query",
                user_message
                    .to_string()
                    .replace(|c: char| c.is_ascii_control(), ""),
            ))
            .insert_header(("TR-QueryID", query_id.to_string().replace("\n", "")))
            .streaming(completion_stream))
    } else {
        Ok(HttpResponse::Ok()
            .insert_header(("TR-QueryID", query_id.to_string().replace("\n", "")))
            .streaming(completion_stream))
    }
}

#[derive(Deserialize, Debug)]
struct PriceFilter {
    min: Option<f32>,
    max: Option<f32>,
}
#[derive(Deserialize, Debug, Default)]
struct SearchParams {
    query: String,
    price_filter: Option<PriceFilter>,
}

async fn search_chunks(
    create_message_req_payload: CreateMessageReqPayload,
    search_params: SearchParams,
    dataset: Dataset,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset_config: DatasetConfiguration,
    event_queue: web::Data<EventQueue>,
) -> Result<(SearchQueryEventClickhouse, Vec<ScoreChunk>), ServiceError> {
    let query = search_params.query;
    let price_filter = search_params.price_filter;

    let filters = create_message_req_payload.filters.clone();
    if let Some(price_filter) = price_filter {
        filters.map(|filter| {
            filter.must.map(|mut must| {
                must.push(ConditionType::Field(FieldCondition {
                    field: "num_value".to_string(),
                    range: Some(Range {
                        gte: price_filter.min.map(|x| RangeCondition::Float(x as f64)),
                        lte: price_filter.max.map(|x| RangeCondition::Float(x as f64)),
                        ..Default::default()
                    }),
                    ..Default::default()
                }))
            })
        });
    }

    let search_type = create_message_req_payload
        .search_type
        .unwrap_or(SearchMethod::Hybrid);

    let query_type =
        if let Some(ref image_urls) = create_message_req_payload.image_urls.and_then(|x| {
            if x.is_empty() {
                None
            } else {
                Some(x)
            }
        }) {
            let image_queries = image_urls.iter().map(|url| MultiQuery {
                query: SearchModalities::Image {
                    image_url: url.clone(),
                    llm_prompt: None,
                },
                weight: 0.5 / image_urls.len() as f32,
            });

            QueryTypes::Multi(
                vec![MultiQuery {
                    query: SearchModalities::Text(query.clone()),
                    weight: 0.5,
                }]
                .into_iter()
                .chain(image_queries)
                .collect(),
            )
        } else {
            QueryTypes::Single(SearchModalities::Text(query.clone()))
        };

    if create_message_req_payload
        .use_group_search
        .is_some_and(|x| x)
    {
        let search_groups_data = SearchOverGroupsReqPayload {
            search_type: search_type.clone(),
            query: query_type.clone(),
            score_threshold: create_message_req_payload.score_threshold,
            page_size: Some(10),
            sort_options: create_message_req_payload.sort_options,
            highlight_options: create_message_req_payload.highlight_options,
            filters: create_message_req_payload.filters,
            group_size: Some(1),
            use_quote_negated_terms: create_message_req_payload.use_quote_negated_terms,
            remove_stop_words: create_message_req_payload.remove_stop_words,
            typo_options: create_message_req_payload.typo_options,
            ..Default::default()
        };

        let parsed_query = ParsedQuery {
            query: query.clone(),
            quote_words: None,
            negated_words: None,
        };

        let mut search_timer = Timer::new();

        let result_groups = match search_type {
            SearchMethod::Hybrid => {
                hybrid_search_over_groups(
                    search_groups_data.clone(),
                    parsed_query,
                    pool.clone(),
                    redis_pool,
                    dataset.clone(),
                    &dataset_config,
                    &mut search_timer,
                )
                .await?
            }
            _ => {
                search_over_groups_query(
                    search_groups_data.clone(),
                    ParsedQueryTypes::Single(parsed_query),
                    pool.clone(),
                    redis_pool,
                    dataset.clone(),
                    &dataset_config,
                    &mut search_timer,
                )
                .await?
            }
        };

        let clickhouse_search_event = SearchQueryEventClickhouse {
            request_params: serde_json::to_string(&search_groups_data.clone()).unwrap_or_default(),
            id: uuid::Uuid::new_v4(),
            search_type: "rag_groups".to_string(),
            tokens: count_tokens(&query),
            query: query.clone(),
            dataset_id: dataset.id,
            organization_id: dataset.organization_id,
            metadata: serde_json::to_string(&create_message_req_payload.metadata.clone())
                .unwrap_or_default(),
            top_score: result_groups
                .group_chunks
                .get(0)
                .map(|x| x.metadata.get(0).map(|y| y.score as f32).unwrap_or(0.0))
                .unwrap_or(0.0),
            latency: get_latency_from_header(search_timer.header_value()),
            results: result_groups
                .group_chunks
                .clone()
                .into_iter()
                .map(|x| {
                    let mut json = serde_json::to_value(&x).unwrap_or_default();
                    escape_quotes(&mut json);
                    json.to_string()
                })
                .collect(),
            created_at: time::OffsetDateTime::now_utc(),
            query_rating: String::from(""),
            user_id: create_message_req_payload
                .user_id
                .clone()
                .unwrap_or_default(),
        };
        if !dataset_config.DISABLE_ANALYTICS {
            event_queue
                .send(ClickHouseEvent::SearchQueryEvent(
                    clickhouse_search_event.clone(),
                ))
                .await;
        }

        Ok((
            clickhouse_search_event,
            result_groups
                .group_chunks
                .into_iter()
                .flat_map(|group_score_chunk| {
                    group_score_chunk
                        .metadata
                        .into_iter()
                        .map(ScoreChunk::from)
                        .collect::<Vec<ScoreChunk>>()
                })
                .collect::<Vec<ScoreChunk>>(),
        ))
    } else {
        let search_chunk_data = SearchChunksReqPayload {
            search_type: search_type.clone(),
            query: query_type.clone(),
            score_threshold: create_message_req_payload.score_threshold,
            sort_options: create_message_req_payload.sort_options,
            page_size: Some(10),
            highlight_options: create_message_req_payload.highlight_options,
            filters: create_message_req_payload.filters,
            use_quote_negated_terms: create_message_req_payload.use_quote_negated_terms,
            remove_stop_words: create_message_req_payload.remove_stop_words,
            typo_options: create_message_req_payload.typo_options,
            ..Default::default()
        };
        let parsed_query = ParsedQuery {
            query: query.clone(),
            quote_words: None,
            negated_words: None,
        };
        let mut search_timer = Timer::new();
        let result_chunks = match search_type {
            SearchMethod::Hybrid => {
                search_hybrid_chunks(
                    search_chunk_data.clone(),
                    parsed_query,
                    pool.clone(),
                    redis_pool,
                    dataset.clone(),
                    &dataset_config,
                    &mut search_timer,
                )
                .await?
            }
            _ => {
                search_chunks_query(
                    search_chunk_data.clone(),
                    ParsedQueryTypes::Single(parsed_query),
                    pool.clone(),
                    redis_pool,
                    dataset.clone(),
                    &dataset_config,
                    &mut search_timer,
                )
                .await?
            }
        };

        let clickhouse_search_event = SearchQueryEventClickhouse {
            request_params: serde_json::to_string(&search_chunk_data.clone()).unwrap_or_default(),
            id: uuid::Uuid::new_v4(),
            search_type: "rag_chunks".to_string(),
            tokens: count_tokens(&query),
            query: query.clone(),
            dataset_id: dataset.id,
            organization_id: dataset.organization_id,
            top_score: result_chunks
                .score_chunks
                .get(0)
                .map(|x| x.score as f32)
                .unwrap_or(0.0),
            latency: get_latency_from_header(search_timer.header_value()),
            metadata: serde_json::to_string(&create_message_req_payload.metadata.clone())
                .unwrap_or_default(),
            results: result_chunks
                .score_chunks
                .clone()
                .into_iter()
                .map(|x| {
                    let mut json = serde_json::to_value(&x).unwrap_or_default();
                    escape_quotes(&mut json);
                    json.to_string()
                })
                .collect(),
            created_at: time::OffsetDateTime::now_utc(),
            query_rating: String::from(""),
            user_id: create_message_req_payload
                .user_id
                .clone()
                .unwrap_or_default(),
        };
        if !dataset_config.DISABLE_ANALYTICS {
            event_queue
                .send(ClickHouseEvent::SearchQueryEvent(
                    clickhouse_search_event.clone(),
                ))
                .await;
        }

        Ok((
            clickhouse_search_event,
            result_chunks
                .score_chunks
                .into_iter()
                .map(ScoreChunk::from)
                .collect::<Vec<ScoreChunk>>(),
        ))
    }
}

async fn handle_search_tool_call(
    search_params: SearchParams,
    create_message_req_payload: CreateMessageReqPayload,
    dataset: Dataset,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset_config: DatasetConfiguration,
    event_queue: web::Data<EventQueue>,
) -> Result<(Vec<ScoreChunk>, String, SearchQueryEventClickhouse), ServiceError> {
    let (clickhouse_search_event, results) = search_chunks(
        create_message_req_payload.clone(),
        search_params,
        dataset.clone(),
        pool.clone(),
        redis_pool.clone(),
        dataset_config.clone(),
        event_queue.clone(),
    )
    .await?;

    let formatted_results = results
        .iter()
        .take(10)
        .map(|score_chunk| {
            let chunk_metadata = ChunkMetadata::from(score_chunk.chunk.clone());
            json!({
                "id": chunk_metadata.id,
                "content": convert_html_to_text(&chunk_metadata.chunk_html.unwrap_or_default()),
                "tag_set": chunk_metadata.tag_set.clone(),
                "num_value": chunk_metadata.num_value,
                "score": score_chunk.score,
                "link": chunk_metadata.link.unwrap_or_default()
            })
        })
        .collect::<Vec<_>>();

    Ok((
        results,
        serde_json::to_string_pretty(&formatted_results).unwrap_or_default(),
        clickhouse_search_event,
    ))
}

#[allow(clippy::too_many_arguments)]
pub async fn stream_response_with_agentic_search(
    messages: Vec<models::Message>,
    topic_id: uuid::Uuid,
    dataset: Dataset,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset_config: DatasetConfiguration,
    create_message_req_payload: CreateMessageReqPayload,
    event_queue: web::Data<EventQueue>,
    #[cfg(feature = "hallucination-detection")] hallucination_detector: web::Data<
        HallucinationDetector,
    >,
) -> Result<HttpResponse, actix_web::Error> {
    let user_message_query = match create_message_req_payload.concat_user_messages_query {
        Some(true) => messages
            .iter()
            .filter(|message| message.role == "user")
            .map(|message| message.content.clone())
            .collect::<Vec<String>>()
            .join("\n\n"),
        _ => match messages.last() {
            Some(last_message) => last_message.content.clone(),
            None => "".to_string(),
        },
    };

    let mut openai_messages: Vec<ChatMessage> = messages
        .iter()
        .map(|message| ChatMessage::from(message.clone()))
        .collect();

    openai_messages.push(ChatMessage::User {
        content: ChatMessageContent::Text(user_message_query.clone()),
        name: None,
    });

    if let Some(image_urls) = create_message_req_payload.image_urls.clone() {
        if !image_urls.is_empty() {
            openai_messages.push(ChatMessage::User {
                name: None,
                content: ChatMessageContent::ContentPart(
                    image_urls
                        .iter()
                        .map(|url| {
                            ChatMessageContentPart::Image(ChatMessageImageContentPart {
                                r#type: "image_url".to_string(),
                                image_url: ImageUrlType {
                                    url: url.to_string(),
                                    detail: None,
                                },
                            })
                        })
                        .chain(std::iter::once(ChatMessageContentPart::Text(
                            ChatMessageTextContentPart {
                                r#type: "text".to_string(),
                                text:
                                    "These are the images that the user provided with their query."
                                        .to_string(),
                            },
                        )))
                        .collect::<Vec<_>>(),
                ),
            });
        }
    }

    let base_url = dataset_config.LLM_BASE_URL.clone();

    let llm_api_key = get_llm_api_key(&dataset_config);

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

    let chosen_model = create_message_req_payload
        .model
        .clone()
        .unwrap_or(dataset_config.LLM_DEFAULT_MODEL.clone());
    let tools = vec![ChatCompletionTool {
        r#type: ChatCompletionToolType::Function,
        function: ChatCompletionFunction {
            name: "search".to_string(),
            description: dataset_config.TOOL_CONFIGURATION.query_tool_options.as_ref().map(|x| x.tool_description.clone()).unwrap_or(Some("Search for relevant information in the knowledge base. You can use this tool multiple times if the information you need is not found in the first search.".to_string())),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": dataset_config.TOOL_CONFIGURATION.query_tool_options.as_ref().map(|x| x.query_parameter_description.clone()).unwrap_or(Some("The search query to find relevant information".to_string()))
                    },
                    "price_filter": {
                        "type": ["object", "null"],
                        "properties": {
                            "min": {
                                "type": ["number", "null"],
                                "description": dataset_config.TOOL_CONFIGURATION.query_tool_options.as_ref().map(|x| x.min_price_option_description.clone()).unwrap_or(Some("The minimum price to filter by".to_string()))
                            },
                            "max": {
                                "type": ["number", "null"],
                                "description": dataset_config.TOOL_CONFIGURATION.query_tool_options.as_ref().map(|x| x.max_price_option_description.clone()).unwrap_or(Some("The maximum price to filter by".to_string()))
                            }
                        },
                        "description": dataset_config.TOOL_CONFIGURATION.query_tool_options.as_ref().map(|x| x.price_filter_description.clone()).unwrap_or(Some("The price filter to use for the search".to_string())),
                        "required": ["min", "max"],
                        "additionalProperties": false
                    }
                },
                "required": ["query", "price_filter"],
                "additionalProperties": false
            }),
        },
    }, ChatCompletionTool {
        r#type: ChatCompletionToolType::Function,
        function: ChatCompletionFunction {
            name: "chunks_used".to_string(),
            description: Some("This is a tool that you can use to tell the user the chunks that you plan to use to generate your response.".to_string()),
            parameters: json!({
                "type": "object",
                "properties": {
                    "chunks": {
                        "type": "array",
                        "description": "The id of the chunks that you plan to use to generate your response",
                        "items": {
                            "type": "string"
                        }
                    }
                },
                "required": ["chunks"],
                "additionalProperties": false
            }),
        },
    }];

    let mut parameters = ChatCompletionParameters {
        model: chosen_model,
        messages: openai_messages.clone(),
        tools: Some(tools.clone()),
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

    let query_id = uuid::Uuid::new_v4();
    let query_id_arb = query_id;
    let completion_first = create_message_req_payload
        .llm_options
        .as_ref()
        .map(|x| x.completion_first)
        .unwrap_or(Some(false))
        .unwrap_or(false);

    let user_message = match &openai_messages
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
    };

    if create_message_req_payload
        .llm_options
        .as_ref()
        .is_some_and(|llm_options| !llm_options.stream_response.unwrap_or(true))
    {
        // Non-streaming response with tool support
        let mut conversation_messages = openai_messages.clone();
        let mut response_text = String::new();
        let mut prompt_tokens = 0;
        let mut completion_tokens = 0;
        let mut searched_chunks = Vec::new();
        let mut search_event: Option<SearchQueryEventClickhouse> = None;

        loop {
            parameters.messages = conversation_messages.clone();

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

            prompt_tokens += assistant_completion
                .usage
                .as_ref()
                .map(|usage| usage.prompt_tokens.unwrap_or(0))
                .unwrap_or(0);
            completion_tokens += assistant_completion
                .usage
                .as_ref()
                .map(|usage| usage.completion_tokens.unwrap_or(0))
                .unwrap_or(0);

            let first_choice = assistant_completion
                .choices
                .get(0)
                .ok_or_else(|| ServiceError::BadRequest("No response from LLM".to_string()))?;

            match &first_choice.message {
                ChatMessage::Assistant {
                    content,
                    tool_calls,
                    ..
                } => {
                    // Add assistant message to conversation
                    conversation_messages.push(first_choice.message.clone());

                    if let Some(tool_calls) = tool_calls {
                        // Handle tool calls
                        for tool_call in tool_calls {
                            if tool_call.function.name == "search" {
                                let search_params = serde_json::from_str::<SearchParams>(
                                    &tool_call.function.arguments,
                                )
                                .map_err(|e| ServiceError::BadRequest(e.to_string()))?;
                                let (results, formatted_results, clickhouse_search_event) =
                                    handle_search_tool_call(
                                        search_params,
                                        create_message_req_payload.clone(),
                                        dataset.clone(),
                                        pool.clone(),
                                        redis_pool.clone(),
                                        dataset_config.clone(),
                                        event_queue.clone(),
                                    )
                                    .await?;
                                search_event = Some(clickhouse_search_event);
                                conversation_messages.push(ChatMessage::Tool {
                                    content: formatted_results.clone(),
                                    tool_call_id: tool_call.id.clone(),
                                });
                                searched_chunks.extend(results);
                            }
                            if tool_call.function.name == "chunks_used" {
                                #[derive(Debug, Serialize, Deserialize)]
                                struct ChunksUsed {
                                    chunks: Vec<String>,
                                }

                                let chunks_used: ChunksUsed =
                                    serde_json::from_str(&tool_call.function.arguments).unwrap();

                                conversation_messages.push(ChatMessage::Tool {
                                    content: format!(
                                        "Chunks to use: {}",
                                        tool_call.function.arguments
                                    ),
                                    tool_call_id: tool_call.id.clone(),
                                });

                                searched_chunks.retain(|chunk| {
                                    chunks_used.chunks.contains(
                                        &ChunkMetadata::from(chunk.chunk.clone()).id.to_string(),
                                    )
                                });

                                let images: Vec<String> = searched_chunks
                                    .iter()
                                    .filter_map(|score_chunk| {
                                        ChunkMetadata::from(score_chunk.chunk.clone())
                                            .image_urls
                                            .clone()
                                    })
                                    .flat_map(|image_urls| {
                                        image_urls
                                            .iter()
                                            .filter_map(|image| image.clone())
                                            .collect::<Vec<_>>()
                                    })
                                    .collect();

                                if !images.is_empty() {
                                    if let Some(image_config) = create_message_req_payload
                                        .llm_options
                                        .as_ref()
                                        .and_then(|x| x.image_config.as_ref())
                                    {
                                        if image_config.use_images.unwrap_or(false) {
                                            conversation_messages.push(ChatMessage::User {
                                                name: None,
                                                content: ChatMessageContent::ContentPart(
                                                    images
                                                        .iter()
                                                        .take(
                                                            image_config
                                                                .images_per_chunk
                                                                .unwrap_or(5),
                                                        )
                                                        .map(|url| {
                                                            ChatMessageContentPart::Image(
                                                                ChatMessageImageContentPart {
                                                                    r#type: "image_url".to_string(),
                                                                    image_url: ImageUrlType {
                                                                        url: url.to_string(),
                                                                        detail: None,
                                                                    },
                                                                },
                                                            )
                                                        })
                                                        .collect::<Vec<_>>(),
                                                ),
                                            })
                                        }
                                    }
                                }
                            }
                        }
                        // Continue conversation with tool results
                        continue;
                    } else {
                        // No tool calls, we have the final response
                        if let Some(ChatMessageContent::Text(text)) = content {
                            response_text = text.clone();
                        }
                        break;
                    }
                }
                _ => {
                    return Err(ServiceError::BadRequest(
                        "Invalid response format from LLM".to_string(),
                    )
                    .into());
                }
            }
        }

        let final_response = if completion_first {
            format!(
                "{}||{}",
                serde_json::to_string(&searched_chunks)
                    .unwrap_or_default()
                    .replace("||", ""),
                response_text
            )
        } else {
            format!(
                "{}||{}",
                response_text,
                serde_json::to_string(&searched_chunks)
                    .unwrap_or_default()
                    .replace("||", ""),
            )
        };

        let new_message = models::Message::from_details(
            final_response.clone(),
            topic_id,
            next_message_order()
                .try_into()
                .expect("usize to i32 conversion should always succeed"),
            "assistant".to_string(),
            Some(prompt_tokens as i32),
            Some(completion_tokens as i32),
            dataset.id,
            query_id,
        );

        #[cfg(feature = "hallucination-detection")]
        let score = {
            let docs = searched_chunks
                .iter()
                .map(|score_chunk| {
                    ChunkMetadata::from(score_chunk.chunk.clone())
                        .chunk_html
                        .clone()
                        .unwrap_or_default()
                })
                .collect::<Vec<String>>();
            hallucination_detector
                .detect_hallucinations(&clean_markdown(&response_text), &docs)
                .await
                .map_err(|err| {
                    ServiceError::BadRequest(format!("Failed to detect hallucinations: {}", err))
                })?
        };

        #[cfg(not(feature = "hallucination-detection"))]
        let score = DummyHallucinationScore {
            total_score: 0.0,
            detected_hallucinations: vec![],
        };

        let filtered_chunks_data = searched_chunks
            .iter()
            .map(|x| {
                let mut json = serde_json::to_value(x).unwrap_or_default();
                escape_quotes(&mut json);
                json.to_string()
            })
            .collect::<Vec<String>>();

        if let Some(search_event) = search_event {
            let clickhouse_rag_event = RagQueryEventClickhouse {
                id: query_id,
                created_at: time::OffsetDateTime::now_utc(),
                dataset_id: dataset.id,
                organization_id: dataset.organization_id,
                search_id: search_event.id,
                top_score: search_event.top_score,
                results: vec![],
                topic_id,
                metadata: search_event.metadata,
                json_results: filtered_chunks_data,
                user_message: user_message_query.clone(),
                query_rating: String::new(),
                rag_type: "all_chunks".to_string(),
                llm_response: final_response.clone(),
                user_id: create_message_req_payload
                    .user_id
                    .clone()
                    .unwrap_or_default(),
                hallucination_score: score.total_score,
                detected_hallucinations: score.detected_hallucinations,
                tokens: count_message_tokens(conversation_messages) + count_tokens(&response_text),
            };

            if !dataset_config.DISABLE_ANALYTICS {
                event_queue
                    .send(ClickHouseEvent::RagQueryEvent(clickhouse_rag_event.clone()))
                    .await;
            }
        }

        create_messages_query(vec![new_message], &pool).await?;

        if create_message_req_payload.audio_input.is_some() {
            return Ok(HttpResponse::Ok()
                .insert_header((
                    "X-TR-Query",
                    user_message
                        .to_string()
                        .replace(|c: char| c.is_ascii_control(), ""),
                ))
                .insert_header(("TR-QueryID", query_id.to_string().replace("\n", "")))
                .json(final_response));
        } else {
            return Ok(HttpResponse::Ok()
                .insert_header(("TR-QueryID", query_id.to_string().replace("\n", "")))
                .json(final_response));
        }
    }

    let (tx, rx) = tokio::sync::mpsc::channel::<web::Bytes>(100);

    let create_message_req_payload_clone = create_message_req_payload.clone();

    Arbiter::new().spawn(async move {
        let mut conversation_messages = openai_messages.clone();
        let mut all_content = String::new();
        let mut needs_tool_response = true;
        let mut searched_chunks = Vec::new();
        let mut search_event: Option<SearchQueryEventClickhouse> = None;

        while needs_tool_response {
            needs_tool_response = false;
            let mut functions: std::collections::HashMap<String, DeltaFunction> =
                std::collections::HashMap::new();
            let mut tool_call_ids: std::collections::HashMap<String, String> =
                std::collections::HashMap::new();
            let mut content_buffer = String::new();

            let mut loop_parameters = parameters.clone();
            loop_parameters.messages = conversation_messages.clone();

            let stream = match client.chat().create_stream(loop_parameters).await {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Error creating stream: {:?}", e);
                    let _ = tx
                        .send(web::Bytes::from(format!(", \"error\": \"{}\"", e)))
                        .await;
                    break;
                }
            };

            let mut tracked_stream = RoleTrackingStream::new(stream);

            while let Some(response) = tracked_stream.next().await {
                match response {
                    Ok(chat_response) => {
                        for choice in &chat_response.choices {
                            if let DeltaChatMessage::Assistant {
                                content,
                                tool_calls,
                                ..
                            } = &choice.delta
                            {
                                if let Some(ChatMessageContent::Text(text)) = content {
                                    let mut response_text = String::new();
                                    if content_buffer.is_empty() && !completion_first {
                                        content_buffer.push_str(&format!(
                                            "{}||",
                                            serde_json::to_string(&searched_chunks)
                                                .unwrap_or_default().replace("||", "")
                                        ));
                                        response_text.push_str(&format!(
                                            "{}||",
                                            serde_json::to_string(&searched_chunks)
                                                .unwrap_or_default().replace("||", "")
                                        ));
                                    }
                                    content_buffer.push_str(text);
                                    response_text.push_str(text);

                                    let _ = tx.send(web::Bytes::from(response_text)).await;
                                }
                                if let Some(delta_tool_calls) = tool_calls {
                                    for delta_call in delta_tool_calls {
                                        if let Some(id) = &delta_call.id {
                                            tool_call_ids.insert(id.clone(), id.clone());
                                            functions.insert(
                                                id.clone(),
                                                DeltaFunction {
                                                    name: None,
                                                    arguments: None,
                                                },
                                            );
                                        }

                                        let call_id = delta_call
                                            .id
                                            .as_ref()
                                            .or_else(|| tool_call_ids.keys().last())
                                            .cloned()
                                            .unwrap_or_default();

                                        if let Some(function) = functions.get_mut(&call_id) {
                                            function.merge(&delta_call.function);
                                        }
                                    }
                                }
                            }

                            // Check if we have complete tool calls
                            match choice.finish_reason {
                                Some(FinishReason::ToolCalls) => {
                                    let mut tool_calls_vec = Vec::new();

                                    for (call_id, function) in &functions {
                                        if let (Some(name), Some(arguments)) =
                                            (&function.name, &function.arguments)
                                        {
                                            tool_calls_vec.push(ToolCall {
                                                id: call_id.clone(),
                                                function: Function {
                                                    name: name.clone(),
                                                    arguments: arguments.clone(),
                                                },
                                                r#type: "function".to_string(),
                                            });
                                        }
                                    }

                                    if !tool_calls_vec.is_empty() {
                                        conversation_messages.push(ChatMessage::Assistant {
                                            content: if content_buffer.is_empty() {
                                                None
                                            } else {
                                                Some(ChatMessageContent::Text(
                                                    content_buffer.clone(),
                                                ))
                                            },
                                            name: None,
                                            audio: None,
                                            reasoning_content: None,
                                            refusal: None,
                                            tool_calls: Some(tool_calls_vec.clone()),
                                        });

                                        all_content.push_str(&content_buffer);

                                        // Process all tool calls
                                        for tool_call in &tool_calls_vec {
                                            if tool_call.function.name == "search" {
                                                let search_params = serde_json::from_str::<SearchParams>(&tool_call.function.arguments)
                                                    .map_err(|e| ServiceError::BadRequest(e.to_string())).unwrap_or_default();
                                                // Send search indicator
                                                let _ = tx
                                                    .send(web::Bytes::from(
                                                        format!("\n\n[Searching for {}...]\n\n", search_params.query),
                                                    ))
                                                    .await;

                                                match handle_search_tool_call(
                                                    search_params,
                                                    create_message_req_payload_clone.clone(),
                                                    dataset.clone(),
                                                    pool.clone(),
                                                    redis_pool.clone(),
                                                    dataset_config.clone(),
                                                    event_queue.clone(),
                                                )
                                                .await
                                                {
                                                    Ok((
                                                        chunks,
                                                        formatted_results,
                                                        clickhouse_search_event,
                                                    )) => {
                                                        search_event =
                                                            Some(clickhouse_search_event);
                                                        // Add tool response
                                                        conversation_messages.push(
                                                            ChatMessage::Tool {
                                                                content: formatted_results,
                                                                tool_call_id: tool_call.id.clone(),
                                                            },
                                                        );
                                                        searched_chunks.extend(chunks);
                                                        needs_tool_response = true;
                                                    }
                                                    Err(e) => {
                                                        log::error!(
                                                            "Error in search tool call: {:?}",
                                                            e
                                                        );
                                                        conversation_messages.push(
                                                            ChatMessage::Tool {
                                                                content: format!(
                                                                    "Error in search tool call: {}",
                                                                    e
                                                                ),
                                                                tool_call_id: tool_call.id.clone(),
                                                            },
                                                        );
                                                        needs_tool_response = true;
                                                    }
                                                }
                                            }
                                            if tool_call.function.name == "chunks_used" {
                                                #[derive(Debug, Serialize, Deserialize)]
                                                struct ChunksUsed {
                                                    chunks: Vec<String>,
                                                }

                                                let chunks_used: ChunksUsed = serde_json::from_str(
                                                    &tool_call.function.arguments,
                                                )
                                                .unwrap();

                                                conversation_messages.push(ChatMessage::Tool {
                                                    content: format!(
                                                        "Chunks To Use: {}",
                                                        tool_call.function.arguments
                                                    ),
                                                    tool_call_id: tool_call.id.clone(),
                                                });

                                                searched_chunks.retain(|chunk| {
                                                    chunks_used.chunks.contains(
                                                        &ChunkMetadata::from(chunk.chunk.clone())
                                                            .id
                                                            .to_string(),
                                                    )
                                                });


                                            let images: Vec<String> = searched_chunks
                                                .iter()
                                                .filter_map(|score_chunk| {
                                                    ChunkMetadata::from(score_chunk.chunk.clone())
                                                        .image_urls
                                                        .clone()
                                                })
                                                .flat_map(|image_urls| {
                                                    image_urls
                                                        .iter()
                                                        .filter_map(|image| image.clone())
                                                        .collect::<Vec<_>>()
                                                })
                                                .collect();

                                                if !images.is_empty() {
                                                    if let Some(image_config) = create_message_req_payload
                                                        .llm_options
                                                        .as_ref()
                                                        .and_then(|x| x.image_config.as_ref())
                                                    {
                                                        if image_config.use_images.unwrap_or(false) {
                                                            conversation_messages.push(ChatMessage::User {
                                                                name: None,
                                                                content: ChatMessageContent::ContentPart(
                                                                    images
                                                                        .iter()
                                                                        .take(image_config.images_per_chunk.unwrap_or(5))
                                                                        .map(|url| {
                                                                            ChatMessageContentPart::Image(ChatMessageImageContentPart {
                                                                                r#type: "image_url".to_string(),
                                                                                image_url: ImageUrlType {
                                                                                    url: url.to_string(),
                                                                                    detail: None,
                                                                                },
                                                                            })
                                                                        })
                                                                        .collect::<Vec<_>>(),
                                                                ),
                                                            })
                                                        }
                                                    }
                                                }

                                                needs_tool_response = true;
                                            }
                                        }
                                        tool_calls_vec.clear();
                                    }
                                }

                                _ => {
                                    if completion_first {
                                        let searched_chunks =
                                            serde_json::to_string(&searched_chunks)
                                                .unwrap_or_default();
                                        content_buffer.push_str(&format!("||{}", searched_chunks));
                                        let _ = tx
                                            .send(web::Bytes::from(format!(
                                                "||{}",
                                                searched_chunks
                                            )))
                                            .await;
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Stream error: {:?}", e);
                        let _ = tx
                            .send(web::Bytes::from(format!(", \"error\": \"{}\"", e)))
                            .await;
                        break;
                    }
                }
            }
            all_content.push_str(&content_buffer);
        }

        let mut split_completion = all_content.split("||");

        #[allow(unused_variables)]
        let (response, chunks) = if completion_first {
            let response = split_completion.next().unwrap_or_default().to_string();
            let chunk_data: Vec<ChunkMetadataStringTagSet> =
                serde_json::from_str(split_completion.next().unwrap_or_default())
                    .unwrap_or_default();

            (response, chunk_data)
        } else {
            let chunk_data: Vec<ChunkMetadataStringTagSet> =
                serde_json::from_str(split_completion.next().unwrap_or_default())
                    .unwrap_or_default();

            let response = split_completion.next().unwrap_or_default().to_string();

            (response, chunk_data)
        };

        let chunk_data: Vec<String> = chunks
            .iter()
            .map(|x| {
                let mut json = serde_json::to_value(x).unwrap_or_default();
                escape_quotes(&mut json);
                json.to_string()
            })
            .collect();

        let new_message = models::Message::from_details(
            all_content.clone(),
            topic_id,
            next_message_order().try_into().unwrap(),
            "assistant".to_string(),
            None,
            Some(all_content.len().try_into().unwrap()),
            dataset.id,
            query_id_arb,
        );

        if let Some(search_event) = search_event {
            if !dataset_config.DISABLE_ANALYTICS {
                #[cfg(feature = "hallucination-detection")]
                let score = {
                    let docs = chunks
                        .iter()
                        .map(|x| x.chunk_html.clone().unwrap_or_default())
                        .collect::<Vec<String>>();

                    hallucination_detector
                        .detect_hallucinations(&clean_markdown(&response), &docs)
                        .await
                        .unwrap_or(HallucinationScore {
                            total_score: 0.0,
                            proper_noun_score: 0.0,
                            number_mismatch_score: 0.0,
                            unknown_word_score: 0.0,
                            detected_hallucinations: vec![],
                        })
                };

                #[cfg(not(feature = "hallucination-detection"))]
                let score = DummyHallucinationScore {
                    total_score: 0.0,
                    detected_hallucinations: vec![],
                };

                let clickhouse_rag_event = RagQueryEventClickhouse {
                    id: query_id_arb,
                    created_at: time::OffsetDateTime::now_utc(),
                    search_id: search_event.id,
                    top_score: search_event.top_score,
                    dataset_id: dataset.id,
                    topic_id,
                    results: vec![],
                    json_results: chunk_data,
                    user_message: user_message_query.clone(),
                    metadata: search_event.metadata,
                    query_rating: String::new(),
                    rag_type: "all_chunks".to_string(),
                    llm_response: all_content.clone(),
                    user_id: create_message_req_payload
                        .user_id
                        .clone()
                        .unwrap_or_default(),
                    hallucination_score: score.total_score,
                    detected_hallucinations: score.detected_hallucinations,
                    tokens: count_message_tokens(conversation_messages) + count_tokens(&response),
                    organization_id: dataset.organization_id,
                };

                event_queue
                    .send(ClickHouseEvent::RagQueryEvent(clickhouse_rag_event.clone()))
                    .await;
            }
        }

        let _ = create_messages_query(vec![new_message], &pool).await;
    });

    let chat_completion_timeout = std::env::var("CHAT_COMPLETION_TIMEOUT_SECS")
        .unwrap_or("120".to_string())
        .parse::<u64>()
        .unwrap_or(120);

    let completion_stream = tokio_stream::wrappers::ReceiverStream::new(rx)
        .take_until(tokio::time::sleep(std::time::Duration::from_secs(
            chat_completion_timeout,
        )))
        .map(|content| -> Result<Bytes, actix_web::Error> { Ok(content) });

    if create_message_req_payload.audio_input.is_some() {
        Ok(HttpResponse::Ok()
            .insert_header((
                "X-TR-Query",
                user_message
                    .to_string()
                    .replace(|c: char| c.is_ascii_control(), ""),
            ))
            .insert_header(("TR-QueryID", query_id.to_string().replace("\n", "")))
            .streaming(completion_stream))
    } else {
        Ok(HttpResponse::Ok()
            .insert_header(("TR-QueryID", query_id.to_string().replace("\n", "")))
            .streaming(completion_stream))
    }
}

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
    let dataset_config = DatasetConfiguration::from_json(dataset.server_configuration.clone());
    let parameters = ChatCompletionParameters {
        model,
        messages: vec![prompt_topic_message],
        stream: Some(false),
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        frequency_penalty: None,
        presence_penalty: None,
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

    let base_url = dataset_config.LLM_BASE_URL.clone();

    let base_url = if base_url.is_empty() {
        "https://openrouter.ai/api/v1".into()
    } else {
        base_url
    };

    let llm_api_key = get_llm_api_key(&dataset_config);

    let client = Client {
        headers: None,
        api_key: llm_api_key,
        project: None,
        http_client: reqwest::Client::new(),
        base_url,
        organization: None,
    };

    let query = client.chat().create(parameters).await.map_err(|err| {
        ServiceError::BadRequest(format!("No LLM Completion for topic {:?}", err))
    })?;

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

pub async fn get_text_from_image(
    image_url: String,
    prompt: Option<String>,
    dataset: &Dataset,
) -> Result<String, ServiceError> {
    let dataset_config = DatasetConfiguration::from_json(dataset.server_configuration.clone());
    let base_url = dataset_config.LLM_BASE_URL.clone();

    let base_url = if base_url.is_empty() {
        "https://openrouter.ai/api/v1".into()
    } else {
        base_url
    };

    let llm_api_key = get_llm_api_key(&dataset_config);

    let client = Client {
        headers: None,
        api_key: llm_api_key,
        project: None,
        http_client: reqwest::Client::new(),
        base_url,
        organization: None,
    };

    let default_system_prompt = "Please describe the image and turn the description into a search query. DO NOT INCLUDE ANY OTHER CONTEXT OR INFORMATION. JUST OUTPUT THE SEARCH QUERY AND NOTHING ELSE".to_string();

    let messages = vec![
        ChatMessage::System {
            content: ChatMessageContent::Text(prompt.unwrap_or(default_system_prompt)),
            name: None,
        },
        ChatMessage::User {
            content: ChatMessageContent::ContentPart(vec![ChatMessageContentPart::Image(
                ChatMessageImageContentPart {
                    r#type: "image_url".to_string(),
                    image_url: ImageUrlType {
                        url: image_url.clone(),
                        detail: None,
                    },
                },
            )]),
            name: None,
        },
    ];

    let parameters = ChatCompletionParameters {
        model: dataset_config.LLM_DEFAULT_MODEL.clone(),
        messages,
        stream: Some(false),
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        frequency_penalty: Some(dataset_config.FREQUENCY_PENALTY.unwrap_or(0.8) as f32),
        presence_penalty: Some(dataset_config.PRESENCE_PENALTY.unwrap_or(0.8) as f32),
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

    let query = client
        .chat()
        .create(parameters)
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Error: {:?}", err)))?;

    let text = match &query
        .choices
        .get(0)
        .ok_or(ServiceError::BadRequest(
            "No response for LLM completion".to_string(),
        ))?
        .message
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
    };

    Ok(text)
}

pub async fn get_text_from_audio(audio_base64: &str) -> Result<String, ServiceError> {
    let client = Client {
        headers: None,
        api_key: get_env!("OPENAI_API_KEY", "OPENAI_API_KEY for openai should be set").into(),
        project: None,
        http_client: reqwest::Client::new(),
        base_url: "https://api.openai.com/v1/".into(),
        organization: None,
    };

    let audio_bytes = base64::decode(audio_base64).map_err(|err| {
        ServiceError::BadRequest(format!("Error decoding audio base64: {:?}", err))
    })?;

    let parameters = AudioTranscriptionParametersBuilder::default()
        .file(FileUpload::Bytes(FileUploadBytes {
            filename: "audio.mp3".to_string(),
            bytes: audio_bytes.into(),
        }))
        .model(WhisperModel::Whisper1.to_string())
        .response_format(AudioOutputFormat::Text)
        .language("en".to_string())
        .build()
        .map_err(|err| {
            ServiceError::InternalServerError(format!("Transcription Error: {:?}", err))
        })?;

    let text = client
        .audio()
        .create_transcription(parameters)
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!("Transcription Error: {:?}", err))
        })?;

    dbg!(&text);

    Ok(text.replace("\n", ""))
}

pub fn get_llm_api_key(dataset_config: &DatasetConfiguration) -> String {
    if !dataset_config.LLM_API_KEY.is_empty() {
        dataset_config.LLM_API_KEY.clone()
    } else if dataset_config.LLM_BASE_URL.contains("openai.com") {
        get_env!("OPENAI_API_KEY", "OPENAI_API_KEY for openai should be set").into()
    } else if dataset_config.LLM_BASE_URL.contains("groq.com") {
        get_env!("GROQ_API_KEY", "GROQ_API_KEY for groq should be set").into()
    } else {
        get_env!(
            "LLM_API_KEY",
            "LLM_API_KEY for openrouter or self-hosted should be set"
        )
        .into()
    }
}

pub async fn suggested_followp_questions(
    payload: SuggestedQueriesReqPayload,
    dataset_config: DatasetConfiguration,
) -> Result<Vec<String>, ServiceError> {
    let base_url = dataset_config.LLM_BASE_URL.clone();
    let default_model = dataset_config.LLM_DEFAULT_MODEL.clone();

    let base_url = if base_url.is_empty() {
        "https://api.openai.com/api/v1".into()
    } else {
        base_url
    };

    let llm_api_key = get_llm_api_key(&dataset_config);

    let followup = if payload.is_followup.unwrap_or(false) {
        " followup"
    } else {
        ""
    };

    let number_of_suggestions_to_create = payload.suggestions_to_create.unwrap_or(10);
    let context = payload
        .context
        .clone()
        .unwrap_or("You are an AI assistant that wants to help the user".to_string());

    let content = ChatMessageContent::Text(format!("Generate {number_of_suggestions_to_create}{followup} varied questions followed by a new line. Your only response should be the {number_of_suggestions_to_create}{followup} questions. \n\n Suggest varied questions with the following context in mind: {context}. Keep your query recommendations short, limited to 3-6 words. This should not be a list, so do not number each question."));

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
        frequency_penalty: None,
        presence_penalty: None,
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

    let query = client
        .chat()
        .create(parameters.clone())
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let queries: Vec<String> = match &query
        .choices
        .get(0)
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

    Ok(queries)
}

pub async fn suggested_new_queries(
    payload: SuggestedQueriesReqPayload,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<Vec<String>, ServiceError> {
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.clone().server_configuration);

    let base_url = dataset_config.LLM_BASE_URL.clone();
    let default_model = dataset_config.LLM_DEFAULT_MODEL.clone();
    let qdrant_only = dataset_config.QDRANT_ONLY;

    let base_url = if base_url.is_empty() {
        "https://api.openai.com/api/v1".into()
    } else {
        base_url
    };

    let llm_api_key = get_llm_api_key(&dataset_config);
    let search_type = payload.search_type.clone().unwrap_or(SearchMethod::Hybrid);
    let filters = payload.filters.clone();

    let chunk_metadatas = match payload.query.clone() {
        Some(query) => {
            let search_req_payload = SearchChunksReqPayload {
                search_type: search_type.clone(),
                query: models::QueryTypes::Single(models::SearchModalities::Text(query.clone())),
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
                .map_err(|e| ServiceError::BadRequest(e.to_string()))?,
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
                .map_err(|e| ServiceError::BadRequest(e.to_string()))?,
            }
            .score_chunks
            .into_iter()
            .filter_map(|chunk| chunk.metadata.clone().get(0).cloned())
            .map(ChunkMetadata::from)
            .collect::<Vec<ChunkMetadata>>()
        }
        None => {
            let random_offset_id = match payload.filters {
                Some(_) => uuid::Uuid::nil(),
                None => get_random_chunk_qdrant_point_id_query(dataset_id, pool.clone()).await?,
            };
            let filter =
                assemble_qdrant_filter(filters, None, None, dataset_id, pool.clone()).await?;

            let (search_results, _) = scroll_dataset_points(
                payload
                    .suggestions_to_create
                    .unwrap_or(5)
                    .try_into()
                    .unwrap(),
                Some(random_offset_id),
                None,
                dataset_config.clone(),
                filter,
            )
            .await?;
            if qdrant_only {
                search_results
                    .iter()
                    .map(|search_result| {
                        ChunkMetadata::from(ChunkMetadataTypes::Metadata(
                            ChunkMetadataStringTagSet::from(QdrantChunkMetadata::from(
                                search_result.clone(),
                            )),
                        ))
                    })
                    .collect()
            } else {
                let qdrant_point_ids: Vec<uuid::Uuid> = search_results
                    .iter()
                    .map(|search_result| search_result.point_id)
                    .collect();
                get_chunk_metadatas_from_point_ids(qdrant_point_ids.clone(), pool)
                    .await?
                    .into_iter()
                    .map(ChunkMetadata::from)
                    .collect()
            }
        }
    };

    let rag_content = chunk_metadatas
        .iter()
        .map(|chunk| {
            format!(
                "- {}",
                convert_html_to_text(&(chunk.chunk_html.clone().unwrap_or_default()))
            )
        })
        .collect::<Vec<String>>()
        .join("\n\n");

    let query_style = match payload
        .suggestion_type
        .clone()
        .unwrap_or(SuggestType::Keyword)
    {
        SuggestType::Question => "question",
        SuggestType::Keyword => "keyword",
        SuggestType::Semantic => "semantic while not question",
    };
    let context_sentence = match payload.context.clone() {
        Some(context) => {
            if payload.is_ecommerce.unwrap_or(false) {
                format!(
                    "Suggest varied {query_style} queries with the following context in mind: {}. Generate one question that requests for a comparison of two products, explicitly including these questions in the query through phrases such as 'compare, versus, different, etc...'. Another question should contain one product title, explicitly include the name of the product in the query and ask questions related to this product within the provided context. For another question, generate a question that usses the format of find me and then a broader category name that makes sense within the provided context. Ensure all queries are short and to the point.",
                    context
                )
            } else {
                format!(
                    "Suggest varied {query_style} queries with the following context in mind: {}. Generate queries that are short, specific, and directly related to the provided context.",
                    context
                )
            }
        }
        None => "".to_string(),
    };

    let number_of_suggestions_to_create = payload.suggestions_to_create.unwrap_or(5);

    let content = ChatMessageContent::Text(format!(
        "Here is some content which the user might be looking for: {rag_content}{context_sentence}. Generate {number_of_suggestions_to_create} varied followup {query_style} style queries based off the domain of this dataset. Your only response should be the {number_of_suggestions_to_create} followup {query_style} style queries which are separated by new lines and are just text and you do not add any other context or information about the followup {query_style} style queries. This should not be a list, so do not number each {query_style} style queries.",
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
        frequency_penalty: None,
        presence_penalty: None,
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
        .get(0)
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

    while queries.len() < number_of_suggestions_to_create {
        query = match client.chat().create(parameters.clone()).await {
            Ok(query) => query,
            Err(err) => {
                log::error!(
                    "Error generating suggested queries when queries are less than 3: {}",
                    err
                );
                return Err(ServiceError::BadRequest(err.to_string()));
            }
        };
        let first_query = match query.choices.get(0) {
            Some(first_query) => first_query,
            None => {
                log::error!("Error generating suggested queries when queries are less than 3: No first query in choices");
                return Err(ServiceError::BadRequest(
                    "No first query in choices on call to LLM".to_string(),
                ));
            }
        };
        queries = match &first_query.message {
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
        .take(number_of_suggestions_to_create)
        .cloned()
        .cloned()
        .collect_vec();

    Ok(queries)
}
