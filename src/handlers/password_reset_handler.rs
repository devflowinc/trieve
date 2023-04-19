use crate::data::models::Pool;
use crate::data::validators::email_regex;
use crate::errors::DefaultError;
use crate::operators::password_reset_operator::{reset_user_password, send_password_reset_email};
use actix_web::{web, HttpResponse};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PasswordResetData {
    pub password_reset_id: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetEmailData {
    pub email: String,
}

pub async fn reset_user_password_handler(
    password_reset_data: web::Json<PasswordResetData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    web::block(move || {
        reset_user_password(
            password_reset_data.password_reset_id.clone(),
            password_reset_data.password.clone(),
            &pool,
        )
    })
    .await??;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn send_password_reset_email_handler(
    password_reset_email_data: web::Path<PasswordResetEmailData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_email = password_reset_email_data.email.clone();
    if !email_regex().is_match(&user_email) {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Invalid Email",
        }));
    }

    let send_reset_email_result =
        web::block(move || send_password_reset_email(user_email.to_string(), &pool)).await?;

    match send_reset_email_result {
        Ok(()) => {
            Ok(HttpResponse::NoContent().finish())
        }
        Err(e) => {
            Ok(HttpResponse::BadRequest().json(e))
        }
    }
}
