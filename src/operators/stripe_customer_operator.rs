use actix_web::web;
use stripe::CreateCustomer;

use crate::data::models::Pool;
use crate::diesel::prelude::*;
use crate::{data::models::StripeCustomer, errors::DefaultError};

pub async fn create_stripe_customer_query(
    email: String,
    pool: &web::Data<Pool>,
) -> Result<StripeCustomer, DefaultError> {
    use crate::data::schema::stripe_customers::dsl::stripe_customers;

    let stripe_api_secret_key =
        std::env::var("STRIPE_API_SECRET_KEY").expect("STRIPE_API_SECRET_KEY must be set");
    let stripe_client = stripe::Client::new(stripe_api_secret_key);
    let new_full_customer = stripe::Customer::create(
        &stripe_client,
        CreateCustomer {
            email: Some(&email),
            ..Default::default()
        },
    )
    .await
    .map_err(|_stripe_error| DefaultError {
        message: "Error creating new stripe customer, try again".into(),
    })?;

    let new_stripe_customer = StripeCustomer::from_details(
        new_full_customer.id.to_string(),
        new_full_customer.email.unwrap_or_else(|| "".into()),
    );

    let mut conn = pool.get().unwrap();

    let inserted_stripe_customer = diesel::insert_into(stripe_customers)
        .values(&new_stripe_customer)
        .get_result(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error inserting new stripe customer, try again".into(),
        })?;

    Ok(inserted_stripe_customer)
}
