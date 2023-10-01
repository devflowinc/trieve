use crate::{
    data::models::{FileUploadCompletedNotificationWithName, Pool},
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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]

pub async fn get_notifications(
    user: LoggedUser,
    page: web::Path<i64>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user.id;

    let notifications =
        web::block(move || get_notifications_query(user_id, page.into_inner(), pool))
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

    web::block(move || {
        mark_notification_as_read_query(user_id, notification_id.into_inner().notification_id, pool)
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

    web::block(move || mark_all_notifications_as_read_query(user_id, pool))
        .await?
        .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    Ok(HttpResponse::NoContent().into())
}
