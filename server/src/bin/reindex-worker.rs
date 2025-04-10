use std::collections::HashMap;

use itertools::{izip, Itertools};
use qdrant_client::qdrant::{PointStruct, Vector};
#[allow(deprecated)]
use qdrant_client::{
    qdrant::{self, GetPointsBuilder, PointId, RetrievedPoint, UpsertPointsBuilder},
    Qdrant,
};
use trieve_server::{
    data::models::{DatasetConfiguration, MigratePointMessage, MigrationMode},
    errors::ServiceError,
    get_env,
    operators::{
        model_operator::{get_bm25_embeddings, get_dense_vectors, get_sparse_vectors},
        qdrant_operator::{bulk_upsert_qdrant_points_query, get_qdrant_connection},
    },
};

#[allow(clippy::print_stdout)]
#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    dotenvy::dotenv().ok();
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();

    let redis_url = get_env!("REDIS_URL", "REDIS_URL is not set");

    let redis_manager =
        bb8_redis::RedisConnectionManager::new(redis_url).expect("Failed to connect to redis");

    let redis_pool = bb8_redis::bb8::Pool::builder()
        .max_size(2)
        .connection_timeout(std::time::Duration::from_secs(2))
        .build(redis_manager)
        .await
        .expect("Failed to create redis pool");

    let redis_pool = actix_web::web::Data::new(redis_pool);
    let mut redis_connection = redis_pool
        .get()
        .await
        .map_err(|e| ServiceError::BadRequest(format!("Failed to connect to redis {}", e)))?;

    let mut broken_pipe_sleep = std::time::Duration::from_secs(10);

    log::info!("Starting reindex worker");
    loop {
        let payload_result: Result<Vec<String>, redis::RedisError> = redis::cmd("brpoplpush")
            .arg("collection_migration")
            .arg("collection_migration_started")
            .arg(1.0)
            .query_async(&mut *redis_connection)
            .await;

        let serialized_message = match payload_result {
            Ok(payload) => {
                broken_pipe_sleep = std::time::Duration::from_secs(10);

                if payload.is_empty() {
                    continue;
                }

                payload
                    .first()
                    .expect("Payload must have an element")
                    .clone()
            }
            Err(err) => {
                log::error!("IO broken pipe error, trying to acquire new connection");
                match redis_pool.get().await {
                    Ok(redis_conn) => {
                        log::info!("Got new redis connection after broken pipe! Resuming polling");
                        redis_connection = redis_conn;
                    }
                    Err(err) => {
                        log::error!(
                            "Failed to get redis connection after broken pipe, will try again after {broken_pipe_sleep:?} secs, err: {:?}",
                            err
                        );
                    }
                }

                if err.is_io_error() {
                    tokio::time::sleep(broken_pipe_sleep).await;
                    broken_pipe_sleep =
                        std::cmp::min(broken_pipe_sleep * 2, std::time::Duration::from_secs(300));
                }

                continue;
            }
        };

        let migration_message: MigratePointMessage = match serde_json::from_str(&serialized_message)
        {
            Ok(message) => message,
            Err(_) => {
                log::error!("Failed to deserialize message {:?}", serialized_message);
                continue;
            }
        };
        log::info!(
            "Migrating {} points from {} to {}",
            migration_message.qdrant_point_ids.len(),
            migration_message.from_collection,
            migration_message.to_collection
        );

        if migration_message.qdrant_point_ids.is_empty() {
            continue;
        }

        let qdrant_client = get_qdrant_connection(
            Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
            Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
        )
        .await?;

        // Get all points in message including Payload & Friends

        let result = match migration_message.mode {
            MigrationMode::BM25 { average_len, k, b } => {
                let points = qdrant_client
                    .get_points(
                        GetPointsBuilder::new(
                            migration_message.from_collection.clone(),
                            migration_message
                                .qdrant_point_ids
                                .iter()
                                .map(|uuid| PointId::from(uuid.to_string()))
                                .collect_vec(),
                        )
                        .with_payload(true)
                        .with_vectors(true)
                        .build(),
                    )
                    .await
                    .map_err(|_err| {
                        ServiceError::BadRequest("Failed to search_points from qdrant".to_string())
                    })?
                    .result;

                migrate_bm25(
                    qdrant_client,
                    points,
                    migration_message.to_collection,
                    average_len,
                    b,
                    k,
                )
                .await
            }
            MigrationMode::Reembed {
                embedding_model_name,
                embedding_base_url,
                embedding_size,
            } => {
                let points = qdrant_client
                    .get_points(
                        GetPointsBuilder::new(
                            migration_message.from_collection.clone(),
                            migration_message
                                .qdrant_point_ids
                                .iter()
                                .map(|uuid| PointId::from(uuid.to_string()))
                                .collect_vec(),
                        )
                        .with_payload(true)
                        .with_vectors(false)
                        .build(),
                    )
                    .await
                    .map_err(|_err| {
                        ServiceError::BadRequest("Failed to search_points from qdrant".to_string())
                    })?
                    .result;
                reembed_points(
                    points,
                    embedding_model_name,
                    embedding_base_url,
                    embedding_size,
                )
                .await
            }
        };

        match result {
            Ok(()) => {
                log::info!(
                    "Succesfully Migrated {} Points",
                    migration_message.qdrant_point_ids.len()
                );
            }
            Err(e) => {
                log::error!(
                    "Error migrating points {:?} {:?}",
                    e,
                    serialized_message.clone()
                );
                redis::cmd("lpush")
                    .arg("collection_migration_error")
                    .arg(serialized_message);
            }
        }
    }
}

pub async fn migrate_bm25(
    qdrant_client: Qdrant,
    points: Vec<RetrievedPoint>,
    to_collection: String,
    average_len: f32,
    b: f32,
    k: f32,
) -> Result<(), ServiceError> {
    // Insert points into new collection
    let new_points = points
        .iter()
        .map(|point| {
            let content = match point.payload.get("content") {
                Some(qdrant::Value {
                    kind: Some(qdrant::value::Kind::StringValue(content)),
                }) => content.clone(),
                _ => {
                    unreachable!()
                }
            };

            // calculate bm25
            let bm25_embeddings = get_bm25_embeddings(vec![(content, None)], average_len, b, k);

            let bm25_embedding = bm25_embeddings.first().expect("BM25 Vectors");

            let new_vectors = match &point.vectors {
                Some(qdrant::Vectors {
                    vectors_options:
                        Some(qdrant::vectors::VectorsOptions::Vectors(qdrant::NamedVectors {
                            vectors: vector_hash,
                        })),
                }) => {
                    let mut vectors_cloned = vector_hash.clone();

                    vectors_cloned.insert(
                        "bm25_vectors".to_string(),
                        qdrant::Vector::from(bm25_embedding.clone()),
                    );

                    vectors_cloned.into()
                }
                _ => {
                    unreachable!()
                }
            };

            qdrant::PointStruct {
                id: point.id.clone(),
                payload: point.payload.clone(),
                vectors: Some(new_vectors),
            }
        })
        .collect_vec();

    qdrant_client
        .upsert_points(UpsertPointsBuilder::new(to_collection, new_points))
        .await
        .map_err(|e| ServiceError::BadRequest(format!("Failed to upsert points {:?}", e)))?;

    Ok(())
}

#[allow(clippy::field_reassign_with_default)]
pub async fn reembed_points(
    points: Vec<RetrievedPoint>,
    embedding_model_name: String,
    embedding_base_url: String,
    embedding_size: usize,
) -> Result<(), ServiceError> {
    let mut mock_dataset_config = DatasetConfiguration::default();

    mock_dataset_config.EMBEDDING_MODEL_NAME = embedding_model_name;
    mock_dataset_config.EMBEDDING_BASE_URL = embedding_base_url;
    mock_dataset_config.EMBEDDING_SIZE = embedding_size;

    // Get all qdrant Ids and content
    let point_and_content = points
        .iter()
        .map(|point| {
            let content = match point.payload.get("content") {
                Some(qdrant::Value {
                    kind: Some(qdrant::value::Kind::StringValue(content)),
                }) => content.clone(),
                _ => {
                    unreachable!()
                }
            };

            (point, content)
        })
        .collect_vec();

    let reqwest_client = reqwest::Client::new();

    let splade_vectors = get_sparse_vectors(
        point_and_content
            .iter()
            .map(|(_, content)| (content.clone(), None))
            .collect(),
        "doc",
        reqwest_client.clone(),
    )
    .await?;

    // Embed content
    let embedding_vectors = get_dense_vectors(
        point_and_content
            .iter()
            .map(|(_, content)| (content.clone(), None))
            .collect(),
        "doc",
        mock_dataset_config.clone(),
        reqwest_client.clone(),
    )
    .await?;

    let bm25_vectors = get_bm25_embeddings(
        point_and_content
            .iter()
            .map(|(_, content)| (content.clone(), None))
            .collect(),
        0.0,
        0.0,
        0.0,
    );

    let new_points = izip!(
        points.clone().iter(),
        embedding_vectors.iter(),
        splade_vectors.iter(),
        bm25_vectors.iter()
    )
    .map(|(point, embedding_vector, splade_vector, bm25_vector)| {
        let vector_name = match embedding_vector.len() {
            384 => "384_vectors",
            512 => "512_vectors",
            768 => "768_vectors",
            1024 => "1024_vectors",
            3072 => "3072_vectors",
            1536 => "1536_vectors",
            _ => "768_vectors",
        };

        let vector_payload = HashMap::from([
            (
                "sparse_vectors".to_string(),
                Vector::from(splade_vector.clone()),
            ),
            (
                vector_name.to_string(),
                Vector::from(embedding_vector.clone()),
            ),
            (
                "bm25_vectors".to_string(),
                Vector::from(bm25_vector.clone()),
            ),
        ]);

        PointStruct::new(
            point
                .id
                .clone()
                .unwrap_or(PointId::from(uuid::Uuid::new_v4().to_string())),
            vector_payload,
            point.payload.clone(),
        )
    })
    .collect::<Vec<PointStruct>>();

    bulk_upsert_qdrant_points_query(new_points, mock_dataset_config).await?;

    Ok(())
}
