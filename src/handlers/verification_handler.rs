use crate::AppMutexStore;
use crate::{errors::ServiceError, operators::verification_operator as op};
use actix_web::{web, HttpResponse};
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum VerifyData {
    ContentVerification { content: String, url_source: String },
    CardVerification { card_uuid: uuid::Uuid },
}

pub async fn get_webpage_score(
    url_source: &str,
    content: &str,
    mutex_store: web::Data<AppMutexStore>,
) -> Result<i64, actix_web::Error> {
    let webpage_content = &op::get_webpage_text_fetch(url_source)
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Could not fetch: {}", err)))?;

    let fuzzy_script_result = Command::new("python3")
        .arg("./vault-python/fuzzy-text-match.py")
        .arg(content)
        .arg(webpage_content)
        .output();

    let mut score;
    match fuzzy_script_result {
        Ok(result) => {
            if result.status.code().unwrap() != 0 {
                return Err(ServiceError::BadRequest(format!(
                    "Could not run fuzzy-text-match.py: {:?}",
                    String::from_utf8(result.stderr).unwrap()
                ))
                .into());
            }
            score = String::from_utf8(result.stdout)
                .unwrap()
                .trim_end_matches('\n')
                .parse::<u8>()
                .map_err(|err| {
                    ServiceError::BadRequest(format!("Could not parse score: {}", err))
                })?;
        }
        Err(_) => {
            return Err(
                ServiceError::BadRequest("Could not run fuzzy-text-match.py".into()).into(),
            );
        }
    }

    if score < 80 {
        let webpage_content = &op::get_webpage_text_headless(url_source, mutex_store)
            .await
            .map_err(|err| ServiceError::BadRequest(format!("Could not fetch: {}", err)))?;

        let fuzzy_script_result = Command::new("python3")
            .arg("./vault-python/fuzzy-text-match.py")
            .arg(content)
            .arg(webpage_content)
            .output();
        match fuzzy_script_result {
            Ok(result) => {
                score = String::from_utf8(result.stdout)
                    .unwrap()
                    .trim_end_matches('\n')
                    .parse::<u8>()
                    .map_err(|err| {
                        ServiceError::BadRequest(format!("Could not parse score: {}", err))
                    })?
            }
            Err(_) => {
                return Err(
                    ServiceError::BadRequest("Could not run fuzzy-text-match.py".into()).into(),
                );
            }
        }
    }

    Ok(score as i64)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VerificationStatus {
    pub score: i64,
}

pub async fn verify_card_content(
    data: web::Json<VerifyData>,
    mutex_store: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    // Try naive html get first then use the headless browser approach
    let data = data.into_inner();
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = redis::Client::open(redis_url)
        .map_err(|err| ServiceError::BadRequest(format!("Could not connect to redis: {}", err)))?;
    let mut con = client
        .get_connection()
        .map_err(|err| ServiceError::BadRequest(format!("Could not connect to redis: {}", err)))?;

    if let VerifyData::CardVerification { card_uuid } = data {
        con.set(format!("Verify: {}", card_uuid), true)
            .map_err(|err| ServiceError::BadRequest(format!("Could not set redis key: {}", err)))?;
    };

    if let VerifyData::ContentVerification {
        content,
        url_source,
    } = data
    {
        return Ok(HttpResponse::Ok().json(VerificationStatus {
            score: get_webpage_score(&url_source, &content, mutex_store).await?,
        }));
    }

    Ok(HttpResponse::NoContent().finish())
}
