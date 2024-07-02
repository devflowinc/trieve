use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use sentry::{Hub, SentryFutureExt};
use signal_hook::consts::SIGTERM;
use std::collections::HashMap;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Arc};
use tracing_subscriber::{prelude::*, EnvFilter, Layer};
use trieve_server::data::models::{self, ChunkData, Event, PGInsertQueueMessage};
use trieve_server::errors::ServiceError;
use trieve_server::operators::chunk_operator::bulk_insert_chunk_metadata_query;
use trieve_server::operators::event_operator::create_event_query;
use trieve_server::operators::qdrant_operator::delete_points_from_qdrant;
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

    let database_url = get_env!("DATABASE_URL", "DATABASE_URL is not set");

    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);

    let mgr = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new_with_config(
        database_url,
        config,
    );

    let pool = diesel_async::pooled_connection::deadpool::Pool::builder(mgr)
        .max_size(10)
        .build()
        .expect("Failed to create diesel_async pool");

    let web_pool = actix_web::web::Data::new(pool.clone());

    let clickhouse_client = if std::env::var("USE_ANALYTICS")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false)
    {
        log::info!("Analytics enabled");

        clickhouse::Client::default()
            .with_url(
                std::env::var("CLICKHOUSE_URL").unwrap_or("http://localhost:8123".to_string()),
            )
            .with_user(std::env::var("CLICKHOUSE_USER").unwrap_or("default".to_string()))
            .with_password(std::env::var("CLICKHOUSE_PASSWORD").unwrap_or("".to_string()))
            .with_database(std::env::var("CLICKHOUSE_DATABASE").unwrap_or("default".to_string()))
            .with_option("async_insert", "1")
            .with_option("wait_for_async_insert", "0")
    } else {
        log::info!("Analytics disabled");
        clickhouse::Client::default()
    };

    let web_clickhouse_client = actix_web::web::Data::new(clickhouse_client);

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime")
        .block_on(
            async move {
                let redis_url = get_env!("REDIS_URL", "REDIS_URL is not set");
                let redis_connections: u32 = std::env::var("REDIS_CONNECTIONS")
                    .unwrap_or("2".to_string())
                    .parse()
                    .unwrap_or(2);

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

                pg_insert_worker(
                    should_terminate,
                    web_redis_pool,
                    web_pool,
                    web_clickhouse_client,
                )
                .await
            }
            .bind_hub(Hub::new_from_top(Hub::current())),
        );
}

#[tracing::instrument(skip(should_terminate, web_pool, redis_pool, clickhouse_client))]
async fn pg_insert_worker(
    should_terminate: Arc<AtomicBool>,
    redis_pool: actix_web::web::Data<models::RedisPool>,
    web_pool: actix_web::web::Data<models::Pool>,
    clickhouse_client: actix_web::web::Data<clickhouse::Client>,
) {
    log::info!("Starting pg insert service thread");

    let mut redis_conn_sleep = std::time::Duration::from_secs(1);

    #[allow(unused_assignments)]
    let mut opt_redis_connection = None;

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

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let payload_result: Result<Vec<String>, redis::RedisError> = redis::cmd("rpop")
            .arg("bulk_pg_queue")
            .arg(1000)
            .query_async(&mut *redis_connection)
            .await;

        let serialized_message = if let Ok(payload) = payload_result {
            broken_pipe_sleep = std::time::Duration::from_secs(10);

            if payload.is_empty() {
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

        redis::cmd("lpush")
            .arg("bulk_pg_processing")
            .arg(serialized_message.clone())
            .query_async::<redis::aio::MultiplexedConnection, ()>(&mut *redis_connection)
            .await
            .map_err(|err| {
                log::error!("Failed to push to processing queue: {:?}", err);
            })
            .ok();

        let processing_chunk_ctx = sentry::TransactionContext::new(
            "ingestion worker processing chunk",
            "ingestion worker processing chunk",
        );
        let transaction = sentry::start_transaction(processing_chunk_ctx);

        let messages: Vec<PGInsertQueueMessage> = serialized_message
            .iter()
            .filter_map(|message| serde_json::from_str(message).ok())
            .collect();

        match upload_bulk_pg_chunk(messages.clone(), web_pool.clone()).await {
            Ok(dataset_chunk_ids) => {
                for (dataset_id, chunk_ids) in dataset_chunk_ids.iter() {
                    log::info!(
                        "Uploaded {:} chunks for dataset {:?}",
                        chunk_ids.len(),
                        dataset_id
                    );

                    let _ = create_event_query(
                        Event::from_details(
                            *dataset_id,
                            models::EventType::ChunksUploaded {
                                chunk_ids: chunk_ids.to_vec(),
                            },
                        ),
                        clickhouse_client.clone(),
                    )
                    .await
                    .map_err(|err| {
                        log::error!("Failed to create event: {:?}", err);
                    });
                }

                for serialized_message in serialized_message.iter() {
                    let _ = redis::cmd("LREM")
                        .arg("bulk_pg_processing")
                        .arg(1)
                        .arg(serialized_message)
                        .query_async::<redis::aio::MultiplexedConnection, usize>(
                            &mut *redis_connection,
                        )
                        .await;
                }
            }
            Err(err) => {
                let qdrant_point_ids = messages
                    .iter()
                    .map(|message| {
                        message
                            .chunk_metadatas
                            .chunk_metadata
                            .qdrant_point_id
                            .unwrap_or_default()
                    })
                    .collect::<Vec<uuid::Uuid>>();

                let _ = delete_points_from_qdrant(
                    qdrant_point_ids,
                    messages.first().unwrap().dataset_config.clone(),
                )
                .await
                .map_err(|err| {
                    log::error!("Failed to delete points from qdrant: {:?}", err);
                });

                log::error!("Failed to upload bulk pg chunk: {:?}", err);
            }
        }

        transaction.finish();
    }
}

#[tracing::instrument(skip(payload, web_pool))]
async fn upload_bulk_pg_chunk(
    payload: Vec<PGInsertQueueMessage>,
    web_pool: actix_web::web::Data<models::Pool>,
) -> Result<HashMap<uuid::Uuid, Vec<uuid::Uuid>>, ServiceError> {
    let tx_ctx = sentry::TransactionContext::new(
        "ingestion worker upload_chunk",
        "ingestion worker upload_chunk",
    );
    let transaction = sentry::start_transaction(tx_ctx);
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone().into())));

    let mut chunk_data_map: HashMap<uuid::Uuid, Vec<ChunkData>> = HashMap::new();

    for payload in payload {
        chunk_data_map
            .entry(payload.dataset_id)
            .or_insert_with(Vec::new)
            .push(payload.chunk_metadatas.clone());
    }

    let mut dataset_chunk_ids: HashMap<uuid::Uuid, Vec<uuid::Uuid>> = HashMap::new();
    for (dataset_id, chunk_data) in chunk_data_map.iter() {
        match bulk_insert_chunk_metadata_query(
            chunk_data.clone(),
            dataset_id.clone(),
            web_pool.clone(),
        )
        .await
        {
            Ok(chunks) => {
                let ids = chunks
                    .iter()
                    .map(|chunk| chunk.chunk_metadata.id)
                    .collect::<Vec<uuid::Uuid>>();
                dataset_chunk_ids.insert(dataset_id.clone(), ids);
            }
            Err(err) => {
                log::error!("Failed to insert chunk metadata: {:?}", err);
                return Err(err);
            }
        }
    }
    Ok(dataset_chunk_ids)
}
