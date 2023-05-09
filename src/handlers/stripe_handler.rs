use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::Pool,
    operators::stripe_customer_operator::{
        create_stripe_checkout_session_query, create_stripe_customer_query,
        get_stripe_customer_query,
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
        create_stripe_checkout_session_query(stripe_customer, plan_id.into_inner())
            .await
            .map_err(actix_web::error::ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(StripeCheckoutSessionResponseDTO {
        checkout_session_url,
    }))
}
