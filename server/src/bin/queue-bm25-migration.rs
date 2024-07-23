use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
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
    tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::fmt::layer().with_filter(
                EnvFilter::from_default_env()
                    .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
            ),
        )
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
    let vector_sizes: Vec<u64> = std::env::var("VECTOR_SIZES")
        .unwrap_or("384,512,768,1024,1536,3072".to_string())
        .split(',')
        .map(|x| x.parse().ok())
        .collect::<Option<Vec<u64>>>()
        .unwrap_or(vec![384, 512, 768, 1024, 1536, 3072]);

    let collections: Vec<String> = vector_sizes
        .iter()
        .map(|size| format!("{}_vectors_bm25", size))
        .collect();

    for collection in collections {
        log::info!("queue'ing collection: {:?}", collection);

        let mut offset = Some(uuid::Uuid::nil().to_string());

        while let Some(cur_offset) = offset.clone() {
            let (qdrant_point_ids, new_offset) = scroll_qdrant_collection_ids(
                collection.clone(),
                Some(cur_offset.to_string()),
                Some(1000),
            )
            .await?;

            let mut conn = web_redis_pool
                .get()
                .await
                .expect("Failed to connect to redis");

            let message = serde_json::to_string(&MigratePointMessage {
                qdrant_point_ids: qdrant_point_ids.clone(),
                from_collection: collection.clone(),
                to_collection: format!("{}_bm25", collection),
                mode: MigrationMode::BM25 {
                    average_len: 256.0,
                    b: 0.75,
                    k: 1.2,
                },
            })
            .expect("Failed to serialze MigratePoint message");

            redis::cmd("lpush")
                .arg("collection_migration")
                .arg(&message)
                .query_async(&mut *conn)
                .await
                .map_err(|_| {
                    ServiceError::BadRequest("Failed to send message to redis".to_string())
                })?;

            log::info!(
                "Migrated {:?} points between {:?} and {:?}",
                qdrant_point_ids.len(),
                offset.clone(),
                new_offset.clone()
            );
            offset = new_offset;
        }
    }

    Ok(())
}
