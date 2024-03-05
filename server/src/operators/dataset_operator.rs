use crate::data::models::{DatasetAndUsage, DatasetUsageCount, ServerDatasetConfiguration};
use crate::diesel::RunQueryDsl;
use crate::get_env;
use crate::operators::qdrant_operator::get_qdrant_connection;
use crate::{
    data::models::{Dataset, Pool},
    errors::ServiceError,
};
use actix_web::web;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use qdrant_client::qdrant::{Condition, Filter};

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
    config: ServerDatasetConfiguration,
) -> Result<(), ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let dataset = get_dataset_by_id_query(id, pool.clone()).await?;

    let redis_url = get_env!("REDIS_URL", "REDIS_URL must be set");
    let client = redis::Client::open(redis_url).map_err(|err| {
        ServiceError::BadRequest(format!("Could not create redis client: {}", err))
    })?;
    let mut redis_conn = client
        .get_async_connection()
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Could not connect to redis: {}", err)))?;
    redis::cmd("DEL")
        .arg(format!("dataset:{}", dataset.id))
        .query_async(&mut redis_conn)
        .await
        .map_err(|err| {
            ServiceError::BadRequest(format!("Could not delete dataset in redis: {}", err))
        })?;

    let qdrant_collection = config.QDRANT_COLLECTION_NAME;

    let qdrant = get_qdrant_connection(Some(&config.QDRANT_URL), Some(&config.QDRANT_API_KEY))
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Could not connect to qdrant: {}", err)))?;

    qdrant
        .delete_points(
            qdrant_collection,
            None,
            &Filter::must([Condition::matches("dataset_id", id.to_string())]).into(),
            None,
        )
        .await
        .map_err(|err| {
            ServiceError::BadRequest(format!("Could not delete points from qdrant: {}", err))
        })?;

    let mut conn = pool.get().unwrap();

    diesel::delete(datasets_columns::datasets.filter(datasets_columns::id.eq(id)))
        .execute(&mut conn)
        .map_err(|err| {
            log::error!("Could not delete dataset: {}", err);
            ServiceError::BadRequest("Could not delete dataset".to_string())
        })?;

    Ok(())
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
    org_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<DatasetAndUsage>, ServiceError> {
    use crate::data::schema::dataset_usage_counts::dsl as dataset_usage_counts_columns;
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let dataset_and_usages: Vec<(Dataset, DatasetUsageCount)> = datasets_columns::datasets
        .inner_join(dataset_usage_counts_columns::dataset_usage_counts)
        .filter(datasets_columns::organization_id.eq(org_id.into_inner()))
        .select((Dataset::as_select(), DatasetUsageCount::as_select()))
        .load::<(Dataset, DatasetUsageCount)>(&mut conn)
        .map_err(|_| ServiceError::BadRequest("Could not find dataset".to_string()))?;

    let dataset_and_usages = dataset_and_usages
        .into_iter()
        .map(|(dataset, usage_count)| DatasetAndUsage::from_components(dataset.into(), usage_count))
        .collect::<Vec<DatasetAndUsage>>();

    Ok(dataset_and_usages)
}
