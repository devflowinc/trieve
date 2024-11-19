use base64::Engine;
use chm::tools::migrations::{run_pending_migrations, SetupArgs};
use lopdf::{Document, Object, ObjectId};
use pdf2md_server::{
    errors::ServiceError,
    get_env,
    models::{self, FileTask, FileTaskStatus},
    operators::{clickhouse::update_task_status, redis::listen_to_redis, s3::get_aws_bucket},
    process_task_with_retry,
};
use signal_hook::consts::SIGTERM;
use std::{
    collections::BTreeMap,
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
    let estimated_size = (task.upload_file_data.base64_file.len() * 3) / 4;
    let mut decoded_file_data = Vec::with_capacity(estimated_size);
    base64::prelude::BASE64_STANDARD
        .decode_vec(
            task.upload_file_data.base64_file.as_bytes(),
            &mut decoded_file_data,
        )
        .map_err(|_e| ServiceError::BadRequest("Could not decode base64 file".to_string()))?;

    let doc = lopdf::Document::load_mem(&decoded_file_data)
        .map_err(|e| ServiceError::BadRequest(format!("Could not load pdf: {}", e)))?;

    let all_pages = doc.get_pages();
    let max_page_num = *all_pages.keys().last().unwrap();
    let pages_per_doc = 10;
    let num_docs = (max_page_num as f64 / pages_per_doc as f64).ceil() as u32;

    let bucket = get_aws_bucket()?;
    let mut buffer = Vec::new();

    // Process each chunk
    for i in 0..num_docs {
        let start_page = i * pages_per_doc + 1;
        let end_page = std::cmp::min((i + 1) * pages_per_doc, max_page_num);

        // Split the document
        let mut split_doc = split_pdf(doc.clone(), start_page, end_page)
            .map_err(|e| ServiceError::BadRequest(format!("Failed to split PDF: {}", e)))?;

        // Clear and reuse buffer
        buffer.clear();

        // Save to reused buffer
        split_doc
            .save_to(&mut buffer)
            .map_err(|_e| ServiceError::BadRequest("Could not save pdf to buffer".to_string()))?;

        let file_name = format!("{}part{}.pdf", task.task_id, i + 1);
        bucket
            .put_object(file_name.clone(), buffer.as_slice())
            .await
            .map_err(|e| {
                log::error!("Could not upload file to S3 {:?}", e);
                ServiceError::BadRequest("Could not upload file to S3".to_string())
            })?;

        let chunking_task = serde_json::to_string(&models::ChunkingTask {
            task_id: task.task_id,
            file_name,
            page_range: (start_page, end_page),
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

        log::info!("Uploaded part {} of {} to S3", i + 1, num_docs);
    }

    update_task_status(
        task.task_id,
        FileTaskStatus::ProcessingFile(max_page_num),
        &clickhouse_client,
    )
    .await?;

    Ok(())
}

pub fn split_pdf(doc: Document, start_page: u32, end_page: u32) -> Result<Document, String> {
    let mut new_document = Document::with_version(doc.version.clone());
    let page_numbers_to_keep: Vec<u32> = (start_page..=end_page).collect();

    // Get mapping of page numbers to object IDs
    let page_map = doc.get_pages();

    // Collect only the pages we want to keep
    let mut documents_pages = BTreeMap::new();
    let mut documents_objects = BTreeMap::new();

    // Filter and collect pages we want to keep
    for page_num in page_numbers_to_keep {
        if let Some(&object_id) = page_map.get(&page_num) {
            if let Ok(page_object) = doc.get_object(object_id) {
                documents_pages.insert(object_id, page_object.clone());
            }
        }
    }

    // Collect all objects from original document
    documents_objects.extend(doc.objects.clone());

    // "Catalog" and "Pages" are mandatory
    let mut catalog_object: Option<(ObjectId, Object)> = None;
    let mut pages_object: Option<(ObjectId, Object)> = None;

    // Process all objects except "Page" type
    for (object_id, object) in documents_objects.iter() {
        match object.type_name().unwrap_or("") {
            "Catalog" => {
                catalog_object = Some((
                    if let Some((id, _)) = catalog_object {
                        id
                    } else {
                        *object_id
                    },
                    object.clone(),
                ));
            }
            "Pages" => {
                if let Ok(dictionary) = object.as_dict() {
                    pages_object = Some((
                        if let Some((id, _)) = pages_object {
                            id
                        } else {
                            *object_id
                        },
                        Object::Dictionary(dictionary.clone()),
                    ));
                }
            }
            "Page" => {} // Handled separately
            _ => {
                // Copy other necessary objects (resources, fonts, etc.)
                new_document.objects.insert(*object_id, object.clone());
            }
        }
    }

    // If no "Pages" object found, abort
    let pages_object = pages_object.ok_or_else(|| "Pages root not found".to_string())?;
    let catalog_object = catalog_object.ok_or_else(|| "Catalog root not found".to_string())?;

    // Add pages to new document
    for (object_id, object) in documents_pages.iter() {
        if let Ok(dictionary) = object.as_dict() {
            let mut dictionary = dictionary.clone();
            dictionary.set("Parent", pages_object.0);
            new_document
                .objects
                .insert(*object_id, Object::Dictionary(dictionary));
        }
    }

    // Build new "Pages" object
    if let Ok(dictionary) = pages_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Count", documents_pages.len() as u32);
        dictionary.set(
            "Kids",
            documents_pages
                .into_keys()
                .map(Object::Reference)
                .collect::<Vec<_>>(),
        );
        new_document
            .objects
            .insert(pages_object.0, Object::Dictionary(dictionary));
    }

    // Build new "Catalog" object
    if let Ok(dictionary) = catalog_object.1.as_dict() {
        let mut dictionary = dictionary.clone();
        dictionary.set("Pages", pages_object.0);
        dictionary.remove(b"Outlines"); // Remove outlines as we're splitting
        new_document
            .objects
            .insert(catalog_object.0, Object::Dictionary(dictionary));
    }

    // Set up trailer and document structure
    new_document.trailer.set("Root", catalog_object.0);
    new_document.max_id = new_document.objects.len() as u32;
    new_document.renumber_objects();
    new_document.compress();

    Ok(new_document)
}
