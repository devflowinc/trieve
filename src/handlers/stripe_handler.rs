use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::Pool,
    operators::stripe_customer_operator::{
        create_stripe_checkout_session_operation, create_stripe_customer_query,
        get_stripe_customer_query, handle_webhook_query,
    },
};

use super::auth_handler::LoggedUser;

#[derive(Debug, Deserialize, Serialize)]
pub struct StripeCheckoutSessionResponseDTO {
    checkout_session_url: String,
}

pub async fn create_stripe_checkout_session(
    plan_id: web::Path<String>,
    user: Option<LoggedUser>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let stripe_customer = match user {
        Some(user) => web::block(move || get_stripe_customer_query(user.email, &pool))
            .await?
            .map_err(actix_web::error::ErrorInternalServerError)?,
        None => create_stripe_customer_query(None, pool)
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?,
    };

    let checkout_session_url =
        create_stripe_checkout_session_operation(stripe_customer, plan_id.into_inner())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(StripeCheckoutSessionResponseDTO {
        checkout_session_url,
    }))
}

pub async fn stripe_webhook(
    req: HttpRequest,
    payload: web::Bytes,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let stripe_signature = match get_header_value(&req, "Stripe-Signature") {
        None => return Ok(HttpResponse::BadRequest().finish()),
        Some(stripe_signature) => stripe_signature,
    };

    let _ = web::block(move || handle_webhook_query(&stripe_signature, payload, &pool))
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().finish())
}

fn get_header_value<'b>(req: &'b HttpRequest, key: &'b str) -> Option<String> {
    let header_val = req.headers().get(key)?.to_str().ok();

    match header_val {
        Some(val) => Some(val.to_string()),
        None => None,
    }
}
