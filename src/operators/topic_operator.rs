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
            message: "Error inserting new topic, try again".into(),
        })?;

    Ok(())
}
