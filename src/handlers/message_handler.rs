use std::{thread, time::Duration};

use crate::{
    data::models,
    data::models::Pool,
    errors::{DefaultError, ServiceError},
    operators::{
        card_operator::{
            create_embedding, get_metadata_and_collided_cards_from_point_ids_query,
            search_card_query,
        },
        message_operator::{
            create_cut_card, create_message_query, create_topic_message_query,
            delete_message_query, get_message_by_sort_for_topic_query,
            get_messages_for_topic_query, get_topic_messages, user_owns_topic_query,
        },
    },
    AppMutexStore,
};
use actix::Arbiter;
use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};
use crossbeam_channel::unbounded;
use futures_util::stream;
use openai_dive::v1::{
    api::Client,
    resources::{
        chat_completion::{ChatCompletionParameters, ChatMessage, Role},
        completion::CompletionParameters,
        shared::StopToken,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_stream::StreamExt;

use super::auth_handler::LoggedUser;

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateMessageData {
    pub new_message_content: String,
    pub topic_id: uuid::Uuid,
}

pub async fn create_message_completion_handler(
    data: web::Json<CreateMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    mutex_store: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    let create_message_data = data.into_inner();
    let new_message = models::Message::from_details(
        create_message_data.new_message_content,
        create_message_data.topic_id,
        0,
        "user".to_string(),
        None,
        None,
    );
    let topic_id = create_message_data.topic_id;
    let second_pool = pool.clone();
    let third_pool = pool.clone();
    let fourth_pool = pool.clone();

    let user_owns_topic = web::block(move || user_owns_topic_query(user.id, topic_id, &pool));
    if let Ok(false) = user_owns_topic.await {
        return Ok(HttpResponse::Unauthorized().json("Unauthorized"));
    }

    // get the previous messages
    let mut previous_messages = web::block(move || get_topic_messages(topic_id, &second_pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    // remove citations from the previous messages
    previous_messages = previous_messages
        .into_iter()
        .map(|message| {
            let mut message = message;
            if message.role == "assistant" {
                message.content = message
                    .content
                    .split("||")
                    .last()
                    .unwrap_or("I give up, I can't find a citation")
                    .to_string();
            }
            message
        })
        .collect::<Vec<models::Message>>();

    // call create_topic_message_query with the new_message and previous_messages
    let new_topic_message_result = web::block(move || {
        create_topic_message_query(previous_messages, new_message, user.id, &third_pool)
    })
    .await?;
    let previous_messages = match new_topic_message_result {
        Ok(messages) => messages,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    stream_response(
        previous_messages,
        user.id,
        topic_id,
        fourth_pool,
        mutex_store,
    )
    .await
}

// get_all_topic_messages_handler
// verify that the user owns the topic for the topic_id they are requesting
// get all the messages for the topic_id
// filter out deleted messages
// return the messages
pub async fn get_all_topic_messages(
    user: LoggedUser,
    messages_topic_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let second_pool = pool.clone();
    let topic_id: uuid::Uuid = messages_topic_id.into_inner();
    // check if the user owns the topic
    let user_owns_topic =
        web::block(move || user_owns_topic_query(user.id, topic_id, &second_pool));
    if let Ok(false) = user_owns_topic.await {
        return Ok(HttpResponse::Unauthorized().json("Unauthorized"));
    }

    let messages = web::block(move || get_messages_for_topic_query(topic_id, &pool)).await?;

    match messages {
        Ok(messages) => Ok(HttpResponse::Ok().json(messages)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RegenerateMessageData {
    topic_id: uuid::Uuid,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EditMessageData {
    topic_id: uuid::Uuid,
    message_sort_order: i32,
    new_message_content: String,
}

pub async fn edit_message_handler(
    data: web::Json<EditMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    mutex_store: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = data.topic_id;
    let message_sort_order = data.message_sort_order;
    let new_message_content = &data.new_message_content;
    let second_pool = pool.clone();
    let third_pool = pool.clone();

    let message_from_sort_order_result = web::block(move || {
        get_message_by_sort_for_topic_query(topic_id, message_sort_order, &pool)
    })
    .await?;

    let message_id = match message_from_sort_order_result {
        Ok(message) => message.id,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    let _ = web::block(move || delete_message_query(&user.id, message_id, topic_id, &second_pool))
        .await?;

    create_message_completion_handler(
        actix_web::web::Json(CreateMessageData {
            new_message_content: new_message_content.to_string(),
            topic_id,
        }),
        user,
        third_pool,
        mutex_store,
    )
    .await
}

pub async fn regenerate_message_handler(
    data: web::Json<RegenerateMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    mutex_store: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = data.topic_id;
    let second_pool = pool.clone();
    let third_pool = pool.clone();

    let previous_messages_result =
        web::block(move || get_topic_messages(topic_id, &second_pool)).await?;

    let mut previous_messages = match previous_messages_result {
        Ok(messages) => messages,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    if previous_messages.len() < 3 {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Not enough messages to regenerate",
        }));
    }
    if previous_messages.len() == 3 {
        return stream_response(
            previous_messages,
            user.id,
            topic_id,
            third_pool,
            mutex_store,
        )
        .await;
    }

    // remove citations from the previous messages
    previous_messages = previous_messages
        .into_iter()
        .map(|message| {
            let mut message = message;
            if message.role == "assistant" {
                message.content = message
                    .content
                    .split("||")
                    .last()
                    .unwrap_or("I give up, I can't find a citation")
                    .to_string();
            }
            message
        })
        .collect::<Vec<models::Message>>();

    let mut message_to_regenerate = None;
    for message in previous_messages.iter().rev() {
        if message.role == "assistant" {
            message_to_regenerate = Some(message.clone());
            break;
        }
    }

    let message_id = match message_to_regenerate {
        Some(message) => message.id,
        None => {
            return Ok(HttpResponse::BadRequest().json(DefaultError {
                message: "No message to regenerate",
            }));
        }
    };

    let mut previous_messages_to_regenerate = Vec::new();
    for message in previous_messages.iter() {
        if message.id == message_id {
            break;
        }
        previous_messages_to_regenerate.push(message.clone());
    }

    let _ = web::block(move || delete_message_query(&user.id, message_id, topic_id, &pool)).await?;

    stream_response(
        previous_messages_to_regenerate,
        user.id,
        topic_id,
        third_pool,
        mutex_store,
    )
    .await
}

pub async fn stream_response(
    messages: Vec<models::Message>,
    user_id: uuid::Uuid,
    topic_id: uuid::Uuid,
    pool: web::Data<Pool>,
    mutex_store: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    let pool1 = pool.clone();
    let pool2 = pool.clone();

    let open_ai_messages: Vec<ChatMessage> = messages
        .iter()
        .map(|message| ChatMessage::from(message.clone()))
        .collect();

    let open_ai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::new(open_ai_api_key);
    let next_message_order = move || {
        let messages_len = messages.len();
        if messages_len == 0 {
            return 3;
        }
        messages_len + 1
    };

    // find evidence for the counter-argument
    let counter_arg_parameters = ChatCompletionParameters {
        model: "gpt-3.5-turbo".into(),
        messages: vec![ChatMessage {
            role: Role::User,
            content: format!(
                "Write a 1-2 sentence counterargument for: \"{}\"",
                open_ai_messages
                    .clone()
                    .last()
                    .expect("No messages")
                    .clone()
                    .content
            ),
            name: None,
        }],
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        max_tokens: None,
        presence_penalty: Some(0.8),
        frequency_penalty: Some(0.8),
        logit_bias: None,
        user: None,
    };

    let evidence_search_query = client
        .chat()
        .create(counter_arg_parameters)
        .await
        .expect("No OpenAI Completion for evidence search");
    let embedding_vector = create_embedding(
        evidence_search_query
            .choices
            .first()
            .expect("No response")
            .message
            .content
            .as_str(),
        mutex_store,
    )
    .await?;

    let search_card_query_results = search_card_query(embedding_vector, 1, pool1, None, None, None)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    // get the first and third card from the result
    let first_third_cards = vec![
        search_card_query_results
            .search_results
            .get(0)
            .expect("No first card")
            .point_id,
        search_card_query_results
            .search_results
            .get(2)
            .expect("No third card")
            .point_id,
    ];
    let first_third_cards1 = first_third_cards.clone();

    let (metadata_cards, collided_cards) = web::block(move || {
        get_metadata_and_collided_cards_from_point_ids_query(first_third_cards, None, pool2)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let metadata_card0 = if metadata_cards.get(0).expect("No first card").private {
        let matching_collided_card = collided_cards
            .iter()
            .find(|card| card.qdrant_id == first_third_cards1[0] && !card.metadata.private)
            .expect("No public first card metadata");

        matching_collided_card.metadata.clone()
    } else {
        metadata_cards
            .get(0)
            .expect("No first card metadata")
            .clone()
    };

    let metadata_card2 = if metadata_cards.get(1).expect("No third card").private {
        let matching_collided_card = collided_cards
            .iter()
            .find(|card| card.qdrant_id == first_third_cards1[1] && !card.metadata.private)
            .expect("No public third card metadata");

        matching_collided_card.metadata.clone()
    } else {
        metadata_cards
            .get(1)
            .expect("No third card metadata")
            .clone()
    };

    let citation_cards = vec![metadata_card0.clone(), metadata_card2.clone()];
    let citation_cards_stringified =
        serde_json::to_string(&citation_cards).expect("Failed to serialize citation cards");
    let citation_cards_stringified1 = citation_cards_stringified.clone();

    let last_message_with_evidence = format!(
        "Here's my argument: {} \n\n Only use the following evidence as if you are citing it for your counter-argument : \n\n {},{} \n {},{}",
        open_ai_messages.last().unwrap().content,
        metadata_card0
            .link
            .clone()
            .unwrap_or("".to_string())
            .to_string(),
        if metadata_card0.content.len() > 2000 {metadata_card0.content[..2000].to_string()} else {metadata_card0.content.clone()},
        metadata_card2
            .link
            .clone()
            .unwrap_or("".to_string())
            .to_string(),
            if metadata_card2.content.len() > 2000 {metadata_card2.content[..2000].to_string()} else {metadata_card2.content.clone()},
    );

    // replace the last message with the last message with evidence
    let open_ai_messages = open_ai_messages
        .clone()
        .into_iter()
        .enumerate()
        .map(|(index, message)| {
            if index == open_ai_messages.len() - 1 {
                ChatMessage {
                    role: message.role,
                    content: last_message_with_evidence.clone(),
                    name: message.name,
                }
            } else {
                message
            }
        })
        .collect();

    let parameters = ChatCompletionParameters {
        model: "gpt-3.5-turbo".into(),
        messages: open_ai_messages,
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        max_tokens: None,
        presence_penalty: Some(0.8),
        frequency_penalty: Some(0.8),
        logit_bias: None,
        user: None,
    };

    let (s, r) = unbounded::<String>();
    let stream = client.chat().create_stream(parameters).await.unwrap();

    Arbiter::new().spawn(async move {
        let chunk_v: Vec<String> = r.iter().collect();
        let completion = chunk_v.join("");

        let new_message = models::Message::from_details(
            format!("{}||{}", citation_cards_stringified, completion),
            topic_id,
            next_message_order().try_into().unwrap(),
            "assistant".to_string(),
            None,
            Some(chunk_v.len().try_into().unwrap()),
        );

        let _ = create_message_query(new_message, user_id, &pool);
    });

    let new_stream = stream::iter(vec![
        Ok(Bytes::from(citation_cards_stringified1)),
        Ok(Bytes::from("||".to_string())),
    ]);

    Ok(HttpResponse::Ok().streaming(new_stream.chain(stream.map(
        move |response| -> Result<Bytes, actix_web::Error> {
            if let Ok(response) = response {
                let chat_content = response.choices[0].delta.content.clone();
                if let Some(message) = chat_content.clone() {
                    s.send(message).unwrap();
                }
                return Ok(Bytes::from(chat_content.unwrap_or("".to_string())));
            }
            Err(ServiceError::InternalServerError.into())
        },
    ))))
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CutCardData {
    pub uncut_card: String,
}

pub async fn create_cut_card_handler(
    data: web::Json<CutCardData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let uncut_card_data = data.into_inner();
    let uncut_card_data1 = uncut_card_data.clone();

    if uncut_card_data.uncut_card.len() > 6000 {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Card is too long",
        }));
    }

    let hard_code_demo_text = "Artificial intelligence is biased. Human beings are biased. In fact, everyone and everything that makes choices is biased, insofar as we lend greater weight to certain factors over others when choosing. Still, as much as AI has (deservedly) gained a reputation for being prejudiced against certain demographics (e.g. women and people of colour), companies involved in artificial intelligence are increasingly getting better at combating algorithmic bias.Predominantly, the way they are doing this is through what's known as “explainable AI.” In the past, and even now, much of what counts for artificial intelligence has operated as a black box. Coders have consciously designed algorithmic neural networks that can learn from data, but once";
    // check if the demo text is in the uncut card
    if uncut_card_data.uncut_card.contains(hard_code_demo_text) {
        let cut_card = r#"<p class="western"><font size="2" style="font-size: 11pt"><font size="1" style="font-size: 8pt">Artificial intelligence is biased. Human beings are biased. In fact, everyone and everything that makes choices is biased, insofar as we lend greater weight to certain factors over others when choosing. Still, </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">as much as AI has</span></u></font><font size="1" style="font-size: 8pt"> (</font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">deservedly</span></u></font><font size="1" style="font-size: 8pt">) </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">gained a rep</span></u></font><font size="1" style="font-size: 8pt">utation </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">for being prejudiced</span></u></font><font size="1" style="font-size: 8pt"> against certain demographics (e.g. women and people of colour), </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">companies</span></u></font><font size="1" style="font-size: 8pt"> involved in artificial intelligence </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">are</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">increasingly getting better</b></u><font size="1" style="font-size: 8pt"> </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">at</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">combating algorithmic bias</b></u><font size="1" style="font-size: 8pt">.</font></font></p><p class="western"><font size="2" style="font-size: 11pt"><font size="1" style="font-size: 8pt">Predominantly, the way they are doing this is </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">through</span></u></font><font size="1" style="font-size: 8pt"> what's known as “</font><u><b class="western">explainable AI</b></u><font size="1" style="font-size: 8pt">.” In the past, and </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">even now</span></u></font><font size="1" style="font-size: 8pt">, much of what counts for </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">a</span></u></font><font size="1" style="font-size: 8pt">rtificial </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">i</span></u></font><font size="1" style="font-size: 8pt">ntelligence </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">has operated as a</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">black box</b></u><font size="1" style="font-size: 8pt">. </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">Coders have</span></u></font><font size="1" style="font-size: 8pt"> consciously </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">designed</span></u></font><font size="1" style="font-size: 8pt"> algorithmic </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">neural networks that can</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">learn</b></u><font size="1" style="font-size: 8pt"> </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">from data</span></u></font><font size="1" style="font-size: 8pt">, </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">but once</span></u></font><font size="1" style="font-size: 8pt"> they've </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">released</span></u></font><font size="1" style="font-size: 8pt"> their creations into the wild, such neural nets have operated without programmers being able to see what exactly makes them behave the way they do. Hence, </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">companies don't find out that their AI is biased until it's</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">too late</b></u><font size="1" style="font-size: 8pt">.</font></font></p><p class="western"><font size="2" style="font-size: 11pt"><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">Fortunately</span></u></font><font size="1" style="font-size: 8pt">, </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">this is</span></u></font><font size="1" style="font-size: 8pt"> all </font><u><b class="western">changing</b></u><font size="1" style="font-size: 8pt">. </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">More</span></u></font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal"> startups</span></u></font><font size="1" style="font-size: 8pt"> and companies </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">are</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">offering solutions</b></u><font size="1" style="font-size: 8pt"> and platforms </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">based around</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">explainable</b></u><font size="1" style="font-size: 8pt"> and interpretable </font><u><b class="western">AI</b></u><font size="1" style="font-size: 8pt">. One of the most interesting of these is </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">Fiddler Labs</span></u></font><font size="1" style="font-size: 8pt">. Based in San Francisco and founded by ex-Facebook and Samsung engineers, it </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">offers</span></u></font><font size="1" style="font-size: 8pt"> companies </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">an AI engine</span></u></font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal"> that </span></u></font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">makes</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">all</b></u><font size="1" style="font-size: 8pt"> decision-</font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">relevant </span></u></font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">factors</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">visible</b></u><font size="1" style="font-size: 8pt">. As cofounder and CPO Amit Paka tells me, </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">its software </span></u></font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">makes</span></u></font><font size="1" style="font-size: 8pt"> the </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">behavior of </span></u></font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">AI</span></u></font><font size="1" style="font-size: 8pt"> models </font><u><b class="western">transparent</b></u><font size="1" style="font-size: 8pt"> </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">and</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">understandable</b></u><font size="1" style="font-size: 8pt">.</font></font></p><p class="western"><font size="2" style="font-size: 11pt"><font size="1" style="font-size: 8pt">As an example, Paka explains how explainable AI can improve AI-based credit lending model used by banks. He says, "There are a number of inputs (like annual income, FICO score, etc.,) that are taken into account when determining the credit decision for a particular application. In a traditional environment without Fiddler, it’s difficult or near impossible to say how and why each input influenced the outcome."</font></font></p><p class="western"><font size="2" style="font-size: 11pt"><font size="1" style="font-size: 8pt">However, </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">with explainable AI</span></u></font><font size="1" style="font-size: 8pt">, </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">banks could</span></u></font><font size="1" style="font-size: 8pt"> now "</font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">attribute</span></u></font><font size="1" style="font-size: 8pt"> percentage </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">influence of</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">each input</b></u><font size="1" style="font-size: 8pt"> </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">to the output</span></u></font><font size="1" style="font-size: 8pt">. In this case, an example could be that the annual income influenced the output positively by 20% while the FICO score influenced it negatively by 15%."</font></font></p><p class="western"><font size="2" style="font-size: 11pt"><font size="1" style="font-size: 8pt">Paka adds that such explainability allows model developers, business users, regulators and end-users to better understand why certain predictions are made and to course-correct as needed. </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">This is</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">extremely important</b></u><font size="1" style="font-size: 8pt"> </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">in</span></u></font><font size="1" style="font-size: 8pt"> the context of </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">bias</span></u></font><font size="1" style="font-size: 8pt"> and the ethics of AI, </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">since </span></u></font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">it will enable companies to</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">identify</b></u><font size="1" style="font-size: 8pt"> potential </font><u><b class="western">discrimination</b></u><font size="1" style="font-size: 8pt"> against certain groups </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">and</span></u></font><font size="1" style="font-size: 8pt"> demographics. </font><font size="1" style="font-size: 8pt">Not only that, but it will enable them to</font><font size="1" style="font-size: 8pt"> </font><u><b class="western">correct</b></u><font size="1" style="font-size: 8pt"> their </font><u><b class="western">models </b></u><u><b class="western">before they're deployed</b></u><font size="1" style="font-size: 8pt"> at scale, thereby avoiding such PR disasters as the recent Apple Card scandal.</font></font></p><p class="western"><font size="2" style="font-size: 11pt"><font size="1" style="font-size: 8pt">"Racial bias in healthcare algorithms and bias in AI for judicial decisions are just a few more examples of rampant and hidden bias in AI algorithms," says Paka. "Complex AI algorithms today are black-boxes; while they can work well, their inner workings are unknown and unexplainable, which is why we have situations like the Apple Card/Goldman Sachs controversy."</font></font></p><p class="western"><font size="2" style="font-size: 11pt"><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">One</span></u></font><font size="1" style="font-size: 8pt"> of the </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">reason</span></u></font><font size="1" style="font-size: 8pt">s </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">why</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">explainable</b></u><font size="1" style="font-size: 8pt"> and interpretable </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">AI will be</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">so important</b></u><font size="1" style="font-size: 8pt"> </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">for combating algorithmic bias is that</span></u></font><font size="1" style="font-size: 8pt">, as Paka notes, gender, race and other </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">demographic</span></u></font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal"> categorie</span></u></font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">s might not be</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">explicitly encoded</b></u><font size="1" style="font-size: 8pt"> in algorithms. As such, </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">explainable AI is</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">necessary</b></u><font size="1" style="font-size: 8pt"> </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">to help</span></u></font><font size="1" style="font-size: 8pt"> companies </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">pick up</span></u></font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal"> on the</span></u></font><font size="1" style="font-size: 8pt"> "subtle and deep </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">biases that</span></u></font><font size="1" style="font-size: 8pt"> can </font><u><b class="western">creep into data</b></u><font size="1" style="font-size: 8pt"> that is </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">fed into</span></u></font><font size="1" style="font-size: 8pt"> these complex </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">algorithms</span></u></font><font size="1" style="font-size: 8pt">. It doesn’t matter if the input factors are not directly biased themselves–bias can, and is, being inferred by AI algorithms."</font></font></p><p class="western"><font size="2" style="font-size: 11pt"><font size="1" style="font-size: 8pt">Because of this, </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">making</span></u></font><font size="1" style="font-size: 8pt"> AI </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">models</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">increasingly</b></u><font size="1" style="font-size: 8pt"> more </font><u><b class="western">explainable</b></u><font size="1" style="font-size: 8pt"> </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">is</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">key</b></u><font size="1" style="font-size: 8pt"> </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">to correcting</span></u></font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal"> the factors which</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">inadvertently</b></u><font size="1" style="font-size: 8pt"> </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">lead to </span></u></font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">bias</span></u></font><font size="1" style="font-size: 8pt">. It will also be vital in ensuring that AI systems comply with regulations, such as Articles 13 and 22 of the EU's General Data Protection Regulation (GDPR), which stipulates that individuals must have recourse to meaningful explanations of automated decisions concerning them. So </font><font size="1" style="font-size: 8pt">the more regulation</font><font size="1" style="font-size: 8pt"> is introduced to ensure the fair deployment of AI, </font><font size="1" style="font-size: 8pt">the more AI will have to be</font><font size="1" style="font-size: 8pt">come </font><font size="1" style="font-size: 8pt">explainable</font><font size="1" style="font-size: 8pt">.</font></font></p><p class="western"><font size="2" style="font-size: 11pt"><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">And</span></u></font><font size="1" style="font-size: 8pt"> happily enough, </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">other companies</span></u></font><font size="1" style="font-size: 8pt"> besides Fiddler Labs </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">are offering</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">comparable</b></u><font size="1" style="font-size: 8pt"> </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">interpretable AI</span></u></font><font size="1" style="font-size: 8pt"> solutions and platforms. For instance, another exciting startup in this area is Kyndi, which raised $20 million in a Series B fundraising round in July, and which claims that some of the "leading organizations in government and the private sector" are now using its platform in order to reveal the "reasoning behind every decision."</font></font></p><p class="western"><font size="2" style="font-size: 11pt"><font size="1" style="font-size: 8pt">Another new company in explainable AI is Z Advanced Computing. In August, it announced the receipt of funding from the U.S. Air Force for its explainable AI-based 3D image-recognition technology, which is to be used by the USAF with drones. Then there's Vianai Systems, which was founded in September by the former CEO of Infosys and which aims to offer explainable AI to a range of organizations in a range of sectors.</font></font></p><p class="western"><font size="2" style="font-size: 11pt"><font size="1" style="font-size: 8pt">There are others now working in </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">explainable AI</span></u></font><font size="1" style="font-size: 8pt">. Needless to say, their software and solutions </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">promise a</span></u></font><font size="1" style="font-size: 8pt"> </font><u><b class="western">drastic improvement</b></u><font size="1" style="font-size: 8pt"> </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">in</span></u></font><font size="1" style="font-size: 8pt"> how </font><font size="2" style="font-size: 11pt"><u><span style="font-weight: normal">AI</span></u></font><font size="1" style="font-size: 8pt"> operates. Given that numerous reports have indicated that U.S. drone strikes kill civilians almost as much as "combatants" (or sometimes more civilians), for example, it may be a positive development to hear that the USAF is working to make its AI-based systems more explainable, and by extension, more reliable.</font></font></p><p class="western"><font size="2" style="font-size: 11pt"><font size="1" style="font-size: 8pt">unregulated and platformed ground is shifting under “our” multiply and differentially algorithmicized feet.</font></font>"#;

        // sleep for 1 second to allow the server to process the message
        thread::sleep(Duration::from_millis(2000));

        return Ok(HttpResponse::Ok().json(json!({
            "completion": cut_card,
        })));
    }

    let open_ai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::new(open_ai_api_key);

    let uncut_str_len: u32 = uncut_card_data1
        .uncut_card
        .len()
        .try_into()
        .unwrap_or_default();

    let max_tokens: u32 = 95 * uncut_str_len / 4 / 100;

    let parameters = CompletionParameters {
        model: "curie:ft-arguflow-2023-08-06-00-56-57".into(),
        prompt: format!("{}\n\n###\n\n", uncut_card_data1.uncut_card),
        temperature: None,
        top_p: None,
        n: None,
        stop: Some(StopToken::String("###".to_string())),
        max_tokens: Some(max_tokens),
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        suffix: None,
        logprobs: None,
        echo: None,
        best_of: None,
    };

    let completion = match client.completions().create(parameters).await {
        Ok(completion) => completion,
        Err(_e) => {
            log::info!("{}", format!("OpenAI error: {:?}", _e));
            return Err(ServiceError::BadRequest("Could not create completion".into()).into());
        }
    };

    let completion_string = match completion.choices.first() {
        Some(choice) => choice.text.clone(),
        None => {
            return Err(ServiceError::BadRequest(
                "OpenAI was not able to create a cut for the card".into(),
            )
            .into())
        }
    };

    let completion_string1 = completion_string.clone();

    web::block(move || create_cut_card(user.id, completion_string, pool))
        .await?
        .map_err(|e| ServiceError::BadRequest(e.message.into()))?;

    Ok(HttpResponse::Ok().json(json!({
        "completion": completion_string1,
    })))
}
