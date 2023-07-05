use crate::{errors::ServiceError, operators::verification_operator as op};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyContentData {
    content: String,
    url_source: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TempContent {
    pub content: String,
}

pub async fn verify_card_content(
    data: web::Json<VerifyContentData>,
) -> Result<HttpResponse, actix_web::Error> {
    // Try naive html get first then use the headless browser approach
    let content = match op::get_webpage_text_fetch(&data.url_source).await {
        Ok(content) => Ok(content),
        Err(fetch_err) => op::get_webpage_text_browser(&data.url_source)
            .await
            .map_err(|_| ServiceError::BadRequest(format!("Could not fetch: {}", fetch_err))),
    }?;

    Ok(HttpResponse::Ok().json(TempContent { content }))
}
