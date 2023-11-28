use std::cmp;

use actix_web::{HttpResponse, web};
use serde::{Serialize, Deserialize};
use actix::prelude::*;
use redis::Client;

use crate::{errors::ServiceError, get_env};


#[derive(Serialize, Deserialize)]
pub struct IngestRequest {

}

// Add things into redis queue
pub async fn ingest(
    data: web::Json<IngestRequest>,
) -> Result<HttpResponse, ServiceError>{


    Ok(HttpResponse::Ok().json("Hello, world!"))
}



#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateCardData {
    pub card_html: Option<String>,
    pub link: Option<String>,
    pub tag_set: Option<String>,
    pub private: Option<bool>,
    pub file_uuid: Option<uuid::Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub collection_id: Option<uuid::Uuid>,
    pub time_stamp: Option<String>,
}

struct QueueActor {
    api_endpoint: String,
    api_key: String,
}

async fn create_card(api_endpoint: &str, api_key: &str, msg: &CreateCardData) -> Result<(), ServiceError> {
    // Hit the monolith with my with the data in ProcessQueueItem
    let url = format!("{}/file", api_endpoint);
    let client = reqwest::Client::new();

    let message = serde_json::to_string(msg).map_err(|err| {
        ServiceError::BadRequest(err.to_string())
    })?;

    client
        .post(&url)
        .header("Accept", "text/html")
        .header("Content-Type", "application/json")
        .header("Authorization", api_key)
        .body(message)
        .send()
        .await
        .map(|_| ())
        .map_err(|err| {
            ServiceError::BadRequest(err.to_string())
        })
}

async fn insert_into_redis(msg: &CreateCardData) {

}

async fn pop_from_redis() -> Option<CreateCardData> {

    Some(CreateCardData {
                card_html: None,
                link: None,
                tag_set: None,
                private: None,
                file_uuid: None,
                metadata: None,
                tracking_id: None,
                collection_id: None,
                time_stamp: None,
            })

}

pub fn start_redis_thread() {

    tokio::spawn(async move {
        let mut sleep_time_ms = 100;
        let api_endpoint = get_env!("API_ENDPOINT", "API_ENDPOINT must exist").to_string();
        let api_key = get_env!("API_KEY", "API_KEY must exist").to_string();


        loop {
            // Poll next item off of redis
            let item = pop_from_redis().await;
            match item {
                Some(item) => {
                    sleep_time_ms = 100;
                    let response = create_card(&api_endpoint, &api_key, &item).await;

                    if let Err(err) = response {
                        insert_into_redis(&item).await;
                    }
                },
                None => {
                    sleep_time_ms = cmp::max(sleep_time_ms * 2, 10_000);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(sleep_time_ms)).await;
        }
    });
}
