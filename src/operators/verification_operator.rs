use soup::prelude::*;
use regex::Regex;

use crate::errors::DefaultError;

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

pub async fn get_webpage_text_browser(url: &String) -> Result<String, DefaultError> {
    Err(DefaultError {
        message: ""
    })
}
