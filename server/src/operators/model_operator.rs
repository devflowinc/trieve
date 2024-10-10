use crate::{
    data::models::{ChunkMetadataTypes, DatasetConfiguration, ScoreChunkDTO},
    errors::ServiceError,
    get_env,
    handlers::chunk_handler::{FullTextBoost, SemanticBoost},
};
use murmur3::murmur3_32;
use openai_dive::v1::resources::embedding::EmbeddingInput;
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
    /// Truncate the input to the maximum length of the model.
    pub truncate: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DenseEmbedData {
    pub data: Vec<EmbeddingInner>,
}

impl DenseEmbedData {
    pub fn to_vec(&self) -> Vec<Vec<f32>> {
        self.data.iter().map(|inner| {
            inner.embedding.clone()
        }).collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingInner {
    embedding: Vec<f32>,
}

#[tracing::instrument]
pub async fn get_dense_vector(
    message: String,
    semantic_boost: Option<SemanticBoost>,
    embed_type: &str,
    dataset_config: DatasetConfiguration,
) -> Result<Vec<f32>, ServiceError> {
    let parent_span = sentry::configure_scope(|scope| scope.get_span());
    let transaction: sentry::TransactionOrSpan = match &parent_span {
        Some(parent) => parent
            .start_child("get_dense_vector", "Create semantic dense embedding")
            .into(),
        None => {
            let ctx = sentry::TransactionContext::new(
                "get_dense_vector",
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

    let clipped_message: String = message.chars().take(20000).collect();
    let mut messages = vec![format!(
        "{}{}",
        dataset_config.EMBEDDING_QUERY_PREFIX, &clipped_message
    )
    .to_string()];
    if let Some(semantic_boost) = semantic_boost.as_ref() {
        if semantic_boost.distance_factor == 0.0 || semantic_boost.phrase.is_empty() {
            return Err(ServiceError::BadRequest(
                "Semantic boost phrase is empty or distance factor is 0. Boost phrase must not be empty and distance factor must be greater than 0".to_string(),
            ));
        }

        let clipped_boost: String = semantic_boost.phrase.chars().take(20000).collect();
        messages.push(clipped_boost);
    }

    let input = EmbeddingInput::StringArray(messages);
    let parameters = EmbeddingParameters {
        model: dataset_config.EMBEDDING_MODEL_NAME.to_string(),
        input,
        truncate: true,
    };

    let embeddings_resp_a = ureq::post(&format!(
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

    println!("data {:?}", embeddings_resp_a);

    let embeddings_resp = embeddings_resp_a
        .into_json::<DenseEmbedData>()
        .map_err(|err| {
            ServiceError::InternalServerError(format!(
                "Failed to format response from embeddings server {:?}",
                err
            ))
        })?;

    let mut vectors: Vec<Vec<f32>> = embeddings_resp.to_vec();
    if vectors.iter().any(|x| x.is_empty()) {
        return Err(ServiceError::InternalServerError(
            "Embedding server responded with Base64 and that is not currently supported for embeddings".to_owned(),
        ));
    }

    if let Some(semantic_boost) = semantic_boost {
        let distance_factor = semantic_boost.distance_factor;
        let boost_vector = match vectors.pop() {
            Some(v) => v,
            None => {
                return Err(ServiceError::InternalServerError(
                    "No dense embedding returned from server for boost_vector".to_owned(),
                ))
            }
        };
        let embedding_vector = match vectors.pop() {
            Some(v) => v,
            None => {
                return Err(ServiceError::InternalServerError(
                    "No dense embedding returned from server for embedding_vector".to_owned(),
                ))
            }
        };

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
    fulltext_boost: Option<FullTextBoost>,
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

    let clipped_message: String = message.chars().take(20000).collect();
    let mut inputs = vec![clipped_message.clone()];
    if let Some(fulltext_boost) = fulltext_boost.as_ref() {
        if fulltext_boost.phrase.is_empty() {
            return Err(ServiceError::BadRequest(
                "Fulltext boost phrase is empty. Non-empty phrase must be specified.".to_string(),
            ));
        }

        let clipped_boost: String = fulltext_boost.phrase.chars().take(20000).collect();
        inputs.push(clipped_boost);
    }

    let embedding_server_call = format!("{}/embed_sparse", server_origin);

    let mut sparse_vectors = ureq::post(&embedding_server_call)
        .set("Content-Type", "application/json")
        .set(
            "Authorization",
            &format!(
                "Bearer {}",
                get_env!("OPENAI_API_KEY", "OPENAI_API should be set")
            ),
        )
        .send_json(CustomSparseEmbedData {
            inputs,
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

    if let Some(fulltext_boost) = fulltext_boost {
        let boost_amt = fulltext_boost.boost_factor;
        let boost_vector = match sparse_vectors.pop() {
            Some(v) => v,
            None => {
                return Err(ServiceError::InternalServerError(
                    "No sparse vector returned from server for boost_vector".to_owned(),
                ))
            }
        };
        let query_vector = match sparse_vectors.pop() {
            Some(v) => v,
            None => {
                return Err(ServiceError::InternalServerError(
                    "No sparse vector returned from server for embedding_vector".to_owned(),
                ))
            }
        };

        let boosted_query_vector = query_vector
            .iter()
            .map(|splade_indice| {
                if boost_vector
                    .iter()
                    .any(|boost_splade_indice| boost_splade_indice.index == splade_indice.index)
                {
                    SpladeIndicies {
                        index: splade_indice.index,
                        value: splade_indice.value * (boost_amt as f32),
                    }
                    .into_tuple()
                } else {
                    SpladeIndicies {
                        index: splade_indice.index,
                        value: splade_indice.value,
                    }
                    .into_tuple()
                }
            })
            .collect();

        return Ok(boosted_query_vector);
    }

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
pub async fn get_dense_vectors(
    content_and_distances: Vec<(String, Option<SemanticBoost>)>,
    embed_type: &str,
    dataset_config: DatasetConfiguration,
    reqwest_client: reqwest::Client,
) -> Result<Vec<Vec<f32>>, ServiceError> {
    let parent_span = sentry::configure_scope(|scope| scope.get_span());
    let transaction: sentry::TransactionOrSpan = match &parent_span {
        Some(parent) => parent
            .start_child("get_dense_vector", "Create semantic dense embedding")
            .into(),
        None => {
            let ctx = sentry::TransactionContext::new(
                "get_dense_vector",
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

    let (contents, distance_phrases): (Vec<_>, Vec<_>) =
        content_and_distances.clone().into_iter().unzip();
    let thirty_content_groups = contents.chunks(30);

    let filtered_distances_with_index = distance_phrases
        .clone()
        .iter()
        .enumerate()
        .filter_map(|(index, distance_phrase)| {
            distance_phrase
                .clone()
                .map(|distance_phrase| (index, distance_phrase))
        })
        .collect::<Vec<(usize, SemanticBoost)>>();
    let thirty_filterted_distances_with_indices = filtered_distances_with_index.chunks(30);

    let vec_distance_futures: Vec<_> = thirty_filterted_distances_with_indices
        .map(|thirty_distances| {
            let distance_phrases = thirty_distances
                .iter()
                .map(|(_, x)| x.phrase.clone())
                .collect::<Vec<String>>();

            let clipped_messages = distance_phrases
                .iter()
                .map(|message| {
                        message.chars().take(12000).collect()
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
                truncate: true
            };

            let cur_client = reqwest_client.clone();
            let url = embedding_base_url.clone();

            let embedding_api_key = embedding_api_key.clone();

            
            async move {
                let embeddings_resp = cur_client
                    .post(format!("{}/embeddings?api-version=2023-05-15", url))
                    .header("Authorization", &format!("Bearer {}", &embedding_api_key.clone()))
                    .header("api-key", &embedding_api_key.clone())
                    .header("Content-Type", "application/json")
                    .json(&parameters)
                    .send()
                    .await
                    .map_err(|_| {
                        ServiceError::BadRequest("Failed to send message to embedding server".to_string())
                    })?
                    .json::<DenseEmbedData>()
                    .await
                    .map_err(|err| {
                        ServiceError::BadRequest(format!("Failed to format text from embeddings {}", err))
                    })?;

                let vectors_and_boosts: Vec<(Vec<f32>, &(usize, SemanticBoost))> = embeddings_resp
                    .to_vec()
                    .into_iter()
                    .zip(thirty_distances)
                    .collect();

                    if vectors_and_boosts.iter().any(|x| x.0.is_empty()) {
                        return Err(ServiceError::InternalServerError(
                            "Embedding server responded with Base64 and that is not currently supported for embeddings".to_owned(),
                        ));
                    }

                Ok(vectors_and_boosts)
            }
        })
        .collect();

    let vec_content_futures: Vec<_> = thirty_content_groups
        .map(|messages| {
            let clipped_messages = messages
                .iter()
                .map(|message| {
                        message.chars().take(12000).collect()
                })
                .collect::<Vec<String>>();

            let input = match embed_type {
                "doc" => EmbeddingInput::StringArray(clipped_messages),
                "query" => EmbeddingInput::String(
                    format!(
                        "{}{}",
                        dataset_config.EMBEDDING_QUERY_PREFIX, &clipped_messages.get(0).unwrap_or(&"".to_string())
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

                

                async move {
                    let embeddings_resp = cur_client
                    .post(format!("{}/embeddings?api-version=2023-05-15", url))
                    .header("Authorization", &format!("Bearer {}", &embedding_api_key.clone()))
                    .header("api-key", &embedding_api_key.clone())
                    .header("Content-Type", "application/json")
                    .json(&parameters)
                    .send()
                    .await
                    .map_err(|_| {
                        ServiceError::BadRequest("Failed to send message to embedding server".to_string())
                    })?
                    .json::<DenseEmbedData>()
                    .await
                    .map_err(|err| {
                        ServiceError::BadRequest(format!("Failed to get text from embeddings {:?}", err))
                    })?;

                    let vectors: Vec<Vec<f32>> = embeddings_resp.to_vec();

                    Ok(vectors)
                }

            })
        .collect();

    let mut content_vectors: Vec<_> = futures::future::join_all(vec_content_futures)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, ServiceError>>()?
        .into_iter()
        .flatten()
        .collect();

    let distance_vectors: Vec<_> = futures::future::join_all(vec_distance_futures)
        .await
        .into_iter()
        .collect::<Result<Vec<_>, ServiceError>>()?
        .into_iter()
        .flatten()
        .collect();

    if !distance_vectors.is_empty() {
        content_vectors = content_vectors
            .into_iter()
            .enumerate()
            .map(|(i, message)| {
                let distance_vector = distance_vectors
                    .iter()
                    .find(|(_, (og_index, _))| *og_index == i);
                match distance_vector {
                    Some((distance_vec, (_, distance_phrase))) => {
                        let distance_factor = distance_phrase.distance_factor;
                        message
                            .iter()
                            .zip(distance_vec)
                            .map(|(vec_elem, distance_elem)| {
                                vec_elem + distance_factor * distance_elem
                            })
                            .collect()
                    }
                    None => message,
                }
            })
            .collect();
    }

    transaction.finish();
    Ok(content_vectors)
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
    content_and_boosts: Vec<(String, Option<FullTextBoost>)>,
    embed_type: &str,
    reqwest_client: reqwest::Client,
) -> Result<Vec<Vec<(u32, f32)>>, ServiceError> {
    if content_and_boosts.is_empty() {
        return Err(ServiceError::BadRequest(
            "No messages to encode".to_string(),
        ));
    }

    let contents = content_and_boosts
        .clone()
        .into_iter()
        .map(|(x, _)| x)
        .collect::<Vec<String>>();
    let thirty_content_groups = contents.chunks(30);

    let filtered_boosts_with_index = content_and_boosts
        .into_iter()
        .enumerate()
        .filter_map(|(i, (_, y))| y.map(|fulltext_boost| (i, fulltext_boost)))
        .collect::<Vec<(usize, FullTextBoost)>>();
    let thirty_filtered_boosts_with_indices = filtered_boosts_with_index.chunks(30);

    let vec_boost_futures: Vec<_> = thirty_filtered_boosts_with_indices
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

                let clipped_messages = thirty_boosts
                    .iter()
                    .map(|(_, message)| message.phrase.chars().take(50000).collect())
                    .collect::<Vec<String>>();

                let sparse_embed_req = CustomSparseEmbedData {
                    inputs: clipped_messages,
                    encode_type: embed_type.to_string(),
                    truncate: true,
                };

                let embedding_response = cur_client
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
                            "Failed sending request from custom embedding server {:?}",
                            err
                        );
                        ServiceError::InternalServerError(format!(
                            "Failed making call to server {:?}",
                            err
                        ))
                    })?
                    .text()
                    .await
                    .map_err(|_| {
                        ServiceError::InternalServerError(
                            "Failed to get text from embeddings".to_string(),
                        )
                    })?;

                let sparse_vectors = serde_json::from_str::<Vec<Vec<SpladeIndicies>>>(
                    &embedding_response,
                )
                .map_err(|_e| {
                    log::error!(
                        "Failed parsing response from custom embedding server {:?}",
                        embedding_response
                    );
                    ServiceError::InternalServerError(format!(
                        "Failed parsing response from custom embedding server {:?}",
                        embedding_response
                    ))
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

                let clipped_messages = thirty_messages
                    .iter()
                    .map(|message| message.chars().take(50000).collect())
                    .collect::<Vec<String>>();

                let sparse_embed_req = CustomSparseEmbedData {
                    inputs: clipped_messages,
                    encode_type: embed_type.to_string(),
                    truncate: true,
                };

                let embedding_response = cur_client
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
                            "Failed sending request from custom embedding server {:?}",
                            err
                        );
                        ServiceError::InternalServerError(format!(
                            "Failed making call to server {:?}",
                            err
                        ))
                    })?
                    .text()
                    .await
                    .map_err(|_| {
                        ServiceError::InternalServerError(
                            "Failed to get text from embeddings".to_string(),
                        )
                    })?;

                let sparse_vectors = serde_json::from_str::<Vec<Vec<SpladeIndicies>>>(
                    &embedding_response,
                )
                .map_err(|_e| {
                    log::error!(
                        "Failed parsing response from custom embedding server {:?}",
                        embedding_response
                    );
                    ServiceError::InternalServerError(format!(
                        "Failed parsing response from custom embedding server {:?}",
                        embedding_response
                    ))
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
    dataset_config: &DatasetConfiguration,
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
    chunks_and_boost: Vec<(String, Option<FullTextBoost>)>,
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
    chunks: Vec<(String, Option<FullTextBoost>)>,
) -> Vec<(Vec<String>, Option<FullTextBoost>)> {
    chunks
        .into_iter()
        .map(|(chunk, boost)| (tokenize(chunk), boost))
        .collect()
}

pub fn term_frequency(
    batched_tokens: Vec<(Vec<String>, Option<FullTextBoost>)>,
    avg_len: f32,
    b: f32,
    k: f32,
) -> Vec<Vec<(u32, f32)>> {
    batched_tokens
        .iter()
        .map(|(batch, fulltext_boost_option)| {
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
                let num_occurences = raw_freqs.get(token).unwrap_or(&0f32);

                let top = num_occurences * (k + 1f32);
                let bottom = num_occurences + k * (1f32 - b + b * doc_len / avg_len);

                tf_map.insert(token_id, top / bottom);
            }

            if let Some(fulltext_boost) = fulltext_boost_option {
                let tokenized_phrase = tokenize(fulltext_boost.phrase.clone());
                for token in tokenized_phrase {
                    let token_id =
                        (murmur3_32(&mut Cursor::new(token), 0).unwrap() as i32).unsigned_abs();

                    let value = tf_map.get(&token_id).unwrap_or(&0f32);
                    tf_map.insert(token_id, fulltext_boost.boost_factor as f32 * value);
                }
            }

            tf_map.into_iter().collect::<Vec<(u32, f32)>>()
        })
        .collect()
}
