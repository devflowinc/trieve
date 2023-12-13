use actix_web::web;
use diesel::RunQueryDsl;

use crate::{
    data::models::{Pool, StripeCustomer},
    errors::DefaultError,
};

pub async fn create_stripe_customer_query(
    email: String,
    pool: web::Data<Pool>,
) -> Result<StripeCustomer, DefaultError> {
    use crate::data::schema::stripe_customers::dsl as stripe_customers_columns;

    let mut create_stripe_customer_data =
        stripe::generated::core::customer::CreateCustomer::default();
    create_stripe_customer_data.email = Some(&email);

    let stripe_secret = std::env::var("STRIPE_SECRET").expect("STRIPE_SECRET must be set");
    let stripe_client = stripe::Client::new(stripe_secret);

    let created_stripe_customer =
        stripe::Customer::create(&stripe_client, create_stripe_customer_data)
            .await
            .map_err(|e| {
                log::error!("Failed to create stripe customer: {}", e);
                DefaultError {
                    message: "Failed to create stripe customer",
                }
            })?;

    let stripe_customer = StripeCustomer::from_details(
        created_stripe_customer.id.to_string(),
        created_stripe_customer.email.expect("Email is required"),
    );

    let mut conn = pool.get().expect("Failed to get connection from pool");
    let created_stripe_customer: StripeCustomer =
        diesel::insert_into(stripe_customers_columns::stripe_customers)
            .values(&stripe_customer)
            .get_result(&mut conn)
            .map_err(|e| {
                log::error!("Failed to insert stripe customer: {}", e);
                DefaultError {
                    message: "Failed to insert stripe customer",
                }
            })?;

    Ok(created_stripe_customer)
}
