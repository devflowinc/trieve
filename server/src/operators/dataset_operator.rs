use crate::data::models::{ChunkCollision, ChunkMetadata, DatasetAndUsage, DatasetUsageCount};
use crate::diesel::RunQueryDsl;

use crate::operators::file_operator::delete_file_query;
use crate::operators::qdrant_operator::get_qdrant_connection;
use crate::{
    data::models::{Dataset, Pool},
    errors::ServiceError,
};
use actix_web::web;
use diesel::{Connection, ExpressionMethods, JoinOnDsl, QueryDsl, SelectableHelper};
use qdrant_client::qdrant::PointId;

pub async fn create_dataset_query(
    new_dataset: Dataset,
    pool: web::Data<Pool>,
) -> Result<Dataset, ServiceError> {
    use crate::data::schema::datasets::dsl::*;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    diesel::insert_into(datasets)
        .values(&new_dataset)
        .execute(&mut conn)
        .map_err(|_| ServiceError::BadRequest("Failed to create dataset".to_string()))?;

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = redis::Client::open(redis_url).map_err(|err| {
        ServiceError::BadRequest(format!("Could not create redis client: {}", err))
    })?;
    let mut redis_conn = client
        .get_async_connection()
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Could not connect to redis: {}", err)))?;
    redis::cmd("SET")
        .arg(format!("dataset:{}", new_dataset.id))
        .arg(serde_json::to_string(&new_dataset).map_err(|err| {
            ServiceError::BadRequest(format!("Could not stringify dataset: {}", err))
        })?)
        .query_async(&mut redis_conn)
        .await
        .map_err(|err| {
            ServiceError::BadRequest(format!("Could not set dataset in redis: {}", err))
        })?;

    Ok(new_dataset)
}

pub async fn get_dataset_by_id_query(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Dataset, ServiceError> {
    // Check cache first
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = redis::Client::open(redis_url)
        .map_err(|_| ServiceError::BadRequest("Could not create redis client".to_string()))?;
    let mut redis_conn = redis_client
        .get_async_connection()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get redis connection".to_string()))?;

    let redis_dataset: Result<String, ServiceError> = redis::cmd("GET")
        .arg(format!("dataset:{}", id))
        .query_async(&mut redis_conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get dataset from redis".to_string()));

    match redis_dataset {
        Ok(dataset) => Ok(serde_json::from_str::<Dataset>(&dataset).map_err(|_| {
            ServiceError::BadRequest("Could not parse dataset from redis".to_string())
        })?),
        Err(_) => {
            use crate::data::schema::datasets::dsl as datasets_columns;
            let mut conn = pool.get().map_err(|_| {
                ServiceError::BadRequest("Could not get database connection".to_string())
            })?;

            let dataset: Dataset = datasets_columns::datasets
                .filter(datasets_columns::id.eq(id))
                .select(Dataset::as_select())
                .first(&mut conn)
                .map_err(|_| ServiceError::BadRequest("Could not find dataset".to_string()))?;

            let dataset_stringified = serde_json::to_string(&dataset)
                .map_err(|_| ServiceError::BadRequest("Could not stringify dataset".to_string()))?;

            redis::cmd("SET")
                .arg(format!("dataset:{}", dataset.id))
                .arg(dataset_stringified)
                .query_async(&mut redis_conn)
                .await
                .map_err(|_| {
                    ServiceError::BadRequest("Could not set dataset in redis".to_string())
                })?;

            Ok(dataset)
        }
    }
}

pub async fn delete_dataset_by_id_query(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::datasets::dsl as datasets_columns;
    use crate::data::schema::files::dsl as files_columns;
    use crate::data::schema::topics::dsl as topics_columns;
    use crate::data::schema::{
        chunk_collisions::dsl as chunk_collisions_columns, chunk_files::dsl as chunk_files_columns,
    };
    use crate::data::schema::{
        chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns, events::dsl as events_columns,
        groups_from_files::dsl as groups_from_files_columns,
    };

    let dataset = get_dataset_by_id_query(id, pool.clone()).await?;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let file_ids = files_columns::files
        .select(files_columns::id)
        .filter(files_columns::dataset_id.eq(dataset.id))
        .load::<uuid::Uuid>(&mut conn)
        .map_err(|_| ServiceError::BadRequest("Could not find files".to_string()))?;

    for file_id in file_ids {
        delete_file_query(file_id, dataset.clone(), Some(false), pool.clone())
            .await
            .map_err(|e| {
                log::error!("Failed to delete files for dataset: {}", e);
                ServiceError::BadRequest("Failed to delete files for dataset".to_string())
            })?;
    }

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::delete(events_columns::events.filter(events_columns::dataset_id.eq(id)))
            .execute(conn)?;

        let group_ids = chunk_group_columns::chunk_group
            .select(chunk_group_columns::id)
            .filter(chunk_group_columns::dataset_id.eq(dataset.id))
            .load::<uuid::Uuid>(conn)?;

        diesel::delete(
            groups_from_files_columns::groups_from_files
                .filter(groups_from_files_columns::group_id.eq_any(group_ids.clone())),
        )
        .execute(conn)?;

        diesel::delete(
            chunk_group_bookmarks_columns::chunk_group_bookmarks
                .filter(chunk_group_bookmarks_columns::group_id.eq_any(group_ids.clone())),
        )
        .execute(conn)?;

        diesel::delete(
            chunk_group_columns::chunk_group
                .filter(chunk_group_columns::id.eq_any(group_ids.clone()))
                .filter(chunk_group_columns::dataset_id.eq(id)),
        )
        .execute(conn)?;
        let chunks = chunk_metadata_columns::chunk_metadata
            .select(ChunkMetadata::as_select())
            .filter(chunk_metadata_columns::dataset_id.eq(dataset.id))
            .load::<ChunkMetadata>(conn)?;

        let chunk_ids = chunks
            .iter()
            .map(|chunk| chunk.id)
            .collect::<Vec<uuid::Uuid>>();

        diesel::delete(
            chunk_files_columns::chunk_files
                .filter(chunk_files_columns::chunk_id.eq_any(chunk_ids.clone())),
        )
        .execute(conn)?;

        diesel::delete(
            chunk_group_bookmarks_columns::chunk_group_bookmarks
                .filter(chunk_group_bookmarks_columns::chunk_metadata_id.eq_any(chunk_ids.clone())),
        )
        .execute(conn)?;

        let deleted_chunk_collision_count = diesel::delete(
            chunk_collisions_columns::chunk_collisions
                .filter(chunk_collisions_columns::chunk_id.eq_any(chunk_ids.clone())),
        )
        .execute(conn)?;

        if deleted_chunk_collision_count > 0 {
            // there cannot be collisions for a collision, just delete the chunk_metadata without issue
            diesel::delete(
                chunk_metadata_columns::chunk_metadata
                    .filter(chunk_metadata_columns::id.eq_any(chunk_ids))
                    .filter(chunk_metadata_columns::dataset_id.eq(id)),
            )
            .execute(conn)?;
        } else {
            let chunk_collisions: Vec<(ChunkCollision, ChunkMetadata)> =
                chunk_collisions_columns::chunk_collisions
                    .inner_join(
                        chunk_metadata_columns::chunk_metadata
                            .on(chunk_metadata_columns::qdrant_point_id
                                .eq(chunk_collisions_columns::collision_qdrant_id)),
                    )
                    .filter(chunk_metadata_columns::id.eq_any(chunk_ids.clone()))
                    .filter(chunk_metadata_columns::dataset_id.eq(id))
                    .select((ChunkCollision::as_select(), ChunkMetadata::as_select()))
                    .order_by(chunk_collisions_columns::created_at.asc())
                    .load::<(ChunkCollision, ChunkMetadata)>(conn)?;

            if !chunk_collisions.is_empty() {
                let chunk_metadata_ids = chunk_collisions
                    .iter()
                    .map(|(_, chunk_metadata)| chunk_metadata.id)
                    .collect::<Vec<uuid::Uuid>>();

                diesel::delete(
                    chunk_metadata_columns::chunk_metadata
                        .filter(chunk_metadata_columns::id.eq_any(chunk_metadata_ids))
                        .filter(chunk_metadata_columns::dataset_id.eq(id)),
                )
                .execute(conn)?;

                let collision_ids = chunk_collisions
                    .iter()
                    .map(|(chunk_collision, _)| chunk_collision.id)
                    .collect::<Vec<uuid::Uuid>>();

                diesel::delete(
                    chunk_collisions_columns::chunk_collisions
                        .filter(chunk_collisions_columns::id.eq_any(collision_ids)),
                )
                .execute(conn)?;
            }

            // if there were no collisions, just delete the chunk_metadata without issue
            diesel::delete(
                chunk_metadata_columns::chunk_metadata
                    .filter(chunk_metadata_columns::id.eq_any(chunk_ids.clone()))
                    .filter(chunk_metadata_columns::dataset_id.eq(dataset.id)),
            )
            .execute(conn)?;
        }

        diesel::delete(topics_columns::topics.filter(topics_columns::dataset_id.eq(id)))
            .execute(conn)?;

        diesel::delete(datasets_columns::datasets)
            .filter(datasets_columns::id.eq(id))
            .execute(conn)?;

        Ok(chunks)
    });

    let qdrant_group = std::env::var("QDRANT_COLLECTION").unwrap_or("debate_chunks".to_owned());

    match transaction_result {
        Ok(chunks) => {
            let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
            let client = redis::Client::open(redis_url).map_err(|err| {
                ServiceError::BadRequest(format!("Could not create redis client: {}", err))
            })?;
            let mut redis_conn = client.get_async_connection().await.map_err(|err| {
                ServiceError::BadRequest(format!("Could not connect to redis: {}", err))
            })?;
            redis::cmd("DEL")
                .arg(format!("dataset:{}", dataset.id))
                .query_async(&mut redis_conn)
                .await
                .map_err(|err| {
                    ServiceError::BadRequest(format!("Could not delete dataset in redis: {}", err))
                })?;

            let qdrant = get_qdrant_connection().await.map_err(|err| {
                ServiceError::BadRequest(format!("Could not connect to qdrant: {}", err))
            })?;

            let selector = chunks
                .iter()
                .map(|chunk| {
                    <String as Into<PointId>>::into(
                        chunk.qdrant_point_id.unwrap_or_default().to_string(),
                    )
                })
                .collect::<Vec<PointId>>();

            let _ = qdrant
                .delete_points(qdrant_group, None, &selector.into(), None)
                .await
                .map_err(|err| {
                    ServiceError::BadRequest(format!(
                        "Could not delete points from qdrant: {}",
                        err
                    ))
                })?;

            Ok(())
        }
        Err(e) => {
            log::error!("Failed to delete dataset: {}", e);
            Err(ServiceError::BadRequest(
                "Failed to delete dataset".to_string(),
            ))
        }
    }
}

pub async fn update_dataset_query(
    id: uuid::Uuid,
    name: String,
    server_configuration: serde_json::Value,
    client_configuration: serde_json::Value,
    pool: web::Data<Pool>,
) -> Result<Dataset, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    // TODO update columns that are not listed
    let new_dataset: Dataset =
        diesel::update(datasets_columns::datasets.filter(datasets_columns::id.eq(id)))
            .set((
                datasets_columns::name.eq(name),
                datasets_columns::updated_at.eq(diesel::dsl::now),
                datasets_columns::server_configuration.eq(server_configuration),
                datasets_columns::client_configuration.eq(client_configuration),
            ))
            .get_result(&mut conn)
            .map_err(|_| ServiceError::BadRequest("Failed to update dataset".to_string()))?;

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");

    let client = redis::Client::open(redis_url).map_err(|err| {
        ServiceError::BadRequest(format!("Could not create redis client: {}", err))
    })?;

    let mut redis_conn = client
        .get_async_connection()
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Could not connect to redis: {}", err)))?;

    redis::cmd("SET")
        .arg(format!("dataset:{}", id))
        .arg(serde_json::to_string(&new_dataset).map_err(|err| {
            ServiceError::BadRequest(format!("Could not stringify dataset: {}", err))
        })?)
        .query_async(&mut redis_conn)
        .await
        .map_err(|err| {
            ServiceError::BadRequest(format!("Could not set dataset in redis: {}", err))
        })?;

    Ok(new_dataset)
}

pub fn get_datasets_by_organization_id(
    id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<DatasetAndUsage>, ServiceError> {
    use crate::data::schema::dataset_usage_counts::dsl as dataset_usage_counts_columns;
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let dataset_and_usages: Vec<(Dataset, DatasetUsageCount)> = datasets_columns::datasets
        .inner_join(dataset_usage_counts_columns::dataset_usage_counts)
        .filter(datasets_columns::organization_id.eq(id.into_inner()))
        .select((Dataset::as_select(), DatasetUsageCount::as_select()))
        .load::<(Dataset, DatasetUsageCount)>(&mut conn)
        .map_err(|_| ServiceError::BadRequest("Could not find dataset".to_string()))?;

    let dataset_and_usages = dataset_and_usages
        .into_iter()
        .map(|(dataset, usage_count)| DatasetAndUsage::from_components(dataset.into(), usage_count))
        .collect::<Vec<DatasetAndUsage>>();

    Ok(dataset_and_usages)
}
