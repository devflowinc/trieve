use crate::data::models::{Pool, Topic};
use crate::{diesel::prelude::*, errors::ServiceError};
use actix_web::web;
use diesel_async::RunQueryDsl;

#[tracing::instrument(skip(pool))]
pub async fn create_topic_query(topic: Topic, pool: &web::Data<Pool>) -> Result<(), ServiceError> {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    diesel::insert_into(topics)
        .values(&topic)
        .execute(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest("Error inserting new topic, try again".to_string())
        })?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn delete_topic_query(
    topic_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    diesel::update(
        topics
            .filter(id.eq(topic_id))
            .filter(dataset_id.eq(given_dataset_id)),
    )
    .set(deleted.eq(true))
    .execute(&mut conn)
    .await
    .map_err(|_db_error| ServiceError::BadRequest("Error deleting topic, try again".to_string()))?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn update_topic_query(
    topic_id: uuid::Uuid,
    topic_name: String,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    diesel::update(
        topics
            .filter(id.eq(topic_id))
            .filter(dataset_id.eq(given_dataset_id)),
    )
    .set((name.eq(topic_name), updated_at.eq(diesel::dsl::now)))
    .execute(&mut conn)
    .await
    .map_err(|_db_error| ServiceError::BadRequest("Error updating topic, try again".to_string()))?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn get_topic_query(
    topic_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Topic, ServiceError> {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    topics
        .filter(id.eq(topic_id))
        .filter(deleted.eq(false))
        .filter(dataset_id.eq(given_dataset_id))
        .first::<Topic>(&mut conn)
        .await
        .map_err(|_db_error| ServiceError::BadRequest("This topic does not exist".to_string()))
}

#[tracing::instrument(skip(pool))]
pub async fn get_topic_for_owner_id_query(
    owner_id: String,
    topic_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Topic, ServiceError> {
    use crate::data::schema::topics::dsl as topics_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    topics_columns::topics
        .filter(topics_columns::id.eq(topic_id))
        .filter(topics_columns::owner_id.eq(owner_id))
        .filter(topics_columns::deleted.eq(false))
        .filter(topics_columns::dataset_id.eq(given_dataset_id))
        .first::<Topic>(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest(
                "This topic does not exist for the specified owner_id".to_string(),
            )
        })
}

#[tracing::instrument(skip(pool))]
pub async fn get_all_topics_for_owner_id_query(
    topic_owner_id: String,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<Topic>, ServiceError> {
    use crate::data::schema::topics::dsl as topics_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    topics_columns::topics
        .filter(topics_columns::owner_id.eq(topic_owner_id))
        .filter(topics_columns::dataset_id.eq(given_dataset_id))
        .filter(topics_columns::deleted.eq(false))
        .order(topics_columns::updated_at.desc())
        .load::<Topic>(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest("Error getting topics for the specified owner_id".to_string())
        })
}
