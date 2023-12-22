use crate::{
    data::models::{
        FileUploadCompletedNotification, FileUploadCompletedNotificationWithName, Pool,
    },
    errors::DefaultError,
    handlers::notification_handler::Notification,
};
use actix_web::web;
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

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
    .map_err(|err| {
        log::error!("Failed to create notification: {:?}", err);
        DefaultError {
            message: "Failed to create notification",
        }
    })?;

    Ok(())
}
#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct NotificationReturn {
    pub notifications: Vec<Notification>,
    pub full_count: i32,
    pub total_pages: i64,
}
pub fn get_notifications_query(
    user_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    page: i64,
    pool: web::Data<Pool>,
) -> Result<NotificationReturn, DefaultError> {
    use crate::data::schema::chunk_collection::dsl as chunk_collection_columns;
    use crate::data::schema::file_upload_completed_notifications::dsl as file_upload_completed_notifications_columns;
    use crate::data::schema::user_notification_counts::dsl as user_notification_counts_columns;

    let mut conn = pool.get().unwrap();

    let file_upload_completed =
        file_upload_completed_notifications_columns::file_upload_completed_notifications
            .left_outer_join(
                chunk_collection_columns::chunk_collection
                    .on(file_upload_completed_notifications_columns::collection_uuid
                        .eq(chunk_collection_columns::id)),
            )
            .left_outer_join(
                user_notification_counts_columns::user_notification_counts
                    .on(file_upload_completed_notifications_columns::user_uuid
                        .eq(user_notification_counts_columns::user_uuid)),
            )
            .filter(file_upload_completed_notifications_columns::user_uuid.eq(user_id))
            .filter(file_upload_completed_notifications_columns::dataset_id.eq(dataset_id))
            .select((
                FileUploadCompletedNotification::as_select(),
                chunk_collection_columns::name.nullable(),
                user_notification_counts_columns::notification_count.nullable(),
            ))
            .order(file_upload_completed_notifications_columns::created_at.desc())
            .limit(10)
            .offset((page - 1) * 10)
            .load::<(FileUploadCompletedNotification, Option<String>, Option<i32>)>(&mut conn)
            .map_err(|_| DefaultError {
                message: "Failed to get notifications",
            })?;

    let combined_notifications: Vec<Notification> = file_upload_completed
        .iter()
        .map(|c| {
            Notification::FileUploadComplete(
                FileUploadCompletedNotificationWithName::from_file_upload_notification(
                    c.0.clone(),
                    c.1.clone().unwrap_or("".to_string()),
                ),
            )
        })
        .collect();
    let notification_count = file_upload_completed.len();

    Ok(NotificationReturn {
        notifications: combined_notifications,
        full_count: notification_count as i32,
        total_pages: (notification_count as f64 / 10.0).ceil() as i64,
    })
}

pub fn mark_notification_as_read_query(
    user_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    notification_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::file_upload_completed_notifications::dsl as file_upload_completed_notifications_columns;
    let mut conn = pool.get().unwrap();

    let file_upload_completed_result = diesel::update(
        file_upload_completed_notifications_columns::file_upload_completed_notifications
            .filter(file_upload_completed_notifications_columns::user_uuid.eq(user_id))
            .filter(file_upload_completed_notifications_columns::dataset_id.eq(dataset_id))
            .filter(file_upload_completed_notifications_columns::id.eq(notification_id)),
    )
    .set(file_upload_completed_notifications_columns::user_read.eq(true))
    .execute(&mut conn);

    match file_upload_completed_result {
        Ok(_) => Ok(()),
        Err(_) => Err(DefaultError {
            message: "Failed to mark notification as read",
        }),
    }
}

pub fn mark_all_notifications_as_read_query(
    user_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::file_upload_completed_notifications::dsl as file_upload_completed_notifications_columns;

    let mut conn = pool.get().unwrap();

    let file_upload_completed_result = diesel::update(
        file_upload_completed_notifications_columns::file_upload_completed_notifications
            .filter(file_upload_completed_notifications_columns::dataset_id.eq(dataset_id))
            .filter(file_upload_completed_notifications_columns::user_uuid.eq(user_id)),
    )
    .set(file_upload_completed_notifications_columns::user_read.eq(true))
    .execute(&mut conn);

    match file_upload_completed_result {
        Ok(_) => Ok(()),
        Err(_) => Err(DefaultError {
            message: "Failed to mark all notifications as read",
        }),
    }
}
