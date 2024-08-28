use chm::tools::migrations::SetupArgs;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use futures::future::join_all;
use itertools::Itertools;
use tracing_subscriber::{prelude::*, EnvFilter, Layer};
use trieve_server::{
    errors::ServiceError,
    establish_connection, get_env,
    operators::{
        chunk_operator::{
            get_last_processed_from_clickhouse, scroll_chunk_ids_for_dictionary_query,
        },
        dataset_operator::get_all_dataset_ids,
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

    let dataset_ids = get_all_dataset_ids(pool.clone()).await?;
    let dataset_ids_and_processed = dataset_ids
        .into_iter()
        .map(|dataset_id| {
            let clickhouse_client = clickhouse_client.clone();
            async move {
                (
                    dataset_id,
                    get_last_processed_from_clickhouse(&clickhouse_client, dataset_id).await,
                )
            }
        })
        .collect_vec();

    let dataset_ids_and_processed = join_all(dataset_ids_and_processed).await;

    for (dataset_id, last_processed) in dataset_ids_and_processed {
        let mut chunk_id_offset = uuid::Uuid::nil();

        let last_processed = last_processed.map_err(|_| {
            ServiceError::InternalServerError(
                "Failed to get last processed from clickhouse".to_string(),
            )
        })?;
        while let Some(chunk_id_dataset_id_list) = scroll_chunk_ids_for_dictionary_query(
            pool.clone(),
            dataset_id,
            last_processed.clone(),
            10000,
            chunk_id_offset,
        )
        .await?
        {
            if let Some((chunk_id, _)) = chunk_id_dataset_id_list.last() {
                chunk_id_offset = *chunk_id
            }
            let redis_futures =
                chunk_id_dataset_id_list
                    .chunks(500)
                    .map(|chunk_id_dataset_id_list| {
                        let pool = redis_pool.clone();
                        async move {
                            let mut redis_conn = pool
                                .get()
                                .await
                                .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
                            let process_words_msg = ProcessWordsFromDatasetMessage {
                                chunks_to_process: chunk_id_dataset_id_list.to_vec(),
                                attempt_number: 0,
                            };

                            match serde_json::to_string(&process_words_msg).map_err(|_| {
                                ServiceError::InternalServerError(
                                    "Failed to serialize message".to_string(),
                                )
                            }) {
                                Ok(serialized_msg) => redis::cmd("LPUSH")
                                    .arg("create_dictionary")
                                    .arg(serialized_msg)
                                    .query_async::<redis::aio::MultiplexedConnection, bool>(
                                        &mut *redis_conn,
                                    )
                                    .await
                                    .map_err(|_| {
                                        ServiceError::InternalServerError(
                                            "Failed to send message to redis".to_string(),
                                        )
                                    }),
                                Err(err) => Err(err),
                            }
                        }
                    });

            let _ = join_all(redis_futures)
                .await
                .into_iter()
                .collect::<Result<Vec<bool>, ServiceError>>()?;
            log::info!("Scrolled {} chunks", chunk_id_dataset_id_list.len());
        }
    }

    Ok(())
}
