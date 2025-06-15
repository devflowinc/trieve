use actix_web::{
    get,
    middleware::Logger,
    web::{self, PayloadConfig},
    App, HttpResponse, HttpServer,
};
use chm::tools::migrations::{run_pending_migrations, SetupArgs};
use errors::{custom_json_error_handler, ErrorResponseBody};
use routes::{create_task::create_task, get_task::get_task, jinja_templates};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_actix_web::AppExt;
use utoipa_redoc::{Redoc, Servable};

pub mod errors;
pub mod middleware;
pub mod models;
pub mod operators;
pub mod routes;

/// Health Check
///
/// Confirmation that the service is healthy and can make embedding vectors
#[utoipa::path(
    get,
    path = "/health",
    context_path = "/api",
    tag = "Health",
    responses(
        (status = 200, description = "Confirmation that the service is healthy and can make embedding vectors"),
        (status = 400, description = "Service error relating to making an embedding or overall service health", body = ErrorResponseBody),
    ),
)]
#[get("")]
pub async fn health_check() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().finish())
}

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

pub type Templates<'a> = web::Data<minijinja::Environment<'a>>;

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    #[derive(OpenApi)]
    #[openapi(info(
        title = "PDF2MD API",
        description = "PDF2MD OpenAPI Specification. This document describes all of the operations available through the PDF2MD API.",
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
        (name = "Task", description = "Task operations. Allow you to interact with tasks."),
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

    let json_cfg = web::JsonConfig::default()
        .limit(134200000)
        .error_handler(custom_json_error_handler);

    HttpServer::new(move || {
        let mut jinja_env = minijinja::Environment::new();
        minijinja_embed::load_templates!(&mut jinja_env);

        App::new()
            .wrap(actix_cors::Cors::permissive())
            .wrap(
                // Set up logger, but avoid logging hot status endpoints
                Logger::new("%r %s %b %{Referer}i %{User-Agent}i %T")
                    .exclude("/")
                    .exclude("/api/health")
                    .exclude("/metrics"),
            )
            .wrap(middleware::api_key_middleware::ApiKeyMiddlewareFactory)
            .into_utoipa_app()
            .openapi(ApiDoc::openapi())
            .app_data(json_cfg.clone())
            .app_data(PayloadConfig::new(1073741824))
            .app_data(web::Data::new(jinja_env))
            .app_data(web::Data::new(redis_pool.clone()))
            .app_data(web::Data::new(clickhouse_client.clone()))
            .default_service(actix_files::Files::new("/static", "."))
            .service(utoipa_actix_web::scope("/api/task").configure(|config| {
                config.service(create_task).service(get_task);
            }))
            .service(utoipa_actix_web::scope("/health").configure(|config| {
                config.service(health_check);
            }))
            .openapi_service(|api| Redoc::with_url("/redoc", api))
            .service(utoipa_actix_web::scope("").configure(|config| {
                config.service(jinja_templates::public_page);
            }))
            .into_app()
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}
