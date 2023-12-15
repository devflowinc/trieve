use crate::{
    data::models::{Pool, StripePlan, StripeSubscription},
    errors::DefaultError,
};
use actix_web::web;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use stripe::{
    CreatePaymentLink, CreatePaymentLinkAfterCompletion, CreatePaymentLinkAfterCompletionRedirect,
    CreatePaymentLinkAfterCompletionType, CreatePaymentLinkLineItems, PaymentLink,
};

pub fn get_stripe_client() -> stripe::Client {
    let stripe_secret = std::env::var("STRIPE_SECRET").expect("STRIPE_SECRET must be set");
    let stripe_client = stripe::Client::new(stripe_secret);

    stripe_client
}

pub fn create_stripe_subscription_query(
    stripe_id: String,
    plan_id: uuid::Uuid,
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

    let stripe_subscription = StripeSubscription::from_details(stripe_id, plan_id, organization_id);

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
    amount: i64,
    pool: web::Data<Pool>,
) -> Result<StripePlan, DefaultError> {
    use crate::data::schema::stripe_plans::dsl as stripe_plans_columns;

    let stripe_plan = StripePlan::from_details(stripe_id, 10000, 1000000000, 100, 1, 10000, amount);

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

pub fn get_plan_by_id_query(
    plan_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<StripePlan, DefaultError> {
    use crate::data::schema::stripe_plans::dsl as stripe_plans_columns;

    let mut conn = pool.get().expect("Failed to get connection from pool");
    let stripe_plan: StripePlan = stripe_plans_columns::stripe_plans
        .filter(stripe_plans_columns::id.eq(plan_id))
        .first(&mut conn)
        .map_err(|e| {
            log::error!("Failed to get stripe plan: {}", e);
            DefaultError {
                message: "Failed to get stripe plan",
            }
        })?;

    Ok(stripe_plan)
}

pub async fn create_stripe_payment_link(
    plan: StripePlan,
    organization_id: uuid::Uuid,
) -> Result<String, DefaultError> {
    let mut payment_link_line_items = CreatePaymentLinkLineItems::default();
    payment_link_line_items.quantity = 1;
    payment_link_line_items.price = plan.stripe_id;

    let admin_dashboard_url =
        std::env::var("ADMIN_DASHBOARD_URL").expect("ADMIN_DASHBOARD_URL must be set");
    let mut create_payment_link = CreatePaymentLink::new(vec![payment_link_line_items]);
    create_payment_link.after_completion = Some(CreatePaymentLinkAfterCompletion {
        redirect: Some(CreatePaymentLinkAfterCompletionRedirect {
            url: admin_dashboard_url,
        }),
        hosted_confirmation: None,
        type_: CreatePaymentLinkAfterCompletionType::Redirect,
    });
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("organization_id".to_string(), organization_id.to_string());
    metadata.insert("plan_id".to_string(), plan.id.to_string());
    create_payment_link.metadata = Some(metadata);

    let stripe_client = get_stripe_client();

    let payment_link = PaymentLink::create(&stripe_client, create_payment_link)
        .await
        .map_err(|e| {
            log::error!("Failed to create stripe payment link: {}", e);
            DefaultError {
                message: "Failed to create stripe payment link",
            }
        })?
        .url;

    Ok(payment_link)
}

pub fn get_subscription_by_id_query(
    subscription_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<StripeSubscription, DefaultError> {
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

    let mut conn = pool.get().expect("Failed to get connection from pool");
    let stripe_subscription: StripeSubscription =
        stripe_subscriptions_columns::stripe_subscriptions
            .filter(stripe_subscriptions_columns::id.eq(subscription_id))
            .first(&mut conn)
            .map_err(|e| {
                log::error!("Failed to get stripe subscription: {}", e);
                DefaultError {
                    message: "Failed to get stripe subscription",
                }
            })?;

    Ok(stripe_subscription)
}

pub async fn cancel_stripe_subscription(
    subscription_stripe_id: String,
) -> Result<(), DefaultError> {
    let stripe_secret = std::env::var("STRIPE_SECRET").expect("STRIPE_SECRET must be set");
    let stripe_client = reqwest::Client::new();
    let stripe_response = stripe_client
        .delete(&format!(
            "https://api.stripe.com/v1/subscriptions/{}",
            subscription_stripe_id
        ))
        .bearer_auth(stripe_secret)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to cancel stripe subscription: {}", e);
            DefaultError {
                message: "Request to stripe failed",
            }
        })?;

    if !stripe_response.status().is_success() {
        log::error!(
            "Failed to cancel stripe subscription: {}",
            stripe_response.text().await.unwrap()
        );
        return Err(DefaultError {
            message: "Request to stripe failed",
        });
    }

    Ok(())
}
