use std::sync::{Arc, Mutex};

use crate::{
    data::models::{Pool, VerificationNotification},
    errors::ServiceError,
    operators::notification_operator::add_verificiation_notification_query,
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct VerificationNotificationData {
    pub card_uuid: uuid::Uuid,
    pub user_uuid: uuid::Uuid,
    pub verification_uuid: uuid::Uuid,
}

pub async fn create_verificiation_notification(
    data: web::Json<VerificationNotificationData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let thread_safe_pool = Arc::new(Mutex::new(pool));
    let data = data.into_inner();

    web::block(move || {
        add_verificiation_notification_query(
            &VerificationNotification::from_details(
                data.card_uuid,
                data.user_uuid,
                data.verification_uuid,
            ),
            thread_safe_pool.lock().unwrap(),
        )
    })
    .await?
    .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    Ok(HttpResponse::NoContent().into())
}
