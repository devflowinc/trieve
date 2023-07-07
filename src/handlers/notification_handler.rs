use std::sync::{Arc, Mutex};

use crate::{
    data::models::{Pool, VerificationNotification},
    errors::ServiceError,
    operators::notification_operator::{
        add_verificiation_notification_query, get_notifications_query,
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use super::auth_handler::LoggedUser;

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
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum NotificationTypes {
    Verification(Vec<VerificationNotification>),
}

pub async fn get_notifications(
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user.id;
    let thread_safe_pool = Arc::new(Mutex::new(pool));

    let notifications =
        web::block(move || get_notifications_query(user_id, thread_safe_pool.lock().unwrap()))
            .await?
            .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    Ok(HttpResponse::Ok().json(notifications))
}
