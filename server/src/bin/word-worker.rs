#![allow(clippy::print_stdout)]
use actix_web::web;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use futures::future::join_all;
use itertools::Itertools;
use sentry::{Hub, SentryFutureExt};
use signal_hook::consts::SIGTERM;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tracing_subscriber::{prelude::*, EnvFilter, Layer};
use trieve_server::{
    data::models::{self, WordInDataset},
    errors::ServiceError,
    establish_connection, get_env,
    operators::{
        chunk_operator::scroll_chunk_metadatas_query,
        dataset_operator::{add_words_to_dataset, update_dataset_last_processed_query},
        parse_operator::convert_html_to_text,
        words_operator::{create_words_query, CreateBkTreeMessage, ProcessWordsFromDatasetMessage},
    },
};

#[allow(clippy::print_stdout)]
fn main() -> Result<(), ServiceError> {
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
        .max_size(3)
        .build()
        .expect("Failed to create diesel_async pool");

    let web_pool = actix_web::web::Data::new(pool.clone());
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
                word_worker(should_terminate, web_redis_pool, web_pool).await
            }
            .bind_hub(Hub::new_from_top(Hub::current())),
        );

    Ok(())
}

async fn word_worker(
    should_terminate: Arc<AtomicBool>,
    redis_pool: actix_web::web::Data<models::RedisPool>,
    web_pool: actix_web::web::Data<models::Pool>,
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
                log::error!("Unable to process {:?}", err);

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

        match process_dataset(msg.clone(), web_pool.clone(), redis_pool.clone()).await {
            Ok(()) => {
                log::info!("Processed dataset: {}", msg.dataset_id);
            }
            Err(err) => {
                log::error!("Failed to process dataset: {:?}", err);
                let _ = readd_error_to_queue(msg.clone(), err, redis_pool.clone()).await;
            }
        }
    }
}

async fn process_dataset(
    message: ProcessWordsFromDatasetMessage,
    pool: web::Data<models::Pool>,
    redis_pool: web::Data<models::RedisPool>,
) -> Result<(), ServiceError> {
    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    let mut chunk_id_offset = uuid::Uuid::nil();
    let mut word_count_map: HashMap<String, i32> = HashMap::new();
    println!("Processing dataset: {}", message.dataset_id);
    while let Some(chunks) =
        scroll_chunk_metadatas_query(message.dataset_id, chunk_id_offset, 1000, pool.clone())
            .await?
    {
        println!("working on {:?}", chunk_id_offset);
        if let Some(last_chunk) = chunks.last() {
            chunk_id_offset = last_chunk.0;
        }

        for (_, chunk) in &chunks {
            let content = convert_html_to_text(chunk);
            for word in content
                .split([' ', '\n', '\t', '\r', ',', '.', ';', ':', '!', '?'].as_ref())
                .filter(|word| !word.is_empty())
            {
                let word = word
                    .replace(|c: char| !c.is_alphabetic(), "")
                    .to_lowercase();

                if let Some(count) = word_count_map.get_mut(&word) {
                    *count += 1;
                } else {
                    word_count_map.insert(word.chars().take(50).join(""), 1);
                }
            }
        }
    }

    let (words, word_counts): (Vec<_>, Vec<_>) = word_count_map.clone().into_iter().unzip();

    let word_ids_futs = words
        .chunks(10000)
        .map(|words| {
            create_words_query(
                words
                    .iter()
                    .map(|word| WordInDataset::from_word(word.to_string()))
                    .collect_vec(),
                pool.clone(),
            )
        })
        .collect_vec();

    let word_ids_and_counts = join_all(word_ids_futs)
        .await
        .into_iter()
        .collect::<Result<Vec<Vec<WordInDataset>>, ServiceError>>()?
        .into_iter()
        .flatten()
        .zip(word_counts)
        .collect_vec();

    let word_dataset_relation_futs = word_ids_and_counts
        .chunks(5000)
        .map(|ids_counts| {
            let (ids, counts): (Vec<WordInDataset>, Vec<i32>) =
                ids_counts.iter().cloned().collect_vec().into_iter().unzip();
            add_words_to_dataset(ids, counts, message.dataset_id, pool.clone())
        })
        .collect_vec();

    join_all(word_dataset_relation_futs)
        .await
        .into_iter()
        .collect::<Result<Vec<()>, ServiceError>>()?;

    update_dataset_last_processed_query(message.dataset_id, pool.clone()).await?;
    let create_tree_msg = CreateBkTreeMessage {
        dataset_id: message.dataset_id,
        attempt_number: 0,
    };

    let serialized_payload = serde_json::to_string(&message).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input".to_string())
    })?;

    let _ = redis::cmd("LREM")
        .arg("process_dictionary")
        .arg(1)
        .arg(serialized_payload)
        .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
        .await;

    let serialized_msg = serde_json::to_string(&create_tree_msg).map_err(|_| {
        ServiceError::InternalServerError("Failed to serialize message".to_string())
    })?;

    redis::cmd("LPUSH")
        .arg("bktree_creation")
        .arg(serialized_msg)
        .query_async(&mut *redis_conn)
        .await
        .map_err(|_| {
            ServiceError::InternalServerError("Failed to send message to redis".to_string())
        })?;
    Ok(())
}

#[tracing::instrument(skip(redis_pool))]
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

    let _ = redis::cmd("LREM")
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
            .query_async(&mut *redis_conn)
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
        .query_async(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}
