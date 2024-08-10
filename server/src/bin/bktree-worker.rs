use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use chm::tools::migrations::SetupArgs;
use sentry::{Hub, SentryFutureExt};
use signal_hook::consts::SIGTERM;
use tracing_subscriber::{prelude::*, EnvFilter, Layer};
use trieve_server::{
    data::models::RedisPool,
    errors::ServiceError,
    get_env,
    operators::{
        dataset_operator::{scroll_words_from_dataset, update_dataset_last_processed_query},
        words_operator::{get_bktree_from_redis_query, BkTree, CreateBkTreeMessage},
    },
};

#[allow(clippy::print_stdout)]
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

    let should_terminate = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGTERM, Arc::clone(&should_terminate))
        .expect("Failed to register shutdown hook");

    tokio::runtime::Builder::new_current_thread()
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

                let args = SetupArgs {
                    url: Some(get_env!("CLICKHOUSE_URL", "CLICKHOUSE_URL is not set").to_string()),
                    user: Some(
                        get_env!("CLICKHOUSE_USER", "CLICKHOUSE_USER is not set").to_string(),
                    ),
                    password: Some(
                        get_env!("CLICKHOUSE_PASSWORD", "CLICKHOUSE_PASSWORD is not set")
                            .to_string(),
                    ),
                    database: Some(
                        get_env!("CLICKHOUSE_DB", "CLICKHOUSE_DB is not set").to_string(),
                    ),
                };

                let clickhouse_client = clickhouse::Client::default()
                    .with_url(args.url.as_ref().unwrap())
                    .with_user(args.user.as_ref().unwrap())
                    .with_password(args.password.as_ref().unwrap())
                    .with_database(args.database.as_ref().unwrap())
                    .with_option("async_insert", "1")
                    .with_option("wait_for_async_insert", "0");

                let should_terminate = Arc::new(AtomicBool::new(false));
                signal_hook::flag::register(SIGTERM, Arc::clone(&should_terminate))
                    .expect("Failed to register shutdown hook");

                bktree_worker(should_terminate, web_redis_pool, clickhouse_client).await
            }
            .bind_hub(Hub::new_from_top(Hub::current())),
        );
}

async fn bktree_worker(
    should_terminate: Arc<AtomicBool>,
    redis_pool: actix_web::web::Data<RedisPool>,
    clickhouse_client: clickhouse::Client,
) {
    log::info!("Starting bk tree service thread");

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
            .arg("bktree_creation")
            .arg("bktree_processing")
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

        let create_tree_msg: CreateBkTreeMessage = match serde_json::from_str(&serialized_message) {
            Ok(message) => message,
            Err(err) => {
                log::error!(
                    "Failed to deserialize message, was not a CreateBkTreeMessage: {:?}",
                    err
                );
                continue;
            }
        };

        let mut id_offset = uuid::Uuid::nil();
        log::info!("Processing dataset {}", create_tree_msg.dataset_id);

        match update_dataset_last_processed_query(create_tree_msg.dataset_id, &clickhouse_client)
            .await
        {
            Ok(_) => {}
            Err(err) => {
                log::error!("Failed to update last processed {:?}", err);
            }
        }

        let mut bk_tree = if let Ok(Some(bktree)) =
            get_bktree_from_redis_query(create_tree_msg.dataset_id, redis_pool.clone()).await
        {
            bktree
        } else {
            BkTree::new()
        };

        let mut failed = false;

        while let Ok(Some(word_and_counts)) = scroll_words_from_dataset(
            create_tree_msg.dataset_id,
            id_offset,
            1000,
            &clickhouse_client,
        )
        .await
        .map_err(|err| {
            let err = err.clone();
            let redis_pool = redis_pool.clone();
            let create_tree_msg = create_tree_msg.clone();
            tokio::spawn(async move {
                let _ = readd_error_to_queue(create_tree_msg.clone(), &err, redis_pool.clone())
                    .await
                    .map_err(|e| {
                        eprintln!("Failed to readd error to queue: {:?}", e);
                    });
            });
            failed = true;
        }) {
            if let Some(last_word) = word_and_counts.last() {
                id_offset = last_word.id;
            }

            let word_and_counts = word_and_counts
                .into_iter()
                .map(|words| (words.word, words.count))
                .collect::<Vec<(String, i32)>>();

            bk_tree.insert_all(word_and_counts);
        }

        if failed {
            continue;
        }

        match rmp_serde::to_vec(&bk_tree) {
            Ok(serialized_tree) => {
                match redis::cmd("SET")
                    .arg(format!("bk_tree_{}", create_tree_msg.dataset_id))
                    .arg(serialized_tree)
                    .query_async::<redis::aio::MultiplexedConnection, String>(
                        &mut *redis_connection,
                    )
                    .await
                {
                    Ok(_) => {
                        let _ = redis::cmd("LREM")
                            .arg("bktree_processing")
                            .arg(1)
                            .arg(serialized_message.clone())
                            .query_async::<redis::aio::MultiplexedConnection, usize>(
                                &mut *redis_connection,
                            )
                            .await;

                        log::info!(
                            "Succesfully created bk-tree for {}",
                            create_tree_msg.dataset_id
                        );
                    }
                    Err(err) => {
                        let _ = readd_error_to_queue(
                            create_tree_msg.clone(),
                            &ServiceError::InternalServerError(format!(
                                "Failed to serialize tree: {:?}",
                                err
                            )),
                            redis_pool.clone(),
                        )
                        .await;
                    }
                }
            }
            Err(err) => {
                let _ = readd_error_to_queue(
                    create_tree_msg.clone(),
                    &ServiceError::InternalServerError(format!(
                        "Failed to serialize tree: {:?}",
                        err
                    )),
                    redis_pool.clone(),
                )
                .await;
            }
        }
    }
}

pub async fn readd_error_to_queue(
    message: CreateBkTreeMessage,
    error: &ServiceError,
    redis_pool: actix_web::web::Data<RedisPool>,
) -> Result<(), ServiceError> {
    let mut message = message;

    let old_payload_message = serde_json::to_string(&message).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input for retry".to_string())
    })?;

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let _ = redis::cmd("LREM")
        .arg("bktree_processing")
        .arg(1)
        .arg(old_payload_message.clone())
        .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
        .await;

    message.attempt_number += 1;

    if message.attempt_number == 3 {
        log::error!("Failed to construct bktree 3 times {:?}", error);
        let mut redis_conn = redis_pool
            .get()
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        redis::cmd("lpush")
            .arg("bktree_dead_letters")
            .arg(old_payload_message)
            .query_async(&mut *redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        return Err(ServiceError::InternalServerError(format!(
            "Failed to construct bktree {:?}",
            error
        )));
    } else {
        let new_payload_message = serde_json::to_string(&message).map_err(|_| {
            ServiceError::InternalServerError("Failed to reserialize input for retry".to_string())
        })?;

        let mut redis_conn = redis_pool
            .get()
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        log::error!(
            "Failed to insert data, re-adding {:?} retry: {:?}",
            error,
            message.attempt_number
        );

        redis::cmd("lpush")
            .arg("bktree_creation")
            .arg(&new_payload_message)
            .query_async(&mut *redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?
    }

    Ok(())
}
