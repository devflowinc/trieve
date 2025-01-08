use batch_etl::{get_env, operators::job::create_openai_job_query};
use broccoli_queue::queue::BroccoliQueue;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();

    let clickhouse_client = clickhouse::Client::default()
        .with_url(std::env::var("CLICKHOUSE_URL").unwrap_or("http://localhost:8123".to_string()))
        .with_user(std::env::var("CLICKHOUSE_USER").unwrap_or("default".to_string()))
        .with_password(std::env::var("CLICKHOUSE_PASSWORD").unwrap_or("".to_string()))
        .with_database(std::env::var("CLICKHOUSE_DATABASE").unwrap_or("default".to_string()))
        .with_option("async_insert", "1")
        .with_option("wait_for_async_insert", "0");

    let redis_url = get_env!("REDIS_URL", "REDIS_URL is not set");

    let redis_connections: u32 = std::env::var("REDIS_CONNECTIONS")
        .unwrap_or("2".to_string())
        .parse()
        .unwrap_or(2);

    let queue = BroccoliQueue::builder(redis_url)
        .pool_connections(redis_connections.try_into().unwrap())
        .failed_message_retry_strategy(Default::default())
        .build()
        .await?;

    queue
        .process_messages_with_handlers(
            "create_job_queue",
            None,
            move |msg| {
                let value = clickhouse_client.clone();
                async move {
                    log::info!("Processing message: {:?}", &msg.task_id);
                    create_openai_job_query(msg.payload, &value).await
                }
            },
            |msg| async move {
                log::info!("Processed message: {:?}", msg.task_id);
                Ok(())
            },
            |_, err| async move {
                log::error!("Error processing message: {:?}", err);
                Ok(())
            },
        )
        .await?;
    Ok(())
}
