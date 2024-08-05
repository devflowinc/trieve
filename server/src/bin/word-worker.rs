use actix_web::web;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use futures::future::join_all;
use itertools::Itertools;
use trieve_server::{
    data::models::{self, ChunkMetadataTable, WordInDataset},
    errors::ServiceError,
    establish_connection, get_env,
    operators::{
        chunk_operator::scroll_chunk_metadatas_query,
        dataset_operator::{add_words_to_dataset, scroll_dataset_ids_query},
        parse_operator::convert_html_to_text,
        words_operator::create_words_query,
    },
};

#[allow(clippy::print_stdout)]
#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    dotenvy::dotenv().ok();

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

    let mut dataset_offset = uuid::Uuid::nil();

    while let Some(dataset_ids) =
        scroll_dataset_ids_query(dataset_offset, 1000, pool.clone()).await?
    {
        println!("Processing {} datasets", dataset_ids.len());
        if let Some(last_dataset_id) = dataset_ids.last() {
            dataset_offset = *last_dataset_id;
        }
        for dataset_id in dataset_ids {
            println!("Processing dataset: {}", dataset_id);
            let mut chunk_id_offset = uuid::Uuid::nil();
            while let Some(chunks) =
                scroll_chunk_metadatas_query(dataset_id, chunk_id_offset, 1000, pool.clone())
                    .await?
            {
                if let Some(last_chunk) = chunks.last() {
                    chunk_id_offset = last_chunk.id;
                }
                push_words_to_redis(dataset_id, chunks, web_redis_pool.clone()).await?;
            }
        }
    }

    pull_words_and_datasets_from_redis(web_redis_pool.clone(), pool.clone()).await?;

    Ok(())
}

async fn push_words_to_redis(
    dataset_id: uuid::Uuid,
    chunks: Vec<ChunkMetadataTable>,
    redis_pool: web::Data<models::RedisPool>,
) -> Result<(), ServiceError> {
    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let _ = redis::cmd("SADD")
        .arg(format!("word_datasets_{}", dataset_id))
        .arg(
            chunks
                .into_iter()
                .filter_map(|chunk| {
                    chunk.chunk_html.map(|html| {
                        convert_html_to_text(&html)
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect_vec()
                    })
                })
                .flatten()
                .unique()
                .collect::<Vec<String>>(),
        )
        .query_async::<redis::aio::MultiplexedConnection, usize>(&mut *redis_conn)
        .await
        .map_err(|_| ServiceError::InternalServerError("Failed to add words to set".to_string()))?;

    Ok(())
}

async fn pull_words_and_datasets_from_redis(
    redis_pool: web::Data<models::RedisPool>,
    pool: web::Data<models::Pool>,
) -> Result<(), ServiceError> {
    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let word_datasets_names = redis::cmd("KEYS")
        .arg("word_datasets*")
        .query_async::<redis::aio::MultiplexedConnection, Vec<String>>(&mut *redis_conn)
        .await
        .map_err(|_| {
            ServiceError::InternalServerError("Failed to get word set names".to_string())
        })?;

    let mut pipeline = redis::pipe();

    for name in &word_datasets_names {
        pipeline.cmd("SCARD").arg(name.clone());
    }

    let word_dataset_counts = pipeline
        .query_async::<redis::aio::MultiplexedConnection, Vec<usize>>(&mut *redis_conn)
        .await
        .map_err(|_| ServiceError::InternalServerError("Failed to get word set names".to_string()))?
        .into_iter()
        .zip(word_datasets_names)
        .collect_vec();

    for (count, dataset) in word_dataset_counts {
        let words = redis::cmd("SPOP")
            .arg(dataset.clone())
            .arg(count)
            .query_async::<redis::aio::MultiplexedConnection, Vec<String>>(&mut *redis_conn)
            .await
            .map_err(|_| {
                ServiceError::InternalServerError("Failed to get word set names".to_string())
            })?;

        let word_ids = create_words_query(
            words
                .into_iter()
                .map(WordInDataset::from_word)
                .collect_vec(),
            pool.clone(),
        )
        .await?;

        let dataset_id = dataset
            .strip_prefix("word_datasets_")
            .expect("Datset ID must be present");

        let dataset_id = uuid::Uuid::parse_str(dataset_id).map_err(|_| {
            ServiceError::InternalServerError("Failed to parse dataset id".to_string())
        })?;

        let add_words_futs = word_ids
            .chunks(10000)
            .map(|word_ids| add_words_to_dataset(word_ids.to_vec(), dataset_id, pool.clone()))
            .collect_vec();

        join_all(add_words_futs)
            .await
            .into_iter()
            .collect::<Result<Vec<()>, ServiceError>>()?;
    }

    Ok(())
}
