use diesel::prelude::*;
use headless_chrome::{Browser, LaunchOptionsBuilder};

use regex::Regex;
use soup::{NodeExt, QueryBuilderExt};
use std::{
    process::Command,
    sync::{Arc, Mutex},
};

use actix_web::web;

use crate::{
    data::models::{CardVerifications, Pool},
    errors::DefaultError,
};

pub async fn get_webpage_text_headless(url: &str) -> Result<String, DefaultError> {
    let options = LaunchOptionsBuilder::default()
        .headless(true)
        .build()
        .unwrap();

    let browser = Browser::new(options).map_err(|_e| DefaultError {
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
        let pdf_file_path = format!("./tmp/{}.pdf", response.url().to_string().replace('/', ""));
        let html_file_path = format!("./tmp/{}.html", response.url().to_string().replace('/', ""));

        let pdf = response.bytes().await.map_err(|_| DefaultError {
            message: "Could not parse pdf",
        })?;

        std::fs::write(&pdf_file_path, &pdf).map_err(|_| DefaultError {
            message: "Could not write file to disk",
        })?;

        let conversion_command_output =
            Command::new(std::env::var("LIBREOFFICE_PATH").expect("LIBREOFFICE_PATH must be set"))
                .arg("--headless")
                .arg("--convert-to")
                .arg("html")
                .arg("--outdir")
                .arg("./tmp")
                .arg(&pdf_file_path)
                .output();

        if conversion_command_output.is_err() {
            return Err(DefaultError {
                message: "Could not convert file",
            });
        }

        html = std::fs::read_to_string(&html_file_path).map_err(|_| DefaultError {
            message: "Could not read text file",
        })?;

        std::fs::remove_file(&pdf_file_path).map_err(|_| DefaultError {
            message: "Could not remove temp docx file",
        })?;
        std::fs::remove_file(&html_file_path).map_err(|_| DefaultError {
            message: "Could not remove temp html file",
        })?;
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
