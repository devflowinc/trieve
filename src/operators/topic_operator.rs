use crate::data::models::{Pool, Topic};
use crate::{diesel::prelude::*, errors::DefaultError};
use actix_web::web;

pub fn create_topic_query(topic: Topic, pool: &web::Data<Pool>) -> Result<(), DefaultError> {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::insert_into(topics)
        .values(&topic)
        .execute(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error inserting new topic, try again",
        })?;

    Ok(())
}

pub fn delete_topic_query(
    topic_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::update(topics.filter(id.eq(topic_id)))
        .set(deleted.eq(true))
        .execute(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error deleting topic, try again",
        })?;

    Ok(())
}

pub fn update_topic_query(
    topic_id: uuid::Uuid,
    topic_resolution: String,
    topic_side: bool,
    pool: &web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::update(topics.filter(id.eq(topic_id)))
        .set((
            resolution.eq(topic_resolution),
            side.eq(topic_side),
            updated_at.eq(diesel::dsl::now),
        ))
        .execute(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error updating topic, try again",
        })?;

    Ok(())
}

pub fn get_topic_query(
    topic_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Topic, DefaultError> {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().unwrap();

    topics
        .filter(id.eq(topic_id))
        .filter(deleted.eq(false))
        .first::<Topic>(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "This topic does not exist",
        })
}

pub fn get_topic_for_user_query(
    topic_user_id: uuid::Uuid,
    topic_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Topic, DefaultError> {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().unwrap();

    topics
        .filter(id.eq(topic_id))
        .filter(user_id.eq(topic_user_id))
        .first::<Topic>(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "This topic does not exist for the authenticated user",
        })
}

pub fn get_all_topics_for_user_query(
    topic_user_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<Topic>, DefaultError> {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().unwrap();

    topics
        .filter(user_id.eq(topic_user_id))
        .filter(deleted.eq(false))
        .order(updated_at.desc())
        .load::<Topic>(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error getting topics for user",
        })
}

pub fn get_total_messages_for_user_query(
    user_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<i64, DefaultError> {
    use crate::data::schema::messages::dsl::{id as message_id, messages};
    use crate::data::schema::topics::dsl::{topics, user_id as topic_user_id};

    let mut conn = pool.get().unwrap();

    topics
        .filter(topic_user_id.eq(user_id))
        .inner_join(messages)
        .select(diesel::dsl::count(message_id))
        .first::<i64>(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error getting total messages for user",
        })
}
