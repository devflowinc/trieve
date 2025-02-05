use base64::Engine;
use chm::tools::migrations::SetupArgs;
use pdf2image::PDF;
use pdf2md_server::{
    errors::ServiceError,
    get_env,
    models::{self, FileTask, FileTaskStatus},
    operators::{clickhouse::update_task_status, redis::listen_to_redis, s3::get_aws_bucket},
    process_task_with_retry,
};
use signal_hook::consts::SIGTERM;
use std::{
    io::Cursor,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
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
    let bucket = get_aws_bucket()?;

    let estimated_size = (task.upload_file_data.base64_file.len() * 3) / 4;
    let mut decoded_file_data = Vec::with_capacity(estimated_size);
    base64::prelude::BASE64_STANDARD
        .decode_vec(
            task.upload_file_data.base64_file.as_bytes(),
            &mut decoded_file_data,
        )
        .map_err(|_e| ServiceError::BadRequest("Could not decode base64 file".to_string()))?;

    bucket
        .put_object(format!("{}.pdf", task.id), decoded_file_data.as_slice())
        .await
        .map_err(|e| {
            log::error!("Could not upload file to S3 {:?}", e);
            ServiceError::BadRequest("Could not upload file to S3".to_string())
        })?;

    let pdf = PDF::from_bytes(decoded_file_data)
        .map_err(|err| ServiceError::BadRequest(format!("Failed to open PDF file {:?}", err)))?;

    let pages = pdf
        .render(pdf2image::Pages::All, None)
        .map_err(|err| ServiceError::BadRequest(format!("Failed to render PDF file {:?}", err)))?
        .into_iter()
        .skip(1);

    let num_pages = pdf.page_count();

    update_task_status(
        task.id,
        FileTaskStatus::ProcessingFile(num_pages),
        &clickhouse_client,
    )
    .await?;

    // Process each chunk
    for (i, page) in pages.enumerate() {
        let file_name = format!("{}page{}.jpeg", task.id, i + 1);
        let mut buffer = Vec::new();
        page.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Jpeg)
            .map_err(|err| {
                ServiceError::BadRequest(format!("Failed to render PDF file {:?}", err))
            })?;
        bucket
            .put_object(file_name.clone(), &buffer)
            .await
            .map_err(|e| {
                log::error!("Could not upload file to S3 {:?}", e);
                ServiceError::BadRequest("Could not upload file to S3".to_string())
            })?;

        let chunking_task = serde_json::to_string(&models::ChunkingTask {
            id: task.id,
            file_name,
            page_num: (i + 1) as u32,
            params: task.upload_file_data.clone().into(),
            attempt_number: 0,
        })
        .map_err(|_e| ServiceError::BadRequest("Failed to serialize chunking task".to_string()))?;

        redis::cmd("lpush")
            .arg("files_to_chunk")
            .arg(&chunking_task)
            .query_async::<String>(&mut redis_connection)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        log::info!("Uploaded page {} of {} to S3", i + 1, num_pages);
    }

    Ok(())
}
