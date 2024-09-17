use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use sentry::{Hub, SentryFutureExt};
use signal_hook::consts::SIGTERM;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use actix_web::web;
use trieve_server::operators::{
    chunk_operator::create_chunk_metadata, crawl_operator::update_next_crawl_at,
};
use trieve_server::operators::{
    dataset_operator::get_dataset_by_id_query, user_operator::hash_function,
};
use trieve_server::{
    data::models::{CrawlRequest, DatasetConfiguration, RedisPool},
    operators::crawl_operator::{get_crawl_from_firecrawl, Status},
};
use trieve_server::{
    data::models::{CrawlStatus, Pool},
    errors::ServiceError,
    establish_connection, get_env,
    operators::crawl_operator::{get_chunk_html, get_images, get_tags, update_crawl_status},
};
use trieve_server::{
    handlers::chunk_handler::ChunkReqPayload, operators::crawl_operator::chunk_markdown,
};
use ureq::json;

async fn crawl(
    scrape_request: CrawlRequest,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<uuid::Uuid, ServiceError> {
    let ingest_result;
    loop {
        let temp_result = get_crawl_from_firecrawl(scrape_request.scrape_id)
            .await
            .map_err(|e| {
                log::error!("Error getting scrape request: {:?}", e);
                ServiceError::InternalServerError("Error getting scrape request".to_string())
            })?;
        if temp_result.status == Status::Completed {
            ingest_result = temp_result;
            break;
        } else if temp_result.status == Status::Scraping {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        } else if temp_result.status == Status::Failed {
            update_crawl_status(scrape_request.id, CrawlStatus::Failed, pool.clone())
                .await
                .map_err(|e| {
                    log::error!("Error updating crawl status: {:?}", e);
                    ServiceError::InternalServerError("Error updating crawl status".to_string())
                })
                .unwrap();

            return Err(ServiceError::InternalServerError(
                "Scrape failed".to_string(),
            ));
        }
    }

    update_crawl_status(
        scrape_request.id,
        CrawlStatus::GotResponseBackFromFirecrawl,
        pool.clone(),
    )
    .await
    .map_err(|e| {
        log::error!("Error updating crawl status: {:?}", e);
        ServiceError::InternalServerError("Error updating crawl status".to_string())
    })
    .unwrap();

    log::info!(
        "Got response back from firecrawl for scrape_id: {}",
        scrape_request.id
    );

    let mut chunks = vec![];

    let data = ingest_result.data.unwrap_or_default();

    log::info!("Processing {} chunks", data.len());

    for page in data {
        if page.is_none() {
            continue;
        }
        let page = page.unwrap();
        if page.metadata.status_code != Some(200) {
            log::error!("Error getting metadata for chunk: {:?}", page.metadata);
            update_crawl_status(scrape_request.id, CrawlStatus::Failed, pool.clone())
                .await
                .map_err(|e| {
                    log::error!("Error updating crawl status: {:?}", e);
                    ServiceError::InternalServerError("Error updating crawl status".to_string())
                })
                .unwrap();
        }

        let page_link = page.metadata.og_url.clone().unwrap_or_default();
        let page_title = page.metadata.og_title.clone().unwrap_or_default();
        let page_description = page.metadata.og_description.clone().unwrap_or_default();
        let page_markdown = page.markdown.clone().unwrap_or_default();
        let page_tags = get_tags(page_link.clone());

        let chunk_html = get_chunk_html(
            page_markdown.clone(),
            page_title.clone(),
            "".to_string(),
            0,
            None,
        );

        let chunked_markdown = chunk_markdown(&chunk_html.clone());

        for chunk in chunked_markdown {
            let chunk = ChunkReqPayload {
                chunk_html: Some(chunk.clone()),
                link: Some(page_link.clone()),
                tag_set: Some(page_tags.clone()),
                image_urls: Some(get_images(&chunk.clone())),
                metadata: Some(json!({
                    "title": page_title.clone(),
                    "description": page_description.clone(),
                    "url": page_link.clone(),
                })),
                tracking_id: Some(hash_function(&chunk.clone())),
                upsert_by_tracking_id: Some(true),
                ..Default::default()
            };
            chunks.push(chunk);
        }
    }

    let dataset = get_dataset_by_id_query(
        trieve_server::data::models::UnifiedId::TrieveUuid(scrape_request.dataset_id),
        pool.clone(),
    )
    .await
    .map_err(|e| {
        log::error!("Error getting dataset config: {:?}", e);
        ServiceError::InternalServerError("Error getting dataset config".to_string())
    })?;

    let dataset_config = DatasetConfiguration::from_json(dataset.server_configuration.clone());

    let (chunk_ingestion_message, chunk_metadatas) = create_chunk_metadata(
        chunks,
        scrape_request.dataset_id,
        dataset_config.clone(),
        pool.clone(),
    )
    .await?;

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    if !chunk_metadatas.is_empty() {
        let serialized_message: String =
            serde_json::to_string(&chunk_ingestion_message).map_err(|_| {
                ServiceError::BadRequest("Failed to Serialize BulkUploadMessage".to_string())
            })?;

        redis::cmd("lpush")
            .arg("ingestion")
            .arg(&serialized_message)
            .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    }

    update_crawl_status(
        scrape_request.scrape_id,
        CrawlStatus::Completed,
        pool.clone(),
    )
    .await?;

    update_next_crawl_at(
        scrape_request.scrape_id,
        scrape_request.next_crawl_at + scrape_request.interval,
        pool.clone(),
    )
    .await?;

    Ok(scrape_request.id)
}

async fn scrape_worker(
    should_terminate: Arc<AtomicBool>,
    redis_pool: web::Data<RedisPool>,
    pool: web::Data<Pool>,
) {
    log::info!("Starting scrape worker service thread");

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
            .arg("scrape_queue")
            .arg("scrape_processing")
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

        let processing_chunk_ctx = sentry::TransactionContext::new(
            "file worker processing file",
            "file worker processing file",
        );
        let transaction = sentry::start_transaction(processing_chunk_ctx);
        let crawl_request: CrawlRequest =
            serde_json::from_str(&serialized_message).expect("Failed to parse file message");

        update_crawl_status(crawl_request.scrape_id, CrawlStatus::Pending, pool.clone())
            .await
            .map_err(|e| {
                log::error!("Error updating crawl status: {:?}", e);
                ServiceError::InternalServerError("Error updating crawl status".to_string())
            })
            .unwrap();

        match crawl(crawl_request.clone(), pool.clone(), redis_pool.clone()).await {
            Ok(scrape_id) => {
                log::info!("Scrape job completed: {:?}", scrape_id);

                update_crawl_status(scrape_id, CrawlStatus::Completed, pool.clone())
                    .await
                    .map_err(|e| {
                        log::error!("Error updating crawl status: {:?}", e);
                        ServiceError::InternalServerError("Error updating crawl status".to_string())
                    })
                    .unwrap();

                let _ = redis::cmd("LREM")
                    .arg("scrape_processing")
                    .arg(1)
                    .arg(serialized_message)
                    .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_connection)
                    .await;
            }
            Err(err) => {
                log::error!("Failed to scrape website: {:?}", err);

                let _ = readd_error_to_queue(crawl_request, err, redis_pool.clone()).await;
            }
        };

        transaction.finish();
    }
}

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

                scrape_worker(should_terminate, web_redis_pool, web_pool).await
            }
            .bind_hub(Hub::new_from_top(Hub::current())),
        );
}

#[tracing::instrument(skip(redis_pool))]
pub async fn readd_error_to_queue(
    mut payload: CrawlRequest,
    error: ServiceError,
    redis_pool: actix_web::web::Data<RedisPool>,
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
        .arg("scrape_processing")
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
            .arg("dead_letters_scrape")
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

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    log::error!(
        "Failed to insert data, re-adding {:?} retry: {:?}",
        error,
        payload.attempt_number
    );

    redis::cmd("lpush")
        .arg("scrape_queue")
        .arg(&new_payload_message)
        .query_async::<redis::aio::MultiplexedConnection, ()>(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}
