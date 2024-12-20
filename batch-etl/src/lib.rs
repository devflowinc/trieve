use actix_web::{
    web::{self, PayloadConfig},
    App, HttpServer,
};
use chm::tools::migrations::{run_pending_migrations, SetupArgs};
use errors::custom_json_error_handler;
use routes::{
    input::{create_input, get_input},
    job::{cancel_job, create_job, get_job, get_job_output},
    schema::{create_schema, get_schema},
};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_actix_web::AppExt;
use utoipa_redoc::{Redoc, Servable};

pub mod errors;
pub mod models;
pub mod operators;
pub mod routes;

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

#[macro_export]
#[cfg(feature = "runtime-env")]
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

    #[derive(OpenApi)]
    #[openapi(info(
        title = "Batch ETL API",
        description = "Batch ETL OpenAPI Specification. This document describes all of the operations available through the Batch ETL API.",
        contact(
            name = "Trieve Team",
            url = "https://trieve.ai",
            email = "developers@trieve.ai",
        ),
        license(
            name = "BSL",
            url = "https://github.com/devflowinc/trieve/blob/main/LICENSE.txt",
        ),
        version = "0.0.0"), 
    modifiers(&SecurityAddon),
    tags(
        (name = "Schema", description = "Schema operations. Allow you to interact with Schema."),
        (name = "Job", description = "Job operations. Allow you to interact with Jobs."),
        (name = "Input", description = "Input operations. Allow you to interact with Inputs."),
    ))]
    struct ApiDoc;

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap(); // we can unwrap safely since there already is components registered.
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("Authorization"))),
            )
        }
    }

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
        .with_option("wait_for_async_insert", "1");

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

    let json_cfg = web::JsonConfig::default()
        .limit(134200000)
        .error_handler(custom_json_error_handler);

    HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
            .openapi(ApiDoc::openapi())
            .app_data(PayloadConfig::new(134200000))
            .app_data(json_cfg.clone())
            .app_data(web::Data::new(redis_pool.clone()))
            .app_data(web::Data::new(clickhouse_client.clone()))
            .service(utoipa_actix_web::scope("/api/schema").configure(|config| {
                config.service(create_schema).service(get_schema);
            }))
            .service(utoipa_actix_web::scope("/api/input").configure(|config| {
                config.service(create_input).service(get_input);
            }))
            .service(utoipa_actix_web::scope("/api/job").configure(|config| {
                config
                    .service(create_job)
                    .service(get_job)
                    .service(cancel_job)
                    .service(get_job_output);
            }))
            .openapi_service(|api| Redoc::with_url("/redoc", api))
            .into_app()
    })
    .bind(("0.0.0.0", 8082))?
    .run()
    .await
}
