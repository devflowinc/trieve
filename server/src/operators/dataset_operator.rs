use crate::diesel::RunQueryDsl;
use crate::{
    data::models::{Dataset, Pool},
    errors::ServiceError,
};
use actix_web::web;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};

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

    let client = redis::Client::open(redis_url)
        .map_err(|_| ServiceError::BadRequest("Could not create redis client".to_string()))?;

    let mut redis_conn = client
        .get_async_connection()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get redis connection".to_string()))?;

    let redis_dataset: Result<String, ServiceError> = {
        redis::cmd("GET")
            .arg(format!("dataset:{}", id))
            .query_async(&mut redis_conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Could not get dataset from redis".to_string()))
    };

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
    configuration: serde_json::Value,
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
                datasets_columns::configuration.eq(configuration),
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
) -> Result<Vec<Dataset>, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let dataset = datasets_columns::datasets
        .filter(datasets_columns::organization_id.eq(id.into_inner()))
        .select(Dataset::as_select())
        .load::<Dataset>(&mut conn)
        .map_err(|_| ServiceError::BadRequest("Could not find dataset".to_string()))?;

    Ok(dataset)
}
