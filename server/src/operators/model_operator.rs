use candle_nn::{var_builder::SimpleBackend, VarBuilder};
use candle_transformers::models::bert::{
    BertModel, Config as BertModelConfig, DTYPE as BertModelDTYPE,
};
use hf_hub::{api::sync::Api, Repo, RepoType};
use openai_dive::v1::{api::Client, resources::embedding::EmbeddingParameters};

use serde::{Deserialize, Serialize};
use tokenizers::{tokenizer::Tokenizer, PaddingParams, PaddingStrategy, TruncationParams}; // a fast, portable hash library

use crate::{
    data::models::ServerDatasetConfiguration,
    errors::ServiceError,
    get_env,
};

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
    dataset_config: ServerDatasetConfiguration,
) -> Result<Vec<f32>, actix_web::Error> {
    let open_ai_api_key = get_env!("OPENAI_API_KEY", "OPENAI_API_KEY should be set").into();
    let base_url = dataset_config
        .EMBEDDING_BASE_URL
        .unwrap_or("https://api.openai.com/v1".to_string());
    let client = Client {
        http_client: reqwest::Client::new(),
        api_key: open_ai_api_key,
        base_url,
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
    let mut embedding_server_call: String = get_env!("SPLADE_EMBEDDING_SERVER_ORIGIN","SPLADE_EMBEDDING_SERVER_ORIGIN should be set if this is called").to_string();
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
    let mut embedding_server_call: String = get_env!("SPLADE_EMBEDDING_SERVER_ORIGIN", "EMBEDDING_SERVER_ORIGIN should be set if this is called").to_string();
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
