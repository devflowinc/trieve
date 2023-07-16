use std::sync::{Arc, Mutex};

use crate::{
    data::models::{FileUploadCompledNotification, Pool, VerificationNotification},
    errors::ServiceError,
    operators::notification_operator::{
        get_notifications_query, mark_all_notifications_as_read_query,
        mark_notification_as_read_query,
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
    pub score: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Notification {
    Verification(VerificationNotification),
    FileUploadComplete(FileUploadCompledNotification),
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NotificationId {
    pub notification_id: uuid::Uuid,
}
pub async fn mark_notification_as_read(
    user: LoggedUser,
    notification_id: web::Json<NotificationId>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user.id;
    let thread_safe_pool = Arc::new(Mutex::new(pool));

    web::block(move || {
        mark_notification_as_read_query(
            user_id,
            notification_id.into_inner().notification_id,
            thread_safe_pool.lock().unwrap(),
        )
    })
    .await?
    .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    Ok(HttpResponse::NoContent().into())
}

pub async fn mark_all_notifications_as_read(
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user.id;
    let thread_safe_pool = Arc::new(Mutex::new(pool));

    web::block(move || {
        mark_all_notifications_as_read_query(user_id, thread_safe_pool.lock().unwrap())
    })
    .await?
    .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    Ok(HttpResponse::NoContent().into())
}
