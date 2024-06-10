use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use redis::aio::MultiplexedConnection;
use sentry::{Hub, SentryFutureExt};
use signal_hook::consts::SIGTERM;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use trieve_server::{
    data::models::{self, FileWorkerMessage},
    errors::ServiceError,
    establish_connection, get_env,
    operators::file_operator::{create_file_chunks, create_file_query, get_aws_bucket},
};

fn main() {
    dotenvy::dotenv().ok();
    let sentry_url = std::env::var("SENTRY_URL");
    let _guard = if let Ok(sentry_url) = sentry_url {
        let guard = sentry::init((
            sentry_url,
            sentry::ClientOptions {
                release: sentry::release_name!(),
                traces_sample_rate: 1.0,
                ..Default::default()
            },
        ));

        tracing_subscriber::Registry::default()
            .with(sentry::integrations::tracing::layer())
            .with(
                tracing_subscriber::fmt::layer().with_filter(
                    EnvFilter::from_default_env()
                        .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
                ),
            )
            .init();

        log::info!("Sentry monitoring enabled");
        Some(guard)
    } else {
        tracing_subscriber::Registry::default()
            .with(
                tracing_subscriber::fmt::layer().with_filter(
                    EnvFilter::from_default_env()
                        .add_directive(tracing_subscriber::filter::LevelFilter::INFO.into()),
                ),
            )
            .init();

        None
    };

    let thread_num = if let Ok(thread_num) = std::env::var("THREAD_NUM") {
        thread_num
            .parse::<usize>()
            .expect("THREAD_NUM must be a number")
    } else {
        std::thread::available_parallelism()
            .expect("Failed to get available parallelism")
            .get()
            * 2
    };

    let database_url = get_env!("DATABASE_URL", "DATABASE_URL is not set");

    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);

    let mgr = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new_with_config(
        database_url,
        config,
    );

    let pool = diesel_async::pooled_connection::deadpool::Pool::builder(mgr)
        .max_size(10)
        .build()
        .expect("Failed to create diesel_async pool");

    let web_pool = actix_web::web::Data::new(pool.clone());

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime")
        .block_on(
            async move {
                let should_terminate = Arc::new(AtomicBool::new(false));
                signal_hook::flag::register(SIGTERM, Arc::clone(&should_terminate))
                    .expect("Failed to register shutdown hook");
                let threads: Vec<_> = (0..thread_num)
                    .map(|i| {
                        let web_pool = web_pool.clone();
                        let should_terminate = Arc::clone(&should_terminate);

                        tokio::spawn(
                            async move { delete_worker(should_terminate, i, web_pool).await },
                        )
                    })
                    .collect();

                while !should_terminate.load(Ordering::Relaxed) {}
                log::info!("Shutdown signal received, killing all children...");
                futures::future::join_all(threads).await
            }
            .bind_hub(Hub::new_from_top(Hub::current())),
        );
}

pub async fn delete_worker(
    should_terminate: Arc<AtomicBool>,
    thread_num: usize,
    web_pool: actix_web::web::Data<models::Pool>,
) {
    let dataset_to_delete = loop {
        let dataset = get_soft_delete_dataset(&web_pool)
            .await
            .expect("Failed to get dataset to delete");

        if let Some(dataset) = dataset {
            break dataset;
        }

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    };
}

pub async fn get_soft_delete_dataset(pool: &models::Pool) -> Result<Option<Dataset>, ServiceError> {
    use crate::schema::datasets::dsl::*;
    use diesel::prelude::*;

    let conn = pool.get().await?;
    let dataset = datasets
        .filter(deleted.eq(true))
        .first::<Dataset>(&conn)
        .optional()?;

    Ok(dataset)
}
