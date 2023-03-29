use crate::data::models::Pool;
use crate::services::password_reset_service::{reset_user_password, send_password_reset_email};
use actix_web::{HttpResponse, web};
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
    web::block(move || send_password_reset_email(password_reset_email_data.email.clone(), &pool))
        .await??;

    Ok(HttpResponse::NoContent().finish())
}
