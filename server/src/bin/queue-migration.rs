use qdrant_client::qdrant::{Condition, Filter};
use trieve_server::{
    data::models::{MigratePointMessage, MigrationMode},
    errors::ServiceError,
    get_env,
    operators::qdrant_operator::scroll_qdrant_collection_ids,
};

#[allow(clippy::print_stdout)]
#[tokio::main]
async fn main() -> Result<(), ServiceError> {
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

    let web_redis_pool = actix_web::web::Data::new(redis_pool);

    let from_collection = get_env!("FROM_COLLECTION", "FROM_COLLECTION is not set");
    let to_collection = get_env!("TO_COLLECTION", "TO_COLLECTION is not set");
    let dataset_ids: Vec<&str> = get_env!("DATASET_IDS", "DATASET_IDS is not set")
        .split(',')
        .collect();

    let new_embedding_model_name = get_env!(
        "NEW_EMBEDDING_MODEL_NAME",
        "NEW_EMBEDDING_MODEL_NAME is not set"
    );
    let new_embedding_base_url =
        get_env!("NEW_EMBEDDING_BASE_URL", "NEWEMBEDDING_BASE_URL is not set");
    let new_embedding_size = get_env!("NEW_EMBEDDING_SIZE", "NEW_EMBEDDING_SIZE is not set")
        .parse()
        .unwrap_or(768);

    log::info!("queue'ing datasets: {:?}", dataset_ids);

    let mut offset = Some(uuid::Uuid::nil().to_string());

    while let Some(cur_offset) = offset.clone() {
        let (qdrant_point_ids, new_offset) = scroll_qdrant_collection_ids(
            from_collection.to_string(),
            Some(cur_offset.to_string()),
            Some(120),
            Some(Filter::any(dataset_ids.iter().map(|dataset_id| {
                Condition::matches("dataset_id", dataset_id.to_string())
            }))),
        )
        .await?;

        let mut conn = web_redis_pool
            .get()
            .await
            .expect("Failed to connect to redis");

        let message = serde_json::to_string(&MigratePointMessage {
            qdrant_point_ids: qdrant_point_ids.clone(),
            from_collection: from_collection.to_string(),
            to_collection: to_collection.to_string(),
            mode: MigrationMode::Reembed {
                embedding_model_name: new_embedding_model_name.to_string(),
                embedding_base_url: new_embedding_base_url.to_string(),
                embedding_size: new_embedding_size,
            },
        })
        .expect("Failed to serialze MigratePoint message");

        redis::cmd("lpush")
            .arg("collection_migration")
            .arg(&message)
            .query_async::<redis::aio::MultiplexedConnection, ()>(&mut *conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to send message to redis".to_string()))?;

        log::info!(
            "Migrated {:?} points between {:?} and {:?}",
            qdrant_point_ids.len(),
            offset.clone(),
            new_offset.clone()
        );
        offset = new_offset;
    }

    Ok(())
}
