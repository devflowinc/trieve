use diesel::prelude::*;
use headless_chrome::Browser;
use regex::Regex;
use soup::{NodeExt, QueryBuilderExt};
use std::sync::{Arc, Mutex};

use actix_web::web;

use crate::{
    data::models::{CardVerifications, Pool},
    errors::DefaultError,
};

pub async fn get_webpage_text_headless(url: &str) -> Result<String, DefaultError> {
    let browser = Browser::default().map_err(|_e| DefaultError {
        message: "Could not create browser",
    })?;

    let tab = browser.new_tab().map_err(|_e| DefaultError {
        message: "Could not create tab",
    })?;

    tab.set_user_agent(
            // first param is "user_agent", second is "accept_language", third is "platform"
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36",
            Some("en-US,en;q=0.7"),
            Some("Windows NT 10.0; Win64; x64"),
        ).map_err(|_e| DefaultError {
            message: "Could not set user agent",
        })?;

    tab.enable_stealth_mode().map_err(|_e| DefaultError {
        message: "Could not enable stealth mode",
    })?;

    tab.navigate_to(url).map_err(|_e| DefaultError {
        message: "Could not navigate to url",
    })?;

    let body_tag = tab.wait_for_element("body").map_err(|_e| DefaultError {
        message: "Could not wait for body",
    })?;

    let body_tag_inner_html = body_tag.get_inner_text().map_err(|_e| DefaultError {
        message: "Could not get inner html",
    })?;

    let re = Regex::new(r"\s+").unwrap();
    let clean_text = re.replace_all(&body_tag_inner_html, " ").to_string();

    Ok(clean_text)
}

pub async fn get_webpage_text_fetch(url: &str) -> Result<String, DefaultError> {
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
) -> Result<CardVerifications, DefaultError> {
    use crate::data::schema::card_verification::dsl::*;

    let mut conn = pool.lock().unwrap().get().unwrap();

    let new_id = uuid::Uuid::new_v4();

    let created_verification = diesel::insert_into(card_verification)
        .values((
            id.eq(new_id),
            card_id.eq(card_uuid),
            similarity_score.eq(new_score),
        ))
        .on_conflict(card_id)
        .do_update()
        .set(similarity_score.eq(new_score))
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
            message: "Could not upsert card verification",
        })?;

    Ok(created_verification)
}
