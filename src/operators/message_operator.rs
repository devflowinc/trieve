use crate::diesel::prelude::*;
use crate::operators::stripe_customer_operator::get_user_plan_query;
use crate::operators::topic_operator::{get_topic_query, get_total_messages_for_user_query};
use crate::{
    data::models::{Message, Pool},
    errors::DefaultError,
};
use actix_web::web;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionDTO {
    pub completion_message: Message,
    pub completion_tokens: i32,
}

pub fn get_topic_messages(
    messages_topic_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<Message>, DefaultError> {
    use crate::data::schema::messages::dsl::*;

    let mut conn = pool.get().unwrap();

    let topic_messages = messages
        .filter(topic_id.eq(messages_topic_id))
        .filter(deleted.eq(false))
        .order(sort_order.asc())
        .load::<Message>(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error getting topic messages",
        })?;

    Ok(topic_messages)
}

pub fn user_owns_topic_query(
    user_given_id: uuid::Uuid,
    topic_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> bool {
    use crate::data::schema::topics::dsl::*;

    let mut conn = pool.get().unwrap();

    let topic = topics
        .filter(id.eq(topic_id))
        .filter(user_id.eq(user_id))
        .first::<crate::data::models::Topic>(&mut conn);
    if topic.is_err() {
        return false;
    }

    topic.unwrap().user_id == user_given_id
}

pub fn is_allowed_to_create_message_query(
    user_id: uuid::Uuid,
    user_email: String,
    pool: &web::Data<Pool>,
) -> Result<(), DefaultError> {
    let user_plan = get_user_plan_query(user_email, pool);
    let mut maximum_messages_allowed = 0;
    match user_plan {
        Ok(plan) => {
            if plan.plan == "silver" {
                maximum_messages_allowed = 1000;
            } else if plan.plan == "gold" {
                maximum_messages_allowed = 5000;
            }
        }
        Err(_error) => {
            maximum_messages_allowed = 2;
        }
    }
    let total_messages_for_user = get_total_messages_for_user_query(user_id, pool)?;

    if total_messages_for_user >= maximum_messages_allowed {
        return Err(DefaultError {
            message: "You must upgrade your plan to get more coaching",
        });
    };

    Ok(())
}

pub fn create_message_query(
    new_message: Message,
    given_user_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::messages::dsl::messages;

    let mut conn = pool.get().unwrap();

    match get_topic_query(new_message.topic_id, pool) {
        Ok(topic) if topic.user_id != given_user_id => {
            return Err(DefaultError {
                message: "Unauthorized",
            })
        }
        Ok(_topic) => {}
        Err(e) => return Err(e),
    };

    diesel::insert_into(messages)
        .values(&new_message)
        .execute(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error creating message, try again",
        })?;

    Ok(())
}

pub fn create_generic_system_and_prompt_message(
    messages_topic_id: uuid::Uuid,
    normal_chat: bool,
    pool: &web::Data<Pool>,
) -> Result<Vec<Message>, DefaultError> {
    let topic = crate::operators::topic_operator::get_topic_query(messages_topic_id, pool)?;
    let system_message_content = if normal_chat {
        "You are Arguflow Assistant, a large language model trained by Arguflow to be a helpful assistant."
    } else {
        "You are Arguflow Debate Coach, a large language model trained by Arguflow to coach students on their debate capabilities."
    };

    let system_message = Message::from_details(
        system_message_content,
        topic.id,
        0,
        "system".into(),
        Some(0),
        Some(0),
    );

    let user_message_content = if normal_chat {
        "Hi".to_string()
    } else {
        format!(
            "We are going to practice debating over the topic \"{}\". We are speaking to a judge. After my messages, you will respond exactly as follows:\n\nfeedback: {{aggressive feedback on how my argument could be improved}}\ncounterargument: {{A simulated counterargument, including evidence, that I can respond to in order to further practice my skills}}",
            topic.resolution,
        )
    };

    let user_message = Message::from_details(
        user_message_content,
        topic.id,
        0,
        "user".into(),
        Some(0),
        Some(0),
    );

    Ok(vec![system_message, user_message])
}

pub fn create_topic_message_query(
    previous_messages: Vec<Message>,
    new_message: Message,
    given_user_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<Message>, DefaultError> {
    let mut ret_messages = previous_messages.clone();
    let mut new_message_copy = new_message.clone();
    let mut previous_messages_len = previous_messages.len();

    if previous_messages.is_empty() {
        // Check if this is a normal chat or not
        let normal_chat = get_topic_query(new_message.topic_id, pool)?.normal_chat;
        let starter_messages =
            create_generic_system_and_prompt_message(new_message.topic_id, normal_chat, pool)?;
        ret_messages.extend(starter_messages.clone());
        for starter_message in starter_messages {
            create_message_query(starter_message, given_user_id, pool)?;
            previous_messages_len += 1;
        }
    }

    new_message_copy.sort_order = (previous_messages_len + 1).try_into().unwrap();

    create_message_query(new_message_copy.clone(), given_user_id, pool)?;
    ret_messages.push(new_message_copy);

    Ok(ret_messages)
}

pub fn get_message_by_sort_for_topic_query(
    message_topic_id: uuid::Uuid,
    message_sort_order: i32,
    pool: &web::Data<Pool>,
) -> Result<Message, DefaultError> {
    use crate::data::schema::messages::dsl::*;

    let mut conn = pool.get().unwrap();

    messages
        .filter(deleted.eq(false))
        .filter(topic_id.eq(message_topic_id))
        .filter(sort_order.eq(message_sort_order))
        .first::<Message>(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "This message does not exist for the authenticated user",
        })
}

pub fn get_messages_for_topic_query(
    message_topic_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Vec<Message>, DefaultError> {
    use crate::data::schema::messages::dsl::*;

    let mut conn = pool.get().unwrap();

    messages
        .filter(topic_id.eq(message_topic_id))
        .filter(deleted.eq(false))
        .order_by(sort_order.asc())
        .load::<Message>(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "This topic does not exist for the authenticated user",
        })
}

pub fn delete_message_query(
    given_user_id: &uuid::Uuid,
    given_message_id: uuid::Uuid,
    given_topic_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::messages::dsl::*;

    let mut conn = pool.get().unwrap();

    match get_topic_query(given_topic_id, pool) {
        Ok(topic) if topic.user_id != *given_user_id => {
            return Err(DefaultError {
                message: "Unauthorized",
            })
        }
        Ok(_topic) => {}
        Err(e) => return Err(e),
    };

    let target_message: Message = messages
        .find(given_message_id)
        .first::<Message>(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error finding message",
        })?;

    diesel::update(
        messages
            .filter(topic_id.eq(given_topic_id))
            .filter(sort_order.ge(target_message.sort_order)),
    )
    .set(deleted.eq(true))
    .execute(&mut conn)
    .map_err(|_| DefaultError {
        message: "Error deleting message",
    })?;

    Ok(())
}
