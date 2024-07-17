use itertools::Itertools;
#[allow(deprecated)]
use qdrant_client::{
    qdrant::{self, GetPointsBuilder, PointId, RetrievedPoint, UpsertPointsBuilder},
    Qdrant,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use trieve_server::{
    data::models::{MigratePointMessage, MigrationMode},
    errors::ServiceError,
    get_env,
    operators::{model_operator::get_bm25_embeddings, qdrant_operator::get_qdrant_connection},
};

#[allow(clippy::print_stdout)]
#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    dotenvy::dotenv().ok();
    tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::fmt::layer().with_filter(
                EnvFilter::from_default_env()
                    .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
            ),
        )
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

    let web_redis_pool = actix_web::web::Data::new(redis_pool);
    let mut connection = web_redis_pool
        .get()
        .await
        .map_err(|e| ServiceError::BadRequest(format!("Failed to connect to redis {}", e)))?;

    let mut broken_pipe_sleep = std::time::Duration::from_secs(10);

    loop {
        let payload_result: Result<Vec<String>, redis::RedisError> = redis::cmd("brpop")
            .arg("collection_migration")
            .arg(1)
            .query_async(&mut *connection)
            .await;

        let serialized_message = match payload_result {
            Ok(payload) => {
                broken_pipe_sleep = std::time::Duration::from_secs(10);

                if payload.is_empty() {
                    log::info!("wait");
                    continue;
                }

                payload
                    .get(1)
                    .expect("Payload must have a first element")
                    .clone()
            }
            Err(err) => {
                log::error!("Unable to process {:?}", err);

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

        let result = match migration_message.mode {
            MigrationMode::BM25 { average_len, k, b } => {
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
                    .arg("dead_letters")
                    .arg(serialized_message);
            }
        }
    }
}

#[tracing::instrument(skip(qdrant_client, points))]
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
