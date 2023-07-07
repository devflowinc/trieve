use diesel::prelude::*;
use regex::Regex;
use soup::prelude::*;
use std::sync::{Arc, Mutex};

use actix_web::web;

use crate::{data::models::Pool, errors::DefaultError};

pub async fn get_webpage_text_fetch(url: &String) -> Result<String, DefaultError> {
    let html = reqwest::get(url)
        .await
        .map_err(|_| DefaultError {
            message: "Could not fetch page",
        })?
        .text()
        .await
        .map_err(|_| DefaultError {
            message: "Could not parse text",
        })?;

    let soup = soup::Soup::new(&html);

    let body = soup.tag("body").find().ok_or(DefaultError {
        message: "Could not find body tag",
    })?;

    // Replace multiple whitesapces chars with a single space
    let text = body.text();
    let re = Regex::new(r"\s+").unwrap();
    let clean_text = re.replace_all(&text, " ").to_string();

    Ok(clean_text)
}

pub fn upsert_card_verification_query(
    pool: Arc<Mutex<web::Data<Pool>>>,
    card_uuid: uuid::Uuid,
    new_score: i64,
) -> Result<(), DefaultError> {
    use crate::data::schema::card_verification::dsl::*;

    let mut conn = pool.lock().unwrap().get().unwrap();

    let new_id = uuid::Uuid::new_v4();

    diesel::insert_into(card_verification)
        .values((
            id.eq(new_id),
            card_id.eq(card_uuid),
            similarity_score.eq(new_score),
        ))
        .on_conflict(card_id)
        .do_update()
        .set(similarity_score.eq(new_score))
        .execute(&mut conn)
        .map_err(|_| DefaultError {
            message: "Could not upsert card verification",
        })?;

    Ok(())
}
