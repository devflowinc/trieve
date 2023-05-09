use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::Pool, operators::stripe_customer_operator::{create_stripe_checkout_session_query, get_stripe_customer_query},
};

use super::auth_handler::LoggedUser;

#[derive(Debug, Deserialize, Serialize)]
pub struct StripeCheckoutSessionResponseDTO {
    checkout_session_url: String,
}



pub async fn create_stripe_checkout_session(
    plan_id: web::Path<String>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let stripe_customer_result = 
        web::block(move || get_stripe_customer_query(user.email, &pool))
            .await?;

    let stripe_customer = match stripe_customer_result {
        Ok(stripe_customer) => stripe_customer,
        Err(e) => {
            return Ok(HttpResponse::BadRequest().json(e));
        }
    };

    let checkout_session_url = 
        create_stripe_checkout_session_query(
            stripe_customer,
            plan_id.into_inner(),
        ).await;

    match checkout_session_url {
        Ok(checkout_session_url) => {
            Ok(HttpResponse::Ok().json(
                StripeCheckoutSessionResponseDTO {
                    checkout_session_url,
                },
            ))
        },
        Err(e) => {
            Ok(HttpResponse::BadRequest().json(e))
        }
    }
}
