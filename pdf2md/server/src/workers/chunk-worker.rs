use chm::tools::migrations::{run_pending_migrations, SetupArgs};
use pdf2md_server::{
    errors::ServiceError,
    get_env,
    models::{ChunkingTask, RedisPool},
    operators::{pdf_chunk::chunk_sub_pages, redis::listen_to_redis, s3::get_aws_bucket},
    process_task_with_retry,
};
use signal_hook::consts::SIGTERM;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();

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

    let args = SetupArgs {
        url: Some(std::env::var("CLICKHOUSE_URL").unwrap_or("http://localhost:8123".to_string())),
        user: Some(std::env::var("CLICKHOUSE_USER").unwrap_or("default".to_string())),
        password: Some(std::env::var("CLICKHOUSE_PASSWORD").unwrap_or("password".to_string())),
        database: Some(std::env::var("CLICKHOUSE_DB").unwrap_or("default".to_string())),
    };

    let clickhouse_client = clickhouse::Client::default()
        .with_url(args.url.as_ref().unwrap())
        .with_user(args.user.as_ref().unwrap())
        .with_password(args.password.as_ref().unwrap())
        .with_database(args.database.as_ref().unwrap())
        .with_option("async_insert", "1")
        .with_option("wait_for_async_insert", "0");

    let _ = run_pending_migrations(args.clone()).await.map_err(|err| {
        log::error!("Failed to run clickhouse migrations: {:?}", err);
    });

    let should_terminate = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGTERM, Arc::clone(&should_terminate))
        .expect("Failed to register shutdown hook");

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

    let redis_connection =
        opt_redis_connection.expect("Failed to get redis connection outside of loop");

    log::info!("Starting chunking worker");

    process_task_with_retry!(
        redis_connection,
        &clickhouse_client.clone(),
        "files_to_chunk",
        |task| chunk_sub_pdf(task, clickhouse_client.clone(), redis_pool.clone()),
        ChunkingTask
    );
}

pub async fn chunk_sub_pdf(
    task: ChunkingTask,
    clickhouse_client: clickhouse::Client,
    redis_pool: RedisPool,
) -> Result<(), pdf2md_server::errors::ServiceError> {
    let bucket = get_aws_bucket()?;
    let file_data = bucket
        .get_object(task.file_name.clone())
        .await
        .map_err(|e| {
            log::error!("Could not get file from S3 {:?}", e);
            ServiceError::BadRequest("File is not present in s3".to_string())
        })?
        .as_slice()
        .to_vec();

    let result = chunk_sub_pages(file_data, task.clone(), &clickhouse_client, &redis_pool).await?;

    log::info!("Got {} pages for {:?}", result.len(), task.task_id);

    Ok(())
}
