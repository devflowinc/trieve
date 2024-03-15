use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use redis::AsyncCommands;
use sentry::{Hub, SentryFutureExt};
use tracing_subscriber::{prelude::*, EnvFilter, Layer};
use trieve_server::data::models::{self, Event, ServerDatasetConfiguration};
use trieve_server::errors::ServiceError;
use trieve_server::handlers::chunk_handler::{UpdateIngestionMessage, UploadIngestionMessage};
use trieve_server::operators::chunk_operator::{
    get_metadata_from_point_ids, get_qdrant_id_from_chunk_id_query, insert_chunk_metadata_query,
    insert_duplicate_chunk_metadata_query, update_chunk_metadata_query,
};
use trieve_server::operators::event_operator::create_event_query;
use trieve_server::operators::model_operator::{create_embedding, get_splade_embedding};
use trieve_server::operators::parse_operator::{average_embeddings, coarse_doc_chunker};
use trieve_server::operators::qdrant_operator::{
    create_new_qdrant_point_query, update_qdrant_point_query,
};
use trieve_server::operators::search_operator::global_unfiltered_top_match_query;
use trieve_server::{establish_connection, get_env};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum IngestionMessage {
    Upload(UploadIngestionMessage),
    Update(UpdateIngestionMessage),
}

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
        thread_num.parse::<usize>().unwrap()
    } else {
        std::thread::available_parallelism().unwrap().get() * 2
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
        .unwrap();

    let web_pool = actix_web::web::Data::new(pool.clone());

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(
            async move {
                let redis_url = get_env!("REDIS_URL", "REDIS_URL is not set");
                let redis_client = redis::Client::open(redis_url).unwrap();
                let redis_connection = redis_client
                    .get_multiplexed_tokio_connection()
                    .await
                    .unwrap();

                let threads: Vec<_> = (0..thread_num)
                    .map(|i| {
                        let web_pool = web_pool.clone();
                        let redis_connection = redis_connection.clone();
                        tokio::spawn(async move {
                            ingestion_service(i, redis_connection, web_pool).await
                        })
                    })
                    .collect();

                futures::future::join_all(threads).await;
            }
            .bind_hub(Hub::new_from_top(Hub::current())),
        );
}

#[tracing::instrument(skip(web_pool, redis_connection))]
async fn ingestion_service(
    thread: usize,
    mut redis_connection: redis::aio::MultiplexedConnection,
    web_pool: actix_web::web::Data<models::Pool>,
) {
    log::info!("Starting ingestion service thread");
    loop {
        let payload_result = redis_connection
            .brpop::<&str, Vec<String>>("ingestion", 0.0)
            .await;

        let payload = if let Ok(payload) = payload_result {
            payload
        } else {
            continue;
        };

        let ctx = sentry::TransactionContext::new("Processing chunk", "Processing chunk");
        let transaction = sentry::start_transaction(ctx);

        let payload: IngestionMessage = serde_json::from_str(&payload[1]).unwrap();
        match payload {
            IngestionMessage::Upload(payload) => {
                match upload_chunk(payload.clone(), web_pool.clone(), payload.dataset_config).await
                {
                    Ok(_) => {
                        log::info!("Uploaded chunk: {:?}", payload.chunk_metadata.id);
                        let _ = create_event_query(
                            Event::from_details(
                                payload.chunk_metadata.dataset_id,
                                models::EventType::CardUploaded {
                                    chunk_id: payload.chunk_metadata.id,
                                },
                            ),
                            web_pool.clone(),
                        )
                        .await
                        .map_err(|err| {
                            log::error!("Failed to create event: {:?}", err);
                        });
                    }
                    Err(err) => {
                        log::error!("Failed to upload chunk: {:?}", err);
                        let _ = create_event_query(
                            Event::from_details(
                                payload.chunk_metadata.dataset_id,
                                models::EventType::CardActionFailed {
                                    chunk_id: payload.chunk_metadata.id,
                                    error: format!("Failed to upload chunk: {:?}", err),
                                },
                            ),
                            web_pool.clone(),
                        )
                        .await
                        .map_err(|err| {
                            log::error!("Failed to create event: {:?}", err);
                        });
                    }
                }
            }

            IngestionMessage::Update(payload) => {
                match update_chunk(
                    payload.clone(),
                    web_pool.clone(),
                    payload.server_dataset_config.clone(),
                )
                .await
                {
                    Ok(_) => {
                        log::info!("Updated chunk: {:?}", payload.chunk_metadata.id);
                        let _ = create_event_query(
                            Event::from_details(
                                payload.dataset_id,
                                models::EventType::CardUpdated {
                                    chunk_id: payload.chunk_metadata.id,
                                },
                            ),
                            web_pool.clone(),
                        )
                        .await
                        .map_err(|err| {
                            log::error!("Failed to create event: {:?}", err);
                        });
                    }
                    Err(err) => {
                        log::error!("Failed to upload chunk: {:?}", err);
                        let _ = create_event_query(
                            Event::from_details(
                                payload.dataset_id,
                                models::EventType::CardActionFailed {
                                    chunk_id: payload.chunk_metadata.id,
                                    error: format!("Failed to upload chunk: {:?}", err),
                                },
                            ),
                            web_pool.clone(),
                        )
                        .await
                        .map_err(|err| {
                            log::error!("Failed to create event: {:?}", err);
                        });
                    }
                }
            }
        }
        transaction.finish();
    }
}

#[tracing::instrument(skip(payload, web_pool, dataset_config))]
async fn upload_chunk(
    mut payload: UploadIngestionMessage,
    web_pool: actix_web::web::Data<models::Pool>,
    dataset_config: ServerDatasetConfiguration,
) -> Result<(), ServiceError> {
    let tx_ctx = sentry::TransactionContext::new("upload_chunk", "Uploading Chunk");
    let transaction = sentry::start_transaction(tx_ctx);
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone().into())));

    let mut qdrant_point_id = payload
        .chunk_metadata
        .qdrant_point_id
        .unwrap_or(uuid::Uuid::new_v4());

    let embedding_vector = if let Some(embedding_vector) = payload.chunk.chunk_vector.clone() {
        embedding_vector
    } else {
        match payload.chunk.split_avg.unwrap_or(false) {
            true => {
                let chunks = coarse_doc_chunker(payload.chunk_metadata.content.clone());
                let mut embeddings: Vec<Vec<f32>> = vec![];
                for chunk in chunks {
                    let embedding = create_embedding(&chunk, "doc", dataset_config.clone())
                        .await
                        .map_err(|err| {
                            ServiceError::InternalServerError(format!(
                                "Failed to create embedding: {:?}",
                                err
                            ))
                        })?;
                    embeddings.push(embedding);
                }

                average_embeddings(embeddings).map_err(|err| {
                    ServiceError::InternalServerError(format!(
                        "Failed to average embeddings: {:?}",
                        err.message
                    ))
                })?
            }
            false => create_embedding(
                &payload.chunk_metadata.content,
                "doc",
                dataset_config.clone(),
            )
            .await
            .map_err(|err| {
                ServiceError::InternalServerError(format!("Failed to create embedding: {:?}", err))
            })?,
        }
    };

    let mut collision: Option<uuid::Uuid> = None;

    let duplicate_distance_threshold = dataset_config.DUPLICATE_DISTANCE_THRESHOLD;

    if duplicate_distance_threshold < 1.0 || dataset_config.COLLISIONS_ENABLED {
        let collision_detection_span = transaction.start_child(
            "collision_check",
            "global_unfiltered_top_match_query and get_metadata_from_point_ids",
        );

        let first_semantic_result = global_unfiltered_top_match_query(
            embedding_vector.clone(),
            payload.chunk_metadata.dataset_id,
            dataset_config.clone(),
        )
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!("Failed to get top match: {:?}", err))
        })?;

        if first_semantic_result.score >= duplicate_distance_threshold {
            //Sets collision to collided chunk id
            collision = Some(first_semantic_result.point_id);

            let score_chunk_result =
                get_metadata_from_point_ids(vec![first_semantic_result.point_id], web_pool.clone())
                    .await;

            match score_chunk_result {
                Ok(chunk_results) => chunk_results.first().unwrap().clone(),
                Err(err) => {
                    return Err(ServiceError::InternalServerError(format!(
                        "Failed to get chunk metadata: {:?}",
                        err
                    )))
                }
            };
        }
        collision_detection_span.finish();
    }

    //if collision is not nil, insert chunk with collision
    if collision.is_some() {
        let update_collision_span = transaction.start_child(
            "update_collision",
            "update_qdrant_point_query and insert_duplicate_chunk_metadata_query",
        );

        let splade_vector = if dataset_config.FULLTEXT_ENABLED {
            match get_splade_embedding(&payload.chunk_metadata.content, "doc").await {
                Ok(v) => v,
                Err(_) => vec![(0, 0.0)],
            }
        } else {
            vec![(0, 0.0)]
        };

        update_qdrant_point_query(
            None,
            collision.expect("Collision must be some"),
            None,
            payload.chunk_metadata.dataset_id,
            splade_vector,
            dataset_config.clone(),
        )
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!("Failed to update qdrant point: {:?}", err))
        })?;

        insert_duplicate_chunk_metadata_query(
            payload.chunk_metadata.clone(),
            collision.expect("Collision should must be some"),
            payload.chunk.file_id,
            payload.chunk.group_ids,
            web_pool.clone(),
        )
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!(
                "Failed to insert duplicate chunk metadata: {:?}",
                err
            ))
        })?;

        update_collision_span.finish();
    }
    //if collision is nil and embedding vector is some, insert chunk with no collision
    else {
        payload.chunk_metadata.qdrant_point_id = Some(qdrant_point_id);

        let insert_tx = transaction.start_child(
            "calling_insert_chunk_metadata_query",
            "calling_insert_chunk_metadata_query",
        );

        let inserted_chunk = insert_chunk_metadata_query(
            payload.chunk_metadata.clone(),
            payload.chunk.file_id,
            payload.chunk.group_ids.clone(),
            payload.dataset_id,
            payload.upsert_by_tracking_id,
            web_pool.clone(),
        )
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!("Failed to insert chunk metadata: {:?}", err))
        })?;

        insert_tx.finish();

        qdrant_point_id = inserted_chunk.qdrant_point_id.unwrap_or(qdrant_point_id);

        let insert_tx =
            transaction.start_child("calling_create_qdrant_point", "calling_create_qdrant_point");

        let splade_vector = if dataset_config.FULLTEXT_ENABLED {
            match get_splade_embedding(&payload.chunk_metadata.content, "doc").await {
                Ok(v) => v,
                Err(_) => vec![(0, 0.0)],
            }
        } else {
            vec![(0, 0.0)]
        };

        create_new_qdrant_point_query(
            qdrant_point_id,
            embedding_vector,
            payload.chunk_metadata.clone(),
            splade_vector,
            payload.chunk.group_ids,
            dataset_config.clone(),
        )
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!(
                "Failed to create new qdrant point: {:?}",
                err
            ))
        })?;

        insert_tx.finish();
    }

    transaction.finish();
    Ok(())
}

#[tracing::instrument(skip(web_pool))]
async fn update_chunk(
    payload: UpdateIngestionMessage,
    web_pool: actix_web::web::Data<models::Pool>,
    server_dataset_config: ServerDatasetConfiguration,
) -> Result<(), ServiceError> {
    let embedding_vector = create_embedding(
        &payload.chunk_metadata.content,
        "doc",
        server_dataset_config.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let qdrant_point_id =
        get_qdrant_id_from_chunk_id_query(payload.chunk_metadata.id, web_pool.clone())
            .await
            .map_err(|_| ServiceError::BadRequest("chunk not found".into()))?;

    update_chunk_metadata_query(
        payload.chunk_metadata.clone(),
        None,
        None,
        payload.dataset_id,
        web_pool.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let splade_vector = if server_dataset_config.FULLTEXT_ENABLED {
        match get_splade_embedding(&payload.chunk_metadata.content, "doc").await {
            Ok(v) => v,
            Err(_) => vec![(0, 0.0)],
        }
    } else {
        vec![(0, 0.0)]
    };

    update_qdrant_point_query(
        // If the chunk is a collision, we don't want to update the qdrant point
        if payload.chunk_metadata.qdrant_point_id.is_none() {
            None
        } else {
            Some(payload.chunk_metadata)
        },
        qdrant_point_id,
        Some(embedding_vector),
        payload.dataset_id,
        splade_vector,
        server_dataset_config,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}
