use actix_web::{middleware::Logger, web, App, HttpServer};
use chm::tools::migrations::{run_pending_migrations, SetupArgs};
use routes::{create_task::create_task, get_task::get_task};

pub mod errors;
pub mod models;
pub mod operators;
pub mod routes;

// #[macro_export]
// #[cfg(not(feature = "runtime-env"))]
// macro_rules! get_env {
//     ($name:expr, $message:expr) => {
//         env!($name, $message)
//     };
// }

#[macro_export]
#[cfg(not(feature = "runtime-env"))]
macro_rules! get_env {
    ($name:expr, $message:expr) => {{
        lazy_static::lazy_static! {
            static ref ENV_VAR: String = {
                std::env::var($name).expect($message)
            };
        }
        ENV_VAR.as_str()
    }};
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();

    let redis_url = get_env!("REDIS_URL", "REDIS_URL should be set");

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

    log::info!("Connecting to redis");

    let redis_manager =
        bb8_redis::RedisConnectionManager::new(redis_url).expect("Failed to connect to redis");

    let redis_connections: u32 = std::env::var("REDIS_CONNECTIONS")
        .unwrap_or("200".to_string())
        .parse()
        .unwrap_or(200);

    let redis_pool = bb8_redis::bb8::Pool::builder()
        .max_size(redis_connections)
        .build(redis_manager)
        .await
        .expect("Failed to create redis pool");

    HttpServer::new(move || {
        App::new()
            .wrap(actix_cors::Cors::permissive())
            .wrap(
                // Set up logger, but avoid logging hot status endpoints
                Logger::new("%r %s %b %{Referer}i %{User-Agent}i %T")
                    .exclude("/")
                    .exclude("/api/health")
                    .exclude("/metrics"),
            )
            .app_data(web::Data::new(redis_pool.clone()))
            .app_data(web::Data::new(clickhouse_client.clone()))
            .service(create_task)
            .service(get_task)
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await
}
