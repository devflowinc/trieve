use std::sync::{Arc, Mutex};

use crate::data::models::VerificationNotification;
use crate::operators::card_operator as card_op;
use crate::operators::notification_operator::add_verificiation_notification_query;
use crate::{data::models::Pool, errors::ServiceError, operators::verification_operator as op};
use actix_web::{web, HttpResponse};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::{Deserialize, Serialize};

use super::auth_handler::LoggedUser;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum VerifyData {
    ContentVerification { content: String, url_source: String },
    CardVerification { card_uuid: uuid::Uuid },
}

pub async fn get_webpage_score(
    url_source: &String,
    content: &str,
) -> Result<i64, actix_web::Error> {
    let webpage_content = op::get_webpage_text_fetch(url_source)
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Could not fetch: {}", err)))?;

    let matcher = SkimMatcherV2::default();
    let (score, _) = matcher
        .fuzzy_indices(&webpage_content, content)
        .unwrap_or((0, vec![]));

    Ok(score)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VerificationStatus {
    pub score: i64,
}

pub async fn verify_card_content(
    data: web::Json<VerifyData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    // Try naive html get first then use the headless browser approach
    let thread_safe_pool = Arc::new(Mutex::new(pool));
    let pool1 = thread_safe_pool.clone();
    let pool2 = thread_safe_pool.clone();
    let data = data.into_inner();

    let score = match data.clone() {
        VerifyData::ContentVerification {
            content,
            url_source,
        } => get_webpage_score(&url_source, &content).await?,

        VerifyData::CardVerification { card_uuid } => {
            let card = web::block(move || {
                card_op::get_metadata_from_id_query(card_uuid, pool1.lock().unwrap())
            })
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
            let link = card
                .link
                .ok_or_else(|| ServiceError::BadRequest("No link on this card to verify".into()))?;

            get_webpage_score(&link, &card.content).await?
        }
    };

    if let VerifyData::CardVerification { card_uuid } = data {
        // This is a vault call, so we need to update the card with the score
        web::block(move || op::upsert_card_verification_query(thread_safe_pool, card_uuid, score))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.to_string()))?;

        web::block(move || {
            add_verificiation_notification_query(
                &VerificationNotification::from_details(card_uuid, user.id, uuid::Uuid::new_v4()),
                pool2.lock().unwrap(),
            )
        })
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.to_string()))?;
    }

    Ok(HttpResponse::Ok().json(VerificationStatus { score }))
}
