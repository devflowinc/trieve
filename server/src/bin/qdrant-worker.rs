use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use qdrant_client::qdrant::{PointStruct, Vector};
use sentry::{Hub, SentryFutureExt};
use signal_hook::consts::SIGTERM;
use std::collections::HashMap;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};
use trieve_server::data::models::{self, Event};
use trieve_server::errors::ServiceError;
use trieve_server::operators::event_operator::create_event_query;
use trieve_server::operators::qdrant_operator::get_qdrant_connection;
use trieve_server::{establish_connection, get_env};

fn main() {
    dotenvy::dotenv().ok();
    let sentry_url = std::env::var("SENTRY_URL");
    let _guard = if let Ok(sentry_url) = sentry_url {
        let guard = sentry::init((
            sentry_url,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                traces_sample_rate: 1.0,
                ..Default::default()
            },
        ));

        tracing_subscriber::Registry::default()
            .with(sentry::integrations::tracing::layer())
            .with(
                tracing_subscriber::fmt::layer().with_filter(
                    EnvFilter::from_default_env()
                        .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
                ),
            )
            .init();

        log::info!("Sentry monitoring enabled");
        Some(guard)
    } else {
        tracing_subscriber::Registry::default()
            .with(
                tracing_subscriber::fmt::layer().with_filter(
                    EnvFilter::from_default_env()
                        .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
                ),
            )
            .init();

        None
    };

    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);

    let database_url = get_env!("DATABASE_URL", "DATABASE_URL is not set");

    let mgr = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new_with_config(
        database_url,
        config,
    );

    let pool = diesel_async::pooled_connection::deadpool::Pool::builder(mgr)
        .max_size(10)
        .build()
        .expect("Failed to create diesel_async pool");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime")
        .block_on(
            async move {
                let redis_url = get_env!("REDIS_URL", "REDIS_URL is not set");
                let redis_connections: u32 = std::env::var("REDIS_CONNECTIONS")
                    .unwrap_or("30".to_string())
                    .parse()
                    .unwrap_or(30);

                let redis_manager = bb8_redis::RedisConnectionManager::new(redis_url)
                    .expect("Failed to connect to redis");

                let redis_pool = bb8_redis::bb8::Pool::builder()
                    .max_size(redis_connections)
                    .connection_timeout(std::time::Duration::from_secs(2))
                    .build(redis_manager)
                    .await
                    .expect("Failed to create redis pool");

                let web_redis_pool = actix_web::web::Data::new(redis_pool);

                let should_terminate = Arc::new(AtomicBool::new(false));
                signal_hook::flag::register(SIGTERM, Arc::clone(&should_terminate))
                    .expect("Failed to register shutdown hook");

                let web_redis_pool = web_redis_pool.clone();
                let web_pool = actix_web::web::Data::new(pool.clone());

                qdrant_ingestion_worker(should_terminate, web_redis_pool, web_pool).await;
                log::info!("Shutdown signal received, killing all children...");
            }
            .bind_hub(Hub::new_from_top(Hub::current())),
        );
}

async fn qdrant_ingestion_worker(
    should_terminate: Arc<AtomicBool>,
    redis_pool: actix_web::web::Data<models::RedisPool>,
    web_pool: actix_web::web::Data<models::Pool>,
) {
    log::info!("Starting qdrant service thread");

    let mut redis_conn_sleep = std::time::Duration::from_secs(1);

    #[allow(unused_assignments)]
    let mut opt_redis_connection = None;

    let qdrant_api_key = get_env!("QDRANT_API_KEY", "QDRANT_API_KEY must be present");
    let batch_size = get_env!("BATCH_SIZE", "BATCH_SIZE must be present");
    let qdrant_url = get_env!("QDRANT_URL", "QDRANT_URL must be present");
    let qdrant_collection = get_env!("QDRANT_COLLECTION", "QDRANT_COLLECTION must be present");

    let qdrant_client = get_qdrant_connection(Some(qdrant_url), Some(qdrant_api_key))
        .await
        .expect("Could not connect to qdrant, exiting");

    let redis_key = format!("{};{}", qdrant_url, qdrant_collection);

    loop {
        let borrowed_redis_connection = match redis_pool.get().await {
            Ok(redis_connection) => Some(redis_connection),
            Err(err) => {
                log::error!("Failed to get redis connection outside of loop: {:?}", err);
                None
            }
        };

        if borrowed_redis_connection.is_some() {
            opt_redis_connection = borrowed_redis_connection;
            break;
        }

        tokio::time::sleep(redis_conn_sleep).await;
        redis_conn_sleep = std::cmp::min(redis_conn_sleep * 2, std::time::Duration::from_secs(300));
    }

    let mut redis_connection =
        opt_redis_connection.expect("Failed to get redis connection outside of loop");

    let mut broken_pipe_sleep = std::time::Duration::from_secs(10);

    loop {
        if should_terminate.load(Ordering::Relaxed) {
            log::info!("Shutting down");
            break;
        }

        let payload_result: Result<Vec<String>, redis::RedisError> = redis::cmd("RPOP")
            .arg(&redis_key)
            .arg(batch_size)
            .query_async(&mut *redis_connection)
            .await;

        let serialized_messages = if let Ok(payload) = payload_result {
            broken_pipe_sleep = std::time::Duration::from_secs(10);

            if payload.is_empty() {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                continue;
            }

            payload
        } else {
            log::error!("Unable to process {:?}", payload_result);

            if payload_result.is_err_and(|err| err.is_io_error()) {
                tokio::time::sleep(broken_pipe_sleep).await;
                broken_pipe_sleep =
                    std::cmp::min(broken_pipe_sleep * 2, std::time::Duration::from_secs(300));
            }

            continue;
        };

        let processing_chunk_ctx = sentry::TransactionContext::new(
            "qdrant worker bulkprocessing chunks",
            "qdrant worker bulkprocessing chunks",
        );
        let transaction = sentry::start_transaction(processing_chunk_ctx);
        let qdrant_ingestion_messages: Vec<models::QdrantMessage> = serialized_messages
            .iter()
            .filter_map(|e| serde_json::from_str::<models::QdrantMessage>(e).ok())
            .collect();

        let point_structs = qdrant_ingestion_messages
            .iter()
            .filter_map(|message| {
                let embedding_vector = message.point_struct_data.clone().dense_vector;
                let splade_vector = message.point_struct_data.clone().splade_vector;

                let vector_name = match embedding_vector.len() {
                    384 => "384_vectors",
                    512 => "512_vectors",
                    768 => "768_vectors",
                    1024 => "1024_vectors",
                    3072 => "3072_vectors",
                    1536 => "1536_vectors",
                    _ => return None,
                };

                let vector_payload = HashMap::from([
                    (vector_name.to_string(), Vector::from(embedding_vector)),
                    ("sparse_vectors".to_string(), Vector::from(splade_vector)),
                ]);

                Some(PointStruct::new(
                    message.point_struct_data.point_id.clone().to_string(),
                    vector_payload,
                    message
                        .clone()
                        .point_struct_data
                        .payload
                        .try_into()
                        .expect("a json! Value must always be a valid Payload"),
                ))
            })
            .collect();

        let upsert_result = qdrant_client
            .upsert_points_blocking(qdrant_collection, None, point_structs, None)
            .await
            .map_err(|err| {
                sentry::capture_message(&format!("Error {:?}", err), sentry::Level::Error);
                log::error!("Failed inserting chunk to qdrant {:?}", err);
                ServiceError::BadRequest(format!("Failed inserting chunk to qdrant {:?}", err))
            });

        transaction.finish();

        if let Err(error) = upsert_result {
            for message in qdrant_ingestion_messages {
                log::error!(
                    "Failed to upload to qdrant chunk_id {:} qdrant_point_id {:}",
                    message.chunk_id,
                    message.point_struct_data.point_id
                );
                let _ = create_event_query(
                    Event::from_details(
                        message.dataset_id,
                        models::EventType::QdrantUploadFailed {
                            chunk_id: message.chunk_id,
                            qdrant_point_id: message.point_struct_data.point_id,
                            error: error.to_string(),
                        },
                    ),
                    web_pool.clone(),
                )
                .await;
            }

            let _ : Result<(), ServiceError> = redis::cmd("lpush")
                .arg(&format!("{:}-failed", redis_key))
                .arg(serialized_messages)
                .query_async(&mut *redis_connection)
                .await
                .map_err(|err| ServiceError::BadRequest(err.to_string()));
        } else {
            log::info!("Bulk inserted {} points", qdrant_ingestion_messages.len());
        }
    }
}
