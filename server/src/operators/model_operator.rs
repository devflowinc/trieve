use crate::{
    data::models::ServerDatasetConfiguration, errors::ServiceError, get_env,
    handlers::chunk_handler::ScoreChunkDTO,
};
use openai_dive::v1::{
    api::Client,
    helpers::format_response,
    resources::embedding::{EmbeddingInput, EmbeddingOutput, EmbeddingResponse},
};
use serde::{Deserialize, Serialize};
use std::ops::IndexMut;

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingParameters {
    /// Input text to embed, encoded as a string or array of tokens.
    /// To embed multiple inputs in a single request, pass an array of strings or array of token arrays.
    pub input: EmbeddingInput,
    /// ID of the model to use.
    pub model: String,
}

#[tracing::instrument]
pub async fn create_embeddings(
    message: Vec<String>,
    embed_type: &str,
    dataset_config: ServerDatasetConfiguration,
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

    let open_ai_api_key = get_env!("OPENAI_API_KEY", "OPENAI_API_KEY should be set").into();
    let base_url = dataset_config.EMBEDDING_BASE_URL;

    let base_url = if base_url.is_empty() || base_url == "https://api.openai.com/v1" {
        get_env!("OPENAI_BASE_URL", "OPENAI_BASE_URL must be set").to_string()
    } else if base_url.contains("https://embedding.trieve.ai") {
        match std::env::var("EMBEDDING_SERVER_ORIGIN")
            .ok()
            .filter(|s| !s.is_empty())
        {
            Some(origin) => origin,
            None => get_env!(
                "GPU_SERVER_ORIGIN",
                "GPU_SERVER_ORIGIN should be set if this is called"
            )
            .to_string(),
        }
    } else {
        base_url
    };

    let client = Client {
        http_client: reqwest::Client::new(),
        api_key: open_ai_api_key,
        base_url,
        organization: None,
    };

    let clipped_messages = message
        .iter()
        .map(|msg| {
            if msg.len() > 7000 {
                msg.chars().take(20000).collect()
            } else {
                msg.clone()
            }
        })
        .collect::<Vec<String>>();

    let input = match embed_type {
        "doc" => EmbeddingInput::StringArray(clipped_messages),
        "query" => EmbeddingInput::String(
            format!(
                "{}{}",
                dataset_config.EMBEDDING_QUERY_PREFIX,
                clipped_messages
                    .first()
                    .unwrap_or(&"Arbitrary because query is empty".to_string())
            )
            .to_string(),
        ),
        _ => EmbeddingInput::StringArray(clipped_messages),
    };

    // Vectorize
    let parameters = EmbeddingParameters {
        model: dataset_config.EMBEDDING_MODEL_NAME.to_string(),
        input,
    };

    let embeddings_resp = ureq::post(&format!(
        "{}/embeddings?api-version=2023-05-15",
        client.base_url
    ))
    .set("Authorization", &format!("Bearer {}", client.api_key))
    .set("api-key", &client.api_key)
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

    let vectors: Vec<Vec<f32>> = embeddings
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

    transaction.finish();
    Ok(vectors)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpladeEmbedding {
    pub embeddings: Vec<(u32, f32)>,
}

#[derive(Debug, Deserialize)]
struct SpladeIndicies {
    index: u32,
    value: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomSparseEmbedData {
    pub inputs: String,
    pub encode_type: String,
    pub truncate: bool,
}

#[tracing::instrument]
pub async fn get_splade_embedding(
    message: &str,
    embed_type: &str,
) -> Result<Vec<(u32, f32)>, ServiceError> {
    if message.is_empty() {
        return Err(ServiceError::BadRequest(
            "Cannot encode empty query".to_string(),
        ));
    }

    let origin_key = match embed_type {
        "doc" => "SPARSE_SERVER_DOC_ORIGIN",
        "query" => "SPARSE_SERVER_QUERY_ORIGIN",
        _ => unreachable!("Invalid embed_type passed"),
    };

    let server_origin = match std::env::var(origin_key).ok().filter(|s| !s.is_empty()) {
        Some(origin) => origin,
        None => get_env!(
            "GPU_SERVER_ORIGIN",
            "GPU_SERVER_ORIGIN should be set if this is called"
        )
        .to_string(),
    };
    let embedding_server_call = format!("{}/embed_sparse", server_origin);

    let resp = ureq::post(&embedding_server_call)
        .set("Content-Type", "application/json")
        .set(
            "Authorization",
            &format!(
                "Bearer {}",
                get_env!("OPENAI_API_KEY", "OPENAI_API should be set")
            ),
        )
        .send_json(CustomSparseEmbedData {
            inputs: message.to_string(),
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

    let first_vector = resp.get(0).ok_or(ServiceError::BadRequest(
        "Failed getting sparse vector from embedding server".to_string(),
    ))?;

    Ok(first_vector
        .iter()
        .map(|splade_idx| (splade_idx.index, splade_idx.value))
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

    let server_origin: String = match std::env::var("RERANKER_SERVER_ORIGIN")
        .ok()
        .filter(|s| !s.is_empty())
    {
        Some(origin) => origin,
        None => get_env!(
            "GPU_SERVER_ORIGIN",
            "GPU_SERVER_ORIGIN should be set if this is called"
        )
        .to_string(),
    };

    let embedding_server_call = format!("{}/rerank", server_origin);

    if results.is_empty() {
        return Ok(vec![]);
    }

    let request_docs = results
        .clone()
        .into_iter()
        .map(|x| x.metadata[0].clone().content)
        .collect::<Vec<String>>();

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
            query,
            texts: request_docs,
            truncate: true,
        })
        .map_err(|err| ServiceError::BadRequest(format!("Failed making call to server {:?}", err)))?
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

    let mut results = results.clone();

    resp.into_iter().for_each(|pair| {
        results.index_mut(pair.index).score = pair.score as f64;
    });

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    results.truncate(page_size.try_into().unwrap());

    transaction.finish();
    Ok(results)
}
