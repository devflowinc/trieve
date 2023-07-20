use diesel::prelude::*;

use actix_web::web;
use regex::Regex;
use serde_json::json;
use soup::{NodeExt, QueryBuilderExt};
use std::{
    process::Command,
    sync::{Arc, Mutex},
};

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

pub async fn get_webpage_text_fetch(
    url: &str,
    mutex_store: web::Data<AppMutexStore>,
) -> Result<String, DefaultError> {
    let response = reqwest::get(url).await.map_err(|_| DefaultError {
        message: "Could not fetch page",
    })?;

    let headers = response.headers().get("content-type").ok_or(DefaultError {
        message: "Could not get content type",
    })?;
    let html;

    if headers.to_str().unwrap_or("").contains("text/html") {
        html = response.text().await.map_err(|_| DefaultError {
            message: "Could not parse text",
        })?;
    } else if headers == "application/pdf" {
        let pdf_file_path = format!(
            "./tmp/{}-{}.pdf",
            uuid::Uuid::new_v4(),
            response.url().to_string().replace('/', "")
        );
        let html_file_path = format!(
            "./tmp/{}-{}.html",
            uuid::Uuid::new_v4(),
            response.url().to_string().replace('/', "")
        );

        let pdf = response.bytes().await.map_err(|_| DefaultError {
            message: "Could not parse pdf",
        })?;

        let delete_files = || {
            let _ = std::fs::remove_file(&pdf_file_path).map_err(|_| DefaultError {
                message: "Could not remove temp docx file",
            });
            let _ = std::fs::remove_file(&html_file_path).map_err(|_| DefaultError {
                message: "Could not remove temp html file",
            });
        };

        std::fs::write(&pdf_file_path, &pdf).map_err(|err| {
            log::error!("Could not write file to disk: {}", err);
            DefaultError {
                message: "Could not write file to disk",
            }
        })?;

        let libreoffice_lock = match mutex_store.libreoffice.lock() {
            Ok(libreoffice_lock) => libreoffice_lock,
            Err(_) => {
                delete_files();
                return Err(DefaultError {
                    message: "Could not lock libreoffice mutex",
                })
            }
        };

        let conversion_command_output =
            Command::new(std::env::var("LIBREOFFICE_PATH").expect("LIBREOFFICE_PATH must be set"))
                .arg("--headless")
                .arg("--convert-to")
                .arg("html")
                .arg("--outdir")
                .arg("./tmp")
                .arg(&pdf_file_path)
                .output();

        drop(libreoffice_lock);

        if conversion_command_output.is_err() {
            delete_files();
            return Err(DefaultError {
                message: "Could not convert file",
            });
        }

        html = std::fs::read_to_string(&html_file_path).map_err(|_| DefaultError {
            message: "Could not read text file",
        })?;

        delete_files();
    } else {
        return Err(DefaultError {
            message: "Could not parse content type",
        });
    }

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
