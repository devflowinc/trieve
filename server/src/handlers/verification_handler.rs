use crate::{errors::ServiceError, get_env};

use actix_web::{web, HttpResponse};
use redis::Commands;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]

#[derive(Debug, Deserialize, Serialize)]

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

    Ok(HttpResponse::NoContent().finish())
}
