use chrono::NaiveDateTime;
use dateparser::DateTimeUtc;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use futures::StreamExt;
use lapin::types::FieldTable;
use lapin::BasicProperties;
use sentry::{Hub, SentryFutureExt};
use tracing_subscriber::{prelude::*, EnvFilter, Layer};
use trieve_server::data::models::{self, ChunkMetadata, Event, ServerDatasetConfiguration};
use trieve_server::errors::ServiceError;
use trieve_server::handlers::chunk_handler::{UpdateIngestionMessage, UploadIngestionMessage};
use trieve_server::handlers::group_handler::dataset_owns_group;
use trieve_server::operators::chunk_operator::{
    get_metadata_from_point_ids, get_qdrant_id_from_chunk_id_query, insert_chunk_metadata_query,
    insert_duplicate_chunk_metadata_query, update_chunk_metadata_query,
};
use trieve_server::operators::event_operator::create_event_query;
use trieve_server::operators::model_operator::{create_embeddings, get_splade_embedding};
use trieve_server::operators::parse_operator::{
    average_embeddings, coarse_doc_chunker, convert_html_to_text,
};
use trieve_server::operators::qdrant_operator::{
    create_new_qdrant_point_query, update_qdrant_point_query,
};
use trieve_server::operators::search_operator::global_unfiltered_top_match_query;
use trieve_server::{establish_connection, get_env, set_up_rabbit};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum IngestionMessage {
    Upload(UploadIngestionMessage),
    Update(UpdateIngestionMessage),
}

async fn send_to_delay_queue(
    payload: IngestionMessage,
    publish_headers: lapin::types::FieldTable,
    channel: &lapin::Channel,
) -> Result<(), Box<dyn std::error::Error>> {
    channel
        .basic_publish(
            "ingestion_exchange",
            "delay",
            lapin::options::BasicPublishOptions::default(),
            serde_json::to_string(&payload)?.as_bytes(),
            BasicProperties::default().with_headers(publish_headers),
        )
        .await
        .expect("Failed to publish message");

    Ok(())
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
                let rabbit_pool = set_up_rabbit().await;
                let web_rabbit_pool = actix_web::web::Data::new(rabbit_pool);
                let threads: Vec<_> = (0..thread_num)
                    .map(|i| {
                        let web_pool = web_pool.clone();
                        let web_rabbit_pool = web_rabbit_pool.clone();
                        tokio::spawn(async move {
                            ingestion_service(i, web_rabbit_pool, web_pool).await
                        })
                    })
                    .collect();

                futures::future::join_all(threads).await;
            }
            .bind_hub(Hub::new_from_top(Hub::current())),
        );
}

#[tracing::instrument(skip(web_pool, rabbit_pool))]
async fn ingestion_service(
    thread: usize,
    rabbit_pool: actix_web::web::Data<models::RabbitPool>,
    web_pool: actix_web::web::Data<models::Pool>,
) {
    log::info!("Starting ingestion service thread");

    let rabbit_connection = rabbit_pool
        .get()
        .await
        .expect("Failed to fetch from rabbit pool");
    let channel = rabbit_connection
        .create_channel()
        .await
        .expect("Failed to create channel");

    let mut consumer = channel
        .basic_consume(
            "ingestion_queue",
            "ingestion_microservice",
            lapin::options::BasicConsumeOptions {
                no_ack: true,
                nowait: true,
                ..Default::default()
            },
            lapin::types::FieldTable::default(),
        )
        .await
        .expect("Failed to consume from queue");

    loop {
        let delivery = match consumer.next().await {
            Some(Ok(delivery)) => delivery,
            Some(Err(err)) => {
                log::error!("Failed to consume message: {:?}", err);
                continue;
            }
            None => {
                continue;
            }
        };

        let ctx = sentry::TransactionContext::new("Processing chunk", "Processing chunk");
        let transaction = sentry::start_transaction(ctx);

        let string_payload = std::str::from_utf8(&delivery.data)
            .map_err(|err| {
                log::error!("Failed to parse payload: {:?}", err);
            })
            .unwrap_or("");

        let payload: IngestionMessage = serde_json::from_str(string_payload).unwrap();
        match payload {
            IngestionMessage::Upload(payload) => {
                match upload_chunk(
                    payload.clone(),
                    web_pool.clone(),
                    payload.clone().dataset_config,
                )
                .await
                {
                    Ok(_) => {
                        log::info!(
                            "Uploaded chunk: {:?}",
                            payload.ingest_specific_chunk_metadata.id
                        );
                        let _ = create_event_query(
                            Event::from_details(
                                payload.ingest_specific_chunk_metadata.dataset_id,
                                models::EventType::CardUploaded {
                                    chunk_id: payload.ingest_specific_chunk_metadata.id,
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
                        let delivery_count = delivery
                            .properties
                            .headers()
                            .clone()
                            .and_then(|h| {
                                if h.contains_key("delivery_count") {
                                    h.inner()
                                        .get("delivery_count")
                                        .and_then(|v| v.as_short_int())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(0);

                        if delivery_count <= 3 {
                            log::info!("Resending to delay queue");
                            let mut publish_headers = FieldTable::default();
                            publish_headers.insert(
                                "delivery_count".into(),
                                lapin::types::AMQPValue::ShortInt(delivery_count + 1),
                            );
                            let _ = send_to_delay_queue(
                                IngestionMessage::Upload(payload.clone()),
                                publish_headers,
                                &channel,
                            )
                            .await
                            .map_err(|err| {
                                log::error!("Failed to send to delay queue: {:?}", err);
                            });
                        }

                        let _ = create_event_query(
                            Event::from_details(
                                payload.ingest_specific_chunk_metadata.dataset_id,
                                models::EventType::CardActionFailed {
                                    chunk_id: payload.ingest_specific_chunk_metadata.id,
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
                        log::error!("Failed to update chunk: {:?}", err);
                        let delivery_count = delivery
                            .properties
                            .headers()
                            .clone()
                            .and_then(|h| {
                                if h.contains_key("delivery_count") {
                                    h.inner()
                                        .get("delivery_count")
                                        .and_then(|v| v.as_short_int())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or(0);

                        if delivery_count <= 3 {
                            let mut publish_headers = FieldTable::default();
                            publish_headers.insert(
                                "delivery_count".into(),
                                lapin::types::AMQPValue::ShortInt(delivery_count + 1),
                            );
                            let _ = send_to_delay_queue(
                                IngestionMessage::Update(payload.clone()),
                                publish_headers,
                                &channel,
                            )
                            .await
                            .map_err(|err| {
                                log::error!("Failed to send to delay queue: {:?}", err);
                            });
                        }
                        let _ = create_event_query(
                            Event::from_details(
                                payload.dataset_id,
                                models::EventType::CardActionFailed {
                                    chunk_id: payload.chunk_metadata.id,
                                    error: format!("Failed to update chunk: {:?}", err),
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
        .ingest_specific_chunk_metadata
        .qdrant_point_id
        .unwrap_or(uuid::Uuid::new_v4());

    let content = convert_html_to_text(&payload.chunk.chunk_html.clone().unwrap_or_default());

    let chunk_tag_set = payload
        .chunk
        .tag_set
        .clone()
        .map(|tag_set| tag_set.join(","));

    let chunk_tracking_id = payload
        .chunk
        .tracking_id
        .clone()
        .filter(|chunk_tracking| !chunk_tracking.is_empty());

    let timestamp = {
        payload
            .chunk
            .time_stamp
            .clone()
            .map(|ts| -> Result<NaiveDateTime, ServiceError> {
                Ok(ts
                    .parse::<DateTimeUtc>()
                    .map_err(|_| ServiceError::BadRequest("Invalid timestamp format".to_string()))?
                    .0
                    .with_timezone(&chrono::Local)
                    .naive_local())
            })
            .transpose()?
    };

    let chunk_metadata = ChunkMetadata {
        id: payload.ingest_specific_chunk_metadata.id,
        content: content.clone(),
        link: payload.chunk.link.clone(),
        qdrant_point_id: Some(qdrant_point_id),
        created_at: chrono::Utc::now().naive_local(),
        updated_at: chrono::Utc::now().naive_local(),
        tag_set: chunk_tag_set,
        chunk_html: payload.chunk.chunk_html.clone(),
        metadata: payload.chunk.metadata.clone(),
        tracking_id: chunk_tracking_id,
        time_stamp: timestamp,
        dataset_id: payload.ingest_specific_chunk_metadata.dataset_id,
        weight: payload.chunk.weight.unwrap_or(0.0),
    };

    let embedding_vector = if let Some(embedding_vector) = payload.chunk.chunk_vector.clone() {
        embedding_vector
    } else {
        match payload.chunk.split_avg.unwrap_or(false) {
            true => {
                let chunks = coarse_doc_chunker(content.clone());

                let embeddings = create_embeddings(chunks, "doc", dataset_config.clone())
                    .await
                    .map_err(|err| {
                        ServiceError::InternalServerError(format!(
                            "Failed to create embedding: {:?}",
                            err
                        ))
                    })?;

                average_embeddings(embeddings).map_err(|err| {
                    ServiceError::InternalServerError(format!(
                        "Failed to average embeddings: {:?}",
                        err.message
                    ))
                })?
            }
            false => {
                let embedding_vectors =
                    create_embeddings(vec![content.clone()], "doc", dataset_config.clone())
                        .await
                        .map_err(|err| {
                            ServiceError::InternalServerError(format!(
                                "Failed to create embedding: {:?}",
                                err
                            ))
                        })?;

                embedding_vectors
                    .first()
                    .ok_or(ServiceError::InternalServerError(
                        "Failed to get first embedding".into(),
                    ))?
                    .clone()
            }
        }
    };

    let splade_vector = if dataset_config.FULLTEXT_ENABLED {
        match get_splade_embedding(&content.clone(), "doc").await {
            Ok(v) => v,
            Err(_) => vec![(0, 0.0)],
        }
    } else {
        vec![(0, 0.0)]
    };
    // let splade_vector = vec![(0, 0.0)];

    let mut collision: Option<uuid::Uuid> = None;

    let duplicate_distance_threshold = dataset_config.DUPLICATE_DISTANCE_THRESHOLD;

    if duplicate_distance_threshold < 1.0 || dataset_config.COLLISIONS_ENABLED {
        let collision_detection_span = transaction.start_child(
            "collision_check",
            "global_unfiltered_top_match_query and get_metadata_from_point_ids",
        );

        let first_semantic_result = global_unfiltered_top_match_query(
            embedding_vector.clone(),
            payload.ingest_specific_chunk_metadata.dataset_id,
            dataset_config.clone(),
        )
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!("Failed to get top match: {:?}", err))
        })?;

        if first_semantic_result.score >= duplicate_distance_threshold as f32 {
            //Sets collision to collided chunk id
            collision = Some(first_semantic_result.point_id);

            let score_chunk_result =
                get_metadata_from_point_ids(vec![first_semantic_result.point_id], web_pool.clone())
                    .await;

            match score_chunk_result {
                Ok(chunk_results) => chunk_results
                    .first()
                    .expect("First chunk must exist on collision check")
                    .clone(),
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

        update_qdrant_point_query(
            None,
            collision.expect("Collision must be some"),
            None,
            None,
            payload.ingest_specific_chunk_metadata.dataset_id,
            splade_vector,
            dataset_config.clone(),
        )
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!("Failed to update qdrant point: {:?}", err))
        })?;

        insert_duplicate_chunk_metadata_query(
            chunk_metadata.clone(),
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
        payload.ingest_specific_chunk_metadata.qdrant_point_id = Some(qdrant_point_id);

        let insert_tx = transaction.start_child(
            "calling_insert_chunk_metadata_query",
            "calling_insert_chunk_metadata_query",
        );

        let inserted_chunk = insert_chunk_metadata_query(
            chunk_metadata.clone(),
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

        create_new_qdrant_point_query(
            qdrant_point_id,
            embedding_vector,
            chunk_metadata.clone(),
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
    let content = convert_html_to_text(
        &payload
            .chunk_metadata
            .clone()
            .chunk_html
            .unwrap_or("".to_string())
            .clone(),
    );
    let mut chunk_metadata = payload.chunk_metadata.clone();
    chunk_metadata.content = content.clone();

    let embedding_vectors = create_embeddings(
        vec![content.to_string()],
        "doc",
        server_dataset_config.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    let embedding_vector = embedding_vectors
        .first()
        .ok_or(ServiceError::BadRequest(
            "Failed to get first embedding due to empty response from create_embedding".into(),
        ))?
        .clone();

    let qdrant_point_id = get_qdrant_id_from_chunk_id_query(chunk_metadata.id, web_pool.clone())
        .await
        .map_err(|_| ServiceError::BadRequest("chunk not found".into()))?;

    let splade_vector = if server_dataset_config.FULLTEXT_ENABLED {
        match get_splade_embedding(&content, "doc").await {
            Ok(v) => v,
            Err(_) => vec![(0, 0.0)],
        }
    } else {
        vec![(0, 0.0)]
    };

    if let Some(group_ids) = payload.group_ids {
        let mut chunk_group_ids: Vec<uuid::Uuid> = vec![];
        for group_id in group_ids {
            let group = dataset_owns_group(group_id, payload.dataset_id, web_pool.clone())
                .await
                .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
            chunk_group_ids.push(group.id);
        }

        let chunk = update_chunk_metadata_query(
            chunk_metadata.clone(),
            None,
            Some(chunk_group_ids.clone()),
            payload.dataset_id,
            web_pool.clone(),
        )
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

        if let Some(qdrant_point_id) = chunk.qdrant_point_id {
            update_qdrant_point_query(
                // If the chunk is a collision, we don't want to update the qdrant point
                if chunk_metadata.qdrant_point_id.is_none() {
                    None
                } else {
                    Some(chunk_metadata)
                },
                qdrant_point_id,
                Some(embedding_vector),
                Some(chunk_group_ids),
                payload.dataset_id,
                splade_vector,
                server_dataset_config,
            )
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
        }
    } else {
        update_chunk_metadata_query(
            chunk_metadata.clone(),
            None,
            None,
            payload.dataset_id,
            web_pool.clone(),
        )
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

        update_qdrant_point_query(
            // If the chunk is a collision, we don't want to update the qdrant point
            if chunk_metadata.qdrant_point_id.is_none() {
                None
            } else {
                Some(chunk_metadata)
            },
            qdrant_point_id,
            Some(embedding_vector),
            None,
            payload.dataset_id,
            splade_vector,
            server_dataset_config,
        )
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    }

    Ok(())
}
