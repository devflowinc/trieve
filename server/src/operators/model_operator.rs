use crate::{
    data::models::ServerDatasetConfiguration, errors::ServiceError, get_env,
    handlers::chunk_handler::ScoreChunkDTO,
};
use openai_dive::v1::{
    api::Client,
    resources::embedding::{EmbeddingInput, EmbeddingParameters},
};
use serde::{Deserialize, Serialize};

pub async fn create_embedding(
    message: &str,
    dataset_config: ServerDatasetConfiguration,
) -> Result<Vec<f32>, actix_web::Error> {
    let open_ai_api_key = get_env!("OPENAI_API_KEY", "OPENAI_API_KEY should be set").into();
    let base_url = dataset_config
        .EMBEDDING_BASE_URL
        .unwrap_or("https://api.openai.com/v1".to_string());

    let base_url = if base_url.is_empty() {
        "https://api.openai.com/v1".to_string()
    } else {
        base_url
    };

    let client = Client {
        http_client: reqwest::Client::new(),
        api_key: open_ai_api_key,
        base_url,
        organization: None,
    };

    // Vectorize
    let parameters = EmbeddingParameters {
        model: "text-embedding-3-small".to_string(),
        input: EmbeddingInput::String(message.to_string()),
        user: None,
        encoding_format: None,
        dimensions: None,
    };

    let embeddings = client
        .embeddings()
        .create(parameters)
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    let vector = embeddings.data.first().unwrap().embedding.clone();
    Ok(vector.iter().map(|&x| x as f32).collect())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SpladeEmbedding {
    pub embeddings: Vec<(u32, f32)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomSparseEmbedData {
    pub input: String,
    pub encode_type: String,
}

pub async fn get_splade_doc_embedding(message: &str) -> Result<Vec<(u32, f32)>, ServiceError> {
    if message.is_empty() {
        return Err(ServiceError::BadRequest(
            "Cannot encode empty query".to_string(),
        ));
    }
    let mut embedding_server_call: String = get_env!(
        "GPU_SERVER_ORIGIN",
        "GPU_SERVER_ORIGIN should be set if this is called"
    )
    .to_string();
    embedding_server_call.push_str("/sparse_encode");

    let client = reqwest::Client::new();
    let resp = client
        .post(embedding_server_call)
        .json(&CustomSparseEmbedData {
            input: message.to_string(),
            encode_type: "doc".to_string(),
        })
        .send()
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Failed making call to server {:?}", err)))?
        .json::<SpladeEmbedding>()
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

    Ok(resp.embeddings)
}

pub async fn get_splade_query_embedding(message: &str) -> Result<Vec<(u32, f32)>, ServiceError> {
    let mut embedding_server_call: String = get_env!(
        "GPU_SERVER_ORIGIN",
        "GPU_SERVER_ORIGIN should be set if this is called"
    )
    .to_string();
    embedding_server_call.push_str("/sparse_encode");

    let client = reqwest::Client::new();
    let resp = client
        .post(embedding_server_call)
        .json(&CustomSparseEmbedData {
            input: message.to_string(),
            encode_type: "query".to_string(),
        })
        .send()
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Failed making call to server {:?}", err)))?
        .json::<SpladeEmbedding>()
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

    Ok(resp.embeddings)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReRankResponse {
    pub docs: Vec<(String, f32)>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CrossEncoderData {
    pub query: String,
    pub docs: Vec<String>,
}

pub async fn cross_encoder(
    query: String,
    results: Vec<ScoreChunkDTO>,
) -> Result<Vec<ScoreChunkDTO>, actix_web::Error> {
    let mut embedding_server_call: String = get_env!(
        "GPU_SERVER_ORIGIN",
        "GPU_SERVER_ORIGIN should be set if this is called"
    )
    .to_string();
    embedding_server_call.push_str("/rerank");

    if results.is_empty() {
        return Err(ServiceError::BadRequest("Cannot rerank empty results".to_string()).into());
    }

    let request_docs = results
        .clone()
        .into_iter()
        .map(|x| x.metadata[0].clone().content)
        .collect::<Vec<String>>();

    let client = reqwest::Client::new();
    let resp = client
        .post(embedding_server_call)
        .json(&CrossEncoderData {
            query: query.to_string(),
            docs: request_docs,
        })
        .send()
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Failed making call to server {:?}", err)))?
        .json::<ReRankResponse>()
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

    let mut results: Vec<ScoreChunkDTO> = results
        .clone()
        .iter_mut()
        .map(|x| {
            x.score = resp
                .docs
                .iter()
                .find(|s| s.0 == x.metadata[0].content)
                .unwrap()
                .1 as f64;

            x.clone()
        })
        .collect();

    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    Ok(results)
}
