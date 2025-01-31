use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use redis::AsyncCommands;
use signal_hook::consts::SIGTERM;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use trieve_server::{
    data::models::{self, DatasetConfiguration},
    errors::ServiceError,
    establish_connection, get_env,
    operators::{
        chunk_operator::bulk_delete_chunks_query,
        clickhouse_operator::{ClickHouseEvent, EventQueue},
        dataset_operator::{
            clear_dataset_query, delete_dataset_by_id_query, get_dataset_by_id_query,
            get_deleted_dataset_by_id_query, ChunkDeleteMessage, DatasetDeleteMessage,
            DeleteMessage,
        },
        organization_operator::{
            delete_actual_organization_query, get_soft_deleted_datasets_for_organization,
        },
    },
};

fn main() {
    dotenvy::dotenv().ok();
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();

    let database_url = get_env!("DATABASE_URL", "DATABASE_URL is not set");

    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);

    let mgr = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new_with_config(
        database_url,
        config,
    );

    let pool = diesel_async::pooled_connection::deadpool::Pool::builder(mgr)
        .max_size(3)
        .build()
        .expect("Failed to create diesel_async pool");

    let web_pool = actix_web::web::Data::new(pool.clone());

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime")
        .block_on(async move {
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

            let event_queue = if std::env::var("USE_ANALYTICS")
                .unwrap_or("false".to_string())
                .parse()
                .unwrap_or(false)
            {
                log::info!("Analytics enabled");

                let clickhouse_client = clickhouse::Client::default()
                    .with_url(
                        std::env::var("CLICKHOUSE_URL")
                            .unwrap_or("http://localhost:8123".to_string()),
                    )
                    .with_user(std::env::var("CLICKHOUSE_USER").unwrap_or("default".to_string()))
                    .with_password(std::env::var("CLICKHOUSE_PASSWORD").unwrap_or("".to_string()))
                    .with_database(
                        std::env::var("CLICKHOUSE_DATABASE").unwrap_or("default".to_string()),
                    )
                    .with_option("async_insert", "1")
                    .with_option("wait_for_async_insert", "0");

                let mut event_queue = EventQueue::new(clickhouse_client.clone());
                event_queue.start_service();
                event_queue
            } else {
                log::info!("Analytics disabled");
                EventQueue::default()
            };

            let web_event_queue = actix_web::web::Data::new(event_queue);

            delete_worker(should_terminate, web_redis_pool, web_pool, web_event_queue).await
        });
}

async fn delete_worker(
    should_terminate: Arc<AtomicBool>,
    redis_pool: actix_web::web::Data<models::RedisPool>,
    web_pool: actix_web::web::Data<models::Pool>,
    event_queue: actix_web::web::Data<EventQueue>,
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

        log::info!(
            "Retrying to get redis connection out of loop after {:?} secs",
            redis_conn_sleep
        );
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

                tokio::time::sleep(broken_pipe_sleep).await;
                broken_pipe_sleep =
                    std::cmp::min(broken_pipe_sleep * 2, std::time::Duration::from_secs(300));
            }

            continue;
        };

        let delete_worker_message: DeleteMessage =
            serde_json::from_str(&serialized_message).expect("Failed to parse file message");

        match delete_worker_message {
            DeleteMessage::DatasetDelete(delete_worker_message) => {
                if let Err(err) = delete_or_clear_dataset(
                    web_pool.clone(),
                    redis_pool.clone(),
                    delete_worker_message.clone(),
                    event_queue.clone(),
                )
                .await
                {
                    let _ = readd_error_to_queue(
                        DeleteMessage::DatasetDelete(delete_worker_message),
                        err,
                        event_queue.clone(),
                        redis_pool.clone(),
                    )
                    .await;
                } else {
                    let _ = redis::cmd("LREM")
                        .arg("delete_dataset_processing")
                        .arg(1)
                        .arg(serialized_message)
                        .query_async::<redis::aio::MultiplexedConnection, usize>(
                            &mut *redis_connection,
                        )
                        .await;
                }
            }
            DeleteMessage::ChunkDelete(chunk_delete_message) => {
                if let Err(err) =
                    bulk_delete_chunks(web_pool.clone(), chunk_delete_message.clone()).await
                {
                    let _ = readd_error_to_queue(
                        DeleteMessage::ChunkDelete(chunk_delete_message),
                        err,
                        event_queue.clone(),
                        redis_pool.clone(),
                    )
                    .await;
                } else {
                    let _ = redis::cmd("LREM")
                        .arg("delete_dataset_processing")
                        .arg(1)
                        .arg(serialized_message)
                        .query_async::<redis::aio::MultiplexedConnection, usize>(
                            &mut *redis_connection,
                        )
                        .await;
                }
            }
        }
    }
}

pub async fn delete_or_clear_dataset(
    web_pool: actix_web::web::Data<models::Pool>,
    redis_pool: actix_web::web::Data<models::RedisPool>,
    delete_worker_message: DatasetDeleteMessage,
    event_queue: actix_web::web::Data<EventQueue>,
) -> Result<(), ServiceError> {
    let dataset =
        get_deleted_dataset_by_id_query(delete_worker_message.dataset_id, web_pool.clone())
            .await
            .map_err(|err| ServiceError::BadRequest(format!("Failed to get dataset: {:?}", err)))?;

    let mut redis_connection = redis_pool.get().await.map_err(|err| {
        ServiceError::BadRequest(format!("Failed to get redis connection: {:?}", err))
    })?;

    let dataset_config = DatasetConfiguration::from_json(dataset.server_configuration);

    if delete_worker_message.empty_dataset {
        log::info!("Clearing dataset {:?}", delete_worker_message.dataset_id);

        if dataset_config.QDRANT_ONLY {
            bulk_delete_chunks_query(
                None,
                delete_worker_message.deleted_at,
                delete_worker_message.dataset_id,
                dataset_config.clone(),
                web_pool.clone(),
            )
            .await
            .map_err(|err| {
                log::error!("Failed to bulk delete chunks: {:?}", err);
                err
            })?;

            log::info!(
                "Bulk deleted chunks for dataset: {:?}",
                delete_worker_message.dataset_id
            );

            return Ok(());
        }

        clear_dataset_query(
            delete_worker_message.dataset_id,
            delete_worker_message.deleted_at,
            web_pool.clone(),
            event_queue.clone(),
            dataset_config.clone(),
        )
        .await
        .map_err(|err| {
            log::error!("Failed to clear dataset: {:?}", err);
            err
        })?;

        log::info!(
            "Cleared all chunks for dataset: {:?}",
            delete_worker_message.dataset_id
        );

        return Ok(());
    }

    log::info!("Deleting dataset {:?}", delete_worker_message.dataset_id);

    let dataset = delete_dataset_by_id_query(
        delete_worker_message.dataset_id,
        delete_worker_message.deleted_at,
        web_pool.clone(),
        event_queue.clone(),
        dataset_config.clone(),
    )
    .await
    .map_err(|err| {
        log::error!("Failed to delete dataset: {:?}", err);
        err
    })?;

    log::info!("Deleted Dataset: {:?}", delete_worker_message.dataset_id);

    if redis_connection
        .sismember("deleted_organizations", dataset.organization_id.to_string())
        .await
        .unwrap_or(false)
    {
        let datasets =
            get_soft_deleted_datasets_for_organization(dataset.organization_id, web_pool.clone())
                .await
                .map_err(|err| {
                    log::error!("Failed to get datasets for organization: {:?}", err);
                    err
                })?;

        if datasets.is_empty() {
            delete_actual_organization_query(dataset.organization_id, web_pool.clone())
                .await
                .map_err(|err| {
                    log::error!("Failed to delete organization: {:?}", err);
                    err
                })?;

            log::info!("Deleted Organization: {:?}", dataset.organization_id);

            let _ = redis_connection
                .srem::<&str, std::string::String, usize>(
                    "deleted_organizations",
                    dataset.organization_id.to_string(),
                )
                .await
                .map_err(|err| {
                    log::error!(
                        "Failed to remove organization from deleted organizations: {:?}",
                        err
                    )
                });
            return Ok(());
        }
    }

    Ok(())
}

pub async fn bulk_delete_chunks(
    web_pool: actix_web::web::Data<models::Pool>,
    chunk_delete_message: ChunkDeleteMessage,
) -> Result<(), ServiceError> {
    log::info!(
        "Bulk deleting chunks for dataset: {:?}",
        chunk_delete_message.dataset_id
    );
    let dataset = get_dataset_by_id_query(chunk_delete_message.dataset_id, web_pool.clone())
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Failed to get dataset: {:?}", err)))?;
    let dataset_config = DatasetConfiguration::from_json(dataset.server_configuration);

    bulk_delete_chunks_query(
        Some(chunk_delete_message.filter),
        chunk_delete_message.deleted_at,
        chunk_delete_message.dataset_id,
        dataset_config,
        web_pool.clone(),
    )
    .await
    .map_err(|err| {
        log::error!("Failed to bulk delete chunks: {:?}", err);
        err
    })?;

    log::info!(
        "Bulk deleted chunks for dataset: {:?}",
        chunk_delete_message.dataset_id
    );

    Ok(())
}

pub async fn readd_error_to_queue(
    mut payload: DeleteMessage,
    error: ServiceError,
    event_queue: actix_web::web::Data<EventQueue>,
    redis_pool: actix_web::web::Data<models::RedisPool>,
) -> Result<(), ServiceError> {
    let old_payload_message = serde_json::to_string(&payload).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input for retry".to_string())
    })?;

    payload.increment_attempt_number();

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let _ = redis::cmd("LREM")
        .arg("delete_dataset_processing")
        .arg(1)
        .arg(old_payload_message.clone())
        .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
        .await;

    if payload.attempt_number() == 3 {
        log::error!("Failed to insert data 3 times quitting {:?}", error);

        let mut redis_conn = redis_pool
            .get()
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        redis::cmd("lpush")
            .arg("dead_letters_delete")
            .arg(old_payload_message)
            .query_async::<redis::aio::MultiplexedConnection, ()>(&mut *redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        event_queue
            .send(ClickHouseEvent::WorkerEvent(
                models::WorkerEvent::from_details(
                    payload.dataset_id(),
                    models::EventType::DatasetDeleteFailed {
                        error: error.to_string(),
                    },
                )
                .into(),
            ))
            .await;

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
        payload.attempt_number()
    );

    redis::cmd("lpush")
        .arg("delete_dataset_queue")
        .arg(&new_payload_message)
        .query_async::<redis::aio::MultiplexedConnection, ()>(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}
