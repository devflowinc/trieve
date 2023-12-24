use super::{
    model_operator::{get_splade_doc_embedding, get_splade_query_embedding},
    search_operator::SearchResult,
};
use crate::{
    data::models::ChunkMetadata,
    errors::{DefaultError, ServiceError},
    get_env,
};
use itertools::Itertools;
use qdrant_client::{
    client::{QdrantClient, QdrantClientConfig},
    qdrant::{
        payload_index_params::IndexParams, point_id::PointIdOptions,
        with_payload_selector::SelectorOptions, Condition, CreateCollection, Distance, FieldType,
        Filter, HnswConfigDiff, PayloadIndexParams, PointId, PointStruct, RecommendPoints,
        SearchPoints, SparseIndexConfig, SparseVectorConfig, SparseVectorParams, TextIndexParams,
        TokenizerType, Vector, VectorParams, VectorParamsMap, VectorsConfig, WithPayloadSelector,
    },
};
use serde_json::json;
use std::{collections::HashMap, str::FromStr};

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

    // check if collection exists
    let collection = qdrant_client
        .collection_info(qdrant_collection.clone())
        .await;
    if let Ok(collection) = collection {
        if collection.result.is_some() {
            return Err(ServiceError::BadRequest(
                "Collection already exists".to_string(),
            ));
        }
    }

    let mut sparse_vector_config = HashMap::new();
    sparse_vector_config.insert(
        "sparse_vectors".to_string(),
        SparseVectorParams {
            index: Some(SparseIndexConfig {
                on_disk: Some(false),
                ..Default::default()
            }),
        },
    );

    qdrant_client
        .create_collection(&CreateCollection {
            collection_name: qdrant_collection.clone(),
            vectors_config: Some(VectorsConfig {
                config: Some(qdrant_client::qdrant::vectors_config::Config::ParamsMap(
                    VectorParamsMap {
                        map: HashMap::from([
                            (
                                "384_vectors".to_string(),
                                VectorParams {
                                    size: 384,
                                    distance: Distance::Cosine.into(),
                                    hnsw_config: None,
                                    quantization_config: None,
                                    on_disk: None,
                                },
                            ),
                            (
                                "768_vectors".to_string(),
                                VectorParams {
                                    size: 768,
                                    distance: Distance::Cosine.into(),
                                    hnsw_config: None,
                                    quantization_config: None,
                                    on_disk: None,
                                },
                            ),
                            (
                                "1024_vectors".to_string(),
                                VectorParams {
                                    size: 1024,
                                    distance: Distance::Cosine.into(),
                                    hnsw_config: None,
                                    quantization_config: None,
                                    on_disk: None,
                                },
                            ),
                            (
                                "1536_vectors".to_string(),
                                VectorParams {
                                    size: 1536,
                                    distance: Distance::Cosine.into(),
                                    hnsw_config: None,
                                    quantization_config: None,
                                    on_disk: None,
                                },
                            ),
                        ]),
                    },
                )),
            }),
            hnsw_config: Some(HnswConfigDiff {
                payload_m: Some(16),
                m: Some(0),
                ..Default::default()
            }),
            sparse_vectors_config: Some(SparseVectorConfig {
                map: sparse_vector_config,
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
            "chunk_html",
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

pub async fn create_new_qdrant_point_query(
    point_id: uuid::Uuid,
    embedding_vector: Vec<f32>,
    chunk_metadata: ChunkMetadata,
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

    let splade_vector = get_splade_doc_embedding(
        chunk_metadata
            .chunk_html
            .as_ref()
            .unwrap_or(&"".to_string()),
    )
    .await?;

    let payload = json!({"authors": vec![author_id.unwrap_or_default().to_string()], "tag_set": chunk_metadata.tag_set.unwrap_or("".to_string()).split(',').collect_vec(), "link": chunk_metadata.link.unwrap_or("".to_string()).split(',').collect_vec(), "chunk_html": chunk_metadata.chunk_html.unwrap_or("".to_string()), "metadata": chunk_metadata.metadata.unwrap_or_default(), "time_stamp": chunk_metadata.time_stamp.unwrap_or_default().timestamp(), "dataset_id": dataset_id.to_string()})
                .try_into()
                .expect("A json! Value must always be a valid Payload");

    let vector_name = match embedding_vector.len() {
        384 => "384_vectors",
        768 => "768_vectors",
        1024 => "1024_vectors",
        1536 => "1536_vectors",
        _ => return Err(ServiceError::BadRequest("Invalid embedding vector size".into()).into()),
    };

    let point = PointStruct::new(
        point_id.clone().to_string(),
        HashMap::from([
            (vector_name.to_string(), Vector::from(embedding_vector)),
            ("sparse_vectors".to_string(), Vector::from(splade_vector)),
        ]),
        payload,
    );

    qdrant
        .upsert_points_blocking(qdrant_collection, None, vec![point], None)
        .await
        .map_err(|err| {
            log::info!("Failed inserting chunk to qdrant {:?}", err);
            ServiceError::BadRequest("Failed inserting chunk to qdrant".into())
        })?;

    Ok(())
}

pub async fn update_qdrant_point_query(
    metadata: Option<ChunkMetadata>,
    point_id: uuid::Uuid,
    author_id: Option<uuid::Uuid>,
    updated_vector: Option<Vec<f32>>,
    dataset_id: uuid::Uuid,
) -> Result<(), actix_web::Error> {
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
                "Failed getting vec.first chunk from qdrant".into(),
            )
            .into())
        }
    };

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

    let payload = if let Some(metadata) = metadata.clone() {
        json!({"authors": current_author_ids, "tag_set": metadata.tag_set.unwrap_or("".to_string()).split(',').collect_vec(), "link": metadata.link.unwrap_or("".to_string()).split(',').collect_vec(), "chunk_html": metadata.chunk_html.unwrap_or("".to_string()), "metadata": metadata.metadata.unwrap_or_default(), "time_stamp": metadata.time_stamp.unwrap_or_default().timestamp(), "dataset_id": dataset_id.to_string()})
    } else {
        json!({"authors": current_author_ids, "tag_set": current_point.payload.get("tag_set").unwrap_or(&qdrant_client::qdrant::Value::from("")), "link": current_point.payload.get("link").unwrap_or(&qdrant_client::qdrant::Value::from("")), "chunk_html": current_point.payload.get("chunk_html").unwrap_or(&qdrant_client::qdrant::Value::from("")), "metadata": current_point.payload.get("metadata").unwrap_or(&qdrant_client::qdrant::Value::from("")), "time_stamp": current_point.payload.get("time_stamp").unwrap_or(&qdrant_client::qdrant::Value::from("")), "dataset_id": current_point.payload.get("dataset_id").unwrap_or(&qdrant_client::qdrant::Value::from(""))})
    };
    let points_selector = qdrant_point_id.into();

    if let Some(updated_vector) = updated_vector {
        let splade_vector = get_splade_doc_embedding(&metadata.unwrap().content).await?;
        let vector_name = match updated_vector.len() {
            384 => "384_vectors",
            768 => "768_vectors",
            1024 => "1024_vectors",
            1536 => "1536_vectors",
            _ => {
                return Err(ServiceError::BadRequest("Invalid embedding vector size".into()).into())
            }
        };
        let point = PointStruct::new(
            point_id.clone().to_string(),
            HashMap::from([
                (vector_name.to_string(), Vector::from(updated_vector)),
                ("sparse_vectors".to_string(), Vector::from(splade_vector)),
            ]),
            payload
                .try_into()
                .expect("A json! value must always be a valid Payload"),
        );

        qdrant
            .upsert_points(qdrant_collection, None, vec![point], None)
            .await
            .map_err(|_err| ServiceError::BadRequest("Failed upserting chunk in qdrant".into()))?;

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
            ServiceError::BadRequest("Failed updating chunk payload in qdrant".into())
        })?;

    Ok(())
}

pub async fn search_semantic_qdrant_query(
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

    let vector_name = match embedding_vector.len() {
        384 => "384_vectors",
        768 => "768_vectors",
        1024 => "1024_vectors",
        1536 => "1536_vectors",
        _ => {
            return Err(DefaultError {
                message: "Invalid embedding vector size",
            })
        }
    };

    let data = qdrant
        .search_points(&SearchPoints {
            collection_name: qdrant_collection.to_string(),
            vector: embedding_vector,
            vector_name: Some(vector_name.to_string()),
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

pub async fn search_full_text_qdrant_query(
    page: u64,
    mut filter: Filter,
    query: String,
    dataset_id: uuid::Uuid,
) -> Result<Vec<SearchResult>, DefaultError> {
    let qdrant = get_qdrant_connection().await?;

    let qdrant_collection = get_env!(
        "QDRANT_COLLECTION",
        "QDRANT_COLLECTION should be set if this is called"
    )
    .to_string();

    let embedding_vector =
        get_splade_query_embedding(&query)
            .await
            .map_err(|_err| DefaultError {
                message: "Failed to get splade query embedding",
            })?;

    filter
        .must
        .push(Condition::matches("dataset_id", dataset_id.to_string()));

    let sparse_vector: Vector = embedding_vector.into();

    let data = qdrant
        .search_points(&SearchPoints {
            collection_name: qdrant_collection.to_string(),
            vector: sparse_vector.data,
            sparse_indices: sparse_vector.indices,
            vector_name: Some("sparse_vectors".to_string()),
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
    embed_size: usize,
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

    let vector_name = match embed_size {
        384 => "384_vectors",
        768 => "768_vectors",
        1024 => "1024_vectors",
        1536 => "1536_vectors",
        _ => {
            return Err(DefaultError {
                message: "Invalid embedding vector size",
            })
        }
    };

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
        using: Some(vector_name.to_string()),
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
