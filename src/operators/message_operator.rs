use crate::diesel::prelude::*;
use crate::{
    data::models::{Message, Pool},
    errors::DefaultError,
};
use actix_web::web;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::chat_completion::{ChatMessage, ChatCompletionParameters};
use openai_dive::v1::resources::chat_completion_stream::ChatCompletionStreamResponse;

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
            message: "Error getting topic messages".into(),
        })?;

    Ok(topic_messages)
}

pub fn create_message_query(
    new_message: Message,
    pool: &web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::messages::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::insert_into(messages)
        .values(&new_message)
        .execute(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error creating message, try again".into(),
        })?;

    Ok(())
}

pub fn create_system_message(
    topic_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Message, DefaultError> {
    let topic = crate::operators::topic_operator::get_topic_query(topic_id, pool)?;
    let system_message_content = format!(
        "We are going to practice lincoln douglas debate over \"{}\". You will be {}. We are speaking to a judge. After my messages, you will respond exactly as follows:\n\nfeedback: {{aggressive feedback on how my argument could be improved}}\ncounterargument: {{A simulated counterargument, including evidence, that I can respond to in order to further practice my skills}}",
        topic.resolution,
        if topic.side { "affirming the resolution" } else { "negating the resolution" }
    );

    let system_message = Message::from_details(
        system_message_content,
        topic.id,
        0,
        "system".into(),
        Some(0),
        Some(0),
    );

    Ok(system_message)
}

pub async fn create_topic_message(
    previous_messages: Vec<Message>,
    new_message: Message,
    pool: &web::Data<Pool>,
) -> Result<ChatCompletionStreamResponse, DefaultError> {
    let mut open_ai_messages: Vec<ChatMessage> = previous_messages
        .iter()
        .map(|message| message.to_open_ai_message())
        .collect();

    if open_ai_messages.len() == 0 {
        let system_message = create_system_message(new_message.topic_id, pool)?;
        create_message_query(system_message.clone(), pool)?;
        open_ai_messages.push(system_message.to_open_ai_message());
    }

    create_message_query(new_message.clone(), pool)?;
    let new_open_ai_message = new_message.to_open_ai_message();
    open_ai_messages.push(new_open_ai_message);

    let open_ai_api_key = std::env::var("OPEN_AI_API_KEY").expect("OPEN_AI_API_KEY must be set");
    let client = Client::new(open_ai_api_key);

    let parameters = ChatCompletionParameters {
        model: "gpt-3.5-turbo".into(),
        messages: open_ai_messages,
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        max_tokens: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
    };

    let mut stream = client.chat().create_stream(parameters).await.map_err(|_error| DefaultError {
        message: "Error creating open ai message".into(),
    }).unwrap();

    


    Ok(stream.next().await.unwrap())
}
