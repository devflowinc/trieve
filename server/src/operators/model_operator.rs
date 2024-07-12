use crate::{
    data::models::{ChunkMetadataTypes, ScoreChunkDTO, ServerDatasetConfiguration},
    errors::ServiceError,
    get_env,
    handlers::chunk_handler::{BoostPhrase, DistancePhrase},
};
use murmur3::murmur3_32;
use openai_dive::v1::{
    helpers::format_response,
    resources::embedding::{EmbeddingInput, EmbeddingOutput, EmbeddingResponse},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::Cursor, ops::IndexMut};

use super::parse_operator::convert_html_to_text;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingParameters {
    /// Input text to embed, encoded as a string or array of tokens.
    /// To embed multiple inputs in a single request, pass an array of strings or array of token arrays.
    pub input: EmbeddingInput,
    /// ID of the model to use.
    pub model: String,
    /// Truncate the input to ensure it fits within the model's limits.
    pub truncate: bool,
}

#[tracing::instrument]
pub async fn create_embedding(
    message: String,
    distance_phrase: Option<DistancePhrase>,
    embed_type: &str,
    dataset_config: ServerDatasetConfiguration,
) -> Result<Vec<f32>, ServiceError> {
    if message.is_empty() || message == "" {
        return Err(ServiceError::BadRequest("No message to encode".to_string()));
    }

    let parent_span = sentry::configure_scope(|scope| scope.get_span());
    let transaction: sentry::TransactionOrSpan = match &parent_span {
        Some(parent) => parent
            .start_child("create_embedding", "Create semantic dense embedding")
            .into(),
        None => {
            let ctx = sentry::TransactionContext::new(
                "create_embedding",
                "Create semantic dense embedding",
            );
            sentry::start_transaction(ctx).into()
        }
    };
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone())));

    let embedding_api_key = get_env!("OPENAI_API_KEY", "OPENAI_API_KEY should be set");
    let config_embedding_base_url = dataset_config.EMBEDDING_BASE_URL;
    transaction.set_data(
        "EMBEDDING_SERVER",
        config_embedding_base_url.as_str().into(),
    );
    transaction.set_data(
        "EMBEDDING_MODEL",
        dataset_config.EMBEDDING_MODEL_NAME.as_str().into(),
    );

    let embedding_base_url = match config_embedding_base_url.as_str() {
        "" => get_env!("OPENAI_BASE_URL", "OPENAI_BASE_URL must be set").to_string(),
        "https://api.openai.com/v1" => {
            get_env!("OPENAI_BASE_URL", "OPENAI_BASE_URL must be set").to_string()
        }
        "https://embedding.trieve.ai" => std::env::var("EMBEDDING_SERVER_ORIGIN")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or("https://embedding.trieve.ai".to_string()),
        "https://embedding.trieve.ai/bge-m3" => std::env::var("EMBEDDING_SERVER_ORIGIN_BGEM3")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or("https://embedding.trieve.ai/bge-m3".to_string()),
        "https://embedding.trieve.ai/jina-code" => {
            std::env::var("EMBEDDING_SERVER_ORIGIN_JINA_CODE")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or("https://embedding.trieve.ai/jina-code".to_string())
        }
        _ => config_embedding_base_url.clone(),
    };

    let embedding_api_key =
        if config_embedding_base_url.as_str() == "https://embedding.trieve.ai/jina-code" {
            std::env::var("JINA_CODE_API_KEY")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or(embedding_api_key.to_string())
        } else {
            embedding_api_key.to_string()
        };

    let clipped_message = if message.len() > 7000 {
        message.chars().take(20000).collect()
    } else {
        message.clone()
    };

    let mut messages = vec![clipped_message.clone()];

    if distance_phrase.is_some() {
        let clipped_boost = if distance_phrase.as_ref().unwrap().phrase.len() > 7000 {
            distance_phrase
                .as_ref()
                .unwrap()
                .phrase
                .chars()
                .take(20000)
                .collect()
        } else {
            distance_phrase.as_ref().unwrap().phrase.clone()
        };
        messages.push(clipped_boost);
    }

    let input = match embed_type {
        "doc" => EmbeddingInput::StringArray(messages),
        "query" => EmbeddingInput::String(
            format!(
                "{}{}",
                dataset_config.EMBEDDING_QUERY_PREFIX, &clipped_message
            )
            .to_string(),
        ),
        _ => EmbeddingInput::StringArray(messages),
    };

    let parameters = EmbeddingParameters {
        model: dataset_config.EMBEDDING_MODEL_NAME.to_string(),
        input,
        truncate: true,
    };

    let embeddings_resp = ureq::post(&format!(
        "{}/embeddings?api-version=2023-05-15",
        embedding_base_url
    ))
    .set("Authorization", &format!("Bearer {}", &embedding_api_key))
    .set("api-key", &embedding_api_key)
    .set("Content-Type", "application/json")
    .send_json(serde_json::to_value(parameters).unwrap())
    .map_err(|e| {
        ServiceError::InternalServerError(format!(
            "Could not get embeddings from server: {:?}, {:?}",
            e,
            e.to_string()
        ))
    })?;

    let embeddings: EmbeddingResponse = format_response(embeddings_resp.into_string().unwrap())
        .map_err(|e| {
            log::error!("Failed to format response from embeddings server {:?}", e);
            ServiceError::InternalServerError(
                "Failed to format response from embeddings server".to_owned(),
            )
        })?;

    let mut vectors: Vec<Vec<f32>> = embeddings
    .data
    .into_iter()
    .map(|x| match x.embedding {
        EmbeddingOutput::Float(v) => v.iter().map(|x| *x as f32).collect(),
        EmbeddingOutput::Base64(_) => {
            log::error!("Embedding server responded with Base64 and that is not currently supported for embeddings");
            vec![]
        }
    })
    .collect();

    if vectors.iter().any(|x| x.is_empty()) {
        return Err(ServiceError::InternalServerError(
            "Embedding server responded with Base64 and that is not currently supported for embeddings".to_owned(),
        ));
    }

    if distance_phrase.is_some() {
        let distance_factor = distance_phrase.unwrap().distance_factor;
        let boost_vector = vectors.pop().unwrap();
        let embedding_vector = vectors.pop().unwrap();

        return Ok(embedding_vector
            .iter()
            .zip(boost_vector)
            .map(|(vec_elem, boost_vec_elem)| vec_elem + distance_factor * boost_vec_elem)
            .collect());
    }

    transaction.finish();

    match vectors.first() {
        Some(v) => Ok(v.clone()),
        None => Err(ServiceError::InternalServerError(
            "No dense embeddings returned from server".to_owned(),
        )),
    }
}

#[tracing::instrument]
pub async fn get_sparse_vector(
    message: String,
    embed_type: &str,
) -> Result<Vec<(u32, f32)>, ServiceError> {
    let origin_key = match embed_type {
        "doc" => "SPARSE_SERVER_DOC_ORIGIN",
        "query" => "SPARSE_SERVER_QUERY_ORIGIN",
        _ => unreachable!("Invalid embed_type passed"),
    };

    let server_origin = std::env::var(origin_key)
        .ok()
        .filter(|s| !s.is_empty())
        .ok_or(ServiceError::BadRequest(format!(
            "{} does not exist",
            origin_key
        )))?;

    let embedding_server_call = format!("{}/embed_sparse", server_origin);

    let sparse_vectors = ureq::post(&embedding_server_call)
        .set("Content-Type", "application/json")
        .set(
            "Authorization",
            &format!(
                "Bearer {}",
                get_env!("OPENAI_API_KEY", "OPENAI_API should be set")
            ),
        )
        .send_json(CustomSparseEmbedData {
            inputs: vec![message],
            encode_type: embed_type.to_string(),
            truncate: true,
        })
        .map_err(|err| {
            log::error!(
                "Failed parsing response from custom embedding server {:?}",
                err
            );
            ServiceError::BadRequest(format!("Failed making call to server {:?}", err))
        })?
        .into_json::<Vec<Vec<SpladeIndicies>>>()
        .map_err(|_e| {
            log::error!(
                "Failed parsing response from custom embedding server {:?}",
                _e
            );
            ServiceError::BadRequest(
                "Failed parsing response from custom embedding server".to_string(),
            )
        })?;

    match sparse_vectors.first() {
        Some(v) => Ok(v
            .iter()
            .map(|splade_idx| (*splade_idx).into_tuple())
            .collect()),
        None => Err(ServiceError::InternalServerError(
            "No sparse embeddings returned from server".to_owned(),
        )),
    }
}

#[tracing::instrument]
pub async fn create_embeddings(
    content_and_boosts: Vec<(String, Option<DistancePhrase>)>,
    embed_type: &str,
    dataset_config: ServerDatasetConfiguration,
    reqwest_client: reqwest::Client,
) -> Result<Vec<Vec<f32>>, ServiceError> {
    let parent_span = sentry::configure_scope(|scope| scope.get_span());
    let transaction: sentry::TransactionOrSpan = match &parent_span {
        Some(parent) => parent
            .start_child("create_embedding", "Create semantic dense embedding")
            .into(),
        None => {
            let ctx = sentry::TransactionContext::new(
                "create_embedding",
                "Create semantic dense embedding",
            );
            sentry::start_transaction(ctx).into()
        }
    };
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone())));

    let embedding_api_key = get_env!("OPENAI_API_KEY", "OPENAI_API_KEY should be set");
    let config_embedding_base_url = dataset_config.EMBEDDING_BASE_URL;
    let embedding_base_url = match config_embedding_base_url.as_str() {
        "" => get_env!("OPENAI_BASE_URL", "OPENAI_BASE_URL must be set").to_string(),
        "https://api.openai.com/v1" => {
            get_env!("OPENAI_BASE_URL", "OPENAI_BASE_URL must be set").to_string()
        }
        "https://embedding.trieve.ai" => std::env::var("EMBEDDING_SERVER_ORIGIN")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or("https://embedding.trieve.ai".to_string()),
        "https://embedding.trieve.ai/bge-m3" => std::env::var("EMBEDDING_SERVER_ORIGIN_BGEM3")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or("https://embedding.trieve.ai/bge-m3".to_string())
            .to_string(),
        "https://embedding.trieve.ai/jina-code" => {
            std::env::var("EMBEDDING_SERVER_ORIGIN_JINA_CODE")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or("https://embedding.trieve.ai/jina-code".to_string())
                .to_string()
        }
        _ => config_embedding_base_url.clone(),
    };

    let embedding_api_key =
        if config_embedding_base_url.as_str() == "https://embedding.trieve.ai/jina-code" {
            std::env::var("JINA_CODE_API_KEY")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or(embedding_api_key.to_string())
        } else {
            embedding_api_key.to_string()
        };

    let thirty_message_groups = content_and_boosts.chunks(30);

    let vec_futures: Vec<_> = thirty_message_groups
        .enumerate()
        .map(|(i, combined_messages)| {
            let messages = combined_messages
                .iter()
                .map(|(x, _)| x)
                .cloned()
                .collect::<Vec<String>>();

            let boost_phrase_and_index = combined_messages
                .iter()
                .enumerate()
                .filter_map(|(i, (_, y))| y.clone().map(|phrase| (i, phrase)))
                .collect::<Vec<(usize, DistancePhrase)>>();

            let boost_phrases = combined_messages
                .iter()
                .filter_map(|(_, y)| y.clone().map(|x| x.phrase.clone()))
                .collect::<Vec<String>>();

            let clipped_messages = messages
                .iter()
                .chain(boost_phrases.iter())
                .map(|message| {
                    if message.len() > 7000 {
                        message.chars().take(20000).collect()
                    } else {
                        message.clone()
                    }
                })
                .collect::<Vec<String>>();

            let input = match embed_type {
                "doc" => EmbeddingInput::StringArray(clipped_messages),
                "query" => EmbeddingInput::String(
                    format!(
                        "{}{}",
                        dataset_config.EMBEDDING_QUERY_PREFIX, &clipped_messages[0]
                    )
                    .to_string(),
                ),
                _ => EmbeddingInput::StringArray(clipped_messages),
            };

            let parameters = EmbeddingParameters {
                model: dataset_config.EMBEDDING_MODEL_NAME.to_string(),
                input,
                truncate: true,
            };

            let cur_client = reqwest_client.clone();
            let url = embedding_base_url.clone();

            let embedding_api_key = embedding_api_key.clone();

            let vectors_resp = async move {
                let embeddings_resp = cur_client
                .post(&format!("{}/embeddings?api-version=2023-05-15", url))
                .header("Authorization", &format!("Bearer {}", &embedding_api_key.clone()))
                .header("api-key", &embedding_api_key.clone())
                .header("Content-Type", "application/json")
                .json(&parameters)
                .send()
                .await
                .map_err(|_| {
                    ServiceError::BadRequest("Failed to send message to embedding server".to_string())
                })?
                .text()
                .await
                .map_err(|_| {
                    ServiceError::BadRequest("Failed to get text from embeddings".to_string())
                })?;

                let embeddings: EmbeddingResponse = format_response(embeddings_resp)
                    .map_err(|e| {
                        log::error!("Failed to format response from embeddings server {:?}", e);
                        ServiceError::InternalServerError(
                            "Failed to format response from embeddings server".to_owned(),
                        )
                    })?;

            let mut vectors: Vec<Vec<f32>> = embeddings
                .data
                .into_iter()
                .map(|x| match x.embedding {
                    EmbeddingOutput::Float(v) => v.iter().map(|x| *x as f32).collect(),
                    EmbeddingOutput::Base64(_) => {
                        log::error!("Embedding server responded with Base64 and that is not currently supported for embeddings");
                        vec![]
                    }
                })
                .collect();

                if vectors.iter().any(|x| x.is_empty()) {
                    return Err(ServiceError::InternalServerError(
                        "Embedding server responded with Base64 and that is not currently supported for embeddings".to_owned(),
                    ));
                }

            if !boost_phrase_and_index.is_empty() {
                let boost_vectors = vectors
                    .split_off(messages.len()).to_vec();

                let mut vectors_sorted = vectors.clone();
                for ((og_index, phrase), boost_vector) in boost_phrase_and_index.iter().zip(boost_vectors) {
                    vectors_sorted[*og_index] = vectors_sorted[*og_index]
                        .iter()
                        .zip(boost_vector)
                        .map(|(vector_elem, boost_vec_elem)| vector_elem + phrase.distance_factor * boost_vec_elem)
                        .collect();
                }

                return Ok((i, vectors_sorted));
            }

                Ok((i, vectors))
            };

            vectors_resp
        })
        .collect();

    let all_chunk_vectors: Vec<(usize, Vec<Vec<f32>>)> = futures::future::join_all(vec_futures)
        .await
        .into_iter()
        .collect::<Result<Vec<(usize, Vec<Vec<f32>>)>, ServiceError>>()?;

    let mut vectors_sorted = vec![];
    for index in 0..all_chunk_vectors.len() {
        let (_, vectors_i) = all_chunk_vectors.iter().find(|(i, _)| *i == index).ok_or(
            ServiceError::InternalServerError(
                "Failed to get index i (this should never happen)".to_string(),
            ),
        )?;

        vectors_sorted.extend(vectors_i.clone());
    }

    transaction.finish();
    Ok(vectors_sorted)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpladeEmbedding {
    pub embeddings: Vec<(u32, f32)>,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
struct SpladeIndicies {
    index: u32,
    value: f32,
}
impl SpladeIndicies {
    pub fn into_tuple(self) -> (u32, f32) {
        (self.index, self.value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomSparseEmbedData {
    pub inputs: Vec<String>,
    pub encode_type: String,
    pub truncate: bool,
}

#[tracing::instrument]
pub async fn get_sparse_vectors(
    messages: Vec<(String, Option<BoostPhrase>)>,
    embed_type: &str,
    reqwest_client: reqwest::Client,
) -> Result<Vec<Vec<(u32, f32)>>, ServiceError> {
    if messages.is_empty() {
        return Err(ServiceError::BadRequest(
            "No messages to encode".to_string(),
        ));
    }

    let contents = messages
        .clone()
        .into_iter()
        .map(|(x, _)| x)
        .collect::<Vec<String>>();
    let thirty_content_groups = contents.chunks(30);

    let filtered_boosts_with_index = messages
        .into_iter()
        .enumerate()
        .filter_map(|(i, (_, y))| y.map(|boost_phrase| (i, boost_phrase)))
        .collect::<Vec<(usize, BoostPhrase)>>();
    let filtered_boosts_with_index_groups = filtered_boosts_with_index.chunks(30);

    let vec_boost_futures: Vec<_> = filtered_boosts_with_index_groups
        .enumerate()
        .map(|(i, thirty_boosts)| {
            let cur_client = reqwest_client.clone();

            let origin_key = match embed_type {
                "doc" => "SPARSE_SERVER_DOC_ORIGIN",
                "query" => "SPARSE_SERVER_QUERY_ORIGIN",
                _ => unreachable!("Invalid embed_type passed"),
            };

            async move {
                let server_origin = std::env::var(origin_key)
                    .ok()
                    .filter(|s| !s.is_empty())
                    .ok_or(ServiceError::BadRequest(format!(
                        "env flag {} is not set",
                        origin_key
                    )))?;
                let embedding_server_call = format!("{}/embed_sparse", server_origin);

                let sparse_embed_req = CustomSparseEmbedData {
                    inputs: thirty_boosts
                        .iter()
                        .map(|(_, y)| y.phrase.clone())
                        .collect(),
                    encode_type: embed_type.to_string(),
                    truncate: true,
                };

                let sparse_vectors = cur_client
                    .post(&embedding_server_call)
                    .header("Content-Type", "application/json")
                    .header(
                        "Authorization",
                        &format!(
                            "Bearer {}",
                            get_env!("OPENAI_API_KEY", "OPENAI_API should be set")
                        ),
                    )
                    .json(&sparse_embed_req)
                    .send()
                    .await
                    .map_err(|err| {
                        log::error!(
                            "Failed parsing response from custom embedding server {:?}",
                            err
                        );
                        ServiceError::BadRequest(format!("Failed making call to server {:?}", err))
                    })?
                    .json::<Vec<Vec<SpladeIndicies>>>()
                    .await
                    .map_err(|_e| {
                        log::error!(
                            "Failed parsing response from custom embedding server {:?}",
                            _e
                        );
                        ServiceError::BadRequest(
                            "Failed parsing response from custom embedding server".to_string(),
                        )
                    })?;

                let index_vector_boosts: Vec<(usize, f64, Vec<SpladeIndicies>)> = thirty_boosts
                    .iter()
                    .zip(sparse_vectors)
                    .map(|((og_index, y), sparse_vector)| {
                        (*og_index, y.boost_factor, sparse_vector)
                    })
                    .collect();

                Ok((i, index_vector_boosts))
            }
        })
        .collect();

    let vec_content_futures: Vec<_> = thirty_content_groups
        .enumerate()
        .map(|(i, thirty_messages)| {
            let cur_client = reqwest_client.clone();

            let origin_key = match embed_type {
                "doc" => "SPARSE_SERVER_DOC_ORIGIN",
                "query" => "SPARSE_SERVER_QUERY_ORIGIN",
                _ => unreachable!("Invalid embed_type passed"),
            };

            async move {
                let server_origin = std::env::var(origin_key)
                    .ok()
                    .filter(|s| !s.is_empty())
                    .ok_or(ServiceError::BadRequest(format!(
                        "env flag {} is not set",
                        origin_key
                    )))?;
                let embedding_server_call = format!("{}/embed_sparse", server_origin);

                let sparse_embed_req = CustomSparseEmbedData {
                    inputs: thirty_messages.to_vec(),
                    encode_type: embed_type.to_string(),
                    truncate: true,
                };

                let sparse_vectors = cur_client
                    .post(&embedding_server_call)
                    .header("Content-Type", "application/json")
                    .header(
                        "Authorization",
                        &format!(
                            "Bearer {}",
                            get_env!("OPENAI_API_KEY", "OPENAI_API should be set")
                        ),
                    )
                    .json(&sparse_embed_req)
                    .send()
                    .await
                    .map_err(|err| {
                        log::error!(
                            "Failed parsing response from custom embedding server {:?}",
                            err
                        );
                        ServiceError::BadRequest(format!("Failed making call to server {:?}", err))
                    })?
                    .json::<Vec<Vec<SpladeIndicies>>>()
                    .await
                    .map_err(|_e| {
                        log::error!(
                            "Failed parsing response from custom embedding server {:?}",
                            _e
                        );
                        ServiceError::BadRequest(
                            "Failed parsing response from custom embedding server".to_string(),
                        )
                    })?;

                Ok((i, sparse_vectors))
            }
        })
        .collect();

    let all_content_vectors: Vec<(usize, Vec<Vec<SpladeIndicies>>)> =
        futures::future::join_all(vec_content_futures)
            .await
            .into_iter()
            .collect::<Result<Vec<(usize, Vec<Vec<SpladeIndicies>>)>, ServiceError>>()?;

    let mut content_vectors_sorted = vec![];
    for index in 0..all_content_vectors.len() {
        let (_, vectors_i) = all_content_vectors
            .iter()
            .find(|(i, _)| *i == index)
            .ok_or(ServiceError::InternalServerError(
                "Failed to get index i (this should never happen)".to_string(),
            ))?;

        content_vectors_sorted.extend(vectors_i.clone());
    }

    #[allow(clippy::type_complexity)]
    let all_boost_vectors: Vec<(usize, Vec<(usize, f64, Vec<SpladeIndicies>)>)> =
        futures::future::join_all(vec_boost_futures)
            .await
            .into_iter()
            .collect::<Result<Vec<(usize, Vec<(usize, f64, Vec<SpladeIndicies>)>)>, ServiceError>>(
            )?;

    for (_, boost_vectors) in all_boost_vectors {
        for (og_index, boost_amt, boost_vector) in boost_vectors {
            content_vectors_sorted[og_index] = content_vectors_sorted[og_index]
                .iter()
                .map(|splade_indice| {
                    // Any is here because we multiply all of the matching indices by the boost amount and the boost amount is not unique to any index
                    if boost_vector
                        .iter()
                        .any(|boost_splade_indice| boost_splade_indice.index == splade_indice.index)
                    {
                        SpladeIndicies {
                            index: splade_indice.index,
                            value: splade_indice.value * (boost_amt as f32),
                        }
                    } else {
                        SpladeIndicies {
                            index: splade_indice.index,
                            value: splade_indice.value,
                        }
                    }
                })
                .collect();
        }
    }

    Ok(content_vectors_sorted
        .iter()
        .map(|sparse_vector| {
            sparse_vector
                .iter()
                .map(|splade_idx| (*splade_idx).into_tuple())
                .collect()
        })
        .collect())
}

#[derive(Debug, Serialize, Deserialize)]
struct ScorePair {
    index: usize,
    score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrossEncoderData {
    pub query: String,
    pub texts: Vec<String>,
    pub truncate: bool,
}

#[tracing::instrument]
pub async fn cross_encoder(
    query: String,
    page_size: u64,
    results: Vec<ScoreChunkDTO>,
    dataset_config: &ServerDatasetConfiguration,
) -> Result<Vec<ScoreChunkDTO>, actix_web::Error> {
    let parent_span = sentry::configure_scope(|scope| scope.get_span());
    let transaction: sentry::TransactionOrSpan = match &parent_span {
        Some(parent) => parent
            .start_child("Cross Encoder", "Cross Encode semantic and hybrid chunks")
            .into(),
        None => {
            let ctx = sentry::TransactionContext::new(
                "Cross Encoder",
                "Cross Encode semantic and hybrid chunks",
            );
            sentry::start_transaction(ctx).into()
        }
    };
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone())));

    let server_origin: String = dataset_config.RERANKER_BASE_URL.clone();

    let embedding_server_call = format!("{}/rerank", server_origin);

    if results.is_empty() {
        return Ok(vec![]);
    }

    let mut results = results.clone();

    if results.len() <= 20 {
        let request_docs = results
            .clone()
            .into_iter()
            .map(|x| {
                let chunk = match x.metadata[0].clone() {
                    ChunkMetadataTypes::Metadata(metadata) => Ok(metadata.clone()),
                    _ => Err(ServiceError::BadRequest("Metadata not found".to_string())),
                }?;

                Ok(convert_html_to_text(
                    &(chunk.chunk_html.unwrap_or_default()),
                ))
            })
            .collect::<Result<Vec<String>, ServiceError>>()?;
        let resp = ureq::post(&embedding_server_call)
            .set("Content-Type", "application/json")
            .set(
                "Authorization",
                &format!(
                    "Bearer {}",
                    get_env!("OPENAI_API_KEY", "OPENAI_API should be set")
                ),
            )
            .send_json(CrossEncoderData {
                query: query.clone(),
                texts: request_docs,
                truncate: true,
            })
            .map_err(|err| {
                ServiceError::BadRequest(format!("Failed making call to server {:?}", err))
            })?
            .into_json::<Vec<ScorePair>>()
            .map_err(|_e| {
                log::error!(
                    "Failed parsing response from custom embedding server {:?}",
                    _e
                );
                ServiceError::BadRequest(
                    "Failed parsing response from custom embedding server".to_string(),
                )
            })?;

        resp.into_iter().for_each(|pair| {
            results.index_mut(pair.index).score = pair.score as f64;
        });
    } else {
        let vec_futures: Vec<_> = results
            .chunks_mut(20)
            .map(|docs_chunk| {
                let query = query.clone();
                let cur_client = reqwest::Client::new();
                let embedding_api_key = get_env!("OPENAI_API_KEY", "OPENAI_API should be set");
                let url = embedding_server_call.clone();

                let vectors_resp = async move {
                    let request_docs = docs_chunk
                        .iter_mut()
                        .map(|x| {
                            let chunk = match x.metadata[0].clone() {
                                ChunkMetadataTypes::Metadata(metadata) => Ok(metadata.clone()),
                                _ => {
                                    Err(ServiceError::BadRequest("Metadata not found".to_string()))
                                }
                            }?;

                            Ok(convert_html_to_text(
                                &(chunk.chunk_html.unwrap_or_default()),
                            ))
                        })
                        .collect::<Result<Vec<String>, ServiceError>>()?;

                    let parameters = CrossEncoderData {
                        query: query.clone(),
                        texts: request_docs,
                        truncate: true,
                    };

                    let embeddings_resp = cur_client
                        .post(&url)
                        .header(
                            "Authorization",
                            &format!(
                                "Bearer {}",
                                get_env!("OPENAI_API_KEY", "OPENAI_API should be set")
                            ),
                        )
                        .header("api-key", &embedding_api_key.to_string())
                        .header("Content-Type", "application/json")
                        .json(&parameters)
                        .send()
                        .await
                        .map_err(|_| {
                            ServiceError::BadRequest(
                                "Failed to send message to embedding server".to_string(),
                            )
                        })?
                        .text()
                        .await
                        .map_err(|_| {
                            ServiceError::BadRequest(
                                "Failed to get text from embeddings".to_string(),
                            )
                        })?;

                    let embeddings: Vec<ScorePair> = serde_json::from_str(&embeddings_resp)
                        .map_err(|e| {
                            log::error!("Failed to format response from embeddings server {:?}", e);
                            ServiceError::InternalServerError(
                                "Failed to format response from embeddings server".to_owned(),
                            )
                        })?;

                    embeddings.into_iter().for_each(|pair| {
                        docs_chunk.index_mut(pair.index).score = pair.score as f64;
                    });

                    Ok(())
                };

                vectors_resp
            })
            .collect();

        futures::future::join_all(vec_futures)
            .await
            .into_iter()
            .collect::<Result<(), ServiceError>>()?;
    }

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    results.truncate(page_size.try_into().unwrap());

    transaction.finish();
    Ok(results)
}

pub fn get_bm25_embeddings(
    chunks_and_boost: Vec<(String, Option<BoostPhrase>)>,
    avg_len: f32,
    b: f32,
    k: f32,
) -> Vec<Vec<(u32, f32)>> {
    term_frequency(tokenize_batch(chunks_and_boost), avg_len, b, k)
}

fn tokenize(text: String) -> Vec<String> {
    let mut en_stem =
        tantivy::tokenizer::TextAnalyzer::builder(tantivy::tokenizer::SimpleTokenizer::default())
            .filter(tantivy::tokenizer::RemoveLongFilter::limit(40))
            .filter(tantivy::tokenizer::LowerCaser)
            .filter(tantivy::tokenizer::Stemmer::new(
                tantivy::tokenizer::Language::English,
            ))
            .build();

    let mut stream = en_stem.token_stream(&text);
    let mut tokens: Vec<String> = vec![];
    while stream.advance() {
        tokens.push(stream.token().text.clone());
    }

    tokens
}

pub fn tokenize_batch(
    chunks: Vec<(String, Option<BoostPhrase>)>,
) -> Vec<(Vec<String>, Option<BoostPhrase>)> {
    chunks
        .into_iter()
        .map(|(chunk, boost)| (tokenize(chunk), boost))
        .collect()
}

pub fn term_frequency(
    batched_tokens: Vec<(Vec<String>, Option<BoostPhrase>)>,
    avg_len: f32,
    b: f32,
    k: f32,
) -> Vec<Vec<(u32, f32)>> {
    batched_tokens
        .iter()
        .map(|(batch, boost_phrase)| {
            // Get Full Counts
            let mut raw_freqs = HashMap::new();
            batch.iter().for_each(|token| {
                match raw_freqs.get(token) {
                    Some(val) => raw_freqs.insert(token, *val + 1f32),
                    None => raw_freqs.insert(token, 1f32),
                };
            });

            let mut tf_map = HashMap::new();

            let doc_len = batch.len() as f32;

            for token in batch.iter() {
                let token_id =
                    (murmur3_32(&mut Cursor::new(token), 0).unwrap() as i32).unsigned_abs();
                let num_occurences = raw_freqs[token];

                let top = num_occurences * (k + 1f32);
                let bottom = num_occurences + k * (1f32 - b + b * doc_len / avg_len);

                tf_map.insert(token_id, top / bottom);
            }

            if let Some(boost_phrase) = boost_phrase {
                let tokenized_phrase = tokenize(boost_phrase.phrase.clone());
                for token in tokenized_phrase {
                    let token_id =
                        (murmur3_32(&mut Cursor::new(token), 0).unwrap() as i32).unsigned_abs();

                    let value = tf_map[&token_id];
                    tf_map.insert(token_id, boost_phrase.boost_factor as f32 * value);
                }
            }

            tf_map.into_iter().collect::<Vec<(u32, f32)>>()
        })
        .collect()
}
