use batch_etl::operators::{
    batch::{get_batch_output, get_pending_batches},
    job::send_webhook,
};

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

    let pending_batches = get_pending_batches(&clickhouse_client).await.unwrap();
    log::info!("Pending batches: {:?}", pending_batches);

    for (batch, id, webhook_url) in pending_batches {
        log::info!("Processing batch: {:?}", batch);
        let batch_url = get_batch_output(&clickhouse_client, batch).await.unwrap();
        log::info!("Batch URL: {:?}", batch_url);
        if let Some(webhook_url) = webhook_url {
            if let Some(batch_url) = batch_url {
                log::info!("Sending webhook to: {:?}", webhook_url);
                send_webhook(webhook_url, id, batch_url).await.unwrap();
            }
        }
    }

    Ok(())
}
