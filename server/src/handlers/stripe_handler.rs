use crate::{
    data::models::Pool,
    errors::ServiceError,
    operators::{
        organization_operator::get_organization_by_id_query,
        stripe_operator::{
            create_stripe_payment_link, create_stripe_plan_query, create_stripe_subscription_query,
            get_plan_by_id_query,
        },
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use stripe::{EventObject, EventType, Webhook};
use utoipa::ToSchema;

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
            EventType::CheckoutSessionCompleted => {
                if let EventObject::CheckoutSession(checkout_session) = event.data.object {
                    let subscription_stripe_id = checkout_session
                        .subscription
                        .expect("Checkout session must have a subscription")
                        .id()
                        .to_string();

                    let metadata = checkout_session
                        .metadata
                        .expect("Checkout session must have metadata");
                    let plan_id = metadata
                        .get("plan_id")
                        .expect("Checkout session must have a plan_id metadata")
                        .parse::<uuid::Uuid>()
                        .expect("plan_id metadata must be a uuid");
                    let organization_id = metadata
                        .get("organization_id")
                        .expect("Checkout session must have an organization_id metadata")
                        .parse::<uuid::Uuid>()
                        .expect("organization_id metadata must be a uuid");

                    create_stripe_subscription_query(
                        subscription_stripe_id,
                        plan_id,
                        organization_id,
                        pool,
                    )
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

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GetDirectPaymentLink {
    pub plan_id: uuid::Uuid,
    pub organization_id: uuid::Uuid,
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
    path_data: web::Path<GetDirectPaymentLink>,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_pool = pool.clone();

    let plan_id = path_data.plan_id.clone();
    let organization_id = path_data.organization_id.clone();
    let organization_id_clone = path_data.organization_id.clone();
    let _organization =
        web::block(move || get_organization_by_id_query(organization_id_clone, organization_pool))
            .await?
            .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    let plan = web::block(move || get_plan_by_id_query(plan_id, pool))
        .await?
        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    let payment_link = create_stripe_payment_link(plan, organization_id)
        .await
        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    Ok(HttpResponse::SeeOther()
        .insert_header(("Location", payment_link))
        .finish())
}
