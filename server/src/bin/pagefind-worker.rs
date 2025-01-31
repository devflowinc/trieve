use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use signal_hook::consts::SIGTERM;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use trieve_server::{
    data::models::{self, DatasetConfiguration, PagefindIndexWorkerMessage, WorkerEvent},
    errors::ServiceError,
    establish_connection, get_env,
    operators::{
        clickhouse_operator::{ClickHouseEvent, EventQueue},
        dataset_operator::get_dataset_by_id_query,
        pagefind_operator,
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

            let should_terminate = Arc::new(AtomicBool::new(false));
            signal_hook::flag::register(SIGTERM, Arc::clone(&should_terminate))
                .expect("Failed to register shutdown hook");

            pagefind_indexer(should_terminate, web_redis_pool, web_pool, web_event_queue).await
        });
}

async fn pagefind_indexer(
    should_terminate: Arc<AtomicBool>,
    redis_pool: actix_web::web::Data<models::RedisPool>,
    web_pool: actix_web::web::Data<models::Pool>,
    event_queue: actix_web::web::Data<EventQueue>,
) {
    log::info!("Starting pagefind worker service thread");

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
            .arg("pagefind-index-ingestion")
            .arg("pagefind-index-processing")
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

        let message: PagefindIndexWorkerMessage =
            serde_json::from_str(&serialized_message).expect("Failed to parse file message");

        match process_pagefind_index(message.clone(), web_pool.clone(), &event_queue).await {
            Ok(_) => {
                log::info!("Successfully processed pagefind index");
            }
            Err(err) => {
                log::error!("Failed to process pagefind index: {:?}", err);
                let _ = readd_error_to_queue(message, err, redis_pool.clone()).await;
            }
        };
    }
}

async fn process_pagefind_index(
    message: PagefindIndexWorkerMessage,
    web_pool: actix_web::web::Data<models::Pool>,
    event_queue: &actix_web::web::Data<EventQueue>,
) -> Result<(), ServiceError> {
    log::info!("Creating pagefind index for {:?}", message.dataset_id);

    event_queue
        .send(ClickHouseEvent::WorkerEvent(
            WorkerEvent::from_details(
                message.dataset_id,
                models::EventType::PagefindIndexingStarted,
            )
            .into(),
        ))
        .await;

    let dataset = get_dataset_by_id_query(message.dataset_id, web_pool.clone())
        .await
        .map_err(|err| {
            ServiceError::BadRequest(format!("Failed to get dataset from id {:?}", err))
        })?;

    let dataset_config = DatasetConfiguration::from_json(dataset.server_configuration.clone());

    pagefind_operator::build_index_for_dataset_id(
        message.dataset_id,
        dataset_config,
        web_pool,
        event_queue,
    )
    .await?;

    Ok(())
}

pub async fn readd_error_to_queue(
    mut payload: PagefindIndexWorkerMessage,
    error: ServiceError,
    redis_pool: actix_web::web::Data<models::RedisPool>,
) -> Result<(), ServiceError> {
    let old_payload_message = serde_json::to_string(&payload).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input for retry".to_string())
    })?;

    payload.attempt_number += 1;

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let _ = redis::cmd("LREM")
        .arg("pagefind-index-processing")
        .arg(1)
        .arg(old_payload_message.clone())
        .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
        .await;

    if payload.attempt_number == 3 {
        log::error!("Failed to insert data 3 times quitting {:?}", error);

        let mut redis_conn = redis_pool
            .get()
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        redis::cmd("lpush")
            .arg("dead_letters_pagefind-index")
            .arg(old_payload_message)
            .query_async::<redis::aio::MultiplexedConnection, ()>(&mut *redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        return Err(ServiceError::InternalServerError(format!(
            "Failed to create new qdrant point: {:?}",
            error
        )));
    }

    let new_payload_message = serde_json::to_string(&payload).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input for retry".to_string())
    })?;

    log::error!(
        "Failed to insert data, re-adding {:?} retry: {:?}",
        error,
        payload.attempt_number
    );

    redis::cmd("lpush")
        .arg("pagefind-index-ingestion")
        .arg(&new_payload_message)
        .query_async::<redis::aio::MultiplexedConnection, ()>(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}
