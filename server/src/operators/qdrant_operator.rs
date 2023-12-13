use super::search_operator::SearchResult;
use crate::{
    data::models::CardMetadata,
    errors::{DefaultError, ServiceError},
    get_env, AppMutexStore,
};
use actix_web::web;
use itertools::Itertools;
use openai_dive::v1::{api::Client, resources::embedding::EmbeddingParameters};
use qdrant_client::{
    client::{QdrantClient, QdrantClientConfig},
    qdrant::{
        payload_index_params::IndexParams, point_id::PointIdOptions,
        with_payload_selector::SelectorOptions, Condition, CreateCollection, Distance, FieldType,
        Filter, HnswConfigDiff, PayloadIndexParams, PointId, PointStruct, RecommendPoints,
        SearchPoints, TextIndexParams, TokenizerType, VectorParams, VectorsConfig,
        WithPayloadSelector,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;

pub async fn get_qdrant_connection() -> Result<QdrantClient, DefaultError> {
    let qdrant_url = get_env!("QDRANT_URL", "QDRANT_URL should be set");
    let qdrant_api_key = get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set").into();
    let mut config = QdrantClientConfig::from_url(qdrant_url);
    config.api_key = Some(qdrant_api_key);
    QdrantClient::new(Some(config)).map_err(|_err| DefaultError {
        message: "Failed to connect to Qdrant",
    })
}

/// Create Qdrant collection and indexes needed
pub async fn create_new_qdrant_collection_query() -> Result<(), ServiceError> {
    let qdrant_collection = get_env!(
        "QDRANT_COLLECTION",
        "QDRANT_COLLECTION should be set if this is called"
    )
    .to_string();

    let qdrant_client = get_qdrant_connection()
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let embedding_size = std::env::var("EMBEDDING_SIZE").unwrap_or("1536".to_owned());
    let embedding_size = embedding_size.parse::<u64>().unwrap_or(1536);

    qdrant_client
        .create_collection(&CreateCollection {
            collection_name: qdrant_collection.clone(),
            vectors_config: Some(VectorsConfig {
                config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                    VectorParams {
                        size: embedding_size,
                        distance: Distance::Cosine.into(),
                        hnsw_config: None,
                        quantization_config: None,
                        on_disk: None,
                    },
                )),
            }),
            hnsw_config: Some(HnswConfigDiff {
                payload_m: Some(16),
                m: Some(0),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
        .map_err(|err| {
            if err.to_string().contains("already exists") {
                return ServiceError::BadRequest("Collection already exists".into());
            }
            ServiceError::BadRequest("Failed to create Collection".into())
        })?;

    qdrant_client
        .create_field_index(
            qdrant_collection.clone(),
            "link",
            FieldType::Text,
            None,
            None,
        )
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

    qdrant_client
        .create_field_index(
            qdrant_collection.clone(),
            "tag_set",
            FieldType::Text,
            None,
            None,
        )
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

    qdrant_client
        .create_field_index(
            qdrant_collection.clone(),
            "dataset_id",
            FieldType::Keyword,
            None,
            None,
        )
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

    qdrant_client
        .create_field_index(
            qdrant_collection.clone(),
            "card_html",
            FieldType::Text,
            Some(&PayloadIndexParams {
                index_params: Some(IndexParams::TextIndexParams(TextIndexParams {
                    tokenizer: TokenizerType::Whitespace as i32,
                    min_token_len: Some(2),
                    max_token_len: Some(10),
                    lowercase: Some(true),
                })),
            }),
            None,
        )
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

    Ok(())
}

pub async fn create_embedding(
    message: &str,
    app_mutex: web::Data<AppMutexStore>,
) -> Result<Vec<f32>, actix_web::Error> {
    let use_custom: u8 = std::env::var("USE_CUSTOM_EMBEDDINGS")
        .unwrap_or("1".to_string())
        .parse::<u8>()
        .unwrap_or(1);

    match &app_mutex.into_inner().embedding_semaphore {
        Some(semaphore) => {
            let lease = semaphore.acquire().await.unwrap();
            if use_custom == 0 {
                let result = create_server_embedding(message).await;
                drop(lease);
                result
            } else {
                let result = create_openai_embedding(message).await;
                drop(lease);
                result
            }
        }
        _ => {
            if use_custom == 0 {
                create_server_embedding(message).await
            } else {
                create_openai_embedding(message).await
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomServerData {
    pub input: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomServerResponse {
    pub embeddings: Vec<f32>,
}

pub async fn create_openai_embedding(message: &str) -> Result<Vec<f32>, actix_web::Error> {
    let open_ai_api_key = get_env!("OPENAI_API_KEY", "OPENAI_API_KEY should be set").into();
    let client = Client::new(open_ai_api_key);

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

pub async fn create_server_embedding(message: &str) -> Result<Vec<f32>, actix_web::Error> {
    let embedding_server_call = std::env::var("EMBEDDING_SERVER_CALL")
        .expect("EMBEDDING_SERVER_CALL should be set if this is called");

    let client = reqwest::Client::new();
    let resp = client
        .post(embedding_server_call)
        .json(&CustomServerData {
            input: message.to_string(),
        })
        .send()
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Failed making call to server {:?}", err)))?
        .json::<CustomServerResponse>()
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

pub async fn create_new_qdrant_point_query(
    point_id: uuid::Uuid,
    embedding_vector: Vec<f32>,
    private: bool,
    card_metadata: CardMetadata,
    author_id: Option<uuid::Uuid>,
    dataset_id: uuid::Uuid,
) -> Result<(), actix_web::Error> {
    let qdrant_collection = get_env!(
        "QDRANT_COLLECTION",
        "QDRANT_COLLECTION should be set if this is called"
    )
    .to_string();

    let qdrant = get_qdrant_connection()
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let payload = json!({"private": private, "authors": vec![author_id.unwrap_or_default().to_string()], "tag_set": card_metadata.tag_set.unwrap_or("".to_string()).split(',').collect_vec(), "link": card_metadata.link.unwrap_or("".to_string()).split(',').collect_vec(), "card_html": card_metadata.card_html.unwrap_or("".to_string()), "metadata": card_metadata.metadata.unwrap_or_default(), "time_stamp": card_metadata.time_stamp.unwrap_or_default().timestamp(), "dataset_id": dataset_id.to_string()})
                .try_into()
                .expect("A json! Value must always be a valid Payload");

    let point = PointStruct::new(point_id.clone().to_string(), embedding_vector, payload);

    qdrant
        .upsert_points_blocking(qdrant_collection, None, vec![point], None)
        .await
        .map_err(|_err| ServiceError::BadRequest("Failed inserting card to qdrant".into()))?;

    Ok(())
}

pub async fn update_qdrant_point_query(
    metadata: Option<CardMetadata>,
    private: bool,
    point_id: uuid::Uuid,
    author_id: Option<uuid::Uuid>,
    updated_vector: Option<Vec<f32>>,
    dataset_id: uuid::Uuid,
) -> Result<(), actix_web::Error> {
    if private && author_id.is_none() {
        return Err(ServiceError::BadRequest("Private card must have an author".into()).into());
    }

    let qdrant_point_id: Vec<PointId> = vec![point_id.to_string().into()];

    let qdrant = get_qdrant_connection()
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let qdrant_collection = get_env!(
        "QDRANT_COLLECTION",
        "QDRANT_COLLECTION should be set if this is called"
    )
    .to_string();

    let current_point_vec = qdrant
        .get_points(
            qdrant_collection.clone(),
            None,
            &qdrant_point_id,
            false.into(),
            true.into(),
            None,
        )
        .await
        .map_err(|_err| ServiceError::BadRequest("Failed to search_points from qdrant".into()))?
        .result;

    let current_point = match current_point_vec.first() {
        Some(point) => point,
        None => {
            return Err(ServiceError::BadRequest(
                "Failed getting vec.first card from qdrant".into(),
            )
            .into())
        }
    };

    let current_private = match current_point.payload.get("private") {
        Some(private) => private.as_bool().unwrap_or(false),
        None => false,
    };

    if !current_private {
        return Ok(());
    }

    let mut current_author_ids = match current_point.payload.get("authors") {
        Some(authors) => match authors.as_list() {
            Some(authors) => authors
                .iter()
                .map(|author| match author.as_str() {
                    Some(author) => author.to_string(),
                    None => "".to_string(),
                })
                .filter(|author| !author.is_empty())
                .collect::<Vec<String>>(),
            None => {
                vec![]
            }
        },
        None => {
            vec![]
        }
    };

    if !current_author_ids.contains(&author_id.unwrap().to_string()) {
        current_author_ids.push(author_id.unwrap().to_string());
    }

    let payload = if let Some(metadata) = metadata {
        json!({"private": private, "authors": current_author_ids, "tag_set": metadata.tag_set.unwrap_or("".to_string()).split(',').collect_vec(), "link": metadata.link.unwrap_or("".to_string()).split(',').collect_vec(), "card_html": metadata.card_html.unwrap_or("".to_string()), "metadata": metadata.metadata.unwrap_or_default(), "time_stamp": metadata.time_stamp.unwrap_or_default().timestamp(), "dataset_id": dataset_id.to_string()})
    } else {
        json!({"private": private, "authors": current_author_ids, "tag_set": current_point.payload.get("tag_set").unwrap_or(&qdrant_client::qdrant::Value::from("")), "link": current_point.payload.get("link").unwrap_or(&qdrant_client::qdrant::Value::from("")), "card_html": current_point.payload.get("card_html").unwrap_or(&qdrant_client::qdrant::Value::from("")), "metadata": current_point.payload.get("metadata").unwrap_or(&qdrant_client::qdrant::Value::from("")), "time_stamp": current_point.payload.get("time_stamp").unwrap_or(&qdrant_client::qdrant::Value::from("")), "dataset_id": current_point.payload.get("dataset_id").unwrap_or(&qdrant_client::qdrant::Value::from(""))})
    };
    let points_selector = qdrant_point_id.into();

    if let Some(embedding_vector) = updated_vector {
        let point = PointStruct::new(
            point_id.clone().to_string(),
            embedding_vector,
            payload
                .try_into()
                .expect("A json! value must always be a valid Payload"),
        );

        qdrant
            .upsert_points(qdrant_collection, None, vec![point], None)
            .await
            .map_err(|_err| ServiceError::BadRequest("Failed upserting card in qdrant".into()))?;

        return Ok(());
    }

    qdrant
        .overwrite_payload(
            qdrant_collection,
            None,
            &points_selector,
            payload
                .try_into()
                .expect("A json! value must always be a valid Payload"),
            None,
        )
        .await
        .map_err(|_err| {
            ServiceError::BadRequest("Failed updating card payload in qdrant".into())
        })?;

    Ok(())
}

pub async fn search_qdrant_query(
    page: u64,
    mut filter: Filter,
    embedding_vector: Vec<f32>,
    dataset_id: uuid::Uuid,
) -> Result<Vec<SearchResult>, DefaultError> {
    let qdrant = get_qdrant_connection().await?;

    let qdrant_collection = get_env!(
        "QDRANT_COLLECTION",
        "QDRANT_COLLECTION should be set if this is called"
    )
    .to_string();

    filter
        .must
        .push(Condition::matches("dataset_id", dataset_id.to_string()));

    let data = qdrant
        .search_points(&SearchPoints {
            collection_name: qdrant_collection.to_string(),
            vector: embedding_vector,
            limit: 10,
            offset: Some((page - 1) * 10),
            with_payload: None,
            filter: Some(filter),
            ..Default::default()
        })
        .await
        .map_err(|e| {
            log::error!("Failed to search points on Qdrant {:?}", e);
            DefaultError {
                message: "Failed to search points on Qdrant",
            }
        })?;

    let point_ids: Vec<SearchResult> = data
        .result
        .iter()
        .filter_map(|point| match point.clone().id?.point_id_options? {
            PointIdOptions::Uuid(id) => Some(SearchResult {
                score: point.score,
                point_id: uuid::Uuid::parse_str(&id).ok()?,
            }),
            PointIdOptions::Num(_) => None,
        })
        .collect();

    Ok(point_ids)
}

pub async fn delete_qdrant_point_id_query(
    point_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
) -> Result<(), DefaultError> {
    let qdrant = get_qdrant_connection().await?;

    let qdrant_point_id: Vec<PointId> = vec![point_id.to_string().into()];
    let points_selector = qdrant_point_id.into();
    let qdrant_collection = dataset_id.to_string();

    qdrant
        .delete_points(qdrant_collection, None, &points_selector, None)
        .await
        .map_err(|_err| DefaultError {
            message: "Failed to delete point from qdrant",
        })?;

    Ok(())
}

pub async fn recommend_qdrant_query(
    positive_ids: Vec<uuid::Uuid>,
    dataset_id: uuid::Uuid,
) -> Result<Vec<uuid::Uuid>, DefaultError> {
    let collection_name = dataset_id.to_string();

    let point_ids: Vec<PointId> = positive_ids
        .iter()
        .map(|id| id.to_string().into())
        .collect();
    let dataset_filter = Some(Filter::must([Condition::matches(
        "dataset_id",
        dataset_id.to_string(),
    )]));

    let recommend_points = RecommendPoints {
        collection_name,
        positive: point_ids,
        negative: vec![],
        filter: dataset_filter,
        limit: 10,
        with_payload: Some(WithPayloadSelector {
            selector_options: Some(SelectorOptions::Enable(true)),
        }),
        params: None,
        score_threshold: None,
        offset: None,
        using: None,
        with_vectors: None,
        lookup_from: None,
        read_consistency: None,
        positive_vectors: vec![],
        negative_vectors: vec![],
        strategy: None,
        timeout: None,
        shard_key_selector: None,
    };

    let qdrant_client = get_qdrant_connection().await?;

    let recommended_point_ids = qdrant_client
        .recommend(&recommend_points)
        .await
        .map_err(|err| {
            log::info!("Failed to recommend points from qdrant: {:?}", err);
            DefaultError {
                message: "Failed to recommend points from qdrant. Your are likely providing an invalid point id.",
            }
        })?
        .result
        .into_iter()
        .filter_map(|point| match point.id?.point_id_options? {
            PointIdOptions::Uuid(id) => uuid::Uuid::from_str(&id).ok(),
            PointIdOptions::Num(_) => None,
        })
        .collect::<Vec<uuid::Uuid>>();

    Ok(recommended_point_ids)
}
