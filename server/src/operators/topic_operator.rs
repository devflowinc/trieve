use crate::data::models::{Pool, Topic};
use crate::{diesel::prelude::*, errors::ServiceError};
use actix_web::web;
use diesel_async::RunQueryDsl;

#[tracing::instrument(skip(pool))]
pub async fn create_topic_query(topic: Topic, pool: &web::Data<Pool>) -> Result<(), ServiceError> {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().await.unwrap();

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

    let mut conn = pool.get().await.unwrap();

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

    let mut conn = pool.get().await.unwrap();

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

    let mut conn = pool.get().await.unwrap();

    topics
        .filter(id.eq(topic_id))
        .filter(deleted.eq(false))
        .filter(dataset_id.eq(given_dataset_id))
        .first::<Topic>(&mut conn)
        .await
        .map_err(|_db_error| ServiceError::BadRequest("This topic does not exist".to_string()))
}

#[tracing::instrument(skip(pool))]
pub async fn get_topic_for_user_query(
    topic_user_id: uuid::Uuid,
    topic_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Topic, ServiceError> {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().await.unwrap();

    topics
        .filter(id.eq(topic_id))
        .filter(user_id.eq(topic_user_id))
        .filter(deleted.eq(false))
        .filter(dataset_id.eq(given_dataset_id))
        .first::<Topic>(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest(
                "This topic does not exist for the authenticated user".to_string(),
            )
        })
}

#[tracing::instrument(skip(pool))]
pub async fn get_all_topics_for_user_query(
    topic_user_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<Topic>, ServiceError> {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().await.unwrap();

    topics
        .filter(user_id.eq(topic_user_id))
        .filter(dataset_id.eq(given_dataset_id))
        .filter(deleted.eq(false))
        .order(updated_at.desc())
        .load::<Topic>(&mut conn)
        .await
        .map_err(|_db_error| ServiceError::BadRequest("Error getting topics for user".to_string()))
}
