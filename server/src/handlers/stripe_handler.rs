use crate::{
    data::models::Pool,
    errors::ServiceError,
    operators::stripe_operator::{create_stripe_plan_query, create_stripe_subscription_query},
};
use actix_web::{web, HttpRequest, HttpResponse};
use stripe::{EventObject, EventType, Webhook};

pub async fn webhook(
    req: HttpRequest,
    payload: web::Bytes,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let payload_str = String::from_utf8(payload.to_vec()).expect("Failed to parse payload");

    let stripe_signature = req
        .headers()
        .get("Stripe-Signature")
        .expect("Stripe-Signature header is required")
        .to_str()
        .expect("Failed to parse Stripe-Signature header");

    let stripe_webhook_secret =
        std::env::var("STRIPE_WEBHOOK_SERCRET").expect("STRIPE_WEBHOOK_SERCRET must be set");

    if let Ok(event) =
        Webhook::construct_event(&payload_str, stripe_signature, &stripe_webhook_secret)
    {
        match event.type_ {
            EventType::CustomerSubscriptionUpdated => {
                if let EventObject::Subscription(subscription) = event.data.object {
                    let subscription_id = subscription.id.to_string();
                    let plan_id = <std::option::Option<stripe::Plan> as Clone>::clone(
                        &subscription
                            .items
                            .data
                            .first()
                            .expect("Subscription must have at least one item")
                            .plan,
                    )
                    .expect("Item must have a plan")
                    .id
                    .to_string();
                    let customer_id = subscription.customer.id().to_string();

                    create_stripe_subscription_query(subscription_id, plan_id, customer_id, pool)
                        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;
                }
            }
            EventType::PlanCreated => {
                if let EventObject::Plan(plan) = event.data.object {
                    let plan_id = plan.id.to_string();
                    create_stripe_plan_query(plan_id, pool)
                        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;
                }
            }
            _ => {}
        }
    }

    Ok(HttpResponse::Ok().finish())
}
