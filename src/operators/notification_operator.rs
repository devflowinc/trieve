use std::sync::MutexGuard;

use actix_web::web;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

use crate::{
    data::models::{CollectionCreatedNotification, Pool, VerificationNotification},
    errors::DefaultError,
    handlers::notification_handler::Notification,
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

#[allow(dead_code)]
pub fn add_collection_created_notification_query(
    collection: CollectionCreatedNotification,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::collection_created_notifications::dsl as collection_created_notifications_columns;

    let mut conn = pool.get().unwrap();

    diesel::insert_into(collection_created_notifications_columns::collection_created_notifications)
        .values(&collection)
        .execute(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to create notification",
        })?;

    Ok(())
}

pub fn get_notifications_query(
    user_id: uuid::Uuid,
    pool: MutexGuard<'_, actix_web::web::Data<Pool>>,
) -> Result<Vec<Notification>, DefaultError> {
    use crate::data::schema::collection_created_notifications::dsl as collection_created_notifications_columns;
    use crate::data::schema::verification_notifications::dsl as verification_notifications_columns;

    let mut conn = pool.get().unwrap();

    let verifications = verification_notifications_columns::verification_notifications
        .filter(verification_notifications_columns::user_uuid.eq(user_id))
        .load::<VerificationNotification>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to get notifications",
        })?;

    let collection_created =
        collection_created_notifications_columns::collection_created_notifications
            .filter(collection_created_notifications_columns::user_uuid.eq(user_id))
            .load::<CollectionCreatedNotification>(&mut conn)
            .map_err(|_| DefaultError {
                message: "Failed to get notifications",
            })?;

    let mut notifications = verifications
        .iter()
        .map(|v| Notification::Verification(v.clone()))
        .collect::<Vec<Notification>>();
    notifications.extend(
        collection_created
            .iter()
            .map(|c| Notification::CollectionCreated(c.clone()))
            .collect::<Vec<Notification>>(),
    );

    Ok(notifications)
}

pub fn mark_notification_as_read_query(
    user_id: uuid::Uuid,
    notification_id: uuid::Uuid,
    pool: MutexGuard<'_, actix_web::web::Data<Pool>>,
) -> Result<(), DefaultError> {
    use crate::data::schema::collection_created_notifications::dsl as collection_created_notifications_columns;
    use crate::data::schema::verification_notifications::dsl as verification_notifications_columns;

    let mut conn = pool.get().unwrap();

    // We have to do both, just in case there is a weird collision between both tables
    let verification_result = diesel::update(
        verification_notifications_columns::verification_notifications
            .filter(verification_notifications_columns::user_uuid.eq(user_id))
            .filter(verification_notifications_columns::id.eq(notification_id)),
    )
    .set(verification_notifications_columns::user_read.eq(true))
    .execute(&mut conn);

    let collection_created_result = diesel::update(
        collection_created_notifications_columns::collection_created_notifications
            .filter(collection_created_notifications_columns::user_uuid.eq(user_id))
            .filter(collection_created_notifications_columns::id.eq(notification_id)),
    )
    .set(collection_created_notifications_columns::user_read.eq(true))
    .execute(&mut conn);

    match verification_result.or(collection_created_result) {
        Ok(_) => Ok(()),
        Err(_) => Err(DefaultError {
            message: "Failed to mark notification as read",
        }),
    }
}

pub fn mark_all_notifications_as_read_query(
    user_id: uuid::Uuid,
    pool: MutexGuard<'_, actix_web::web::Data<Pool>>,
) -> Result<(), DefaultError> {
    use crate::data::schema::collection_created_notifications::dsl as collection_created_notifications_columns;
    use crate::data::schema::verification_notifications::dsl as verification_notifications_columns;

    let mut conn = pool.get().unwrap();

    let verification_result = diesel::update(
        verification_notifications_columns::verification_notifications
            .filter(verification_notifications_columns::user_uuid.eq(user_id)),
    )
    .set(verification_notifications_columns::user_read.eq(true))
    .execute(&mut conn);

    let collection_created_result = diesel::update(
        collection_created_notifications_columns::collection_created_notifications
            .filter(collection_created_notifications_columns::user_uuid.eq(user_id)),
    )
    .set(collection_created_notifications_columns::user_read.eq(true))
    .execute(&mut conn);

    match verification_result.or(collection_created_result) {
        Ok(_) => Ok(()),
        Err(_) => Err(DefaultError {
            message: "Failed to mark all notifications as read",
        }),
    }
}
