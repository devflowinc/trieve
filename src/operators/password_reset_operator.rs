use crate::data::models::{PasswordReset, Pool, User};
use crate::diesel::prelude::*;
use crate::errors::DefaultError;
use crate::handlers::register_handler::hash_password;
use crate::operators::email_operator::send_password_reset;
use actix_web::web;

pub fn reset_user_password(
    password_reset_id: String,
    password: String,
    pool: &web::Data<Pool>,
) -> Result<(), DefaultError> {
    let password_reset = get_password_reset_query(password_reset_id, pool)?;

    // check if password reset is expired
    if password_reset.expires_at < chrono::Local::now().naive_local() {
        return Err(DefaultError {
            message: "Password reset request expired",
        });
    }

    reset_user_password_query(password_reset, password, pool)?;

    Ok(())
}

pub fn send_password_reset_email(
    user_email: String,
    pool: &web::Data<Pool>,
) -> Result<(), DefaultError> {
	let user = get_user_query(&user_email, pool)?;

    let password_reset = create_password_reset_query(user.email, pool)?;

    send_password_reset(&password_reset)?;

    Ok(())
}

pub fn get_user_query(user_email: &String, pool: &web::Data<Pool>) -> Result<User, DefaultError> {
    use crate::data::schema::users::dsl::*;

    let mut conn = pool.get().unwrap();

    let user: User = users
        .filter(email.eq(user_email))
        .first::<User>(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "There is no account associated with that email",
        })?;

    Ok(user)
}

fn create_password_reset_query(
    email: String,
    pool: &web::Data<Pool>,
) -> Result<PasswordReset, DefaultError> {
    use crate::data::schema::password_resets::dsl::password_resets;

    let mut conn = pool.get().unwrap();

    let new_password_reset = PasswordReset::from(email);

    let inserted_password_reset = diesel::insert_into(password_resets)
        .values(&new_password_reset)
        .get_result(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error inserting new password reset request, try again",
        })?;

    Ok(inserted_password_reset)
}

fn get_password_reset_query(
    password_reset_id: String,
    pool: &web::Data<Pool>,
) -> Result<PasswordReset, DefaultError> {
    use crate::data::schema::password_resets::dsl::*;

    let mut conn = pool.get().unwrap();

    let password_reset_id =
        uuid::Uuid::try_parse(&password_reset_id).map_err(|_uuid_error| DefaultError {
            message: "Invalid password reset id",
        })?;

    let password_reset = password_resets
        .find(password_reset_id)
        .first(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Invalid password reset invitation",
        })?;

    Ok(password_reset)
}

fn reset_user_password_query(
    password_reset: PasswordReset,
    password: String,
    pool: &web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::users::dsl::*;

    let mut conn = pool.get().unwrap();

    let password: String = hash_password(&password)?;

    let user: User = users
        .filter(email.eq(password_reset.email))
        .first::<User>(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "There is no account associated with that email",
        })?;

    diesel::update(users.find(user.id))
        .set(hash.eq(password))
        .execute(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error updating user password",
        })?;

    Ok(())
}
