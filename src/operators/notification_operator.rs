use std::sync::MutexGuard;

use diesel::RunQueryDsl;

use crate::{
    data::models::{Pool, VerificationNotification},
    errors::DefaultError,
};

pub fn add_verificiation_notification_query(
    notification: &VerificationNotification,
    pool: MutexGuard<'_, actix_web::web::Data<Pool>>,
) -> Result<(), DefaultError> {
    use crate::data::schema::verification_notifications::dsl as verification_notifications_columns;

    let mut conn = pool.get().unwrap();

    diesel::insert_into(verification_notifications_columns::verification_notifications)
        .values(notification)
        .execute(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to create notification",
        })?;

    Ok(())
}
