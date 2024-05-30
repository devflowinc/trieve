use crate::diesel::prelude::*;
use crate::{
    data::models::{Message, Pool},
    errors::ServiceError,
};
use actix_web::web;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionDTO {
    pub completion_message: Message,
    pub completion_tokens: i32,
}

#[tracing::instrument(skip(pool))]
pub async fn get_topic_messages(
    messages_topic_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<Message>, ServiceError> {
    use crate::data::schema::messages::dsl::*;

    let mut conn = pool.get().await.unwrap();

    let topic_messages = messages
        .filter(topic_id.eq(messages_topic_id))
        .filter(dataset_id.eq(given_dataset_id))
        .filter(deleted.eq(false))
        .order(sort_order.asc())
        .load::<Message>(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest("Error getting topic messages".to_string())
        })?;

    Ok(topic_messages)
}

#[tracing::instrument(skip(pool))]
pub async fn create_message_query(
    new_message: Message,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::messages::dsl::messages;

    let mut conn = pool.get().await.unwrap();

    diesel::insert_into(messages)
        .values(&new_message)
        .execute(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest("Error creating message, try again".to_string())
        })?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn create_generic_system_message(
    messages_topic_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Message, ServiceError> {
    let topic =
        crate::operators::topic_operator::get_topic_query(messages_topic_id, dataset_id, pool)
            .await?;
    let system_message_content =
        "You are Trieve retrieval augmented chatbot, a large language model trained by Trieve to respond in the same tone as and with the context of retrieved information.";

    let system_message = Message::from_details(
        system_message_content,
        topic.id,
        0,
        "system".into(),
        Some(0),
        Some(0),
        dataset_id,
    );

    Ok(system_message)
}

#[tracing::instrument(skip(pool))]
pub async fn create_topic_message_query(
    previous_messages: Vec<Message>,
    new_message: Message,
    dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<Message>, ServiceError> {
    let mut ret_messages = previous_messages.clone();
    let mut new_message_copy = new_message.clone();
    let mut previous_messages_len = previous_messages.len();

    if previous_messages.is_empty() {
        let system_message =
            create_generic_system_message(new_message.topic_id, dataset_id, pool).await?;
        ret_messages.extend(vec![system_message.clone()]);
        create_message_query(system_message, pool).await?;
        previous_messages_len = 1;
    }

    new_message_copy.sort_order = previous_messages_len as i32;

    create_message_query(new_message_copy.clone(), pool).await?;
    ret_messages.push(new_message_copy);

    Ok(ret_messages)
}

#[tracing::instrument(skip(pool))]
pub async fn get_message_by_sort_for_topic_query(
    message_topic_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    message_sort_order: i32,
    pool: &web::Data<Pool>,
) -> Result<Message, ServiceError> {
    use crate::data::schema::messages::dsl::*;

    let mut conn = pool.get().await.unwrap();

    messages
        .filter(deleted.eq(false))
        .filter(topic_id.eq(message_topic_id))
        .filter(sort_order.eq(message_sort_order))
        .filter(dataset_id.eq(given_dataset_id))
        .first::<Message>(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest(
                "This message does not exist for the authenticated user".to_string(),
            )
        })
}

#[tracing::instrument(skip(pool))]
pub async fn get_messages_for_topic_query(
    message_topic_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<Message>, ServiceError> {
    use crate::data::schema::messages::dsl::*;

    let mut conn = pool.get().await.unwrap();

    messages
        .filter(topic_id.eq(message_topic_id))
        .filter(deleted.eq(false))
        .filter(dataset_id.eq(given_dataset_id))
        .order_by(sort_order.asc())
        .load::<Message>(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest(
                "This topic does not exist for the authenticated user".to_string(),
            )
        })
}

#[tracing::instrument(skip(pool))]
pub async fn delete_message_query(
    given_message_id: uuid::Uuid,
    given_topic_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::messages::dsl::*;

    let mut conn = pool.get().await.unwrap();

    let target_message: Message = messages
        .find(given_message_id)
        .first::<Message>(&mut conn)
        .await
        .map_err(|_db_error| ServiceError::BadRequest("Error finding message".to_string()))?;

    diesel::update(
        messages
            .filter(topic_id.eq(given_topic_id))
            .filter(dataset_id.eq(given_dataset_id))
            .filter(sort_order.ge(target_message.sort_order)),
    )
    .set(deleted.eq(true))
    .execute(&mut conn)
    .await
    .map_err(|_| ServiceError::BadRequest("Error deleting message".to_string()))?;

    Ok(())
}
