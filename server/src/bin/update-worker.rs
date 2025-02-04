use actix_web::web;
use broccoli_queue::error::BroccoliError;
use broccoli_queue::queue::BroccoliQueue;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use trieve_server::data::models::{ChunkBoost, EventType, WorkerEvent};
use trieve_server::errors::ServiceError;
use trieve_server::handlers::group_handler::dataset_owns_group;
use trieve_server::operators::chunk_operator::{
    update_chunk_boost_query, update_chunk_metadata_query,
};
use trieve_server::operators::clickhouse_operator::ClickHouseEvent;
use trieve_server::operators::dataset_operator::get_dataset_config_query;
use trieve_server::operators::model_operator::{
    get_bm25_embeddings, get_dense_vector, get_sparse_vectors,
};
use trieve_server::operators::parse_operator::convert_html_to_text;
use trieve_server::operators::qdrant_operator::update_qdrant_point_query;

use std::error::Error;
use trieve_server::{
    data::models::Pool, establish_connection, get_env,
    handlers::chunk_handler::UpdateIngestionMessage, operators::clickhouse_operator::EventQueue,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    let web_pool = actix_web::web::Data::new(pool.clone());

    let event_queue = if std::env::var("USE_ANALYTICS")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false)
    {
        log::info!("Analytics enabled");

        let clickhouse_client = clickhouse::Client::default()
            .with_url(
                std::env::var("CLICKHOUSE_URL").unwrap_or("http://localhost:8123".to_string()),
            )
            .with_user(std::env::var("CLICKHOUSE_USER").unwrap_or("default".to_string()))
            .with_password(std::env::var("CLICKHOUSE_PASSWORD").unwrap_or("".to_string()))
            .with_database(std::env::var("CLICKHOUSE_DATABASE").unwrap_or("default".to_string()))
            .with_option("async_insert", "1")
            .with_option("wait_for_async_insert", "0");

        let mut event_queue = EventQueue::new(clickhouse_client.clone());
        event_queue.start_service();
        event_queue
    } else {
        log::info!("Analytics disabled");
        EventQueue::default()
    };

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
            "update_chunk_queue",
            None,
            None,
            {
                move |msg| {
                    let pool = web_pool.clone();
                    async move { update_chunk(msg.payload, pool.clone()).await }
                }
            },
            {
                let event_queue = event_queue.clone();
                move |msg| {
                    let value = event_queue.clone();
                    async move {
                        log::info!("Updated chunk: {:?}", msg.payload.chunk_metadata.id);
                        value
                            .send(ClickHouseEvent::WorkerEvent(
                                WorkerEvent::from_details(
                                    msg.payload.dataset_id,
                                    EventType::ChunkUpdated {
                                        chunk_id: msg.payload.chunk_metadata.id,
                                    },
                                )
                                .into(),
                            ))
                            .await;
                        Ok(())
                    }
                }
            },
            move |msg, err| {
                let value = event_queue.clone();
                async move {
                    log::error!("Error processing message: {:?}", err);
                    value
                        .send(ClickHouseEvent::WorkerEvent(
                            WorkerEvent::from_details(
                                msg.payload.dataset_id,
                                EventType::ChunkUpdateFailed {
                                    chunk_id: msg.payload.chunk_metadata.id,
                                    message: err.to_string(),
                                },
                            )
                            .into(),
                        ))
                        .await;
                    Ok(())
                }
            },
        )
        .await?;

    Ok(())
}

async fn update_chunk(
    payload: UpdateIngestionMessage,
    pool: web::Data<Pool>,
) -> Result<(), BroccoliError> {
    let dataset_config = get_dataset_config_query(payload.dataset_id, pool.clone()).await?;
    let content = match payload.convert_html_to_text.unwrap_or(true) {
        true => convert_html_to_text(
            &(payload
                .chunk_metadata
                .chunk_html
                .clone()
                .unwrap_or_default()),
        ),
        false => payload
            .chunk_metadata
            .chunk_html
            .clone()
            .unwrap_or_default(),
    };

    if content.is_empty() {
        return Err(BroccoliError::Job("Content is empty".to_string()));
    }

    let chunk_metadata = payload.chunk_metadata.clone();

    let embedding_vector = match dataset_config.SEMANTIC_ENABLED {
        true => {
            let embedding = get_dense_vector(
                content.to_string(),
                payload.semantic_boost.clone(),
                "doc",
                dataset_config.clone(),
            )
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
            Some(embedding)
        }
        false => None,
    };

    let splade_vector = if dataset_config.FULLTEXT_ENABLED {
        let reqwest_client = reqwest::Client::new();

        match get_sparse_vectors(
            vec![(content.clone(), payload.fulltext_boost.clone())],
            "doc",
            reqwest_client,
        )
        .await
        {
            Ok(v) => v.first().unwrap_or(&vec![(0, 0.0)]).clone(),
            Err(_) => vec![(0, 0.0)],
        }
    } else {
        vec![(0, 0.0)]
    };

    let bm25_vector = if dataset_config.BM25_ENABLED
        && std::env::var("BM25_ACTIVE").unwrap_or("false".to_string()) == "true"
    {
        let vecs = get_bm25_embeddings(
            vec![(content, payload.fulltext_boost.clone())],
            dataset_config.BM25_AVG_LEN,
            dataset_config.BM25_B,
            dataset_config.BM25_K,
        );

        vecs.first().cloned()
    } else {
        None
    };

    if let Some(group_ids) = payload.group_ids {
        let mut chunk_group_ids: Vec<uuid::Uuid> = vec![];
        for group_id in group_ids {
            let group = dataset_owns_group(group_id, payload.dataset_id, pool.clone())
                .await
                .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
            chunk_group_ids.push(group.id);
        }

        update_chunk_metadata_query(
            chunk_metadata.clone().into(),
            Some(chunk_group_ids.clone()),
            payload.dataset_id,
            pool.clone(),
        )
        .await?;

        update_qdrant_point_query(
            // If the chunk is a collision, we don't want to update the qdrant point
            chunk_metadata.into(),
            embedding_vector,
            Some(chunk_group_ids),
            payload.dataset_id,
            splade_vector,
            bm25_vector,
            dataset_config,
            pool.clone(),
        )
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    } else {
        update_chunk_metadata_query(
            chunk_metadata.clone().into(),
            None,
            payload.dataset_id,
            pool.clone(),
        )
        .await?;

        update_qdrant_point_query(
            // If the chunk is a collision, we don't want to update the qdrant point
            chunk_metadata.into(),
            embedding_vector,
            None,
            payload.dataset_id,
            splade_vector,
            bm25_vector,
            dataset_config,
            pool.clone(),
        )
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    }

    // If boosts are changed, reflect changes to chunk_boosts table
    if payload.fulltext_boost.is_some() || payload.semantic_boost.is_some() {
        update_chunk_boost_query(
            ChunkBoost {
                chunk_id: payload.chunk_metadata.id,
                fulltext_boost_phrase: payload.fulltext_boost.clone().map(|x| x.phrase),
                fulltext_boost_factor: payload.fulltext_boost.map(|x| x.boost_factor),
                semantic_boost_phrase: payload.semantic_boost.clone().map(|x| x.phrase),
                semantic_boost_factor: payload.semantic_boost.map(|x| x.distance_factor as f64),
            },
            pool,
        )
        .await?;
    }

    Ok(())
}
