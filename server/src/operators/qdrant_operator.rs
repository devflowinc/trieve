use super::{
    group_operator::get_groups_from_group_ids_query,
    search_operator::{assemble_qdrant_filter, SearchResult, SearchResultTrait},
};
use crate::{
    data::models::{
        ChunkMetadata, DatasetConfiguration, DistanceMetric, Pool, QdrantPayload, RecommendType,
        RecommendationStrategy, SortByField, SortOrder,
    },
    errors::ServiceError,
    get_env,
    handlers::chunk_handler::ChunkFilter,
};
use actix_web::web;
use futures::future::try_join_all;
use itertools::Itertools;
use qdrant_client::{
    qdrant::{
        group_id::Kind, point_id::PointIdOptions, quantization_config::Quantization, query,
        vectors::VectorsOptions, BinaryQuantization, CreateCollectionBuilder,
        CreateFieldIndexCollectionBuilder, DeleteFieldIndexCollectionBuilder, DeletePointsBuilder,
        Distance, FieldType, Filter, GetPointsBuilder, HnswConfigDiff, OrderBy, PointId,
        PointStruct, PrefetchQuery, QuantizationConfig, Query, QueryBatchPoints, QueryPointGroups,
        QueryPoints, RecommendPointGroups, RecommendPoints, RecommendStrategy, RetrievedPoint,
        ScrollPointsBuilder, SearchBatchPoints, SearchParams, SearchPointGroups, SearchPoints,
        SetPayloadPointsBuilder, SparseIndexConfig, SparseVectorConfig, SparseVectorParams,
        TextIndexParamsBuilder, TokenizerType, UpsertPointsBuilder, UuidIndexParamsBuilder, Value,
        Vector, VectorInput, VectorParams, VectorParamsMap, VectorsConfig, WithPayloadSelector,
        WithVectorsSelector,
    },
    Payload, Qdrant,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};

pub async fn get_qdrant_connection(
    qdrant_url: Option<&str>,
    qdrant_api_key: Option<&str>,
) -> Result<Qdrant, ServiceError> {
    let qdrant_url = qdrant_url.unwrap_or(get_env!(
        "QDRANT_URL",
        "QDRANT_URL should be set if this is called"
    ));
    let qdrant_api_key = qdrant_api_key.unwrap_or(get_env!(
        "QDRANT_API_KEY",
        "QDRANT_API_KEY should be set if this is called"
    ));

    Qdrant::from_url(qdrant_url)
        .api_key(qdrant_api_key)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|_err| ServiceError::BadRequest("Failed to connect to Qdrant".to_string()))
}

pub fn get_qdrant_collection_from_dataset_config(dataset_config: &DatasetConfiguration) -> String {
    match dataset_config.DISTANCE_METRIC {
        DistanceMetric::Euclidean => {
            format!("{}_vectors_euclidian", dataset_config.EMBEDDING_SIZE)
        }
        DistanceMetric::Manhattan => {
            format!("{}_vectors_manhattan", dataset_config.EMBEDDING_SIZE)
        }
        DistanceMetric::Dot => {
            format!("{}_vectors_dot", dataset_config.EMBEDDING_SIZE)
        }
        DistanceMetric::Cosine => {
            format!("{}_vectors", dataset_config.EMBEDDING_SIZE)
        }
    }
}

/// Create Qdrant collection and indexes needed
pub async fn create_new_qdrant_collection_query(
    qdrant_url: Option<&str>,
    qdrant_api_key: Option<&str>,
    quantize: bool,
    recreate_indexes: bool,
    replication_factor: u32,
    accepted_vectors: Vec<u64>,
    shard_number: u32,
) -> Result<(), ServiceError> {
    let qdrant_client = get_qdrant_connection(qdrant_url, qdrant_api_key).await?;

    let qdrant_collections: Vec<(String, u64, Distance)> = accepted_vectors
        .iter()
        .flat_map(|size| {
            vec![
                (format!("{}_vectors", *size), *size, Distance::Cosine),
                (
                    format!("{}_vectors_manhattan", *size),
                    *size,
                    Distance::Manhattan,
                ),
                (format!("{}_vectors_dot", *size), *size, Distance::Dot),
                (
                    format!("{}_vectors_euclidian", *size),
                    *size,
                    Distance::Euclid,
                ),
            ]
        })
        .collect();

    for (collection_name, size, distance) in qdrant_collections {
        // check if collection exists
        let collection = qdrant_client
            .collection_exists(collection_name.clone())
            .await
            .map_err(|e| {
                ServiceError::BadRequest(format!("Failed to see if collection exists {}", e))
            })?;

        match collection {
            true => log::info!("Avoided creating collection as it already exists"),
            false => {
                let mut sparse_vector_config = HashMap::new();
                sparse_vector_config.insert(
                    "sparse_vectors".to_string(),
                    SparseVectorParams {
                        modifier: None,
                        index: Some(SparseIndexConfig {
                            on_disk: Some(false),
                            ..Default::default()
                        }),
                    },
                );

                sparse_vector_config.insert(
                    "bm25_vectors".to_string(),
                    SparseVectorParams {
                        modifier: Some(1),
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
                        format!("{}_vectors", size).to_string(),
                        VectorParams {
                            size,
                            distance: distance.into(),
                            quantization_config,
                            on_disk,
                            ..Default::default()
                        },
                    )]
                    .into_iter(),
                );

                qdrant_client
                    .create_collection(
                        CreateCollectionBuilder::new(collection_name.clone())
                            .vectors_config(VectorsConfig {
                                config: Some(
                                    qdrant_client::qdrant::vectors_config::Config::ParamsMap(
                                        VectorParamsMap {
                                            map: vectors_hash_map,
                                        },
                                    ),
                                ),
                            })
                            .sparse_vectors_config(SparseVectorConfig {
                                map: sparse_vector_config,
                            })
                            .hnsw_config(HnswConfigDiff {
                                payload_m: Some(16),
                                m: Some(0),
                                ..Default::default()
                            })
                            .write_consistency_factor(1)
                            .replication_factor(replication_factor)
                            .shard_number(shard_number),
                    )
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
                .delete_field_index(DeleteFieldIndexCollectionBuilder::new(
                    collection_name.clone(),
                    "link",
                ))
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(DeleteFieldIndexCollectionBuilder::new(
                    collection_name.clone(),
                    "tag_set",
                ))
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(DeleteFieldIndexCollectionBuilder::new(
                    collection_name.clone(),
                    "dataset_id",
                ))
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(DeleteFieldIndexCollectionBuilder::new(
                    collection_name.clone(),
                    "metadata",
                ))
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(DeleteFieldIndexCollectionBuilder::new(
                    collection_name.clone(),
                    "time_stamp",
                ))
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(DeleteFieldIndexCollectionBuilder::new(
                    collection_name.clone(),
                    "group_ids",
                ))
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(DeleteFieldIndexCollectionBuilder::new(
                    collection_name.clone(),
                    "location",
                ))
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(DeleteFieldIndexCollectionBuilder::new(
                    collection_name.clone(),
                    "content",
                ))
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;

            qdrant_client
                .delete_field_index(DeleteFieldIndexCollectionBuilder::new(
                    collection_name.clone(),
                    "num_value",
                ))
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to delete index".into()))?;
        }

        qdrant_client
            .create_field_index(CreateFieldIndexCollectionBuilder::new(
                collection_name.clone(),
                "link",
                FieldType::Keyword,
            ))
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(CreateFieldIndexCollectionBuilder::new(
                collection_name.clone(),
                "tag_set",
                FieldType::Keyword,
            ))
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(
                CreateFieldIndexCollectionBuilder::new(
                    collection_name.clone(),
                    "dataset_id",
                    FieldType::Uuid,
                )
                .field_index_params(
                    UuidIndexParamsBuilder::default()
                        .is_tenant(true)
                        .on_disk(false)
                        .build(),
                ),
            )
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(CreateFieldIndexCollectionBuilder::new(
                collection_name.clone(),
                "metadata",
                FieldType::Keyword,
            ))
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(CreateFieldIndexCollectionBuilder::new(
                collection_name.clone(),
                "time_stamp",
                FieldType::Integer,
            ))
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(CreateFieldIndexCollectionBuilder::new(
                collection_name.clone(),
                "group_ids",
                FieldType::Keyword,
            ))
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(CreateFieldIndexCollectionBuilder::new(
                collection_name.clone(),
                "location",
                FieldType::Geo,
            ))
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(
                CreateFieldIndexCollectionBuilder::new(
                    collection_name.clone(),
                    "content",
                    FieldType::Text,
                )
                .field_index_params(
                    TextIndexParamsBuilder::new(TokenizerType::Prefix)
                        .min_token_len(2)
                        .max_token_len(10)
                        .lowercase(true)
                        .on_disk(false),
                )
                .build(),
            )
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(CreateFieldIndexCollectionBuilder::new(
                collection_name.clone(),
                "num_value",
                FieldType::Float,
            ))
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;

        qdrant_client
            .create_field_index(CreateFieldIndexCollectionBuilder::new(
                collection_name.clone(),
                "group_tag_set",
                FieldType::Keyword,
            ))
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create index".into()))?;
    }

    Ok(())
}

pub async fn bulk_upsert_qdrant_points_query(
    points: Vec<PointStruct>,
    dataset_config: DatasetConfiguration,
) -> Result<(), ServiceError> {
    if points.is_empty() {
        return Err(ServiceError::BadRequest(
            "No points were created for QDRANT, this is due to the incorrect embedding vector size"
                .into(),
        ));
    }

    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    qdrant_client
        .upsert_points(UpsertPointsBuilder::new(qdrant_collection, points))
        .await
        .map_err(|err| {
            log::error!("Failed inserting chunk to qdrant {:?}", err);
            ServiceError::BadRequest(format!("Failed inserting chunk to qdrant {:?}", err))
        })?;

    Ok(())
}

pub async fn create_new_qdrant_point_query(
    point_id: uuid::Uuid,
    embedding_vector: Vec<f32>,
    chunk_metadata: ChunkMetadata,
    splade_vector: Vec<(u32, f32)>,
    group_ids: Option<Vec<uuid::Uuid>>,
    dataset_config: DatasetConfiguration,
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

    let payload = QdrantPayload::new(chunk_metadata, group_ids, None, chunk_tags);
    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);

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
        .upsert_points(UpsertPointsBuilder::new(qdrant_collection, vec![point]))
        .await
        .map_err(|err| {
            log::error!("Failed inserting chunk to qdrant {:?}", err);
            ServiceError::BadRequest(format!("Failed inserting chunk to qdrant {:?}", err))
        })?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]

pub async fn update_qdrant_point_query(
    metadata: ChunkMetadata,
    updated_vector: Option<Vec<f32>>,
    group_ids: Option<Vec<uuid::Uuid>>,
    dataset_id: uuid::Uuid,
    splade_vector: Vec<(u32, f32)>,
    bm25_vector: Option<Vec<(u32, f32)>>,
    dataset_config: DatasetConfiguration,
    web_pool: web::Data<Pool>,
) -> Result<(), actix_web::Error> {
    let qdrant_point_id: Vec<PointId> = vec![metadata.qdrant_point_id.to_string().clone().into()];

    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let current_point_vec = qdrant_client
        .get_points(
            GetPointsBuilder::new(qdrant_collection.clone(), qdrant_point_id.clone())
                .with_payload(true)
                .with_vectors(false)
                .build(),
        )
        .await
        .map_err(|_err| ServiceError::BadRequest("Failed to search_points from qdrant".into()))?
        .result;

    let current_point = current_point_vec.first();

    let payload = {
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
                .filter_map(|id| id.to_string().parse::<uuid::Uuid>().ok())
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
            metadata.clone(),
            group_ids.into(),
            Some(dataset_id),
            Some(chunk_tags),
        )
    };

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
        let mut vector_payload = HashMap::from([
            (vector_name.to_string(), Vector::from(updated_vector)),
            ("sparse_vectors".to_string(), Vector::from(splade_vector)),
        ]);

        if let Some(bm25_vector) = bm25_vector.clone() {
            vector_payload.insert(
                "bm25_vectors".to_string(),
                Vector::from(bm25_vector.clone()),
            );
        }

        let point = PointStruct::new(
            metadata.qdrant_point_id.clone().to_string(),
            vector_payload,
            payload,
        );

        qdrant_client
            .upsert_points(UpsertPointsBuilder::new(qdrant_collection, vec![point]))
            .await
            .map_err(|_err| ServiceError::BadRequest("Failed upserting chunk in qdrant".into()))?;

        return Ok(());
    }

    qdrant_client
        .overwrite_payload(
            SetPayloadPointsBuilder::new(
                qdrant_collection,
                <QdrantPayload as std::convert::Into<Payload>>::into(payload),
            )
            .points_selector(qdrant_point_id),
        )
        .await
        .map_err(|_err| {
            ServiceError::BadRequest("Failed updating chunk payload in qdrant".into())
        })?;

    Ok(())
}

pub async fn add_bookmark_to_qdrant_query(
    point_id: uuid::Uuid,
    group_id: uuid::Uuid,
    dataset_config: DatasetConfiguration,
) -> Result<(), ServiceError> {
    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let qdrant_point_id: Vec<PointId> = vec![point_id.to_string().into()];

    let current_point_vec = qdrant_client
        .get_points(
            GetPointsBuilder::new(qdrant_collection.clone(), qdrant_point_id.clone())
                .with_payload(true)
                .with_vectors(false)
                .build(),
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

    qdrant_client
        .overwrite_payload(
            SetPayloadPointsBuilder::new(
                qdrant_collection,
                <QdrantPayload as std::convert::Into<Payload>>::into(payload),
            )
            .points_selector(qdrant_point_id),
        )
        .await
        .map_err(|_err| {
            ServiceError::BadRequest("Failed updating chunk payload in qdrant".into())
        })?;

    Ok(())
}

pub async fn remove_bookmark_from_qdrant_query(
    point_id: uuid::Uuid,
    group_id: uuid::Uuid,
    dataset_config: DatasetConfiguration,
) -> Result<(), ServiceError> {
    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let qdrant_point_id: Vec<PointId> = vec![point_id.to_string().into()];

    let current_point_vec = qdrant_client
        .get_points(
            GetPointsBuilder::new(qdrant_collection.clone(), qdrant_point_id.clone())
                .with_payload(true)
                .with_vectors(false)
                .build(),
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

    qdrant_client
        .overwrite_payload(
            SetPayloadPointsBuilder::new(
                qdrant_collection,
                <QdrantPayload as std::convert::Into<Payload>>::into(payload),
            )
            .points_selector(qdrant_point_id),
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

impl SearchResultTrait for GroupSearchResults {
    fn score(&self) -> f32 {
        self.hits.get(0).map_or(0.0, |hit| hit.score)
    }

    fn point_id(&self) -> uuid::Uuid {
        self.hits
            .get(0)
            .map_or(uuid::Uuid::default(), |hit| hit.point_id)
    }

    fn payload(&self) -> HashMap<String, qdrant_client::qdrant::Value> {
        self.hits
            .get(0)
            .map_or(HashMap::new(), |hit| hit.payload.clone())
    }

    fn set_score(&mut self, score: f32) {
        if let Some(hit) = self.hits.get_mut(0) {
            hit.score = score;
        }
    }

    fn embedding(&self) -> Option<Vec<f32>> {
        self.hits.get(0).and_then(|hit| hit.embedding.clone())
    }
}

#[derive(Debug, Clone)]
pub enum VectorType {
    SpladeSparse(Vec<(u32, f32)>),
    BM25Sparse(Vec<(u32, f32)>),
    Dense(Vec<f32>),
}

#[derive(Debug, Clone)]
pub struct QdrantSearchQuery {
    pub filter: Filter,
    pub limit: u64,
    pub score_threshold: Option<f32>,
    pub rerank_by: Box<Option<QdrantSearchQuery>>,
    pub sort_by: Option<SortByField>,
    pub vector: VectorType,
    pub group_size: Option<u64>,
}

#[allow(clippy::too_many_arguments)]
pub async fn search_over_groups_qdrant_query(
    page: u64,
    queries: Vec<QdrantSearchQuery>,
    dataset_config: DatasetConfiguration,
    get_total_pages: bool,
    use_mmr: bool,
) -> Result<(Vec<GroupSearchResults>, u64, Vec<usize>), ServiceError> {
    if queries.is_empty() || queries.iter().all(|query| query.limit == 0) {
        return Ok((vec![], 0, vec![]));
    }

    let group_size = queries
        .iter()
        .map(|query| query.group_size.unwrap_or(1))
        .max()
        .unwrap_or(3);

    let get_payload = dataset_config.QDRANT_ONLY;

    let limit = queries.iter().map(|query| query.limit).max().unwrap_or(10);

    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let count_limit = if !get_total_pages { 0_u64 } else { 100000_u64 };

    let count_future =
        count_qdrant_group_query(count_limit, queries.clone(), dataset_config.clone());

    let search_point_req_payloads: Vec<QueryPointGroups> = queries
        .into_iter()
        .map(|query| {
            let (mut prefetch, (vector_name, qdrant_query)) =
                get_prefetch_query(query.clone(), dataset_config.clone());

            let offset = query.limit * page.saturating_sub(1);

            if let Some(prefetch) = prefetch.get_mut(0) {
                let new_page = if offset / prefetch.limit.unwrap_or(1) > 0 {
                    (offset / prefetch.limit.unwrap_or(1)) + 1
                } else {
                    1
                };

                prefetch.limit = Some(prefetch.limit.unwrap_or(1) * new_page);
            }

            let score_threshold = match qdrant_query.variant {
                Some(query::Variant::OrderBy(_)) => None,
                _ => query.score_threshold,
            };

            let group_size = query.group_size.unwrap_or(3);

            QueryPointGroups {
                collection_name: qdrant_collection.to_string(),
                limit: if use_mmr && query.limit < 20 {
                    Some(query.limit * 2)
                } else {
                    Some(query.limit * page)
                },
                prefetch,
                using: vector_name,
                query: Some(qdrant_query),
                score_threshold,
                with_payload: Some(WithPayloadSelector::from(get_payload)),
                with_vectors: Some(WithVectorsSelector::from(use_mmr)),
                timeout: Some(60),
                filter: Some(query.filter.clone()),
                params: Some(SearchParams {
                    exact: Some(false),
                    indexed_only: Some(dataset_config.INDEXED_ONLY),
                    ..Default::default()
                }),
                group_by: "group_ids".to_string(),
                group_size: Some(if group_size == 0 { 1 } else { group_size }),
                ..Default::default()
            }
        })
        .collect::<Vec<QueryPointGroups>>();

    let search_batch_future = search_point_req_payloads
        .into_iter()
        .map(|search_point_req_payload| qdrant_client.query_groups(search_point_req_payload))
        .collect::<Vec<_>>();

    let search_batch_futures = try_join_all(search_batch_future);

    let (count, search_batch_response) =
        futures::future::join(count_future, search_batch_futures).await;

    let batch_lengths: Vec<usize> = search_batch_response
        .iter()
        .flatten()
        .map(|batch_result| {
            batch_result
                .result
                .as_ref()
                .map(|x| x.groups.len())
                .unwrap_or(0)
        })
        .collect();

    let point_ids: Vec<GroupSearchResults> = search_batch_response
        .map_err(|e| {
            log::error!("Failed to search points on Qdrant {:?}", e);
            ServiceError::BadRequest("Failed to search points on Qdrant".to_string())
        })?
        .into_iter()
        .flat_map(|response| {
            response
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
                                payload: hit.payload.clone(),
                                embedding: hit.vectors.clone().map(|v| match v.vectors_options {
                                    Some(VectorsOptions::Vectors(named_v)) => named_v
                                        .vectors
                                        .into_iter()
                                        .filter(|v| v.1.indices.is_none())
                                        .map(|v| v.1.data)
                                        .collect::<Vec<_>>()
                                        .get(0)
                                        .unwrap_or(&vec![])
                                        .clone(),
                                    _ => vec![],
                                }),
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
                .collect_vec()
        })
        .collect_vec();

    Ok((point_ids, count?, batch_lengths))
}

fn get_qdrant_vector(query: QdrantSearchQuery) -> (String, VectorInput) {
    match query.vector {
        VectorType::SpladeSparse(vector) => {
            let indices = vector.iter().map(|(index, _)| *index).collect::<Vec<u32>>();
            let data = vector.iter().map(|(_, value)| *value).collect::<Vec<f32>>();
            (
                "sparse_vectors".to_string(),
                VectorInput::new_sparse(indices, data),
            )
        }
        VectorType::BM25Sparse(vector) => {
            let indices = vector.iter().map(|(index, _)| *index).collect::<Vec<u32>>();
            let data = vector.iter().map(|(_, value)| *value).collect::<Vec<f32>>();
            (
                "bm25_vectors".to_string(),
                VectorInput::new_sparse(indices, data),
            )
        }
        VectorType::Dense(embedding_vector) => {
            let vector_name = match embedding_vector.len() {
                384 => "384_vectors",
                512 => "512_vectors",
                768 => "768_vectors",
                1024 => "1024_vectors",
                3072 => "3072_vectors",
                1536 => "1536_vectors",
                _ => "invalid",
            };
            (
                vector_name.to_string(),
                VectorInput::new_dense(embedding_vector),
            )
        }
    }
}

fn get_prefetch_query(
    query: QdrantSearchQuery,
    dataset_config: DatasetConfiguration,
) -> (Vec<PrefetchQuery>, (Option<String>, Query)) {
    if let Some(ref rerank_query) = *query.rerank_by {
        let (rerank_vector_name, rerank_vector) = get_qdrant_vector(rerank_query.clone());
        let (name, vector) = get_qdrant_vector(query.clone());
        (
            vec![PrefetchQuery {
                query: Some(Query::new_nearest(vector)),
                limit: Some(rerank_query.limit),
                using: Some(name),
                filter: Some(query.filter.clone()),
                ..Default::default()
            }],
            (Some(rerank_vector_name), Query::new_nearest(rerank_vector)),
        )
    } else if let Some(ref sort_by) = query.sort_by {
        let (name, vector) = get_qdrant_vector(query.clone());
        let prefetch_amount = sort_by.prefetch_amount.unwrap_or(1000);
        let prefetch_amount = if prefetch_amount > dataset_config.MAX_LIMIT {
            dataset_config.MAX_LIMIT
        } else {
            prefetch_amount
        };

        (
            vec![PrefetchQuery {
                query: Some(Query::new_nearest(vector)),
                limit: Some(prefetch_amount),
                using: Some(name),
                filter: Some(query.filter.clone()),
                score_threshold: query.score_threshold,
                ..Default::default()
            }],
            (
                None,
                Query::new_order_by(OrderBy {
                    key: sort_by.field.clone(),
                    direction: Some(sort_by.direction.clone().unwrap_or(SortOrder::Desc).into()),
                    ..Default::default()
                }),
            ),
        )
    } else {
        let (name, vector) = get_qdrant_vector(query.clone());
        (vec![], (Some(name), Query::new_nearest(vector)))
    }
}

pub async fn search_qdrant_query(
    page: u64,
    queries: Vec<QdrantSearchQuery>,
    dataset_config: DatasetConfiguration,
    get_total_pages: bool,
    use_mmr: bool,
) -> Result<(Vec<SearchResult>, u64, Vec<usize>), ServiceError> {
    if queries.is_empty() || queries.iter().all(|query| query.limit == 0) {
        return Ok((vec![], 0, vec![]));
    }

    let get_payload = dataset_config.QDRANT_ONLY;

    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let count_limit = if !get_total_pages { 0_u64 } else { 100000_u64 };

    let count_future = count_qdrant_query(count_limit, queries.clone(), dataset_config.clone());

    let search_point_req_payloads: Vec<QueryPoints> = queries
        .into_iter()
        .map(|query| {
            let (mut prefetch, (vector_name, qdrant_query)) =
                get_prefetch_query(query.clone(), dataset_config.clone());

            let offset = query.limit * page.saturating_sub(1);
            if let Some(prefetch) = prefetch.get_mut(0) {
                let new_page = if offset / prefetch.limit.unwrap_or(1) > 0 {
                    (offset / prefetch.limit.unwrap_or(1)) + 1
                } else {
                    1
                };

                prefetch.limit = Some(prefetch.limit.unwrap_or(1) * new_page);
            }

            let score_threshold = match qdrant_query.variant {
                Some(query::Variant::OrderBy(_)) => None,
                _ => query.score_threshold,
            };

            QueryPoints {
                collection_name: qdrant_collection.to_string(),
                limit: if use_mmr && query.limit < 20 {
                    Some(query.limit * 2)
                } else {
                    Some(query.limit * page)
                },
                offset: Some(offset),
                prefetch,
                using: vector_name,
                query: Some(qdrant_query),
                score_threshold,
                with_payload: Some(WithPayloadSelector::from(get_payload)),
                with_vectors: Some(WithVectorsSelector::from(use_mmr)),
                timeout: Some(60),
                filter: Some(query.filter.clone()),
                params: Some(SearchParams {
                    exact: Some(false),
                    indexed_only: Some(dataset_config.INDEXED_ONLY),
                    ..Default::default()
                }),
                ..Default::default()
            }
        })
        .collect::<Vec<QueryPoints>>();

    let batch_points = QueryBatchPoints {
        collection_name: qdrant_collection.to_string(),
        query_points: search_point_req_payloads.clone(),
        timeout: Some(60),
        ..Default::default()
    };

    let search_batch_future = qdrant_client.query_batch(batch_points);

    let (count, search_batch_response) =
        futures::future::join(count_future, search_batch_future).await;

    let search_batch_response = search_batch_response.map_err(|e| {
        log::error!("Failed to search points on Qdrant {:?}", e);
        ServiceError::BadRequest(format!("Failed to search points on Qdrant {:?}", e))
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
                            payload: scored_point.payload.clone(),
                            embedding: scored_point.vectors.clone().map(|v| {
                                match v.vectors_options {
                                    Some(VectorsOptions::Vectors(named_v)) => named_v
                                        .vectors
                                        .into_iter()
                                        .filter(|v| v.1.indices.is_none())
                                        .map(|v| v.1.data)
                                        .collect::<Vec<_>>()
                                        .get(0)
                                        .unwrap_or(&vec![])
                                        .clone(),
                                    _ => vec![],
                                }
                            }),
                        }),
                        PointIdOptions::Num(_) => None,
                    },
                )
                .collect::<Vec<SearchResult>>()
        })
        .unique_by(|point| point.point_id)
        .collect();

    Ok((search_results, count?, batch_lengths))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QdrantRecommendResult {
    pub point_id: uuid::Uuid,
    pub score: f32,
}

#[allow(clippy::too_many_arguments)]

pub async fn recommend_qdrant_query(
    positive_ids: Vec<uuid::Uuid>,
    negative_ids: Vec<uuid::Uuid>,
    strategy: Option<RecommendationStrategy>,
    recommend_type: Option<RecommendType>,
    filters: Option<ChunkFilter>,
    limit: u64,
    dataset_id: uuid::Uuid,
    dataset_config: DatasetConfiguration,
    pool: web::Data<Pool>,
) -> Result<Vec<SearchResult>, ServiceError> {
    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);

    let recommend_strategy = match strategy {
        Some(strategy) => match strategy {
            RecommendationStrategy::BestScore => Some(RecommendStrategy::BestScore.into()),
            RecommendationStrategy::AverageVector => Some(RecommendStrategy::AverageVector.into()),
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

    let recommend_type = recommend_type.unwrap_or(RecommendType::Semantic);

    let vector_name = match recommend_type {
        RecommendType::Semantic => match dataset_config.EMBEDDING_SIZE {
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
        RecommendType::FullText => "sparse_vectors",
        RecommendType::BM25 => "bm25_vectors",
    };

    let recommend_points = RecommendPoints {
        collection_name: qdrant_collection,
        positive: positive_point_ids.clone(),
        negative: negative_point_ids.clone(),
        filter: Some(filter),
        limit,
        with_payload: Some(WithPayloadSelector::from(false)),
        with_vectors: Some(WithVectorsSelector::from(false)),
        params: Some(SearchParams {
            exact: Some(false),
            indexed_only: Some(dataset_config.INDEXED_ONLY),
            ..Default::default()
        }),
        score_threshold: None,
        offset: None,
        using: Some(vector_name.to_string()),
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
        .recommend(recommend_points)
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

            Some(SearchResult {
                point_id,
                score: point.score,
                payload: point.payload.clone(),
                embedding: None,
            })
        })
        .collect::<Vec<SearchResult>>();

    Ok(recommended_point_ids)
}

#[allow(clippy::too_many_arguments)]
pub async fn recommend_qdrant_groups_query(
    positive_ids: Vec<uuid::Uuid>,
    negative_ids: Vec<uuid::Uuid>,
    strategy: Option<RecommendationStrategy>,
    recommend_type: Option<RecommendType>,
    filter: Option<ChunkFilter>,
    limit: u64,
    group_size: u32,
    dataset_id: uuid::Uuid,
    dataset_config: DatasetConfiguration,
    pool: web::Data<Pool>,
) -> Result<Vec<GroupSearchResults>, ServiceError> {
    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);

    let recommend_strategy = match strategy {
        Some(RecommendationStrategy::BestScore) => Some(RecommendStrategy::BestScore.into()),
        _ => None,
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

    let recommend_type = recommend_type.unwrap_or(RecommendType::Semantic);

    let vector_name = match recommend_type {
        RecommendType::Semantic => match dataset_config.EMBEDDING_SIZE {
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
        RecommendType::FullText => "sparse_vectors",
        RecommendType::BM25 => "bm25_vectors",
    };

    let recommend_points = RecommendPointGroups {
        collection_name: qdrant_collection,
        positive: positive_point_ids.clone(),
        negative: negative_point_ids.clone(),
        filter: Some(filters),
        limit: limit.try_into().unwrap_or(10),
        with_payload: Some(WithPayloadSelector::from(false)),
        with_vectors: Some(WithVectorsSelector::from(false)),
        params: Some(SearchParams {
            exact: Some(false),
            indexed_only: Some(dataset_config.INDEXED_ONLY),
            ..Default::default()
        }),
        score_threshold: None,
        using: Some(vector_name.to_string()),
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
        .recommend_groups(recommend_points)
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
                        payload: hit.payload.clone(),
                        embedding: None,
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

pub async fn point_ids_exists_in_qdrant(
    point_ids: Vec<uuid::Uuid>,
    dataset_config: DatasetConfiguration,
) -> Result<bool, ServiceError> {
    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let points: Vec<PointId> = point_ids.iter().map(|x| x.to_string().into()).collect();

    let data = qdrant_client
        .get_points(
            GetPointsBuilder::new(qdrant_collection.clone(), points.clone())
                .with_payload(false)
                .with_vectors(false)
                .build(),
        )
        .await
        .map_err(|err| {
            log::info!("Failed to fetch points from qdrant {:?}", err);
            ServiceError::BadRequest("Failed to fetch points from qdrant".to_string())
        })?;

    Ok(data.result.len() == point_ids.len())
}

pub fn get_collection_name_from_config(config: &DatasetConfiguration) -> String {
    format!("{}_vectors", config.EMBEDDING_SIZE)
}

pub async fn delete_points_from_qdrant(
    point_ids: Vec<uuid::Uuid>,
    qdrant_collection: String,
) -> Result<(), ServiceError> {
    if point_ids.is_empty() {
        return Ok(());
    }

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let points: Vec<PointId> = point_ids.iter().map(|x| x.to_string().into()).collect();

    qdrant_client
        .delete_points(
            DeletePointsBuilder::new(qdrant_collection.clone())
                .points(points)
                .build(),
        )
        .await
        .map_err(|err| {
            log::info!("Failed to delete points from qdrant {:?}", err);
            ServiceError::BadRequest("Failed to delete points from qdrant".to_string())
        })?;

    Ok(())
}

pub async fn get_qdrant_collections() -> Result<Vec<String>, ServiceError> {
    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let qdrant_collections = qdrant_client.list_collections().await.map_err(|err| {
        log::info!("Failed to list collections from qdrant {:?}", err);
        ServiceError::BadRequest("Failed to list collections from qdrant".to_string())
    })?;

    let collection_names = qdrant_collections
        .collections
        .iter()
        .map(|collection| collection.name.clone())
        .collect();

    Ok(collection_names)
}

pub async fn scroll_qdrant_collection_ids_custom_url(
    collection_name: String,
    offset_id: Option<String>,
    limit: Option<u32>,
    qdrant_client: Qdrant,
) -> Result<(Vec<RetrievedPoint>, Option<String>), ServiceError> {
    let mut scroll_points_params = ScrollPointsBuilder::new(collection_name);

    if let Some(offset_id) = offset_id {
        scroll_points_params = scroll_points_params.offset(offset_id);
    };

    if let Some(limit) = limit {
        scroll_points_params = scroll_points_params.limit(limit);
    };
    let qdrant_point_ids = qdrant_client
        .scroll(scroll_points_params.with_payload(true).with_vectors(true))
        .await
        .map_err(|err| {
            log::info!("Failed to scroll points from qdrant {:?}", err);
            ServiceError::BadRequest("Failed to scroll points from qdrant".to_string())
        })?;

    let points = qdrant_point_ids
        .result
        .iter()
        .filter_map(|point| {
            match point.id.clone()?.point_id_options? {
                PointIdOptions::Uuid(id) => uuid::Uuid::parse_str(&id).ok()?,
                PointIdOptions::Num(_) => {
                    return None;
                }
            };

            Some(point.clone())
        })
        .collect::<Vec<RetrievedPoint>>();

    let offset = qdrant_point_ids
        .next_page_offset
        .map(|id| match id.point_id_options {
            Some(PointIdOptions::Uuid(id)) => id,
            _ => "".to_string(),
        });

    Ok((points, offset))
}

pub async fn scroll_qdrant_collection_ids(
    collection_name: String,
    offset_id: Option<String>,
    limit: Option<u32>,
) -> Result<(Vec<uuid::Uuid>, Option<String>), ServiceError> {
    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let mut scroll_points_params = ScrollPointsBuilder::new(collection_name);

    if let Some(offset_id) = offset_id {
        scroll_points_params = scroll_points_params.offset(offset_id);
    };

    if let Some(limit) = limit {
        scroll_points_params = scroll_points_params.limit(limit);
    };
    let qdrant_point_ids = qdrant_client
        .scroll(scroll_points_params.with_payload(false).with_vectors(false))
        .await
        .map_err(|err| {
            log::info!("Failed to scroll points from qdrant {:?}", err);
            ServiceError::BadRequest("Failed to scroll points from qdrant".to_string())
        })?;

    let point_ids = qdrant_point_ids
        .result
        .iter()
        .filter_map(|point| {
            let point_id = match point.id.clone()?.point_id_options? {
                PointIdOptions::Uuid(id) => uuid::Uuid::parse_str(&id).ok()?,
                PointIdOptions::Num(_) => {
                    return None;
                }
            };

            Some(point_id)
        })
        .collect::<Vec<uuid::Uuid>>();

    let offset = qdrant_point_ids
        .next_page_offset
        .map(|id| match id.point_id_options {
            Some(PointIdOptions::Uuid(id)) => id,
            _ => "".to_string(),
        });

    Ok((point_ids, offset))
}

pub async fn count_qdrant_group_query(
    limit: u64,
    queries: Vec<QdrantSearchQuery>,
    dataset_config: DatasetConfiguration,
) -> Result<u64, ServiceError> {
    if limit == 0 {
        return Ok(0);
    }

    let limit = if limit > dataset_config.MAX_LIMIT {
        dataset_config.MAX_LIMIT
    } else {
        limit
    };

    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let search_point_req_payloads: Vec<SearchPointGroups> = queries
        .into_iter()
        .map(|query| match query.vector {
            VectorType::SpladeSparse(vector) => {
                let sparse_vector: Vector = vector.into();
                Ok(SearchPointGroups {
                    collection_name: qdrant_collection.to_string(),
                    vector: sparse_vector.data,
                    sparse_indices: sparse_vector.indices,
                    vector_name: Some("sparse_vectors".to_string()),
                    limit: limit as u32,
                    score_threshold: query.score_threshold,
                    with_payload: Some(WithPayloadSelector::from(false)),
                    with_vectors: Some(WithVectorsSelector::from(false)),
                    filter: Some(query.filter.clone()),
                    group_by: "group_ids".to_string(),
                    group_size: 1,
                    timeout: Some(60),
                    params: None,
                    ..Default::default()
                })
            }
            VectorType::BM25Sparse(vector) => {
                let sparse_vector: Vector = vector.into();
                Ok(SearchPointGroups {
                    collection_name: qdrant_collection.to_string(),
                    vector: sparse_vector.data,
                    sparse_indices: sparse_vector.indices,
                    vector_name: Some("bm25_vectors".to_string()),
                    limit: limit as u32,
                    score_threshold: query.score_threshold,
                    with_payload: Some(WithPayloadSelector::from(false)),
                    with_vectors: Some(WithVectorsSelector::from(false)),
                    filter: Some(query.filter.clone()),
                    group_by: "group_ids".to_string(),
                    group_size: 1,
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

                Ok(SearchPointGroups {
                    collection_name: qdrant_collection.to_string(),
                    vector: embedding_vector,
                    vector_name: Some(vector_name.to_string()),
                    limit: limit as u32,
                    score_threshold: query.score_threshold,
                    with_payload: Some(WithPayloadSelector::from(false)),
                    with_vectors: Some(WithVectorsSelector::from(false)),
                    filter: Some(query.filter.clone()),
                    group_by: "group_ids".to_string(),
                    group_size: 1,
                    timeout: Some(60),
                    params: Some(SearchParams {
                        exact: Some(false),
                        indexed_only: Some(dataset_config.INDEXED_ONLY),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            }
        })
        .collect::<Result<Vec<SearchPointGroups>, ServiceError>>()?;

    let search_futures = search_point_req_payloads
        .iter()
        .map(|search_point_req_payload| {
            qdrant_client.search_groups(search_point_req_payload.clone())
        })
        .collect::<Vec<_>>();

    let search_responses = futures::future::join_all(search_futures).await;

    let max_count = search_responses
        .into_iter()
        .map(|search_response| match search_response {
            Ok(search_response) => match search_response.result {
                Some(result) => result.groups.len() as u64,
                None => 0,
            },
            Err(e) => {
                log::error!("Failed to search points on Qdrant to get count {:?}", e);
                0
            }
        })
        .max()
        .unwrap_or(0);

    Ok(max_count)
}

pub async fn count_qdrant_query(
    limit: u64,
    queries: Vec<QdrantSearchQuery>,
    dataset_config: DatasetConfiguration,
) -> Result<u64, ServiceError> {
    if limit == 0 {
        return Ok(0);
    }

    let limit = if limit > dataset_config.MAX_LIMIT {
        dataset_config.MAX_LIMIT
    } else {
        limit
    };

    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let search_point_req_payloads: Vec<SearchPoints> = queries
        .into_iter()
        .map(|query| match query.vector {
            VectorType::SpladeSparse(vector) => {
                let sparse_vector: Vector = vector.into();
                Ok(SearchPoints {
                    collection_name: qdrant_collection.to_string(),
                    vector: sparse_vector.data,
                    sparse_indices: sparse_vector.indices,
                    vector_name: Some("sparse_vectors".to_string()),
                    limit,
                    score_threshold: query.score_threshold,
                    with_payload: Some(WithPayloadSelector::from(false)),
                    with_vectors: Some(WithVectorsSelector::from(false)),
                    filter: Some(query.filter.clone()),
                    timeout: Some(60),
                    params: None,
                    ..Default::default()
                })
            }
            VectorType::BM25Sparse(vector) => {
                let sparse_vector: Vector = vector.into();
                Ok(SearchPoints {
                    collection_name: qdrant_collection.to_string(),
                    vector: sparse_vector.data,
                    sparse_indices: sparse_vector.indices,
                    vector_name: Some("bm25_vectors".to_string()),
                    limit,
                    score_threshold: query.score_threshold,
                    with_payload: Some(WithPayloadSelector::from(false)),
                    with_vectors: Some(WithVectorsSelector::from(false)),
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
                    with_payload: Some(WithPayloadSelector::from(false)),
                    with_vectors: Some(WithVectorsSelector::from(false)),
                    filter: Some(query.filter.clone()),
                    timeout: Some(60),
                    params: Some(SearchParams {
                        exact: Some(false),
                        indexed_only: Some(dataset_config.INDEXED_ONLY),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
            }
        })
        .collect::<Result<Vec<SearchPoints>, ServiceError>>()?;

    let batch_points = SearchBatchPoints {
        collection_name: qdrant_collection.to_string(),
        search_points: search_point_req_payloads.clone(),
        timeout: Some(60),
        ..Default::default()
    };

    let search_batch_response = qdrant_client
        .search_batch_points(batch_points)
        .await
        .map_err(|e| {
            log::error!("Failed to search points on Qdrant to get count {:?}", e);
            ServiceError::BadRequest("Failed to search points on Qdrant to get count".to_string())
        })?;

    let max_count = search_batch_response
        .result
        .iter()
        .map(|batch_result| batch_result.result.len() as u64)
        .collect::<Vec<u64>>()
        .into_iter()
        .max()
        .unwrap_or(0);

    Ok(max_count)
}

pub async fn update_group_tag_sets_in_qdrant_query(
    collection_name: String,
    prev_group_tag_set: Vec<String>,
    new_group_tag_set: Vec<String>,
    point_ids: Vec<uuid::Uuid>,
) -> Result<(), ServiceError> {
    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let points: Vec<PointId> = point_ids.iter().map(|x| x.to_string().into()).collect();

    let results = qdrant_client
        .get_points(
            GetPointsBuilder::new(collection_name.clone(), points.clone())
                .with_payload(true)
                .with_vectors(false)
                .build(),
        )
        .await
        .map_err(|e| {
            log::info!("Failed to fetch points from qdrant {:?}", e);
            ServiceError::BadRequest("Failed to fetch points from qdrant".to_string())
        })?;
    let qdrant_payloads: Vec<QdrantPayload> = results
        .result
        .iter()
        .map(|x| x.clone().into())
        .collect::<Vec<QdrantPayload>>();

    for (point, payload) in points.iter().zip(qdrant_payloads) {
        let mut payload_tags: Vec<Option<String>> = payload
            .group_tag_set
            .clone()
            .unwrap_or_default()
            .iter()
            .filter(|tag| match tag {
                Some(tag) => !prev_group_tag_set.contains(tag),
                None => false,
            })
            .cloned()
            .collect();

        let new_tags = new_group_tag_set.iter().map(|x| Some(x.clone()));
        payload_tags.extend(new_tags);
        payload_tags.dedup();

        let new_payload = QdrantPayload {
            group_tag_set: Some(payload_tags.clone()),
            ..payload
        };
        qdrant_client
            .overwrite_payload(
                SetPayloadPointsBuilder::new(
                    collection_name.clone(),
                    <QdrantPayload as std::convert::Into<Payload>>::into(new_payload),
                )
                .points_selector(vec![point.clone()]),
            )
            .await
            .map_err(|_| {
                ServiceError::BadRequest("Failed updating chunk payload in qdrant".into())
            })?;
    }

    Ok(())
}

pub async fn scroll_dataset_points(
    limit: u64,
    offset: Option<uuid::Uuid>,
    sort_by: Option<SortByField>,
    dataset_config: DatasetConfiguration,
    filter: Filter,
) -> Result<(Vec<SearchResult>, Option<uuid::Uuid>), ServiceError> {
    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);
    let mut scroll_points_params = ScrollPointsBuilder::new(qdrant_collection);

    scroll_points_params = scroll_points_params.limit(limit as u32);

    if let Some(offset_id) = offset {
        scroll_points_params = scroll_points_params.offset(offset_id.to_string());
    };

    if let Some(sort_by) = sort_by {
        scroll_points_params = scroll_points_params.order_by(OrderBy {
            key: sort_by.field,
            direction: Some(sort_by.direction.unwrap_or(SortOrder::Desc).into()),
            ..Default::default()
        });
    };

    scroll_points_params = scroll_points_params.filter(filter);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    let scroll_response = qdrant_client
        .scroll(
            scroll_points_params
                .with_payload(dataset_config.QDRANT_ONLY)
                .with_vectors(false),
        )
        .await
        .map_err(|err| {
            log::error!("Failed to scroll points from qdrant: {:?}", err);
            ServiceError::BadRequest(format!("Failed to scroll points from qdrant: {:?}", err))
        })?;

    let point_ids = scroll_response
        .result
        .iter()
        .filter_map(|point| {
            let point_id = match point.id.clone()?.point_id_options? {
                PointIdOptions::Uuid(id) => uuid::Uuid::parse_str(&id).ok()?,
                PointIdOptions::Num(_) => {
                    return None;
                }
            };
            let payload = point.payload.clone();

            Some(SearchResult {
                score: 0 as f32,
                point_id,
                payload,
                embedding: None,
            })
        })
        .collect::<Vec<SearchResult>>();

    Ok((
        point_ids,
        scroll_response
            .next_page_offset
            .map(|point| match point.point_id_options {
                Some(PointIdOptions::Uuid(id)) => uuid::Uuid::parse_str(&id).unwrap(),
                _ => uuid::Uuid::nil(),
            }),
    ))
}
