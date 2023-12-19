use crate::{
    data::models::Pool,
    errors::ServiceError,
    operators::{
        organization_operator::get_organization_by_id_query,
        stripe_operator::{
            cancel_stripe_subscription, create_stripe_payment_link, create_stripe_plan_query,
            create_stripe_subscription_query, delete_subscription_by_id_query,
            get_option_subscription_by_organization_id_query, get_plan_by_id_query,
            get_subscription_by_id_query, set_stripe_subscription_current_period_end,
            update_stripe_subscription, update_stripe_subscription_plan_query,
        },
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use stripe::{EventObject, EventType, Webhook};
use utoipa::ToSchema;

use super::auth_handler::OwnerOnly;

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
                    let optional_subscription_pool = pool.clone();
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

                    let fetch_subscription_organization_id = organization_id;

                    let optional_existing_subscription = web::block(move || {
                        get_option_subscription_by_organization_id_query(
                            fetch_subscription_organization_id,
                            optional_subscription_pool,
                        )
                    })
                    .await?
                    .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

                    if let Some(existing_subscription) = optional_existing_subscription {
                        let delete_subscription_pool = pool.clone();

                        delete_subscription_by_id_query(
                            existing_subscription.id,
                            delete_subscription_pool,
                        )
                        .await
                        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;
                    }

                    create_stripe_subscription_query(
                        subscription_stripe_id,
                        plan_id,
                        organization_id,
                        pool,
                    )
                    .await
                    .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?
                }
            }
            EventType::PlanCreated => {
                if let EventObject::Plan(plan) = event.data.object {
                    let plan_id = plan.id.to_string();
                    let plan_amount = plan.amount.expect("Plan must have an amount");

                    web::block(move || create_stripe_plan_query(plan_id, plan_amount, pool))
                        .await?
                        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;
                }
            }
            EventType::CustomerSubscriptionDeleted => {
                if let EventObject::Subscription(subscription) = event.data.object {
                    let subscription_stripe_id = subscription.id.to_string();

                    let current_period_end = chrono::NaiveDateTime::from_timestamp_micros(
                        subscription.current_period_end,
                    )
                    .expect("Failed to convert current_period_end to NaiveDateTime");

                    set_stripe_subscription_current_period_end(
                        subscription_stripe_id,
                        current_period_end,
                        pool,
                    )
                    .await
                    .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;
                }
            }
            _ => {}
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GetDirectPaymentLinkData {
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
        ("plan_id" = uuid, Path, description = "id of the plan you want to subscribe to"),
        ("organization_id" = uuid, Path, description = "id of the organization you want to subscribe to the plan"),
    ),
)]
pub async fn direct_to_payment_link(
    path_data: web::Path<GetDirectPaymentLinkData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_pool = pool.clone();
    let subscription_pool = pool.clone();
    let subscription_org_id = path_data.organization_id;

    let current_subscription = web::block(move || {
        get_option_subscription_by_organization_id_query(subscription_org_id, subscription_pool)
    })
    .await?
    .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    if current_subscription.is_some_and(|s| s.current_period_end.is_none()) {
        return Ok(HttpResponse::Conflict().finish());
    }

    let plan_id = path_data.plan_id;
    let organization_id = path_data.organization_id;
    let organization_id_clone = path_data.organization_id;
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

#[utoipa::path(
    delete,
    path = "/stripe/subscription/{subscription_id}",
    context_path = "/api",
    tag = "stripe",
    responses(
        (status = 200, description = "Confirmation that the subscription was cancelled"),
        (status = 400, description = "Service error relating to creating a URL for a stripe checkout page", body = [DefaultError]),
    ),
    params(
        ("subscription_id" = uuid, Path, description = "id of the subscription you want to cancel"),
    ),
)]
pub async fn cancel_subscription(
    subscription_id: web::Path<uuid::Uuid>,
    user: OwnerOnly,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let subscription = web::block(move || {
        get_subscription_by_id_query(subscription_id.into_inner(), pool.clone())
    })
    .await?
    .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    if subscription.organization_id != user.0.organization_id {
        return Ok(HttpResponse::Forbidden().finish());
    }

    cancel_stripe_subscription(subscription.stripe_id)
        .await
        .map_err(|e| {
            ServiceError::BadRequest(format!(
                "Failed to cancel stripe subscription: {}",
                e.message
            ))
        })?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateSubscriptionData {
    pub subscription_id: uuid::Uuid,
    pub plan_id: uuid::Uuid,
}

#[utoipa::path(
    patch,
    path = "/stripe/subscription_plan/{subscription_id}/{plan_id}",
    context_path = "/api",
    tag = "stripe",
    responses(
        (status = 200, description = "Confirmation that the subscription was updated to the new plan"),
        (status = 400, description = "Service error relating to updating the subscription to the new plan", body = [DefaultError]),
    ),
    params(
        ("subscription_id" = uuid, Path, description = "id of the subscription you want to update"),
        ("plan_id" = uuid, Path, description = "id of the plan you want to subscribe to"),
    ),
)]
pub async fn update_subscription_plan(
    path_data: web::Path<UpdateSubscriptionData>,
    user: OwnerOnly,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let get_subscription_pool = pool.clone();
    let get_plan_pool = pool.clone();
    let update_subscription_plan_pool = pool.clone();

    let subscription_id = path_data.subscription_id;
    let subscription =
        web::block(move || get_subscription_by_id_query(subscription_id, get_subscription_pool))
            .await?
            .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    if subscription.organization_id != user.0.organization_id {
        return Ok(HttpResponse::Forbidden().finish());
    }

    let plan_id = path_data.plan_id;
    let plan = web::block(move || get_plan_by_id_query(plan_id, get_plan_pool))
        .await?
        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    update_stripe_subscription(subscription.stripe_id, plan.stripe_id)
        .await
        .map_err(|e| {
            ServiceError::BadRequest(format!(
                "Failed to update stripe subscription: {}",
                e.message
            ))
        })?;

    update_stripe_subscription_plan_query(subscription.id, plan.id, update_subscription_plan_pool)
        .await
        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    Ok(HttpResponse::Ok().finish())
}
