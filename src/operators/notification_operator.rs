use actix_web::web;
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::{
        FileUploadCompletedNotification, FileUploadCompletedNotificationWithName, Pool,
        VerificationNotification,
    },
    errors::DefaultError,
    handlers::notification_handler::Notification,
};

pub fn add_collection_created_notification_query(
    collection: FileUploadCompletedNotification,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::file_upload_completed_notifications::dsl as file_upload_completed_notifications_columns;

    let mut conn = pool.get().unwrap();

    diesel::insert_into(
        file_upload_completed_notifications_columns::file_upload_completed_notifications,
    )
    .values(&collection)
    .execute(&mut conn)
    .map_err(|_| DefaultError {
        message: "Failed to create notification",
    })?;

    Ok(())
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NotificationReturn {
    pub notifications: Vec<Notification>,
    pub full_count: i32,
    pub total_pages: i64,
}
pub fn get_notifications_query(
    user_id: uuid::Uuid,
    page: i64,
    pool: web::Data<Pool>,
) -> Result<NotificationReturn, DefaultError> {
    use crate::data::schema::card_collection::dsl as card_collection_columns;
    use crate::data::schema::file_upload_completed_notifications::dsl as file_upload_completed_notifications_columns;
    use crate::data::schema::user_notification_counts::dsl as user_notification_counts_columns;
    use crate::data::schema::verification_notifications::dsl as verification_notifications_columns;

    let mut conn = pool.get().unwrap();

    let verifications = verification_notifications_columns::verification_notifications
        .filter(verification_notifications_columns::user_uuid.eq(user_id))
        .select(VerificationNotification::as_select())
        .limit(10)
        .offset((page - 1) * 10)
        .order(verification_notifications_columns::created_at.desc())
        .load::<VerificationNotification>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to get notifications",
        })?;

    let file_upload_completed =
        file_upload_completed_notifications_columns::file_upload_completed_notifications
            .left_outer_join(
                card_collection_columns::card_collection
                    .on(file_upload_completed_notifications_columns::collection_uuid
                        .eq(card_collection_columns::id)),
            )
            .left_outer_join(
                user_notification_counts_columns::user_notification_counts
                    .on(file_upload_completed_notifications_columns::user_uuid
                        .eq(user_notification_counts_columns::user_uuid)),
            )
            .filter(file_upload_completed_notifications_columns::user_uuid.eq(user_id))
            .select((
                FileUploadCompletedNotification::as_select(),
                card_collection_columns::name.nullable(),
                user_notification_counts_columns::notification_count.nullable(),
            ))
            .order(file_upload_completed_notifications_columns::created_at.desc())
            .limit(10)
            .offset((page - 1) * 10)
            .load::<(FileUploadCompletedNotification, Option<String>, Option<i32>)>(&mut conn)
            .map_err(|_| DefaultError {
                message: "Failed to get notifications",
            })?;

    let combined_count = match file_upload_completed.first() {
        Some((_, _, Some(count))) => count + verifications.len() as i32,
        _ => 0 as i32,
    };

    // Combine file_upload_completed and verifications in order of their created_at date property
    let mut combined_notifications: Vec<Notification> = file_upload_completed
        .iter()
        .map(|c| {
            Notification::FileUploadComplete(
                FileUploadCompletedNotificationWithName::from_file_upload_notification(
                    c.0.clone(),
                    c.1.clone().unwrap_or("".to_string()),
                ),
            )
        })
        .chain(
            verifications
                .iter()
                .map(|v| Notification::Verification(v.clone())),
        )
        .collect();

    // Sort the combined_notifications by their created_at date property
    combined_notifications.sort_by(|a, b| match (a, b) {
        (Notification::FileUploadComplete(a_data), Notification::Verification(b_data)) => {
            a_data.created_at.cmp(&b_data.created_at).reverse()
        }
        (Notification::Verification(a_data), Notification::FileUploadComplete(b_data)) => {
            a_data.created_at.cmp(&b_data.created_at).reverse()
        }
        (Notification::Verification(a_data), Notification::Verification(b_data)) => {
            a_data.created_at.cmp(&b_data.created_at).reverse()
        }
        (Notification::FileUploadComplete(a_data), Notification::FileUploadComplete(b_data)) => {
            a_data.created_at.cmp(&b_data.created_at).reverse()
        }
    });

    Ok(NotificationReturn {
        notifications: combined_notifications,
        full_count: combined_count,
        total_pages: ((combined_count) as f64 / 10.0).ceil() as i64,
    })
}

pub fn mark_notification_as_read_query(
    user_id: uuid::Uuid,
    notification_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::file_upload_completed_notifications::dsl as file_upload_completed_notifications_columns;
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

    let file_upload_completed_result = diesel::update(
        file_upload_completed_notifications_columns::file_upload_completed_notifications
            .filter(file_upload_completed_notifications_columns::user_uuid.eq(user_id))
            .filter(file_upload_completed_notifications_columns::id.eq(notification_id)),
    )
    .set(file_upload_completed_notifications_columns::user_read.eq(true))
    .execute(&mut conn);

    match verification_result.or(file_upload_completed_result) {
        Ok(_) => Ok(()),
        Err(_) => Err(DefaultError {
            message: "Failed to mark notification as read",
        }),
    }
}

pub fn mark_all_notifications_as_read_query(
    user_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::file_upload_completed_notifications::dsl as file_upload_completed_notifications_columns;
    use crate::data::schema::verification_notifications::dsl as verification_notifications_columns;

    let mut conn = pool.get().unwrap();

    let verification_result = diesel::update(
        verification_notifications_columns::verification_notifications
            .filter(verification_notifications_columns::user_uuid.eq(user_id)),
    )
    .set(verification_notifications_columns::user_read.eq(true))
    .execute(&mut conn);

    let file_upload_completed_result = diesel::update(
        file_upload_completed_notifications_columns::file_upload_completed_notifications
            .filter(file_upload_completed_notifications_columns::user_uuid.eq(user_id)),
    )
    .set(file_upload_completed_notifications_columns::user_read.eq(true))
    .execute(&mut conn);

    match verification_result.or(file_upload_completed_result) {
        Ok(_) => Ok(()),
        Err(_) => Err(DefaultError {
            message: "Failed to mark all notifications as read",
        }),
    }
}
