use chm::tools::migrations::SetupArgs;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use tracing_subscriber::{prelude::*, EnvFilter, Layer};
use trieve_server::{
    errors::ServiceError,
    establish_connection, get_env,
    operators::{
        dittofeed_operator::{get_user_ditto_identity, send_user_ditto_identity},
        user_operator::get_all_users_query,
    },
};

#[allow(clippy::print_stdout)]
#[tokio::main]
async fn main() -> Result<(), ServiceError> {
    dotenvy::dotenv().ok();
    log::info!("Starting ditto sync worker service thread");
    tracing_subscriber::Registry::default()
        .with(
            tracing_subscriber::fmt::layer().with_filter(
                EnvFilter::from_default_env()
                    .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
            ),
        )
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

    let args = SetupArgs {
        url: Some(get_env!("CLICKHOUSE_URL", "CLICKHOUSE_URL is not set").to_string()),
        user: Some(get_env!("CLICKHOUSE_USER", "CLICKHOUSE_USER is not set").to_string()),
        password: Some(
            get_env!("CLICKHOUSE_PASSWORD", "CLICKHOUSE_PASSWORD is not set").to_string(),
        ),
        database: Some(get_env!("CLICKHOUSE_DB", "CLICKHOUSE_DB is not set").to_string()),
    };

    let clickhouse_client = clickhouse::Client::default()
        .with_url(args.url.as_ref().unwrap())
        .with_user(args.user.as_ref().unwrap())
        .with_password(args.password.as_ref().unwrap())
        .with_database(args.database.as_ref().unwrap())
        .with_option("async_insert", "1")
        .with_option("wait_for_async_insert", "0");

    let pool = actix_web::web::Data::new(pool.clone());

    let users = get_all_users_query(pool.clone()).await?;

    log::info!("Fetched {} users", users.len());

    for user in users {
        match get_user_ditto_identity(user.clone(), pool.clone(), &clickhouse_client).await {
            Ok(identify_request) => {
                match send_user_ditto_identity(identify_request).await {
                    Ok(_) => {
                        log::info!("Sent ditto identity for user {}", user.email);
                    }
                    Err(e) => {
                        log::info!(
                            "Failed to send ditto identity for user {}. Error: {}",
                            user.email,
                            e
                        );
                    }
                };
            }
            Err(e) => {
                log::info!("No ditto identity for user {}. Error: {}", user.email, e);
            }
        }
    }

    log::info!("Finished sending ditto identities");
    Ok(())
}
