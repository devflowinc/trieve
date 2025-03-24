use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use signal_hook::consts::SIGTERM;
use std::sync::{atomic::AtomicBool, Arc};
use trieve_server::errors::ServiceError;
use trieve_server::operators::{organization_operator, stripe_operator};
use trieve_server::{establish_connection, get_env};

#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    dotenvy::dotenv().ok();
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();

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

    let clickhouse_client = clickhouse::Client::default()
        .with_url(std::env::var("CLICKHOUSE_URL").unwrap_or("http://localhost:8123".to_string()))
        .with_user(std::env::var("CLICKHOUSE_USER").unwrap_or("default".to_string()))
        .with_password(std::env::var("CLICKHOUSE_PASSWORD").unwrap_or("".to_string()))
        .with_database(std::env::var("CLICKHOUSE_DATABASE").unwrap_or("default".to_string()))
        .with_option("async_insert", "1")
        .with_option("wait_for_async_insert", "0");

    let should_terminate = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGTERM, Arc::clone(&should_terminate))
        .expect("Failed to register shutdown hook");

    let organization_ids = organization_operator::get_all_organization_ids(pool.clone()).await?;

    for organization_id in organization_ids {
        let usage_based_subscription =
            stripe_operator::get_option_usage_based_subscription_by_organization_id_query(
                organization_id,
                pool.clone(),
            )
            .await?;

        if let Some(usage_based_subscription) = usage_based_subscription {
            let result = stripe_operator::send_stripe_billing(
                usage_based_subscription,
                &clickhouse_client,
                pool.clone(),
            )
            .await;

            if let Err(e) = result {
                log::error!(
                    "Failed to send stripe billing for organization_id: {}: {}",
                    organization_id,
                    e
                );
            }
        }
    }

    Ok(())
}
