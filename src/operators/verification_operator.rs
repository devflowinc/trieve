use diesel::prelude::*;

use actix_web::web;
use regex::Regex;
use serde_json::json;
use soup::{NodeExt, QueryBuilderExt};
use std::sync::{Arc, Mutex};

use crate::{
    data::models::{CardVerifications, Pool},
    errors::DefaultError,
    AppMutexStore,
};

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
struct HeadlessData {
    pub content: String,
}

pub async fn get_webpage_text_headless(
    url: &str,
    mutex_store: web::Data<AppMutexStore>,
) -> Result<String, DefaultError> {
    let browser_semaphore_permit = match mutex_store.headless_chrome.acquire().await {
        Ok(browser) => browser,
        Err(_) => {
            return Err(DefaultError {
                message: "Could not lock browser mutex",
            })
        }
    };

    let webpage_url =
        std::env::var("VERIFICATION_SERVER_URL").expect("VERIFICATION_SERVER_URL must be set");
    let client = reqwest::Client::new();
    let response = client
        .post(webpage_url)
        .header("Content-Type", "application/json")
        .body(json!({ "url": url }).to_string())
        .send()
        .await
        .map_err(|_| DefaultError {
            message: "Failed to get html body",
        })?;

    let json = response
        .json::<HeadlessData>()
        .await
        .map_err(|_| DefaultError {
            message: "Failed to parse cleaned text",
        })?;

    let clean_text = json.content;

    drop(browser_semaphore_permit);
    Ok(clean_text)
}

pub async fn get_webpage_text_fetch(url: &str) -> Result<String, DefaultError> {
    let response = reqwest::get(url).await.map_err(|_| DefaultError {
        message: "Could not fetch page",
    })?;

    let headers = response.headers().get("content-type").ok_or(DefaultError {
        message: "Could not get content type",
    })?;

    if !headers.to_str().unwrap_or("").contains("text/html") {
        return Err(DefaultError {
            message: "Can not verify source formats other than html",
        });
    }

    let html = response.text().await.map_err(|_| DefaultError {
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
