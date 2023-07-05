use crate::{errors::ServiceError, operators::verification_operator as op};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VerifyContentData {
    content: String,
    url_source: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VerificationStatus {
    pub verified: bool,
    pub score: i64
}

pub async fn verify_card_content(
    data: web::Json<VerifyContentData>,
) -> Result<HttpResponse, actix_web::Error> {
    // Try naive html get first then use the headless browser approach
    let webpage_content = op::get_webpage_text_fetch(&data.url_source).await
        .map_err(|err| ServiceError::BadRequest(format!("Could not fetch: {}", err)))?;

    let matcher = SkimMatcherV2::default();
    let (score, _) = matcher.fuzzy_indices(&webpage_content, &data.content).unwrap();

    Ok(HttpResponse::Ok().json(VerificationStatus {
        score,
        verified: score > 3000,
    }))
}
