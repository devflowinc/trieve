use actix_web::web;
use diesel::RunQueryDsl;

use crate::{
    data::models::{Pool, StripeCustomer, StripePlan, StripeSubscription},
    errors::DefaultError,
};

pub async fn create_stripe_customer_query(
    email: String,
    pool: web::Data<Pool>,
) -> Result<StripeCustomer, DefaultError> {
    use crate::data::schema::stripe_customers::dsl as stripe_customers_columns;

    let create_stripe_customer_data = stripe::generated::core::customer::CreateCustomer {
        email: Some(&email),
        ..Default::default()
    };

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

pub fn create_stripe_subscription_query(
    stripe_id: String,
    stripe_plan_id: String,
    stripe_customer_id: String,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

    let stripe_subscription =
        StripeSubscription::from_details(stripe_id, stripe_plan_id, stripe_customer_id);

    let mut conn = pool.get().expect("Failed to get connection from pool");
    diesel::insert_into(stripe_subscriptions_columns::stripe_subscriptions)
        .values(&stripe_subscription)
        .execute(&mut conn)
        .map_err(|e| {
            log::error!("Failed to insert stripe subscription: {}", e);
            DefaultError {
                message: "Failed to insert stripe subscription",
            }
        })?;

    Ok(())
}

pub fn create_stripe_plan_query(
    stripe_id: String,
    pool: web::Data<Pool>,
) -> Result<StripePlan, DefaultError> {
    use crate::data::schema::stripe_plans::dsl as stripe_plans_columns;

    let stripe_plan = StripePlan::from_details(stripe_id, 10000, 1000000000, 100, 1, 10000);

    let mut conn = pool.get().expect("Failed to get connection from pool");
    let created_stripe_plan: StripePlan = diesel::insert_into(stripe_plans_columns::stripe_plans)
        .values(&stripe_plan)
        .get_result(&mut conn)
        .map_err(|e| {
            log::error!("Failed to insert stripe plan: {}", e);
            DefaultError {
                message: "Failed to insert stripe plan",
            }
        })?;

    Ok(created_stripe_plan)
}
