use actix_web::web;
use candle_nn::{var_builder::SimpleBackend, VarBuilder};
use candle_transformers::models::bert::{
    BertModel, Config as BertModelConfig, DTYPE as BertModelDTYPE,
};
use hf_hub::{api::sync::Api, Repo, RepoType};
use openai_dive::v1::{api::Client, resources::embedding::EmbeddingParameters};

use serde::{Deserialize, Serialize};
use tokenizers::{tokenizer::Tokenizer, PaddingParams, PaddingStrategy, TruncationParams}; // a fast, portable hash library

use crate::{data::models::DatasetConfiguration, errors::ServiceError, get_env, AppMutexStore};

pub struct CrossEncoder {
    pub tokenizer: Tokenizer,
    pub model: BertModel,
}

pub fn initalize_cross_encoder() -> CrossEncoder {
    let model_id = "cross-encoder/ms-marco-MiniLM-L-4-v2".to_string();
    let revision = "refs/pr/1".to_string();

    let repo = Repo::with_revision(model_id.clone(), RepoType::Model, revision);
    let (config_filename, weights_filename) = {
        let api = Api::new().expect("Failed to create API");
        let api = api.repo(repo);
        let config = api.get("config.json").expect("Failed to get config.json");
        let weights = api
            .get("model.safetensors")
            .expect("Failed to get model.safetensors");
        (config, weights)
    };
    let config = std::fs::read_to_string(config_filename).unwrap();
    let config: BertModelConfig =
        serde_json::from_str(&config).expect("Failed to parse config.json");

    let mut tokenizer = Tokenizer::from_pretrained("sentence-transformers/all-MiniLM-L6-v2", None)
        .expect("Error while load tokenizer");

    let vb: candle_nn::var_builder::VarBuilderArgs<'_, Box<dyn SimpleBackend>> = unsafe {
        VarBuilder::from_mmaped_safetensors(
            &[weights_filename],
            BertModelDTYPE,
            &candle_core::Device::Cpu,
        )
        .expect("Failed to load model")
    };

    let model = BertModel::load(vb, &config).expect("Failed to load model");
    let tokenizer = tokenizer
        .with_padding(Some(PaddingParams {
            strategy: PaddingStrategy::BatchLongest,
            ..Default::default()
        }))
        .with_truncation(Some(TruncationParams {
            max_length: 512,
            ..Default::default()
        }))
        .expect("Failed to set padding and truncation")
        .clone()
        .into();

    CrossEncoder { tokenizer, model }
}

pub async fn create_embedding(
    message: &str,
    app_mutex: web::Data<AppMutexStore>,
    dataset_config: Option<DatasetConfiguration>,
) -> Result<Vec<f32>, actix_web::Error> {
    //TODO: make custom embedding pull from dataset config instead of env var and make openai embedding pull from dataset config instead of env var
    match &app_mutex.into_inner().embedding_semaphore {
        Some(semaphore) => {
            let lease = semaphore.acquire().await.unwrap();
            if let Some(dataset_config) = dataset_config {
                if dataset_config.USE_CUSTOM_EMBED.unwrap_or(false) {
                    let result =
                        create_openai_embedding(message, dataset_config.OPENAI_BASE_URL).await;
                    drop(lease);
                    return result;
                }
            }

            let result = create_openai_embedding(message, None).await;
            drop(lease);
            result
        }
        _ => {
            if let Some(dataset_config) = dataset_config {
                if dataset_config.USE_CUSTOM_EMBED.unwrap_or(false) {
                    let result =
                        create_openai_embedding(message, dataset_config.OPENAI_BASE_URL).await;
                    return result;
                }
            }

            create_openai_embedding(message, None).await
        }
    }
}

pub async fn create_openai_embedding(
    message: &str,
    base_url: Option<String>,
) -> Result<Vec<f32>, actix_web::Error> {
    let open_ai_api_key = get_env!("OPENAI_API_KEY", "OPENAI_API_KEY should be set").into();
    let client = Client {
        http_client: reqwest::Client::new(),
        api_key: open_ai_api_key,
        base_url: base_url.unwrap_or("https://api.openai.com/v1".to_string()),
    };

    // Vectorize
    let parameters = EmbeddingParameters {
        model: "text-embedding-ada-002".to_string(),
        input: message.to_string(),
        user: None,
        encoding_format: None,
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
    let mut embedding_server_call: String = std::env::var("EMBEDDING_SERVER_ORIGIN")
        .expect("EMBEDDING_SERVER_ORIGIN should be set if this is called");
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
    let mut embedding_server_call: String = std::env::var("EMBEDDING_SERVER_ORIGIN")
        .expect("EMBEDDING_SERVER_ORIGIN should be set if this is called");
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
