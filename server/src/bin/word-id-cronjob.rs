use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use tracing_subscriber::{prelude::*, EnvFilter, Layer};
use trieve_server::{
    errors::ServiceError,
    establish_connection, get_env,
    operators::{
        dataset_operator::scroll_dataset_ids_for_dictionary_query,
        words_operator::ProcessWordsFromDatasetMessage,
    },
};

#[allow(clippy::print_stdout)]
#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    dotenvy::dotenv().ok();
    log::info!("Starting id worker service thread");
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

    let web_redis_pool = actix_web::web::Data::new(redis_pool);

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

    if let Some(dataset_ids) = scroll_dataset_ids_for_dictionary_query(pool.clone()).await? {
        for dataset_id in &dataset_ids {
            let mut redis_conn = web_redis_pool
                .get()
                .await
                .expect("Failed to get redis connection");

            let process_words_msg = ProcessWordsFromDatasetMessage {
                dataset_id: *dataset_id,
                attempt_number: 0,
            };

            let serialized_msg = serde_json::to_string(&process_words_msg).map_err(|_| {
                ServiceError::InternalServerError("Failed to serialize message".to_string())
            })?;

            let _ = redis::cmd("LPUSH")
                .arg("create_dictionary")
                .arg(serialized_msg)
                .query_async::<redis::aio::MultiplexedConnection, bool>(&mut *redis_conn)
                .await;
        }
        log::info!("Scrolled {} datasets", dataset_ids.len());
    }

    Ok(())
}
