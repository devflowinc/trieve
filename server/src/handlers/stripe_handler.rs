use crate::{
    data::models::Pool,
    errors::ServiceError,
    get_env,
    operators::{
        organization_operator::get_organization_by_key_query,
        stripe_operator::{
            cancel_stripe_subscription, create_stripe_payment_link, create_stripe_plan_query,
            create_stripe_subscription_query, delete_subscription_by_id_query, get_all_plans_query,
            get_option_subscription_by_organization_id_query, get_plan_by_id_query,
            get_subscription_by_id_query, refresh_redis_org_plan_sub,
            set_stripe_subscription_current_period_end, update_stripe_subscription,
            update_stripe_subscription_plan_query,
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
    let payload_str = String::from_utf8(payload.to_vec())
        .map_err(|_| ServiceError::BadRequest("Failed to parse payload".to_string()))?;

    let stripe_signature = req
        .headers()
        .get("Stripe-Signature")
        .ok_or(ServiceError::BadRequest(
            "Stripe-Signature header is required".to_string(),
        ))?
        .to_str()
        .map_err(|_| {
            ServiceError::BadRequest("Failed to parse Stripe-Signature header".to_string())
        })?;

    let stripe_webhook_secret =
        get_env!("STRIPE_WEBHOOK_SECRET", "STRIPE_WEBHOOK_SECRET must be set");

    if let Ok(event) =
        Webhook::construct_event(&payload_str, stripe_signature, stripe_webhook_secret)
    {
        match event.type_ {
            EventType::CheckoutSessionCompleted => {
                if let EventObject::CheckoutSession(checkout_session) = event.data.object {
                    let optional_subscription_pool = pool.clone();
                    let subscription_stripe_id = checkout_session
                        .subscription
                        .ok_or(ServiceError::BadRequest(
                            "Checkout session must have a subscription".to_string(),
                        ))?
                        .id()
                        .to_string();

                    let metadata = checkout_session.metadata.ok_or(ServiceError::BadRequest(
                        "Checkout session must have metadata".to_string(),
                    ))?;
                    let plan_id = metadata
                        .get("plan_id")
                        .ok_or(ServiceError::BadRequest(
                            "Checkout session must have a plan_id metadata".to_string(),
                        ))?
                        .parse::<uuid::Uuid>()
                        .map_err(|_| {
                            ServiceError::BadRequest("plan_id metadata must be a uuid".to_string())
                        })?;
                    let organization_id = metadata
                        .get("organization_id")
                        .ok_or(ServiceError::BadRequest(
                            "Checkout session must have an organization_id metadata".to_string(),
                        ))?
                        .parse::<uuid::Uuid>()
                        .map_err(|_| {
                            ServiceError::BadRequest(
                                "organization_id metadata must be a uuid".to_string(),
                            )
                        })?;

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
                    let plan_amount = plan.amount.ok_or(ServiceError::BadRequest(
                        "Plan must have an amount".to_string(),
                    ))?;

                    web::block(move || create_stripe_plan_query(plan_id, plan_amount, pool))
                        .await?
                        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;
                }
            }
            EventType::CustomerSubscriptionDeleted => {
                if let EventObject::Subscription(subscription) = event.data.object {
                    let subscription_stripe_id = subscription.id.to_string();

                    let current_period_end = chrono::NaiveDateTime::from_timestamp_opt(
                        subscription.current_period_end,
                        0,
                    )
                    .ok_or(ServiceError::BadRequest(
                        "Failed to convert current_period_end to NaiveDateTime".to_string(),
                    ))?;

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
        (status = 400, description = "Service error relating to creating a URL for a stripe checkout page", body = DefaultError),
    ),
    params(
        ("plan_id" = uuid::Uuid, Path, description = "id of the plan you want to subscribe to"),
        ("organization_id" = uuid::Uuid, Path, description = "id of the organization you want to subscribe to the plan"),
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
    let _org_plan_sub =
        get_organization_by_key_query(organization_id_clone.into(), organization_pool)
            .await
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
        (status = 400, description = "Service error relating to creating a URL for a stripe checkout page", body = DefaultError),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
        ("subscription_id" = uuid, Path, description = "id of the subscription you want to cancel"),
    ),
    security(
        ("api_key" = ["owner"]),
        ("cookie" = ["owner"])
    )
)]
pub async fn cancel_subscription(
    subscription_id: web::Path<uuid::Uuid>,
    _user: OwnerOnly,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let get_sub_pool = pool.clone();
    let subscription = web::block(move || {
        get_subscription_by_id_query(subscription_id.into_inner(), get_sub_pool)
    })
    .await?
    .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    cancel_stripe_subscription(subscription.stripe_id)
        .await
        .map_err(|e| {
            ServiceError::BadRequest(format!(
                "Failed to cancel stripe subscription: {}",
                e.message
            ))
        })?;

    let _ = refresh_redis_org_plan_sub(subscription.organization_id, pool.clone()).await;

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
        (status = 400, description = "Service error relating to updating the subscription to the new plan", body = DefaultError),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
        ("subscription_id" = uuid::Uuid, Path, description = "id of the subscription you want to update"),
        ("plan_id" = uuid::Uuid, Path, description = "id of the plan you want to subscribe to"),
    ),
    security(
        ("api_key" = ["readonly"]),
        ("cookie" = ["readonly"])
    )
)]
pub async fn update_subscription_plan(
    path_data: web::Path<UpdateSubscriptionData>,
    _user: OwnerOnly,
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

#[utoipa::path(
    get,
    path = "/stripe/plans",
    context_path = "/api",
    tag = "stripe",
    responses(
        (status = 200, description = "List of all plans", body = Vec<StripePlan>),
        (status = 400, description = "Service error relating to getting all plans", body = DefaultError),
    ),
)]
pub async fn get_all_plans(pool: web::Data<Pool>) -> Result<HttpResponse, actix_web::Error> {
    let stripe_plans = web::block(move || get_all_plans_query(pool))
        .await?
        .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    Ok(HttpResponse::Ok().json(stripe_plans))
}
