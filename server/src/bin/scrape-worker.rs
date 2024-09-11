use actix_web::middleware::Logger;
use actix_web::{post, web, App, HttpResponse, HttpServer};
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};
use trieve_server::data::models::{DatasetConfiguration, RedisPool};
use trieve_server::handlers::chunk_handler::ChunkReqPayload;
use trieve_server::operators::chunk_operator::create_chunk_metadata;
use trieve_server::operators::dataset_operator::get_dataset_by_id_query;
use trieve_server::{
    data::models::{CrawlStatus, Pool},
    errors::ServiceError,
    establish_connection, get_env,
    operators::crawl_operator::{
        get_chunk_html, get_crawl_request, get_images, get_tags, update_crawl_status, IngestResult,
    },
};
use ureq::json;

#[post("/")]
async fn chunk(
    req_body: web::Json<IngestResult>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let ingest_result = req_body.into_inner();
    if ingest_result.r#type != "crawl.page" {
        log::error!("Firecrawl sent a message that was not a crawl.page {:?}", ingest_result);
        return Ok(HttpResponse::NoContent().finish());
    }

    let scrape_request = get_crawl_request(ingest_result.id, pool.clone())
        .await
        .map_err(|e| {
            log::error!("Error getting crawl request: {:?}", e);
            ServiceError::InternalServerError("Error getting crawl request".to_string())
        })?;

    update_crawl_status(
        ingest_result.id,
        CrawlStatus::GotResponseBackFromFirecrawl,
        pool.clone(),
    )
    .await
    .map_err(|e| {
        log::error!("Error updating crawl status: {:?}", e);
        ServiceError::InternalServerError("Error updating crawl status".to_string())
    })?;

    log::info!(
        "Got response back from firecrawl for scrape_id: {}",
        ingest_result.id
    );

    let mut chunks = vec![];

    for page in ingest_result.data {
        if page.metadata.status_code != Some(200) {
            log::error!("Error getting metadata for chunk: {:?}", page.metadata);
            update_crawl_status(ingest_result.id, CrawlStatus::Failed, pool.clone())
                .await
                .map_err(|e| {
                    log::error!("Error updating crawl status: {:?}", e);
                    ServiceError::InternalServerError("Error updating crawl status".to_string())
                })?;
        }

        let page_link = page.metadata.og_url.clone().unwrap_or_default();
        let page_title = page.metadata.og_title.clone().unwrap_or_default();
        let page_description = page.metadata.og_description.clone().unwrap_or_default();
        let page_markdown = page.markdown.clone().unwrap_or_default();
        let page_tags = get_tags(page_link.clone());

        let chunk = ChunkReqPayload {
            chunk_html: Some(get_chunk_html(
                page_markdown.clone(),
                page_title.clone(),
                "".to_string(),
                0,
                None,
            )),
            link: Some(page_link.clone()),
            tag_set: Some(page_tags),
            image_urls: Some(get_images(&page_markdown.clone())),
            metadata: Some(json!({
                "title": page_title.clone(),
                "description": page_description.clone(),
                "url": page_link.clone(),
            })),
            ..Default::default()
        };
        chunks.push(chunk);
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
            .query_async(&mut *redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    }

    Ok(HttpResponse::NoContent().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let database_url = get_env!("DATABASE_URL", "DATABASE_URL should be set");
    let redis_url = get_env!("REDIS_URL", "REDIS_URL should be set");

    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);

    let mgr = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new_with_config(
        database_url,
        config,
    );

    let pool = diesel_async::pooled_connection::deadpool::Pool::builder(mgr)
        .max_size(10)
        .build()
        .unwrap();

    let redis_manager =
        bb8_redis::RedisConnectionManager::new(redis_url).expect("Failed to connect to redis");

    let redis_connections: u32 = std::env::var("REDIS_CONNECTIONS")
        .unwrap_or("2".to_string())
        .parse()
        .unwrap_or(2);

    let redis_pool = bb8_redis::bb8::Pool::builder()
        .max_size(redis_connections)
        .build(redis_manager)
        .await
        .expect("Failed to create redis pool");

    tracing_subscriber::Registry::default()
            .with(
                tracing_subscriber::fmt::layer().with_filter(
                    EnvFilter::from_default_env()
                        .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
                ),
            )
            .init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(redis_pool.clone()))
            .service(chunk)
    })
    .bind(("127.0.0.1", 54324))?
    .run()
    .await
}
