use crate::diesel::prelude::*;
use crate::{
    data::models::{Pool, User},
    errors::DefaultError,
};
use actix_web::web;

pub fn get_user_by_email_query(
    user_email: &String,
    pool: &web::Data<Pool>,
) -> Result<User, DefaultError> {
    use crate::data::schema::users::dsl::*;

    let mut conn = pool.get().unwrap();

    let user: Option<User> = users
        .filter(email.eq(user_email))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| DefaultError {
            message: "Error loading user",
        })?;
    match user {
        Some(user) => Ok(user),
        None => Err(DefaultError {
            message: "User not found",
        }),
    }
}
