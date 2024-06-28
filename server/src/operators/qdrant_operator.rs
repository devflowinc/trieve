use super::{
    group_operator::get_groups_from_group_ids_query,
    search_operator::{assemble_qdrant_filter, SearchResult},
};
use crate::{
    data::models::{ChunkMetadata, Pool, QdrantPayload, ServerDatasetConfiguration},
    errors::ServiceError,
    get_env,
    handlers::chunk_handler::ChunkFilter,
};
use actix_web::web;
use itertools::Itertools;
use qdrant_client::{
    client::{QdrantClient, QdrantClientConfig},
    qdrant::{
        group_id::Kind, payload_index_params::IndexParams, point_id::PointIdOptions,
        quantization_config::Quantization, BinaryQuantization, CountPoints, CreateCollection,
        Distance, FieldType, Filter, HnswConfigDiff, PayloadIndexParams, PointId, PointStruct,
        QuantizationConfig, RecommendPointGroups, RecommendPoints, RecommendStrategy,
        SearchBatchPoints, SearchPointGroups, SearchPoints, SparseIndexConfig, SparseVectorConfig,
        SparseVectorParams, TextIndexParams, TokenizerType, Value, Vector, VectorParams,
        VectorParamsMap, VectorsConfig,
    },
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

#[tracing::instrument(skip(qdrant_url, qdrant_api_key))]
pub async fn get_qdrant_connection(
    qdrant_url: Option<&str>,
    qdrant_api_key: Option<&str>,
) -> Result<QdrantClient, ServiceError> {
    let qdrant_url = qdrant_url.unwrap_or(get_env!(
        "QDRANT_URL",
        "QDRANT_URL should be set if this is called"
    ));
    let qdrant_api_key = qdrant_api_key.unwrap_or(get_env!(
        "QDRANT_API_KEY",
        "QDRANT_API_KEY should be set if this is called"
    ));
    let mut config = QdrantClientConfig::from_url(qdrant_url);
    config.api_key = Some(qdrant_api_key.to_owned());
    config.timeout = std::time::Duration::from_secs(60);
    QdrantClient::new(Some(config))
        .map_err(|_err| ServiceError::BadRequest("Failed to connect to Qdrant".to_string()))
}

/// Create Qdrant collection and indexes needed
#[tracing::instrument(skip(qdrant_url, qdrant_api_key))]
pub async fn create_new_qdrant_collection_query(
    qdrant_url: Option<&str>,
    qdrant_api_key: Option<&str>,
    quantize: bool,
    recreate_indexes: bool,
    replication_factor: u32,
    accepted_vectors: Vec<u64>,
) -> Result<(), ServiceError> {
    let qdrant_client = get_qdrant_connection(qdrant_url, qdrant_api_key).await?;
    for vector in accepted_vectors.iter() {
        let qdrant_collection = format!("{}_vectors", vector);
        // check if collection exists
        let collection = qdrant_client
            .collection_exists(qdrant_collection.clone())
            .await
            .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

        match collection {
            true => log::info!("Avoided creating collection as it already exists"),
            false => {
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

                let quantization_config = if quantize {
                    //TODO: make this scalar
                    Some(QuantizationConfig {
                        quantization: Some(Quantization::Binary(BinaryQuantization {
                            always_ram: Some(true),
                        })),
                    })
                } else {
                    None
                };

                let on_disk = if quantize {
                    //TODO: make this scalar
                    Some(true)
                } else {
                    None
                };

                let vectors_hash_map = HashMap::from_iter(
                    vec![(
                        qdrant_collection.to_string(),
                        VectorParams {
                            size: *vector,
                            distance: Distance::Cosine.into(),
                            quantization_config: quantization_config.clone(),
                            on_disk,
                            ..Default::default()
                        },
                    )]
                    .into_iter(),
                );

                qdrant_client
                    .create_collection(&CreateCollection {
                        collection_name: qdrant_collection.clone(),
                        vectors_config: Some(VectorsConfig {
                            config: Some(qdrant_client::qdrant::vectors_config::Config::ParamsMap(
                                VectorParamsMap {
                                    map: vectors_hash_map,
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
                        write_consistency_factor: Some(1),
                        replication_factor: Some(replication_factor),
                        ..Default::default()
                    })
                    .await
                    .map_err(|err| {
                        if err.to_string().contains("already exists") {
                            return ServiceError::BadRequest("Collection already exists".into());
                        }
                        ServiceError::BadRequest(err.to_string())
                    })?;
            }
        };

        if recreate_indexes {
            qdrant_client
                .delete_field_index(qdrant_collection.clone(), "link", None)
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(qdrant_collection.clone(), "tag_set", None)
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(qdrant_collection.clone(), "dataset_id", None)
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(qdrant_collection.clone(), "metadata", None)
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(qdrant_collection.clone(), "time_stamp", None)
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(qdrant_collection.clone(), "group_ids", None)
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(qdrant_collection.clone(), "location", None)
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(qdrant_collection.clone(), "content", None)
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(qdrant_collection.clone(), "num_value", None)
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;
        }

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
                FieldType::Keyword,
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
                "metadata",
                FieldType::Keyword,
                None,
                None,
            )
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(
                qdrant_collection.clone(),
                "time_stamp",
                FieldType::Integer,
                None,
                None,
            )
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(
                qdrant_collection.clone(),
                "group_ids",
                FieldType::Keyword,
                None,
                None,
            )
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(
                qdrant_collection.clone(),
                "location",
                FieldType::Geo,
                None,
                None,
            )
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(
                qdrant_collection.clone(),
                "content",
                FieldType::Text,
                Some(&PayloadIndexParams {
                    index_params: Some(IndexParams::TextIndexParams(TextIndexParams {
                        tokenizer: TokenizerType::Prefix as i32,
                        min_token_len: Some(2),
                        max_token_len: Some(10),
                        lowercase: Some(true),
                    })),
                }),
                None,
            )
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(
                qdrant_collection.clone(),
                "num_value",
                FieldType::Float,
                None,
                None,
            )
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(
                qdrant_collection.clone(),
                "group_tag_set",
                FieldType::Keyword,
                None,
                None,
            )
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;
    }

    Ok(())
}

#[tracing::instrument(skip(points))]
pub async fn bulk_upsert_qdrant_points_query(
    points: Vec<PointStruct>,
    config: ServerDatasetConfiguration,
) -> Result<(), ServiceError> {
    if points.is_empty() {
        return Err(ServiceError::BadRequest(
            "No points were created for QDRANT, this is due to the incorrect embedding vector size"
                .into(),
        ));
    }

    let qdrant_collection = format!("{}_vectors", config.EMBEDDING_SIZE);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    qdrant_client
        .upsert_points_blocking(qdrant_collection, None, points, None)
        .await
        .map_err(|err| {
            sentry::capture_message(&format!("Error {:?}", err), sentry::Level::Error);
            log::error!("Failed inserting chunk to qdrant {:?}", err);
            ServiceError::BadRequest(format!("Failed inserting chunk to qdrant {:?}", err))
        })?;

    Ok(())
}

#[tracing::instrument(skip(embedding_vector, pool))]
pub async fn create_new_qdrant_point_query(
    point_id: uuid::Uuid,
    embedding_vector: Vec<f32>,
    chunk_metadata: ChunkMetadata,
    splade_vector: Vec<(u32, f32)>,
    group_ids: Option<Vec<uuid::Uuid>>,
    config: ServerDatasetConfiguration,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    let chunk_tags: Option<Vec<Option<String>>> = if let Some(ref group_ids) = group_ids {
        Some(
            get_groups_from_group_ids_query(group_ids.clone(), pool.clone())
                .await?
                .iter()
                .filter_map(|group| group.tag_set.clone())
                .flatten()
                .dedup()
                .collect(),
        )
    } else {
        None
    };

    let payload = QdrantPayload::new(chunk_metadata, group_ids, None, chunk_tags).into();
    let qdrant_collection = format!("{}_vectors", config.EMBEDDING_SIZE);

    let vector_name = match embedding_vector.len() {
        384 => "384_vectors",
        512 => "512_vectors",
        768 => "768_vectors",
        1024 => "1024_vectors",
        3072 => "3072_vectors",
        1536 => "1536_vectors",
        _ => {
            return Err(ServiceError::BadRequest(
                "Invalid embedding vector size".into(),
            ))
        }
    };

    let vector_payload = HashMap::from([
        (vector_name.to_string(), Vector::from(embedding_vector)),
        ("sparse_vectors".to_string(), Vector::from(splade_vector)),
    ]);

    let point = PointStruct::new(point_id.clone().to_string(), vector_payload, payload);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    qdrant_client
        .upsert_points_blocking(qdrant_collection, None, vec![point], None)
        .await
        .map_err(|err| {
            sentry::capture_message(&format!("Error {:?}", err), sentry::Level::Error);
            log::error!("Failed inserting chunk to qdrant {:?}", err);
            ServiceError::BadRequest(format!("Failed inserting chunk to qdrant {:?}", err))
        })?;

    Ok(())
}

#[tracing::instrument(skip(updated_vector, web_pool))]
pub async fn update_qdrant_point_query(
    metadata: Option<ChunkMetadata>,
    point_id: uuid::Uuid,
    updated_vector: Option<Vec<f32>>,
    group_ids: Option<Vec<uuid::Uuid>>,
    dataset_id: uuid::Uuid,
    splade_vector: Vec<(u32, f32)>,
    config: ServerDatasetConfiguration,
    web_pool: web::Data<Pool>,
) -> Result<(), actix_web::Error> {
    let qdrant_point_id: Vec<PointId> = vec![point_id.to_string().into()];

    let qdrant_collection = format!("{}_vectors", config.EMBEDDING_SIZE);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let current_point_vec = qdrant_client
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

    let current_point = current_point_vec.first();

    let payload = if let Some(metadata) = metadata.clone() {
        let group_ids = if let Some(group_ids) = group_ids.clone() {
            group_ids
        } else if let Some(current_point) = current_point {
            current_point
                .payload
                .get("group_ids")
                .unwrap_or(&Value::from(vec![] as Vec<String>))
                .to_owned()
                .iter_list()
                .unwrap()
                .map(|id| {
                    id.to_string()
                        .parse::<uuid::Uuid>()
                        .expect("group_id must be a valid uuid")
                })
                .collect::<Vec<uuid::Uuid>>()
        } else {
            vec![]
        };

        let chunk_tags: Vec<Option<String>> =
            get_groups_from_group_ids_query(group_ids.clone(), web_pool.clone())
                .await?
                .iter()
                .filter_map(|group| group.tag_set.clone())
                .flatten()
                .collect();

        QdrantPayload::new(
            metadata,
            group_ids.into(),
            Some(dataset_id),
            Some(chunk_tags),
        )
    } else if let Some(current_point) = current_point {
        QdrantPayload::from(current_point.clone())
    } else {
        return Err(ServiceError::BadRequest("No metadata points found".into()).into());
    };

    let points_selector = qdrant_point_id.into();

    if let Some(updated_vector) = updated_vector {
        let vector_name = match updated_vector.len() {
            384 => "384_vectors",
            512 => "512_vectors",
            768 => "768_vectors",
            1024 => "1024_vectors",
            3072 => "3072_vectors",
            1536 => "1536_vectors",
            _ => {
                return Err(ServiceError::BadRequest("Invalid embedding vector size".into()).into())
            }
        };
        let vector_payload = HashMap::from([
            (vector_name.to_string(), Vector::from(updated_vector)),
            ("sparse_vectors".to_string(), Vector::from(splade_vector)),
        ]);

        let point = PointStruct::new(point_id.clone().to_string(), vector_payload, payload.into());

        qdrant_client
            .upsert_points(qdrant_collection, None, vec![point], None)
            .await
            .map_err(|_err| ServiceError::BadRequest("Failed upserting chunk in qdrant".into()))?;

        return Ok(());
    }

    qdrant_client
        .overwrite_payload(
            qdrant_collection,
            None,
            &points_selector,
            payload.into(),
            None,
            None,
        )
        .await
        .map_err(|_err| {
            ServiceError::BadRequest("Failed updating chunk payload in qdrant".into())
        })?;

    Ok(())
}

#[tracing::instrument]
pub async fn add_bookmark_to_qdrant_query(
    point_id: uuid::Uuid,
    group_id: uuid::Uuid,
    config: ServerDatasetConfiguration,
) -> Result<(), ServiceError> {
    let qdrant_collection = format!("{}_vectors", config.EMBEDDING_SIZE);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let qdrant_point_id: Vec<PointId> = vec![point_id.to_string().into()];

    let current_point_vec = qdrant_client
        .get_points(
            qdrant_collection.clone(),
            None,
            &qdrant_point_id,
            false.into(),
            true.into(),
            None,
        )
        .await
        .map_err(|_err| {
            ServiceError::BadRequest("Failed to search_points from qdrant".to_string())
        })?
        .result;

    let current_point = match current_point_vec.first() {
        Some(point) => point,
        None => {
            return Err(ServiceError::BadRequest(
                "Failed getting vec.first chunk from qdrant".to_string(),
            ))
        }
    };

    let group_ids = if current_point.payload.contains_key("group_ids") {
        let mut group_ids_qdrant = current_point
            .payload
            .get("group_ids")
            .unwrap_or(&Value::from(vec![] as Vec<&str>))
            .iter_list()
            .unwrap_or(Value::from(vec![] as Vec<&str>).iter_list().unwrap())
            .map(|id| {
                id.as_str()
                    .unwrap_or(&"".to_owned())
                    .parse::<uuid::Uuid>()
                    .unwrap_or_default()
            })
            .collect::<Vec<uuid::Uuid>>();
        group_ids_qdrant.append(&mut vec![group_id]);
        group_ids_qdrant
    } else {
        vec![group_id]
    };

    let payload = QdrantPayload::new_from_point(current_point.clone(), Some(group_ids));

    let points_selector = qdrant_point_id.into();

    qdrant_client
        .overwrite_payload(
            qdrant_collection,
            None,
            &points_selector,
            payload.into(),
            None,
            None,
        )
        .await
        .map_err(|_err| {
            ServiceError::BadRequest("Failed updating chunk payload in qdrant".to_string())
        })?;

    Ok(())
}

#[tracing::instrument]
pub async fn remove_bookmark_from_qdrant_query(
    point_id: uuid::Uuid,
    group_id: uuid::Uuid,
    config: ServerDatasetConfiguration,
) -> Result<(), ServiceError> {
    let qdrant_collection = format!("{}_vectors", config.EMBEDDING_SIZE);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let qdrant_point_id: Vec<PointId> = vec![point_id.to_string().into()];

    let current_point_vec = qdrant_client
        .get_points(
            qdrant_collection.clone(),
            None,
            &qdrant_point_id,
            false.into(),
            true.into(),
            None,
        )
        .await
        .map_err(|_err| {
            ServiceError::BadRequest("Failed to search_points from qdrant".to_string())
        })?
        .result;

    let current_point = match current_point_vec.first() {
        Some(point) => point,
        None => {
            return Err(ServiceError::BadRequest(
                "Failed getting vec.first chunk from qdrant".to_string(),
            ))
        }
    };

    let group_ids = if current_point.payload.contains_key("group_ids") {
        let mut group_ids_qdrant = current_point
            .payload
            .get("group_ids")
            .unwrap_or(&Value::from(vec![] as Vec<&str>))
            .iter_list()
            .unwrap()
            .map(|id| {
                id.as_str()
                    .unwrap_or(&"".to_owned())
                    .parse::<uuid::Uuid>()
                    .unwrap_or_default()
            })
            .collect::<Vec<uuid::Uuid>>();
        group_ids_qdrant.retain(|id| id != &group_id);
        group_ids_qdrant
    } else {
        vec![]
    };

    let payload = QdrantPayload::new_from_point(current_point.clone(), Some(group_ids));

    let points_selector = qdrant_point_id.into();

    qdrant_client
        .overwrite_payload(
            qdrant_collection,
            None,
            &points_selector,
            payload.into(),
            None,
            None,
        )
        .await
        .map_err(|_err| {
            ServiceError::BadRequest("Failed updating chunk payload in qdrant".to_string())
        })?;

    Ok(())
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GroupSearchResults {
    pub group_id: uuid::Uuid,
    pub hits: Vec<SearchResult>,
}

#[derive(Debug)]
pub enum VectorType {
    Sparse(Vec<(u32, f32)>),
    Dense(Vec<f32>),
}

#[derive(Debug)]
pub struct QdrantSearchQuery {
    pub filter: Filter,
    pub score_threshold: Option<f32>,
    pub vector: VectorType,
}

#[tracing::instrument]
pub async fn search_over_groups_query(
    page: u64,
    filter: Filter,
    limit: u32,
    score_threshold: Option<f32>,
    group_size: u32,
    vector: VectorType,
    config: ServerDatasetConfiguration,
) -> Result<Vec<GroupSearchResults>, ServiceError> {
    let qdrant_collection = format!("{}_vectors", config.EMBEDDING_SIZE);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let vector_name = match vector {
        VectorType::Sparse(_) => "sparse_vectors",
        VectorType::Dense(ref embedding_vector) => match embedding_vector.len() {
            384 => "384_vectors",
            512 => "512_vectors",
            768 => "768_vectors",
            1024 => "1024_vectors",
            3072 => "3072_vectors",
            1536 => "1536_vectors",
            _ => {
                return Err(ServiceError::BadRequest(
                    "Invalid embedding vector size".to_string(),
                ))
            }
        },
    };

    let qdrant_search_results = match vector {
        VectorType::Dense(embedding_vector) => {
            qdrant_client
                .search_groups(&SearchPointGroups {
                    collection_name: qdrant_collection.to_string(),
                    vector: embedding_vector,
                    vector_name: Some(vector_name.to_string()),
                    limit: (limit * page as u32),
                    score_threshold,
                    with_payload: None,
                    filter: Some(filter),
                    group_by: "group_ids".to_string(),
                    group_size: if group_size == 0 { 1 } else { group_size },
                    timeout: Some(60),
                    ..Default::default()
                })
                .await
        }

        VectorType::Sparse(sparse_vector) => {
            let sparse_vector: Vector = sparse_vector.into();
            qdrant_client
                .search_groups(&SearchPointGroups {
                    collection_name: qdrant_collection.to_string(),
                    vector: sparse_vector.data,
                    sparse_indices: sparse_vector.indices,
                    vector_name: Some(vector_name.to_string()),
                    limit: (limit * page as u32),
                    score_threshold,
                    with_payload: None,
                    filter: Some(filter),
                    group_by: "group_ids".to_string(),
                    group_size: if group_size == 0 { 1 } else { group_size },
                    timeout: Some(60),
                    ..Default::default()
                })
                .await
        }
    }
    .map_err(|e| {
        log::error!("Failed to search points on Qdrant {:?}", e);
        ServiceError::BadRequest("Failed to search points on Qdrant".to_string())
    })?;

    let point_ids: Vec<GroupSearchResults> = qdrant_search_results
        .result
        .unwrap()
        .groups
        .iter()
        .filter_map(|point| {
            let group_id = match &point.id.clone()?.kind? {
                Kind::StringValue(id) => uuid::Uuid::from_str(id).unwrap_or_default(),
                _ => {
                    return None;
                }
            };

            let hits: Vec<SearchResult> = point
                .hits
                .iter()
                .filter_map(|hit| match hit.id.clone()?.point_id_options? {
                    PointIdOptions::Uuid(id) => Some(SearchResult {
                        score: hit.score,
                        point_id: uuid::Uuid::parse_str(&id).ok()?,
                    }),
                    PointIdOptions::Num(_) => None,
                })
                .collect();

            if group_size == 0 {
                Some(GroupSearchResults {
                    group_id,
                    hits: vec![],
                })
            } else {
                Some(GroupSearchResults { group_id, hits })
            }
        })
        .skip((page - 1) as usize * limit as usize)
        .collect();

    Ok(point_ids)
}

#[tracing::instrument]
pub async fn search_qdrant_query(
    page: u64,
    limit: u64,
    queries: Vec<QdrantSearchQuery>,
    config: ServerDatasetConfiguration,
    get_total_pages: bool,
) -> Result<(Vec<SearchResult>, u64, Vec<usize>), ServiceError> {
    let qdrant_collection = format!("{}_vectors", config.EMBEDDING_SIZE);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let data: Vec<SearchPoints> = queries
        .into_iter()
        .map(|query| match query.vector {
            VectorType::Sparse(vector) => {
                let sparse_vector: Vector = vector.into();
                Ok(SearchPoints {
                    collection_name: qdrant_collection.to_string(),
                    vector: sparse_vector.data,
                    sparse_indices: sparse_vector.indices,
                    vector_name: Some("sparse_vectors".to_string()),
                    limit,
                    score_threshold: query.score_threshold,
                    offset: Some((page - 1) * limit),
                    with_payload: None,
                    filter: Some(query.filter.clone()),
                    timeout: Some(60),
                    params: None,
                    ..Default::default()
                })
            }
            VectorType::Dense(embedding_vector) => {
                let vector_name = match embedding_vector.len() {
                    384 => "384_vectors",
                    512 => "512_vectors",
                    768 => "768_vectors",
                    1024 => "1024_vectors",
                    3072 => "3072_vectors",
                    1536 => "1536_vectors",
                    _ => {
                        return Err(ServiceError::BadRequest(
                            "Invalid embedding vector size".to_string(),
                        ))
                    }
                };

                Ok(SearchPoints {
                    collection_name: qdrant_collection.to_string(),
                    vector: embedding_vector,
                    vector_name: Some(vector_name.to_string()),
                    limit,
                    score_threshold: query.score_threshold,
                    offset: Some((page - 1) * limit),
                    with_payload: None,
                    filter: Some(query.filter.clone()),
                    timeout: Some(60),
                    params: None,
                    ..Default::default()
                })
            }
        })
        .collect::<Result<Vec<SearchPoints>, ServiceError>>()?;

    let batch_points = SearchBatchPoints {
        collection_name: qdrant_collection.to_string(),
        search_points: data.clone(),
        timeout: Some(60),
        ..Default::default()
    };

    let search_response_future = qdrant_client.search_batch_points(&batch_points);

    let count_query = data
        .iter()
        .map(|query| CountPoints {
            collection_name: qdrant_collection.to_string(),
            filter: query.filter.clone(),
            exact: Some(true),
            read_consistency: None,
            shard_key_selector: None,
        })
        .collect::<Vec<_>>();

    let point_count_futures = count_query
        .iter()
        .map(|query| async {
            if !get_total_pages {
                return Ok(0);
            }

            Ok(qdrant_client
                .count(query)
                .await
                .map_err(|e| {
                    log::error!("Failed to count points on Qdrant {:?}", e);
                    ServiceError::BadRequest("Failed to count points on Qdrant".to_string())
                })?
                .result
                .map(|count| count.count)
                .unwrap_or(0))
        })
        .collect::<Vec<_>>();

    let (search_batch_response, point_count_response) = futures::join!(
        search_response_future,
        futures::future::join_all(point_count_futures)
    );

    let search_batch_response = search_batch_response.map_err(|e| {
        log::error!("Failed to search points on Qdrant {:?}", e);
        ServiceError::BadRequest("Failed to search points on Qdrant".to_string())
    })?;

    let batch_lengths = search_batch_response
        .result
        .iter()
        .map(|batch_result| batch_result.result.len())
        .collect();

    let search_results: Vec<SearchResult> = search_batch_response
        .result
        .iter()
        .flat_map(|batch_result| {
            batch_result
                .result
                .iter()
                .filter_map(
                    |scored_point| match scored_point.id.clone()?.point_id_options? {
                        PointIdOptions::Uuid(id) => Some(SearchResult {
                            score: scored_point.score,
                            point_id: uuid::Uuid::parse_str(&id).ok()?,
                        }),
                        PointIdOptions::Num(_) => None,
                    },
                )
                .collect::<Vec<SearchResult>>()
        })
        .unique_by(|point| point.point_id)
        .collect();

    let point_count = point_count_response
        .into_iter()
        .map(|count: Result<u64, ServiceError>| count.unwrap_or(0))
        .min()
        .unwrap_or(0);

    Ok((search_results, point_count, batch_lengths))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QdrantRecommendResult {
    pub point_id: uuid::Uuid,
    pub score: f32,
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool))]
pub async fn recommend_qdrant_query(
    positive_ids: Vec<uuid::Uuid>,
    negative_ids: Vec<uuid::Uuid>,
    strategy: Option<String>,
    recommend_type: Option<String>,
    filters: Option<ChunkFilter>,
    limit: u64,
    dataset_id: uuid::Uuid,
    config: ServerDatasetConfiguration,
    pool: web::Data<Pool>,
) -> Result<Vec<QdrantRecommendResult>, ServiceError> {
    let qdrant_collection = format!("{}_vectors", config.EMBEDDING_SIZE);

    let recommend_strategy = match strategy {
        Some(strategy) => match strategy.as_str() {
            "best_score" => Some(RecommendStrategy::BestScore.into()),
            _ => None,
        },
        None => None,
    };

    let filter = assemble_qdrant_filter(filters, None, None, dataset_id, pool).await?;

    let positive_point_ids: Vec<PointId> = positive_ids
        .iter()
        .map(|id| id.to_string().into())
        .collect();
    let negative_point_ids: Vec<PointId> = negative_ids
        .iter()
        .map(|id| id.to_string().into())
        .collect();

    let vector_name = if let Some(recommend_type) = recommend_type {
        if recommend_type == "fulltext" {
            "sparse_vectors"
        } else if recommend_type == "semantic" {
            match config.EMBEDDING_SIZE {
                384 => "384_vectors",
                512 => "512_vectors",
                768 => "768_vectors",
                1024 => "1024_vectors",
                3072 => "3072_vectors",
                1536 => "1536_vectors",
                _ => {
                    return Err(ServiceError::BadRequest(
                        "Invalid embedding vector size".to_string(),
                    ))
                }
            }
        } else {
            return Err(ServiceError::BadRequest(
                "Invalid recommend type".to_string(),
            ));
        }
    } else {
        match config.EMBEDDING_SIZE {
            384 => "384_vectors",
            512 => "512_vectors",
            768 => "768_vectors",
            1024 => "1024_vectors",
            3072 => "3072_vectors",
            1536 => "1536_vectors",
            _ => {
                return Err(ServiceError::BadRequest(
                    "Invalid embedding vector size".to_string(),
                ))
            }
        }
    };

    let recommend_points = RecommendPoints {
        collection_name: qdrant_collection,
        positive: positive_point_ids.clone(),
        negative: negative_point_ids.clone(),
        filter: Some(filter),
        limit,
        with_payload: None,
        params: None,
        score_threshold: None,
        offset: None,
        using: Some(vector_name.to_string()),
        with_vectors: None,
        lookup_from: None,
        read_consistency: None,
        positive_vectors: vec![],
        negative_vectors: vec![],
        strategy: recommend_strategy,
        timeout: Some(60),
        shard_key_selector: None,
    };

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let recommended_point_ids = qdrant_client
        .recommend(&recommend_points)
        .await
        .map_err(|err| {
            log::error!("Failed to recommend points from qdrant: {:?}", err);
            ServiceError::BadRequest("Failed to recommend points from qdrant.".to_string())
        })?
        .result
        .into_iter()
        .filter_map(|point| {
            let point_id = match point.id.clone()?.point_id_options? {
                PointIdOptions::Uuid(id) => uuid::Uuid::parse_str(&id).ok()?,
                PointIdOptions::Num(_) => {
                    return None;
                }
            };

            Some(QdrantRecommendResult {
                point_id,
                score: point.score,
            })
        })
        .collect::<Vec<QdrantRecommendResult>>();

    Ok(recommended_point_ids)
}

#[allow(clippy::too_many_arguments)]
pub async fn recommend_qdrant_groups_query(
    positive_ids: Vec<uuid::Uuid>,
    negative_ids: Vec<uuid::Uuid>,
    strategy: Option<String>,
    recommend_type: Option<String>,
    filter: Option<ChunkFilter>,
    limit: u64,
    group_size: u32,
    dataset_id: uuid::Uuid,
    config: ServerDatasetConfiguration,
    pool: web::Data<Pool>,
) -> Result<Vec<GroupSearchResults>, ServiceError> {
    let qdrant_collection = format!("{}_vectors", config.EMBEDDING_SIZE);

    let recommend_strategy = match strategy {
        Some(strategy) => match strategy.as_str() {
            "best_score" => Some(RecommendStrategy::BestScore.into()),
            _ => None,
        },
        None => None,
    };

    let filters = assemble_qdrant_filter(filter, None, None, dataset_id, pool).await?;

    let positive_point_ids: Vec<PointId> = positive_ids
        .iter()
        .map(|id| id.to_string().into())
        .collect();
    let negative_point_ids: Vec<PointId> = negative_ids
        .iter()
        .map(|id| id.to_string().into())
        .collect();

    let vector_name = if let Some(recommend_type) = recommend_type {
        if recommend_type == "fulltext" {
            "sparse_vectors"
        } else if recommend_type == "semantic" {
            match config.EMBEDDING_SIZE {
                384 => "384_vectors",
                512 => "512_vectors",
                768 => "768_vectors",
                1024 => "1024_vectors",
                3072 => "3072_vectors",
                1536 => "1536_vectors",
                _ => {
                    return Err(ServiceError::BadRequest(
                        "Invalid embedding vector size".to_string(),
                    ))
                }
            }
        } else {
            return Err(ServiceError::BadRequest(
                "Invalid recommend type".to_string(),
            ));
        }
    } else {
        match config.EMBEDDING_SIZE {
            384 => "384_vectors",
            512 => "512_vectors",
            768 => "768_vectors",
            1024 => "1024_vectors",
            3072 => "3072_vectors",
            1536 => "1536_vectors",
            _ => {
                return Err(ServiceError::BadRequest(
                    "Invalid embedding vector size".to_string(),
                ))
            }
        }
    };

    let recommend_points = RecommendPointGroups {
        collection_name: qdrant_collection,
        positive: positive_point_ids.clone(),
        negative: negative_point_ids.clone(),
        filter: Some(filters),
        limit: limit.try_into().unwrap_or(10),
        with_payload: None,
        params: None,
        score_threshold: None,
        using: Some(vector_name.to_string()),
        with_vectors: None,
        lookup_from: None,
        read_consistency: None,
        positive_vectors: vec![],
        negative_vectors: vec![],
        strategy: recommend_strategy,
        timeout: Some(60),
        shard_key_selector: None,
        group_by: "group_ids".to_string(),
        group_size: if group_size == 0 { 1 } else { group_size },
        with_lookup: None,
    };

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let data = qdrant_client
        .recommend_groups(&recommend_points)
        .await
        .map_err(|err| {
            log::error!("Failed to recommend groups points from qdrant: {:?}", err);
            ServiceError::BadRequest("Failed to recommend groups points from qdrant.".to_string())
        })?;

    let group_recommendation_results = data
        .result
        .ok_or(ServiceError::BadRequest(
            "Failed to recommend groups points from qdrant with no result on data response"
                .to_string(),
        ))?
        .groups
        .iter()
        .filter_map(|point| {
            let group_id = match &point.id.clone()?.kind? {
                Kind::StringValue(id) => uuid::Uuid::from_str(id).unwrap_or_default(),
                _ => {
                    return None;
                }
            };

            let hits: Vec<SearchResult> = point
                .hits
                .iter()
                .filter_map(|hit| match hit.id.clone()?.point_id_options? {
                    PointIdOptions::Uuid(id) => Some(SearchResult {
                        score: hit.score,
                        point_id: uuid::Uuid::parse_str(&id).ok()?,
                    }),
                    PointIdOptions::Num(_) => None,
                })
                .collect();

            if group_size == 0 {
                Some(GroupSearchResults {
                    group_id,
                    hits: vec![],
                })
            } else {
                Some(GroupSearchResults { group_id, hits })
            }
        })
        .collect();

    Ok(group_recommendation_results)
}

#[tracing::instrument]
pub async fn get_point_count_qdrant_query(
    filters: Filter,
    config: ServerDatasetConfiguration,
    get_total_pages: bool,
) -> Result<u64, ServiceError> {
    if !get_total_pages {
        return Ok(0);
    };

    let qdrant_collection = format!("{}_vectors", config.EMBEDDING_SIZE);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let data = qdrant_client
        .count(&CountPoints {
            collection_name: qdrant_collection,
            filter: Some(filters),
            exact: Some(true),
            read_consistency: None,
            shard_key_selector: None,
        })
        .await
        .map_err(|err| {
            log::info!("Failed to count points from qdrant: {:?}", err);
            ServiceError::BadRequest("Failed to count points from qdrant".to_string())
        })?;

    Ok(data.result.expect("Failed to get result from qdrant").count)
}

#[tracing::instrument]
pub async fn point_ids_exists_in_qdrant(
    point_ids: Vec<uuid::Uuid>,
    config: ServerDatasetConfiguration,
) -> Result<bool, ServiceError> {
    let qdrant_collection = format!("{}_vectors", config.EMBEDDING_SIZE);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let points: Vec<PointId> = point_ids.iter().map(|x| x.to_string().into()).collect();

    let data = qdrant_client
        .get_points(
            qdrant_collection,
            None,
            &points,
            false.into(),
            false.into(),
            None,
        )
        .await
        .map_err(|err| {
            log::info!("Failed to fetch points from qdrant {:?}", err);
            ServiceError::BadRequest("Failed to fetch points from qdrant".to_string())
        })?;

    Ok(data.result.len() == point_ids.len())
}
