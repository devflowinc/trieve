use crate::{
    data::models,
    data::models::{Message, Pool},
    errors::{DefaultError, ServiceError},
    operators::message_operator::{
        create_cut_card, create_message_query, create_topic_message_query, delete_message_query,
        get_message_by_sort_for_topic_query, get_messages_for_topic_query, get_topic_messages,
        user_owns_topic_query,
    },
};
use actix::Arbiter;
use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};
use crossbeam_channel::unbounded;
use openai_dive::v1::{
    api::Client,
    resources::chat_completion::{ChatCompletionParameters, ChatMessage},
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
    let previous_messages = web::block(move || get_topic_messages(topic_id, &second_pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    // call create_topic_message_query with the new_message and previous_messages
    let previous_messages_result = web::block(move || {
        create_topic_message_query(previous_messages, new_message, user.id, &third_pool)
    })
    .await?;
    let previous_messages = match previous_messages_result {
        Ok(messages) => messages,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    stream_response(previous_messages, user.id, topic_id, fourth_pool).await
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
    )
    .await
}

pub async fn regenerate_message_handler(
    data: web::Json<RegenerateMessageData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let topic_id = data.topic_id;
    let second_pool = pool.clone();
    let third_pool = pool.clone();

    let previous_messages_result =
        web::block(move || get_topic_messages(topic_id, &second_pool)).await?;
    let previous_messages = match previous_messages_result {
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
        return stream_response(previous_messages, user.id, topic_id, third_pool).await;
    }

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
    )
    .await
}

pub async fn stream_response(
    messages: Vec<models::Message>,
    user_id: uuid::Uuid,
    topic_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
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
            completion,
            topic_id,
            next_message_order().try_into().unwrap(),
            "assistant".to_string(),
            None,
            Some(chunk_v.len().try_into().unwrap()),
        );

        let _ = create_message_query(new_message, user_id, &pool);
    });

    Ok(HttpResponse::Ok().streaming(stream.map(
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
    )))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CutCardData {
    pub uncut_card: String,
}

pub async fn create_cut_card_handler(
    data: web::Json<CutCardData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let uncut_card_data = data.into_inner();

    let system_message = Message::from_details(
        "You are Arguflow syntax highlighter, a large language model trained by Arguflow to inline syntax highlight information",
        uuid::Uuid::default(),
        0,
        "system".into(),
        Some(0),
        Some(0),
    );

    let user_training_message_content = r#"I have the following examples of content which has been transformed to syntax highlighted card_html: 

    ```
    [
      {
        "content": "In recent months, major political events have shaken our assumptions of the world; and as is often the case during such times, there is a risk for more division and misunderstandings. Arts can be a powerful tool here: it helps us appreciate other points of view, challenge our perspectives, and ultimately unite us. It is therefore crucial to re-evaluate the accessibility and weight given to arts in our everyday life. More importantly, we want to provide our next generation with tools to understand and challenge our reality. In recent years, studies have shown that arts education may carry some of the most important benefits — in comparison to other fields — for the development of children's and young adults' critical thinking skills. And yet too often we see the arts treated as optional. Out in the business world too, companies start to acknowledge design and creative thinking as essential skills to challenge their vision, and ultimately perform better. At a time when more and more consulting firms are buying design agencies and tech start-ups are hiring UX designers, the traditional boundaries between business, tech, and arts are finally starting to blur. So why not offer arts programs in schools the value and resources they deserve? One of the goals we've set for ourselves at WeTransfer is to commit to providing arts students with the best conditions to succeed and follow their passion in creative disciplines. Here is why. 1. Benefits of arts in education Arts classes can greatly benefit children and teens, as they are still growing physically and mentally. Music for example, has been proven to connect both hemispheres of the brain and therefore producing powerful changes in the brain structure. Learning to play an instrument at a young age enables stronger connections to form between various regions in the brain, and has a lifelong impact — thus highly improving communication and listening skills as an adult. It is similar to learning a new language as a child, which influences thought, consciousness and memory. Arts also encourage students to value different perspectives, distinguish between reality and interpretations, and become more culturally aware. Research has shown that in low-income neighborhoods, kids in arts programs improve their overall academic performance (including math), score higher on the SAT, and are three times more likely to earn a Bachelor's degree. Overall, arts classes provide a safe and fun environment that motivates children and young adults to go to school. Perhaps most relevant to the ever-evolving world we live in, is how arts education fosters creativity and innovative minds. As Nelly Ben Hayoun, Wired Innovation fellow, director of NBH studios and Head of Experiences at WeTransfer, puts it: “What our society needs are forward-thinking, disruptive and innovative minds, not more people who can only follow directions. We have the duty to inspire the next generation in science and tech to be bold, impolite and ambitious, and arts can provide that experience for them.” Employers agree that so-called soft skills — like the ability to innovate — are crucial for young hires. In the workplace, we need people that are able to think of innovative ways to tackle problems, understand different perspectives and make conscientious decisions. Arts programs can help provide these skills early on in one's life.",
        "card_html": "<p class=\"western\" style=\"line-height: 100%; margin-bottom: 0in\"><font size=\"1\" style=\"font-size: 8pt\">In\nrecent months, major political events have shaken our assumptions of\nthe world; and as is often the case during such times, there is a\nrisk for more division and misunderstandings. Arts can be a powerful\ntool here: it helps us appreciate other points of view, challenge our\nperspectives, and ultimately unite us. </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">It\nis</span></u></font><font size=\"1\" style=\"font-size: 8pt\"> therefore\n</font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">crucial\nto re-evaluate the</span></u></font><font size=\"1\" style=\"font-size: 8pt\">\naccessibility and </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">weight\ngiven to arts</span></u></font><font size=\"1\" style=\"font-size: 8pt\">\nin our everyday life. More importantly, we want to provide our next\ngeneration with tools to understand and challenge our reality. In\nrecent years, studies have shown that </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">arts\neducation may carry some of the most </span></u></font><u><b class=\"western\">important\nbenefits</b></u><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\"> — in\ncomparison to other fields — for the development of children's\nand young adults' </span></u></font><u><b class=\"western\">critical\nthinking skills</b></u><font size=\"1\" style=\"font-size: 8pt\">. And yet\ntoo often we see the arts treated as optional. </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">Out\n</span></u></font><u><b class=\"western\">in the business world</b></u><font size=\"1\" style=\"font-size: 8pt\">\ntoo, </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">companies</span></u></font><font size=\"1\" style=\"font-size: 8pt\">\nstart to </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">acknowledge\ndesign and creative thinking as </span></u></font><u><b class=\"western\">essential</b></u><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">\nskills to</span></u></font><font size=\"1\" style=\"font-size: 8pt\">\nchallenge their vision, and ultimately </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">perform\nbetter</span></u></font><font size=\"1\" style=\"font-size: 8pt\">. At a\ntime when more and more consulting firms are buying design agencies\nand tech start-ups are hiring UX designers, </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">the\ntraditional boundaries between business, tech, and arts are finally\nstarting to blur. So why not offer arts programs in schools the value\nand resources they deserve?</span></u></font><font size=\"1\" style=\"font-size: 8pt\">\nOne of the goals we've set for ourselves at WeTransfer is to commit\nto providing arts students with the best conditions to succeed and\nfollow their passion in creative disciplines. Here is why. 1.\nBenefits of arts in education </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">Arts\nclasses can greatly benefit children and teens</span></u></font><font size=\"1\" style=\"font-size: 8pt\">,\nas they are still growing physically and mentally. </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">Music</span></u></font><font size=\"1\" style=\"font-size: 8pt\">\nfor example, </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">has\nbeen proven to connect both hemispheres of the brain and therefore\nproducing powerful changes in the brain structure</span></u></font><font size=\"1\" style=\"font-size: 8pt\">.\nLearning to play an instrument at a young age enables stronger\nconnections to form between various regions in the brain, </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">and\nhas a lifelong impact</span></u></font><font size=\"1\" style=\"font-size: 8pt\"> — thus\n</font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">highly\nimproving communication and listening skills as an adult. It is\nsimilar to learning a new language as a child, which influences\nthought, consciousness and memory.</span></u></font><font size=\"1\" style=\"font-size: 8pt\">\nArts also encourage students to value different perspectives,\ndistinguish between reality and interpretations, and become more\nculturally aware. </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">Research\nhas shown that in low-income neighborhoods, kids in arts programs\n</span></u></font><u><b class=\"western\">improve</b></u><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">\ntheir </span></u></font><u><b class=\"western\">overall academic\nperformance</b></u><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">\n(including math), </span></u></font><u><b class=\"western\">score higher\non the SAT</b></u><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">,\nand are </span></u></font><u><b class=\"western\">three times more likely\nto earn a Bachelor's degree</b></u><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">.</span></u></font><font size=\"1\" style=\"font-size: 8pt\">\nOverall, </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">arts\nclasses provide a safe and fun environment that </span></u></font><u><b class=\"western\">motivates</b></u><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">\nchildren and young adults to go to school</span></u></font><font size=\"1\" style=\"font-size: 8pt\">.\n</font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">Perhaps\nmost relevant to the ever-evolving world we live in, is how arts\neducation fosters creativity and innovative minds</span></u></font><font size=\"1\" style=\"font-size: 8pt\">.\nAs Nelly Ben Hayoun, Wired Innovation fellow, director of NBH studios\nand Head of Experiences at WeTransfer, puts it: </font><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">“What\nour society needs are forward-thinking, disruptive and innovative\nminds, not more people who can only follow directions. We have the\nduty to inspire the next generation in science and tech to be bold,\nimpolite and ambitious, and arts can provide that experience for\nthem.” </span></u></font><font size=\"1\" style=\"font-size: 8pt\">Employers\nagree that so-called </font><u><b class=\"western\">soft skills</b></u><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\"> — like\nthe ability to innovate — </span></u></font><u><b class=\"western\">are\ncrucial for young hires.</b></u><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">\n</span></u></font><u><b class=\"western\">In the workplace</b></u><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">,\nwe need people that are able to think of innovative ways to tackle\nproblems, understand different perspectives and make conscientious\ndecisions. </span></u></font><u><b class=\"western\">Arts programs can\nhelp provide these skills</b></u><font size=\"2\" style=\"font-size: 10pt\"><u><span style=\"font-weight: normal\">\nearly on in one's life.</span></u></font>"
      },
      {
        "content": "According to regression estimates of the data compiled by the UN, a 1% change in GDP per capita will cause only a 0.3 unit change in happiness (happiness is calculated on a scale from 0 to 10). However, when GDP per capita is included with other variables the model explains nearly 75% of the variance in happiness. These other variables consist of social support, life expectancy, freedom to make life choices, generosity, and freedom from corruption. This demonstrates that quality of life, and not just material wealth, plays a huge role in happiness. Alternate variables seem to matter–a lotThe impact is poor poverty.",
        "card_html": "<p style=\"margin-bottom: 0in\"><font face=\"Times New Roman, serif\"><font size=\"1\" style=\"font-size: 8pt\">According\nto regression estimates of the data compiled by the UN,</font></font><font face=\"Times New Roman, serif\"><font size=\"3\" style=\"font-size: 12pt\"><u><b>\na 1% change in GDP per capita will cause only a 0.3 unit change in\nhappiness</b></u></font></font><font face=\"Times New Roman, serif\"><font size=\"3\" style=\"font-size: 12pt\"><b>\n</b></font></font><font face=\"Times New Roman, serif\"><font size=\"1\" style=\"font-size: 8pt\">(happiness\nis calculated on a scale from 0 to 10). However, </font></font><font face=\"Times New Roman, serif\"><font size=\"3\" style=\"font-size: 12pt\"><u><b>when\nGDP per capita is included with other variables the model explains\nnearly 75% of the variance in happiness.</b></u></font></font><font face=\"Times New Roman, serif\"><font size=\"3\" style=\"font-size: 12pt\"><b>\n</b></font></font><font face=\"Times New Roman, serif\"><font size=\"3\" style=\"font-size: 12pt\"><u><b>These\nother variables consist of social support, life expectancy, freedom\nto make life choices, generosity, and freedom from corruption.</b></u></font></font><font face=\"Times New Roman, serif\"><font size=\"3\" style=\"font-size: 12pt\"><b>\n</b></font></font><font face=\"Times New Roman, serif\"><font size=\"3\" style=\"font-size: 12pt\"><u><b>This\ndemonstrates that quality of life</b></u></font></font><font face=\"Times New Roman, serif\"><font size=\"3\" style=\"font-size: 12pt\"><b>,\n</b></font></font><font face=\"Times New Roman, serif\"><font size=\"1\" style=\"font-size: 8pt\">and\nnot just material wealth,</font></font><font face=\"Times New Roman, serif\"><font size=\"3\" style=\"font-size: 12pt\"><b>\n</b></font></font><font face=\"Times New Roman, serif\"><font size=\"3\" style=\"font-size: 12pt\"><u><b>plays\na huge role in happiness.</b></u></font></font><font face=\"Times New Roman, serif\"><font size=\"3\" style=\"font-size: 12pt\"><b>\n</b></font></font><font face=\"Times New Roman, serif\"><font size=\"1\" style=\"font-size: 8pt\">Alternate\nvariables seem to matter–a lot</font></font></p><p><br/></p><p style=\"margin-bottom: 0in\"><font face=\"Times New Roman, serif\"><font size=\"3\" style=\"font-size: 12pt\"><b>The\nimpact is poor poverty.</b></font></font>"
      },
      {
        "content": "Moore, a self professed 'threshold deontologist', supports the notion of catastrophic exceptions. He suggests that 'deontological norms govern up to a point despite adverse consequences; but when the consequences become so dire that they cross the stipulated threshold, consequentialism takes over'.219 Moore holds, that 'consequentialist considerations can override deontological judgements under the condition that extremely harmful outcomes are inevitable consequences of enacting the alleged duty.'220 That is to say, whilst torture is generally prohibited, when 'innocent lives are at stake, torturing a terrorist is deontologically justified and even morally required.'221 Cocks, a contributor to a draft of the European Convention of Human Rights, declares that '[a]ll forms of torture, whether inflicted by policy, military authorities [or] members of private organisations… are inconsistent with civilised society, are offences against heaven and humanity and must be prohibited.'222 Cocks' moralistic ideal, is to a certain extent accurate; torture is without a doubt 'inconsistent with society', and it is certainly an 'offence against humanity', however this does not automatically prohibit it in all circumstances. Herein lays the crux of the issue upon which moral absolutism is based; the theory assumes that because something is wrong, it absolutely cannot be done, but this is simply not the case. For example, it is possible to give an account of the inherent wrongness of torture; it 'instrumentalizes the pain and terror of human beings; it involves the deliberate, studied, and sustained imposition of pain to the point of agony on a person who is utterly vulnerable… and it aims to use that agony to shatter and mutilate the subject's will'.223 However, giving an account which shows that what is inherently wrong may never in any circumstances be done is another. 'Inherently' does not mean the same as 'absolutely'.224 Therefore, it is arguable that acts classified as 'inherently wrong', may still be permissible in specific situations. Elshtain, a modern moral philosopher, asserts that a refusal to approve the use of these techniques in the aforementioned situations, amounts to 'a form of moral laziness,' 'a moralistic code-fetishism,' or 'a legalistic version of pietistic rigorism in which one's own moral purity is ranked above other goods.'225 Elshtain paints a self-righteous and egotistical picture of the absolutist position. This, however, is an unfair portrayal; the absolutist position is dictated by law and convention. The international legal instruments which forbid torture 'are public documents; they are not treaties of personal ethics but conventions establishing minimum legal standards for the exercise of state power. As such, they prohibit torture categorically and absolutely'.226",
        "card_html": "<p class=\"western\" style=\"text-decoration: none\"><font size=\"2\" style=\"font-size: 11pt\"><u><b>Moore,\na self professed 'threshold deontologist', supports the notion of\ncatastrophic exceptions. He suggests that 'deontological norms\ngovern up to a point despite adverse consequences</b></u></font><font size=\"1\" style=\"font-size: 8pt\">;\n</font><u><b class=\"western\">but when the consequences become so dire\nthat they cross the stipulated threshold, consequentialism takes\nover'</b></u><font size=\"1\" style=\"font-size: 8pt\">.219 Moore holds,\nthat </font><u><b class=\"western\">'consequentialist considerations\ncan override deontological judgements under the condition that\nextremely harmful outcomes are inevitable consequences of enacting\nthe alleged duty.</b></u><font size=\"1\" style=\"font-size: 8pt\">'220\nThat is to say, whilst torture is generally prohibited, when\n'innocent lives are at stake, torturing a terrorist is\ndeontologically justified and even morally required.'221 Cocks, a\ncontributor to a draft of the European Convention of Human Rights,\ndeclares that </font><font size=\"2\" style=\"font-size: 11pt\"><u><b>'[a]ll\nforms of torture, whether inflicted by policy, military authorities\n[or] members of private organisations… are inconsistent with\ncivilised society</b></u></font><font size=\"1\" style=\"font-size: 8pt\">,\nare offences against heaven and humanity and must be prohibited.'222\nCocks' moralistic ideal, is to a certain extent accurate; torture\nis without a doubt 'inconsistent with society', and it is\ncertainly an 'offence against humanity', </font><u><b class=\"western\">however\nthis does not automatically prohibit it in all circumstances</b></u><font size=\"1\" style=\"font-size: 8pt\">.\nHerein lays the crux of the issue upon which moral absolutism is\nbased; the theory assumes that because something is wrong, it\nabsolutely cannot be done, but this is simply not the case. </font><font size=\"2\" style=\"font-size: 11pt\"><u><b>For\nexample, it is possible to give an account of the inherent wrongness\nof torture; it 'instrumentalizes the pain and terror of human\nbeings; it involves the deliberate, studied, and sustained imposition\nof pain to the point of agony on a person who is utterly vulnerable…\nand it aims to use that agony to shatter and mutilate the subject's\nwill'</b></u></font><font size=\"1\" style=\"font-size: 8pt\">.223\nHowever, giving an account which shows that what is inherently wrong\nmay never in any circumstances be done is another. 'Inherently'\ndoes not mean the same as 'absolutely'.224 </font><font size=\"2\" style=\"font-size: 11pt\"><u><b>Therefore,\nit is arguable that acts classified as 'inherently wrong', may\nstill be permissible in specific situations. </b></u></font><font size=\"1\" style=\"font-size: 8pt\">Elshtain,\na modern moral philosopher, asserts that a refusal to approve the use\nof these techniques in the aforementioned situations, amounts to 'a\nform of moral laziness,' 'a moralistic code-fetishism,' or 'a\nlegalistic version of pietistic rigorism in which one's own moral\npurity is ranked above other goods.'225 Elshtain paints a\nself-righteous and egotistical picture of the absolutist position</font><font size=\"2\" style=\"font-size: 11pt\"><u><b>.\nThis, however, is an unfair portrayal; the absolutist position is\ndictated by law and convention. The international legal instruments\nwhich forbid torture 'are public documents; they are not treaties\nof personal ethics but conventions establishing </b></u></font><font size=\"1\" style=\"font-size: 8pt\">minimum\nlegal standards for the exercise of state power. As such, they\nprohibit torture categorically and absolutely'.226</font>"
      }
    ]
    ```
    
    I will now provide you a value for "content" and you will transform it into a value for "card_html"  by underlining, shrinking, bolding, and enlarging the plaintext. Do not use `<i>`"#;

    let user_training_message = Message::from_details(
        user_training_message_content.to_string(),
        uuid::Uuid::default(),
        0,
        "user".into(),
        Some(0),
        Some(0),
    );

    let raw_card_content_message = Message::from_details(
        uncut_card_data.uncut_card,
        uuid::Uuid::default(),
        1,
        "user".into(),
        Some(0),
        Some(0),
    );

    let server_messages = vec![
        system_message,
        user_training_message,
        raw_card_content_message,
    ];

    let open_ai_messages: Vec<ChatMessage> = server_messages
        .iter()
        .map(|message| ChatMessage::from(message.clone()))
        .collect();

    let open_ai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::new(open_ai_api_key);

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

    let completion = client
        .chat()
        .create(parameters)
        .await
        .expect("Failed to create completion");

    let completion_string = completion
        .choices
        .first()
        .expect("No choices returned from completion")
        .message
        .content
        .clone();

    web::block(move || create_cut_card(user.id, completion_string, pool))
        .await?
        .map_err(|e| ServiceError::BadRequest(e.message.into()))?;

    Ok(HttpResponse::Ok().json(json!({
        "completion": completion,
    })))
}
