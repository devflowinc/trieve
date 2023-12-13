use crate::diesel::RunQueryDsl;
use crate::errors::DefaultError;
use crate::{
    data::models::{Dataset, Pool},
    errors::ServiceError,
};
use actix_web::web;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use tokio::sync::RwLock;

use super::tantivy_operator::TantivyIndexMap;

/// Creates all indexes between Pg, Qdrant and tantivy
pub async fn new_dataset_operation(
    dataset: Dataset,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    pool: web::Data<Pool>,
) -> Result<Dataset, ServiceError> {
    tantivy_index_map
        .write()
        .await
        .create_index(&dataset.id.to_string())
        .map_err(|err| {
            ServiceError::BadRequest(format!(
                "Failed to create tantivy index: {:?}",
                err.to_string()
            ))
        })?;

    create_dataset_query(dataset, pool).await
}

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

#[allow(dead_code)]
pub async fn get_dataset_by_id_query(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Dataset, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let organization: Dataset = datasets_columns::datasets
        .filter(datasets_columns::id.eq(id))
        .select(Dataset::as_select())
        .first(&mut conn)
        .map_err(|_| ServiceError::BadRequest("Could not find dataset".to_string()))?;

    Ok(organization)
}

pub async fn load_datasets_into_redis(pool: Pool) -> Result<(), DefaultError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = redis::Client::open(redis_url).map_err(|_| DefaultError {
        message: "Could not create redis client",
    })?;
    let mut redis_conn = client
        .get_async_connection()
        .await
        .map_err(|_| DefaultError {
            message: "Could not connect to redis",
        })?;

    let mut pg_conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let datasets: Vec<Dataset> = datasets_columns::datasets
        .select(Dataset::as_select())
        .load(&mut pg_conn)
        .map_err(|_| DefaultError {
            message: "Could not load datasets from database",
        })?;

    for dataset in datasets {
        let dataset_stringified = serde_json::to_string(&dataset).map_err(|_| DefaultError {
            message: "Could not stringify dataset",
        })?;

        redis::cmd("SET")
            .arg(format!("dataset:{}", dataset.id))
            .arg(dataset_stringified)
            .query_async(&mut redis_conn)
            .await
            .map_err(|_| DefaultError {
                message: "Could not set dataset in redis",
            })?;
    }

    Ok(())
}

pub async fn delete_dataset_by_id_query(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    diesel::delete(datasets_columns::datasets)
        .filter(datasets_columns::id.eq(id))
        .execute(&mut conn)
        .map_err(|_| ServiceError::BadRequest("Failed to delete dataset".to_string()))?;

    Ok(())
}

pub async fn update_dataset_query(
    id: uuid::Uuid,
    name: String,
    pool: web::Data<Pool>,
) -> Result<Dataset, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let new_dataset: Dataset =
        diesel::update(datasets_columns::datasets.filter(datasets_columns::id.eq(id)))
            .set((
                datasets_columns::name.eq(name),
                datasets_columns::updated_at.eq(diesel::dsl::now),
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
