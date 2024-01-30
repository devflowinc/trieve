use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use trieve_server::data::models::{self, ChunkGroupBookmark};
use trieve_server::get_env;
use trieve_server::handlers::chunk_handler::IngestionMessage;
use trieve_server::operators::chunk_operator::{
    get_metadata_from_point_ids, insert_chunk_metadata_query, insert_duplicate_chunk_metadata_query,
};
use trieve_server::operators::group_operator::create_chunk_bookmark_query;
use trieve_server::operators::model_operator::create_embedding;
use trieve_server::operators::qdrant_operator::{
    create_new_qdrant_point_query, delete_qdrant_point_id_query, update_qdrant_point_query,
};
use trieve_server::operators::search_operator::global_unfiltered_top_match_query;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let redis_url = get_env!("REDIS_URL", "REDIS_URL is not set");
    let redis_client = redis::Client::open(redis_url).unwrap();
    let mut redis_connection = redis_client.get_connection().unwrap();
    let mut pubsub = redis_connection.as_pubsub();
    pubsub.subscribe("ingestion").unwrap();

    let database_url = get_env!("DATABASE_URL", "DATABASE_URL is not set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: models::Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let web_pool = actix_web::web::Data::new(pool.clone());

    loop {
        let msg = pubsub.get_message().unwrap();
        let payload: String = msg.get_payload().unwrap();
        println!("recieved payload");
        let mut payload: IngestionMessage = serde_json::from_str(&payload).unwrap();
        let embedding_vector = if let Some(embedding_vector) = payload.chunk.chunk_vector.clone() {
            embedding_vector
        } else {
            create_embedding(
                &payload.chunk_metadata.content,
                payload.dataset_config.clone(),
            )
            .await
            .map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Could not create embedding: {}", err),
                )
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
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Could not get top match: {}", err),
                )
            })?;

            if first_semantic_result.score >= duplicate_distance_threshold {
                //Sets collision to collided chunk id
                collision = Some(first_semantic_result.point_id);

                let score_chunk_result = get_metadata_from_point_ids(
                    vec![first_semantic_result.point_id],
                    web_pool.clone(),
                );

                match score_chunk_result {
                    Ok(chunk_results) => {
                        if chunk_results.is_empty() {
                            delete_qdrant_point_id_query(
                                first_semantic_result.point_id,
                                payload.chunk_metadata.dataset_id,
                            )
                            .await
                            .map_err(|_| {
                                std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    "Could not delete chunk metadata for chunk id",
                                )
                            })?;

                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Could not find chunk metadata for chunk id",
                            ));
                        }
                        chunk_results.first().unwrap().clone()
                    }
                    Err(err) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            err.to_string(),
                        ));
                    }
                };
            }
        }

        //if collision is not nil, insert chunk with collision
        if collision.is_some() {
            update_qdrant_point_query(
                None,
                collision.expect("Collision must be some"),
                Some(payload.chunk_metadata.author_id),
                None,
                payload.chunk_metadata.dataset_id,
            )
            .await
            .map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Could not update qdrant point: {}", err),
                )
            })?;

            insert_duplicate_chunk_metadata_query(
                payload.chunk_metadata.clone(),
                collision.expect("Collision should must be some"),
                payload.chunk.file_uuid,
                web_pool.clone(),
            )
            .map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Could not insert duplicate chunk metadata: {}", err),
                )
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
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Could not insert chunk metadata: {}", err),
                )
            })?;

            create_new_qdrant_point_query(
                qdrant_point_id,
                embedding_vector,
                payload.chunk_metadata.clone(),
                Some(payload.chunk_metadata.author_id),
                payload.chunk_metadata.dataset_id,
            )
            .await
            .map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Could not create qdrant point: {}", err),
                )
            })?;
        }

        if let Some(group_id_to_bookmark) = payload.chunk.group_id {
            let chunk_group_bookmark =
                ChunkGroupBookmark::from_details(group_id_to_bookmark, payload.chunk_metadata.id);

            create_chunk_bookmark_query(web_pool.clone(), chunk_group_bookmark).map_err(|err| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Could not create chunk bookmark: {}", err),
                )
            })?;
        }
    }
}
