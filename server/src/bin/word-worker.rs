#![allow(clippy::print_stdout)]
use actix_web::web;
use chm::tools::migrations::SetupArgs;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use futures::future::join_all;
use itertools::Itertools;
use signal_hook::consts::SIGTERM;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use trieve_server::{
    data::models,
    errors::ServiceError,
    establish_connection, get_env,
    operators::{
        chunk_operator::get_chunk_html_from_ids_query,
        dataset_operator::add_words_to_dataset,
        parse_operator::convert_html_to_text,
        typo_operator::{CreateBkTreeMessage, ProcessWordsFromDatasetMessage},
    },
};

#[allow(clippy::print_stdout)]
fn main() -> Result<(), ServiceError> {
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

            let redis_manager = bb8_redis::RedisConnectionManager::new(redis_url)
                .expect("Failed to connect to redis");

            let redis_pool = bb8_redis::bb8::Pool::builder()
                .connection_timeout(std::time::Duration::from_secs(2))
                .build(redis_manager)
                .await
                .expect("Failed to create redis pool");

            let web_redis_pool = actix_web::web::Data::new(redis_pool);

            let args = SetupArgs {
                url: Some(get_env!("CLICKHOUSE_URL", "CLICKHOUSE_URL is not set").to_string()),
                user: Some(get_env!("CLICKHOUSE_USER", "CLICKHOUSE_USER is not set").to_string()),
                password: Some(
                    get_env!("CLICKHOUSE_PASSWORD", "CLICKHOUSE_PASSWORD is not set").to_string(),
                ),
                database: Some(get_env!("CLICKHOUSE_DB", "CLICKHOUSE_DB is not set").to_string()),
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
            word_worker(
                should_terminate,
                web_redis_pool,
                web_pool,
                clickhouse_client,
            )
            .await
        });

    Ok(())
}

async fn word_worker(
    should_terminate: Arc<AtomicBool>,
    redis_pool: actix_web::web::Data<models::RedisPool>,
    web_pool: actix_web::web::Data<models::Pool>,
    clickhouse_client: clickhouse::Client,
) {
    log::info!("Starting word worker service thread");
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
            .arg("create_dictionary")
            .arg("process_dictionary")
            .arg(1.0)
            .query_async(&mut *redis_connection)
            .await;

        let serialized_msg = match payload_result {
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

        let msg: ProcessWordsFromDatasetMessage = match serde_json::from_str(&serialized_msg) {
            Ok(message) => message,
            Err(err) => {
                log::error!(
                    "Failed to deserialize message, was not an IngestionMessage: {:?}",
                    err
                );
                continue;
            }
        };

        match process_chunks(
            msg.clone(),
            web_pool.clone(),
            redis_pool.clone(),
            clickhouse_client.clone(),
        )
        .await
        {
            Ok(()) => {
                log::info!("Processing {} chunks", msg.chunks_to_process.len());
            }
            Err(err) => {
                log::error!("Failed to process dataset: {:?}", err);
                let _ = readd_error_to_queue(msg.clone(), err, redis_pool.clone()).await;
            }
        }
    }
}

async fn process_chunks(
    message: ProcessWordsFromDatasetMessage,
    pool: web::Data<models::Pool>,
    redis_pool: web::Data<models::RedisPool>,
    clickhouse_client: clickhouse::Client,
) -> Result<(), ServiceError> {
    let mut word_count_map: HashMap<(uuid::Uuid, String), i32> = HashMap::new();
    if let Some(chunks) = get_chunk_html_from_ids_query(
        message
            .chunks_to_process
            .clone()
            .into_iter()
            .map(|x| x.0)
            .collect(),
        pool.clone(),
    )
    .await?
    {
        let chunks = chunks
            .into_iter()
            // add dataset_id back to chunks
            .zip(message.chunks_to_process.clone().into_iter().map(|x| x.1))
            .collect_vec();

        for ((_, chunk), dataset_id) in &chunks {
            let content = convert_html_to_text(chunk);
            for word in content
                .split([' ', '\n', '\t', '\r', ',', '.', ';', ':', '!', '?'].as_ref())
                .filter(|word| !word.is_empty())
            {
                let word = word
                    .replace(|c: char| !c.is_alphabetic(), "")
                    .to_lowercase()
                    .chars()
                    .take(50)
                    .join("");
                if let Some(count) = word_count_map.get_mut(&(*dataset_id, word.clone())) {
                    *count += 1;
                } else {
                    word_count_map.insert((*dataset_id, word), 1);
                }
            }
        }
    }

    let (dataset_id_word, counts): (Vec<_>, Vec<_>) = word_count_map
        .into_iter()
        .sorted_by_key(|((_, word), _)| word.clone())
        .unzip();

    let words_and_counts = dataset_id_word
        .into_iter()
        .zip(counts.into_iter())
        .dedup_by(|((_, word1), _), ((_, word2), _)| word1 == word2)
        .collect_vec();

    let word_dataset_relation_futs = words_and_counts
        .chunks(5000)
        .map(|ids_counts| {
            let words = ids_counts.iter().map(|((_, w), _)| w.clone()).collect_vec();
            let dataset_ids = ids_counts
                .iter()
                .map(|((d, _), _)| d.to_owned())
                .collect_vec();
            let counts = ids_counts
                .iter()
                .map(|((_, _), c)| c.to_owned())
                .collect_vec();
            add_words_to_dataset(words, counts, dataset_ids, &clickhouse_client)
        })
        .collect_vec();

    join_all(word_dataset_relation_futs)
        .await
        .into_iter()
        .collect::<Result<Vec<()>, ServiceError>>()?;

    let serialized_payload = serde_json::to_string(&message).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input".to_string())
    })?;

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let _ = redis::cmd("LREM")
        .arg("process_dictionary")
        .arg(1)
        .arg(serialized_payload)
        .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
        .await;

    let create_tree_msgs = words_and_counts
        .iter()
        .map(|((dataset_id, _), _)| *dataset_id)
        .unique()
        .map(|id| {
            let msg = CreateBkTreeMessage {
                dataset_id: id,
                attempt_number: 0,
            };

            serde_json::to_string(&msg).map_err(|_| {
                ServiceError::InternalServerError("Failed to serialize message".to_string())
            })
        })
        .collect::<Result<Vec<String>, ServiceError>>()?;

    redis::cmd("SADD")
        .arg("bktree_creation")
        .arg(create_tree_msgs)
        .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
        .await
        .map_err(|_| {
            ServiceError::InternalServerError("Failed to send message to redis".to_string())
        })?;

    Ok(())
}

pub async fn readd_error_to_queue(
    mut message: ProcessWordsFromDatasetMessage,
    error: ServiceError,
    redis_pool: actix_web::web::Data<models::RedisPool>,
) -> Result<(), ServiceError> {
    let old_payload_message = serde_json::to_string(&message).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input for retry".to_string())
    })?;

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let _ = redis::cmd("lrem")
        .arg("process_dictionary")
        .arg(1)
        .arg(old_payload_message.clone())
        .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
        .await;

    message.attempt_number += 1;

    if message.attempt_number == 3 {
        log::error!("Failed to process dataset 3 times: {:?}", error);
        redis::cmd("lpush")
            .arg("dictionary_dead_letters")
            .arg(old_payload_message)
            .query_async::<redis::aio::MultiplexedConnection, ()>(&mut *redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
        return Err(ServiceError::InternalServerError(format!(
            "Failed to create new qdrant point: {:?}",
            error
        )));
    }

    let new_payload_message = serde_json::to_string(&message).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input for retry".to_string())
    })?;

    redis::cmd("lpush")
        .arg("create_dictionary")
        .arg(&new_payload_message)
        .query_async::<redis::aio::MultiplexedConnection, ()>(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}
