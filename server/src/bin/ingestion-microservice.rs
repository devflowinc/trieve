use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use redis::AsyncCommands;
use trieve_server::data::models::{self, ChunkGroupBookmark, Event};
use trieve_server::errors::ServiceError;
use trieve_server::get_env;
use trieve_server::handlers::chunk_handler::IngestionMessage;
use trieve_server::operators::chunk_operator::{
    get_metadata_from_point_ids, insert_chunk_metadata_query, insert_duplicate_chunk_metadata_query,
};
use trieve_server::operators::event_operator::create_event_query;
use trieve_server::operators::group_operator::create_chunk_bookmark_query;
use trieve_server::operators::model_operator::create_embedding;
use trieve_server::operators::qdrant_operator::{
    add_bookmark_to_qdrant_query, create_new_qdrant_point_query, delete_qdrant_point_id_query,
    update_qdrant_point_query,
};
use trieve_server::operators::search_operator::global_unfiltered_top_match_query;

static THREAD_NUM: i32 = 4;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let redis_url = get_env!("REDIS_URL", "REDIS_URL is not set");
    let redis_client = redis::Client::open(redis_url).unwrap();
    let redis_connection = redis_client
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    let database_url = get_env!("DATABASE_URL", "DATABASE_URL is not set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: models::Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let web_pool = actix_web::web::Data::new(pool.clone());

    let threads: Vec<_> = (0..THREAD_NUM)
        .map(|_i| {
            let redis_connection = redis_connection.clone();
            let web_pool = web_pool.clone();
            ingestion_service(redis_connection, web_pool)
        })
        .collect();

    futures::future::join_all(threads).await;

    Ok(())
}

async fn ingestion_service(
    mut redis_connection: redis::aio::MultiplexedConnection,
    web_pool: actix_web::web::Data<models::Pool>,
) {
    log::info!("Starting ingestion service thread");
    loop {
        let payload_result = redis_connection
            .blpop::<&str, Vec<String>>("ingestion", 0.0)
            .await
            .map_err(|err| {
                log::error!("Failed to get payload from redis: {:?}", err);
                ServiceError::InternalServerError("Failed to get payload from redis".into())
            });

        let payload = if let Ok(payload) = payload_result {
            payload
        } else {
            continue;
        };

        let payload: IngestionMessage = serde_json::from_str(&payload[1]).unwrap();
        match upload_chunk(payload.clone(), web_pool.clone()).await {
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
                .map_err(|err| {
                    log::error!("Failed to create event: {:?}", err);
                });
            }
            Err(err) => {
                log::error!("Failed to upload chunk: {:?}", err);
                let _ = create_event_query(
                    Event::from_details(
                        payload.chunk_metadata.dataset_id,
                        models::EventType::CardUploadFailed {
                            chunk_id: payload.chunk_metadata.id,
                            error: format!("Failed to upload chunk: {:?}", err),
                        },
                    ),
                    web_pool.clone(),
                )
                .map_err(|err| {
                    log::error!("Failed to create event: {:?}", err);
                });
            }
        }
    }
}

async fn upload_chunk(
    mut payload: IngestionMessage,
    web_pool: actix_web::web::Data<models::Pool>,
) -> Result<(), ServiceError> {
    let embedding_vector = if let Some(embedding_vector) = payload.chunk.chunk_vector.clone() {
        embedding_vector
    } else {
        create_embedding(
            &payload.chunk_metadata.content,
            payload.dataset_config.clone(),
        )
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!("Failed to create embedding: {:?}", err))
        })?
    };

    let mut collision: Option<uuid::Uuid> = None;

    let duplicate_distance_threshold = payload
        .dataset_config
        .DUPLICATE_DISTANCE_THRESHOLD
        .unwrap_or(1.1);

    if duplicate_distance_threshold > 1.0
        || payload.dataset_config.COLLISIONS_ENABLED.unwrap_or(false)
    {
        let first_semantic_result = global_unfiltered_top_match_query(
            embedding_vector.clone(),
            payload.chunk_metadata.dataset_id,
        )
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!("Failed to get top match: {:?}", err))
        })?;

        if first_semantic_result.score >= duplicate_distance_threshold {
            //Sets collision to collided chunk id
            collision = Some(first_semantic_result.point_id);

            let score_chunk_result =
                get_metadata_from_point_ids(vec![first_semantic_result.point_id], web_pool.clone());

            match score_chunk_result {
                Ok(chunk_results) => {
                    if chunk_results.is_empty() {
                        delete_qdrant_point_id_query(
                            first_semantic_result.point_id,
                            payload.chunk_metadata.dataset_id,
                        )
                        .await
                        .map_err(|_| {
                            ServiceError::InternalServerError(
                                "Failed to delete qdrant point".into(),
                            )
                        })?;
                    }
                    chunk_results.first().unwrap().clone()
                }
                Err(err) => {
                    return Err(ServiceError::InternalServerError(format!(
                        "Failed to get chunk metadata: {:?}",
                        err
                    )))
                }
            };
        }
    }

    //if collision is not nil, insert chunk with collision
    if collision.is_some() {
        update_qdrant_point_query(
            None,
            collision.expect("Collision must be some"),
            None,
            payload.chunk_metadata.dataset_id,
        )
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!("Failed to update qdrant point: {:?}", err))
        })?;

        insert_duplicate_chunk_metadata_query(
            payload.chunk_metadata.clone(),
            collision.expect("Collision should must be some"),
            payload.chunk.file_uuid,
            web_pool.clone(),
        )
        .map_err(|err| {
            ServiceError::InternalServerError(format!(
                "Failed to insert duplicate chunk metadata: {:?}",
                err
            ))
        })?;
    }
    //if collision is nil and embedding vector is some, insert chunk with no collision
    else {
        let qdrant_point_id = uuid::Uuid::new_v4();

        payload.chunk_metadata.qdrant_point_id = Some(qdrant_point_id);

        insert_chunk_metadata_query(
            payload.chunk_metadata.clone(),
            payload.chunk.file_uuid,
            web_pool.clone(),
        )
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!("Failed to insert chunk metadata: {:?}", err))
        })?;

        create_new_qdrant_point_query(
            qdrant_point_id,
            embedding_vector,
            payload.chunk_metadata.clone(),
            payload.chunk_metadata.dataset_id,
        )
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!(
                "Failed to create new qdrant point: {:?}",
                err
            ))
        })?;
    }

    if let Some(group_ids_to_bookmark) = payload.chunk.group_ids {
        for group_id_to_bookmark in group_ids_to_bookmark {
            let chunk_group_bookmark = ChunkGroupBookmark::from_details(
                group_id_to_bookmark,
                payload.chunk_metadata.id,
            );

            let _ = create_chunk_bookmark_query(web_pool.clone(), chunk_group_bookmark).map_err(
                |err| {
                    log::error!("Failed to create chunk bookmark: {:?}", err);
                    ServiceError::InternalServerError(format!(
                        "Failed to create chunk bookmark: {:?}",
                        err
                    ))
                },
            );

            if let Some(qdrant_point_id) = payload.chunk_metadata.qdrant_point_id {
                add_bookmark_to_qdrant_query(qdrant_point_id, group_id_to_bookmark)
                    .await
                    .map_err(|err| {
                        ServiceError::InternalServerError(format!(
                            "Failed to add bookmark to qdrant: {:?}",
                            err
                        ))
                    })?;
            }
        }
    }

    Ok(())
}
