use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use tracing_subscriber::{prelude::*, EnvFilter, Layer};
use trieve_server::{
    errors::ServiceError,
    establish_connection, get_env,
    operators::crawl_operator::{crawl_site, get_crawl_requests_to_rerun, update_scrape_id},
};

#[allow(clippy::print_stdout)]
#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    dotenvy::dotenv().ok();
    log::info!("Starting crawl worker service thread");
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

    let redis_url = get_env!("REDIS_URL", "REDIS_URL is not set");
    let redis_connections: u32 = std::env::var("REDIS_CONNECTIONS")
        .unwrap_or("2".to_string())
        .parse()
        .unwrap_or(2);

    let redis_manager =
        bb8_redis::RedisConnectionManager::new(redis_url).expect("Failed to connect to redis");

    let redis_pool = bb8_redis::bb8::Pool::builder()
        .max_size(redis_connections)
        .connection_timeout(std::time::Duration::from_secs(2))
        .build(redis_manager)
        .await
        .expect("Failed to create redis pool");

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

    let pool = actix_web::web::Data::new(pool.clone());

    let new_requests = get_crawl_requests_to_rerun(pool.clone()).await?;

    for request in new_requests {
        log::info!("Re-crawling site: {}", request.url);
        let new_scrape_id = crawl_site(request.url.clone())
            .await
            .expect("Failed to crawl site");

        let updated_request = update_scrape_id(request.scrape_id, new_scrape_id, pool.clone())
            .await
            .expect("Failed to update scrape id");

        let serialized_message = serde_json::to_string(&updated_request).unwrap();
        let mut redis_conn = redis_pool
            .get()
            .await
            .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

        redis::cmd("lpush")
            .arg("scrape_queue")
            .arg(&serialized_message)
            .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    }

    Ok(())
}
