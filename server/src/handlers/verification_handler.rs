use crate::{errors::ServiceError, get_env};

use actix_web::{web, HttpResponse};
use redis::Commands;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum VerifyData {
    CardVerification { card_uuid: uuid::Uuid },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VerificationStatus {
    pub score: i64,
}

pub async fn verify_card_content(
    data: web::Json<VerifyData>,
) -> Result<HttpResponse, actix_web::Error> {
    // Try naive html get first then use the headless browser approach
    let data = data.into_inner();
    let redis_url = get_env!("REDIS_URL", "REDIS_URL should be set");
    let client = redis::Client::open(redis_url)
        .map_err(|err| ServiceError::BadRequest(format!("Could not connect to redis: {}", err)))?;
    let mut con = client
        .get_connection()
        .map_err(|err| ServiceError::BadRequest(format!("Could not connect to redis: {}", err)))?;

    match data {
        VerifyData::CardVerification { card_uuid } => {
            con.set(format!("Verify: {}", card_uuid), true)
                .map_err(|err| {
                    ServiceError::BadRequest(format!("Could not set redis key: {}", err))
                })?;
        }
    };

    Ok(HttpResponse::NoContent().finish())
}
