use crate::{
    data::models::Pool,
    errors::ServiceError,
    operators::stripe_operator::{
        create_stripe_payment_link, create_stripe_plan_query, create_stripe_subscription_query,
        get_plan_by_id_query,
    },
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
                    let plan_amount = plan.amount.expect("Plan must have an amount");

                    create_stripe_plan_query(plan_id, plan_amount, pool)
                        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;
                }
            }
            _ => {}
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    get,
    path = "/stripe/payment_link/{plan_id}/{organization_id}",
    context_path = "/api",
    tag = "stripe",
    responses(
        (status = 303, description = "SeeOther response redirecting user to stripe checkout page"),
        (status = 400, description = "Service error relating to creating a URL for a stripe checkout page", body = [DefaultError]),
    ),
    params(
        ("plan_id" = Option<uuid>, Path, description = "id of the plan you want to subscribe to"),
        ("organization_id" = Option<uuid>, Path, description = "id of the organization you want to subscribe to the plan"),
    ),
)]
pub async fn direct_to_payment_link(
    pool: web::Data<Pool>,
    plan_id: web::Path<uuid::Uuid>,
    organization_id: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let plan = web::block(move || get_plan_by_id_query(plan_id.into_inner(), pool))
        .await?
        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    let payment_link = create_stripe_payment_link(plan, organization_id.into_inner())
        .await
        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    Ok(HttpResponse::SeeOther()
        .insert_header(("Location", payment_link))
        .finish())
}
