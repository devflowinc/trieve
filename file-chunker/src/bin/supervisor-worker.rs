use base64::Engine;
use chm::tools::migrations::{run_pending_migrations, SetupArgs};
use file_chunker::{
    errors::ServiceError,
    get_env,
    models::{self, FileTask, FileTaskStatus},
    operators::{clickhouse::update_task_status, redis::listen_to_redis, s3::get_aws_bucket},
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

    log::info!("Starting supervisor worker");

    process_task_with_retry!(
        redis_connection,
        &clickhouse_client.clone(),
        "files_to_process",
        |task| chunk_pdf(task, redis_connection.clone(), clickhouse_client.clone()),
        FileTask
    );
}

pub async fn chunk_pdf(
    task: FileTask,
    mut redis_connection: redis::aio::MultiplexedConnection,
    clickhouse_client: clickhouse::Client,
) -> Result<(), ServiceError> {
    let decoded_file_data = base64::prelude::BASE64_STANDARD
        .decode(task.upload_file_data.base64_file.clone())
        .map_err(|_e| ServiceError::BadRequest("Could not decode base64 file".to_string()))?;

    let doc = lopdf::Document::load_mem(decoded_file_data.as_slice())
        .map_err(|_e| ServiceError::BadRequest("Could not load pdf".to_string()))?;

    update_task_status(
        task.task_id,
        FileTaskStatus::ProcessingFile,
        &clickhouse_client,
    )
    .await?;

    let all_pages = doc.get_pages();
    let max_page_num = *all_pages.iter().last().unwrap().0;
    let pages_per_doc = 10;

    // Calculate how many documents we'll create
    let num_docs = (max_page_num as f64 / pages_per_doc as f64).ceil() as u32;

    for i in 0..num_docs {
        let mut new_doc = doc.clone();
        let start_page = i * pages_per_doc + 1;
        let end_page = std::cmp::min((i + 1) * pages_per_doc, max_page_num);

        // Create vector of pages to delete before start_page
        let mut pages_to_delete = (1..start_page).collect::<Vec<_>>();
        // Add pages after end_page
        pages_to_delete.extend((end_page + 1)..=max_page_num);

        // Delete pages outside our range
        if !pages_to_delete.is_empty() {
            new_doc.delete_pages(&pages_to_delete);
        }

        new_doc.prune_objects();
        new_doc.delete_zero_length_streams();
        new_doc.renumber_objects();
        new_doc.compress();

        let file_name = format!("{}_part_{}.pdf", task.task_id, i + 1);
        let mut buffer = Vec::new();
        new_doc
            .save_to(&mut buffer)
            .map_err(|_e| ServiceError::BadRequest("Could not save pdf to buffer".to_string()))?;

        let bucket = get_aws_bucket()?;
        bucket
            .put_object(file_name.clone(), buffer.as_slice())
            .await
            .map_err(|e| {
                log::error!("Could not upload file to S3 {:?}", e);
                ServiceError::BadRequest("Could not upload file to S3".to_string())
            })?;

        let task = models::ChunkingTask {
            task_id: task.task_id,
            file_name: file_name.clone(),
            attempt_number: 0,
        };

        let chunking_task = serde_json::to_string(&task).map_err(|_e| {
            ServiceError::BadRequest("Failed to serialize chunking task".to_string())
        })?;

        let pos_in_queue = redis::cmd("lpush")
            .arg("files_to_chunk")
            .arg(&chunking_task)
            .query_async::<String>(&mut redis_connection)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        log::info!("Added chunking task to queue: {:?}", pos_in_queue);
    }

    Ok(())
}
