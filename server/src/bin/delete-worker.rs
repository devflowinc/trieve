use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use redis::AsyncCommands;
use sentry::{Hub, SentryFutureExt};
use signal_hook::consts::SIGTERM;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use trieve_server::{
    data::models,
    errors::ServiceError,
    establish_connection, get_env,
    operators::{
        dataset_operator::{delete_chunks_in_dataset, delete_dataset_by_id_query, DeleteMessage},
        event_operator::create_event_query,
        organization_operator::{
            delete_actual_organization_query, get_soft_deleted_datasets_for_organization,
        },
    },
};

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

                delete_worker(
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

async fn delete_worker(
    should_terminate: Arc<AtomicBool>,
    redis_pool: actix_web::web::Data<models::RedisPool>,
    web_pool: actix_web::web::Data<models::Pool>,
    clickhouse_client: actix_web::web::Data<clickhouse::Client>,
) {
    log::info!("Starting delete worker service thread");

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

        let payload_result: Result<Vec<String>, redis::RedisError> = redis::cmd("brpoplpush")
            .arg("delete_dataset_queue")
            .arg("delete_dataset_processing")
            .arg(1.0)
            .query_async(&mut redis_connection.clone())
            .await;

        let serialized_message = if let Ok(payload) = payload_result {
            broken_pipe_sleep = std::time::Duration::from_secs(10);

            if payload.is_empty() {
                continue;
            }

            payload
                .first()
                .expect("Payload must have a first element")
                .clone()
        } else {
            log::error!("Unable to process {:?}", payload_result);

            if payload_result.is_err_and(|err| err.is_io_error()) {
                tokio::time::sleep(broken_pipe_sleep).await;
                broken_pipe_sleep =
                    std::cmp::min(broken_pipe_sleep * 2, std::time::Duration::from_secs(300));
            }

            continue;
        };

        let processing_chunk_ctx =
            sentry::TransactionContext::new("delete worker processing", "delete worker processing");
        let transaction = sentry::start_transaction(processing_chunk_ctx);
        let delete_worker_message: DeleteMessage =
            serde_json::from_str(&serialized_message).expect("Failed to parse file message");

        log::info!(
            "Deleting chunks for dataset {:?}",
            delete_worker_message.dataset_id
        );

        if delete_worker_message.empty_dataset {
            match delete_chunks_in_dataset(
                delete_worker_message.dataset_id,
                web_pool.clone(),
                clickhouse_client.clone(),
                delete_worker_message.server_config.clone(),
            )
            .await
            {
                Ok(_) => {
                    log::info!(
                        "Deleted all chunks for dataset: {:?}",
                        delete_worker_message.dataset_id
                    );
                    let _ = redis::cmd("LREM")
                        .arg("delete_dataset_processing")
                        .arg(1)
                        .arg(serialized_message)
                        .query_async::<redis::aio::MultiplexedConnection, usize>(
                            &mut *redis_connection,
                        )
                        .await;

                    continue;
                }
                Err(err) => {
                    log::error!("Failed to delete all chunks for dataset: {:?}", err);
                    let _ = create_event_query(
                        models::Event::from_details(
                            delete_worker_message.dataset_id,
                            models::EventType::DatasetDeleteFailed {
                                error: err.to_string(),
                            },
                        ),
                        clickhouse_client.clone(),
                    )
                    .await
                    .map_err(|err| {
                        log::error!("Failed to create event: {:?}", err);
                    });
                    let _ =
                        readd_error_to_queue(delete_worker_message, err, redis_pool.clone()).await;
                    continue;
                }
            }
        }

        match delete_dataset_by_id_query(
            delete_worker_message.dataset_id,
            web_pool.clone(),
            clickhouse_client.clone(),
            delete_worker_message.server_config.clone(),
        )
        .await
        {
            Ok(dataset) => {
                log::info!("Deleted Dataset: {:?}", delete_worker_message.dataset_id);

                let _ = redis::cmd("LREM")
                    .arg("delete_dataset_processing")
                    .arg(1)
                    .arg(serialized_message)
                    .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_connection)
                    .await;

                if redis_connection
                    .sismember("deleted_organizations", dataset.organization_id.to_string())
                    .await
                    .unwrap_or(false)
                {
                    match get_soft_deleted_datasets_for_organization(
                        dataset.organization_id,
                        web_pool.clone(),
                    )
                    .await
                    {
                        Ok(datasets) => {
                            if datasets.is_empty() {
                                match delete_actual_organization_query(
                                    dataset.organization_id,
                                    web_pool.clone(),
                                )
                                .await
                                {
                                    Ok(_) => {
                                        log::info!(
                                            "Deleted Organization: {:?}",
                                            dataset.organization_id
                                        );

                                        let _ = redis_connection
                                            .srem::<&str, std::string::String, usize>("deleted_organizations", dataset.organization_id.to_string())
                                            .await
                                            .map_err(|err| log::error!("Failed to remove organization from deleted organizations: {:?}", err));
                                    }
                                    Err(err) => {
                                        log::error!("Failed to delete organization: {:?}", err);
                                        continue;
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            log::error!("Failed to get datasets for organization: {:?}", err);
                            continue;
                        }
                    }
                }
            }
            Err(err) => {
                log::error!("Failed to delete dataset: {:?}", err);

                let _ = readd_error_to_queue(delete_worker_message, err, redis_pool.clone()).await;
            }
        };

        transaction.finish();
    }
}

#[tracing::instrument(skip(redis_pool))]
pub async fn readd_error_to_queue(
    mut payload: DeleteMessage,
    error: ServiceError,
    redis_pool: actix_web::web::Data<models::RedisPool>,
) -> Result<(), ServiceError> {
    let old_payload_message = serde_json::to_string(&payload).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input for retry".to_string())
    })?;

    payload.attempt_number += 1;

    if payload.attempt_number == 3 {
        log::error!("Failed to insert data 3 times quitting {:?}", error);
        return Err(ServiceError::InternalServerError(format!(
            "Failed to create new qdrant point: {:?}",
            error
        )));
    }

    let new_payload_message = serde_json::to_string(&payload).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input for retry".to_string())
    })?;

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    log::error!(
        "Failed to insert data, re-adding {:?} retry: {:?}",
        error,
        payload.attempt_number
    );

    let _ = redis::cmd("LREM")
        .arg("delete_dataset_processing")
        .arg(1)
        .arg(old_payload_message)
        .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
        .await;

    redis::cmd("lpush")
        .arg("delete_dataset_queue")
        .arg(&new_payload_message)
        .query_async(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}
