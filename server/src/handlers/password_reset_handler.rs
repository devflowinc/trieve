use crate::data::models::Pool;
use crate::data::validators::email_regex;
use crate::errors::DefaultError;
use crate::operators::password_reset_operator::{reset_user_password, send_password_reset_email};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct PasswordResetData {
    pub password_reset_id: String,
    pub password: String,
    pub password_confirmation: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PasswordResetEmailData {
    pub email: String,
}

#[utoipa::path(
    post,
    path = "/password",
    context_path = "/api",
    tag = "password",
    request_body(content = PasswordResetData, description = "JSON request payload to reset password", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the password was reset"),
        (status = 400, description = "Password either too short or did not match confirmation password", body = [DefaultError]),
    )
)]
pub async fn reset_user_password_handler(
    password_reset_data: web::Json<PasswordResetData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let password_reset_data_inner = password_reset_data.into_inner();
    if password_reset_data_inner.password != password_reset_data_inner.password_confirmation {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Passwords do not match",
        }));
    }
    if password_reset_data_inner.password.len() < 8 {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Password must be at least 8 characters long",
        }));
    }

    let reset_user_pass_result = web::block(move || {
        reset_user_password(
            password_reset_data_inner.password_reset_id,
            password_reset_data_inner.password,
            &pool,
        )
    })
    .await?;

    match reset_user_pass_result {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[utoipa::path(
    get,
    path = "/password/{email}",
    context_path = "/api",
    tag = "password",
    request_body(content = PasswordResetEmailData, description = "JSON request payload to send email with password reset link", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the password reset email was sent"),
        (status = 400, description = "Email was invalid or sendgrid had an error", body = [DefaultError]),
    ),
    params(
        ("email" = string, description = "Email of the user to send the password reset email to"),
    )
)]
pub async fn send_password_reset_email_handler(
    request: HttpRequest,
    password_reset_email_data: web::Path<PasswordResetEmailData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_email = password_reset_email_data.email.clone();
    if !email_regex().is_match(&user_email) {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Invalid email",
        }));
    }

    // get the host from the request
    let app_url = request
        .headers()
        .get("Origin")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let send_reset_email_result =
        web::block(move || send_password_reset_email(app_url, user_email.to_string(), &pool))
            .await?;

    match send_reset_email_result {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}
