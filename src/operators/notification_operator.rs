use std::sync::MutexGuard;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::{
    data::models::{Pool, VerificationNotification},
    errors::DefaultError,
    handlers::notification_handler::NotificationTypes,
};

pub fn add_verificiation_notification_query(
    notification: VerificationNotification,
    pool: MutexGuard<'_, actix_web::web::Data<Pool>>,
) -> Result<(), DefaultError> {
    use crate::data::schema::verification_notifications::dsl as verification_notifications_columns;

    let mut conn = pool.get().unwrap();

    diesel::insert_into(verification_notifications_columns::verification_notifications)
        .values(&notification)
        .execute(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to create notification",
        })?;

    Ok(())
}

pub fn get_notifications_query(
    user_id: uuid::Uuid,
    pool: MutexGuard<'_, actix_web::web::Data<Pool>>,
) -> Result<NotificationTypes, DefaultError> {
    use crate::data::schema::verification_notifications::dsl as verification_notifications_columns;

    let mut conn = pool.get().unwrap();

    let notifications = verification_notifications_columns::verification_notifications
        .filter(verification_notifications_columns::user_uuid.eq(user_id))
        .load::<VerificationNotification>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to get notifications",
        })?;

    Ok(NotificationTypes::Verification(notifications))
}

pub fn mark_notification_as_read_query(
    user_id: uuid::Uuid,
    notification_id: uuid::Uuid,
    pool: MutexGuard<'_, actix_web::web::Data<Pool>>,
) -> Result<(), DefaultError> {
    use crate::data::schema::verification_notifications::dsl as verification_notifications_columns;

    let mut conn = pool.get().unwrap();

    diesel::update(
        verification_notifications_columns::verification_notifications
            .filter(verification_notifications_columns::user_uuid.eq(user_id))
            .filter(verification_notifications_columns::id.eq(notification_id)),
    )
    .set(verification_notifications_columns::user_read.eq(true))
    .execute(&mut conn)
    .map_err(|_| DefaultError {
        message: "Failed to mark notification as read",
    })?;

    Ok(())
}

pub fn mark_all_notifications_as_read_query(
    user_id: uuid::Uuid,
    pool: MutexGuard<'_, actix_web::web::Data<Pool>>,
) -> Result<(), DefaultError> {
    use crate::data::schema::verification_notifications::dsl as verification_notifications_columns;

    let mut conn = pool.get().unwrap();

    diesel::update(
        verification_notifications_columns::verification_notifications
            .filter(verification_notifications_columns::user_uuid.eq(user_id)),
    )
    .set(verification_notifications_columns::user_read.eq(true))
    .execute(&mut conn)
    .map_err(|_| DefaultError {
        message: "Failed to mark all notifications as read",
    })?;

    Ok(())
}
