use crate::{
    data::models::{DatasetAndOrgWithSubAndPlan, Pool},
    errors::ServiceError,
    get_env,
    middleware::auth_middleware::verify_owner,
    operators::{
        organization_operator::{get_org_from_id_query, get_org_from_subscription_id_query},
        stripe_operator::{
            cancel_stripe_subscription, create_invoice_query, create_stripe_payment_link, create_stripe_plan_query, create_stripe_subscription_query, delete_subscription_by_id_query, get_all_plans_query, get_invoices_for_org_query, get_option_subscription_by_organization_id_query, get_plan_by_id_query, get_subscription_by_id_query, set_stripe_subscription_current_period_end, update_stripe_subscription, update_stripe_subscription_plan_query
        },
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use stripe::{EventObject, EventType, Webhook};

use super::auth_handler::OwnerOnly;

#[tracing::instrument(skip(pool))]
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
                        .clone()
                        .subscription
                        .ok_or(ServiceError::BadRequest(
                            "Checkout session must have a subscription".to_string(),
                        ))?
                        .id()
                        .to_string();


                    let metadata = checkout_session.clone().metadata.ok_or(ServiceError::BadRequest(
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

                    let optional_existing_subscription =
                        get_option_subscription_by_organization_id_query(
                            fetch_subscription_organization_id,
                            optional_subscription_pool,
                        )
                        .await?;

                    if let Some(existing_subscription) = optional_existing_subscription {
                        let delete_subscription_pool = pool.clone();

                        delete_subscription_by_id_query(
                            existing_subscription.id,
                            delete_subscription_pool,
                        )
                        .await?;
                    }

                    create_stripe_subscription_query(
                        subscription_stripe_id,
                        plan_id,
                        organization_id,
                        pool.clone(),
                    )
                    .await?;


                    let invoice = checkout_session.clone().invoice;
                    if invoice.is_some() {
                        let invoice_id = invoice.unwrap().id();
                        create_invoice_query(organization_id, invoice_id, pool).await?;
                    }
                }
            }
            EventType::PlanCreated => {
                if let EventObject::Plan(plan) = event.data.object {
                    let plan_id = plan.id.to_string();
                    let plan_amount = plan.amount.ok_or(ServiceError::BadRequest(
                        "Plan must have an amount".to_string(),
                    ))?;

                    create_stripe_plan_query(plan_id, plan_amount, pool).await?;
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
                    .await?;
                }
            }
            _ => {}
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetDirectPaymentLinkData {
    pub plan_id: uuid::Uuid,
    pub organization_id: uuid::Uuid,
}

/// Checkout
///
/// Get a direct link to the stripe checkout page for the plan and organization

#[utoipa::path(
    get,
    path = "/stripe/payment_link/{plan_id}/{organization_id}",
    context_path = "/api",
    tag = "Stripe",
    responses(
        (status = 303, description = "SeeOther response redirecting user to stripe checkout page"),
        (status = 400, description = "Service error relating to creating a URL for a stripe checkout page", body = ErrorResponseBody),
    ),
    params(
        ("plan_id" = uuid::Uuid, Path, description = "id of the plan you want to subscribe to"),
        ("organization_id" = uuid::Uuid, Path, description = "id of the organization you want to subscribe to the plan"),
    ),
)]
#[tracing::instrument(skip(pool))]
pub async fn direct_to_payment_link(
    path_data: web::Path<GetDirectPaymentLinkData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_pool = pool.clone();
    let subscription_pool = pool.clone();
    let subscription_org_id = path_data.organization_id;

    let current_subscription =
        get_option_subscription_by_organization_id_query(subscription_org_id, subscription_pool)
            .await?;

    if current_subscription.is_some_and(|s| s.current_period_end.is_none()) {
        return Ok(HttpResponse::Conflict().finish());
    }

    let plan_id = path_data.plan_id;
    let organization_id = path_data.organization_id;
    let organization_id_clone = path_data.organization_id;
    let _org_plan_sub = get_org_from_id_query(organization_id_clone, organization_pool).await?;

    let plan = get_plan_by_id_query(plan_id, pool).await?;

    let payment_link = create_stripe_payment_link(plan, organization_id).await?;

    Ok(HttpResponse::SeeOther()
        .insert_header(("Location", payment_link))
        .finish())
}

/// Cancel Subscription
///
/// Cancel a subscription by its id
#[utoipa::path(
    delete,
    path = "/stripe/subscription/{subscription_id}",
    context_path = "/api",
    tag = "Stripe",
    responses(
        (status = 200, description = "Confirmation that the subscription was cancelled"),
        (status = 400, description = "Service error relating to creating a URL for a stripe checkout page", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
        ("subscription_id" = uuid, Path, description = "id of the subscription you want to cancel"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn cancel_subscription(
    subscription_id: web::Path<uuid::Uuid>,
    user: OwnerOnly,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let get_sub_pool = pool.clone();
    let subscription =
        get_subscription_by_id_query(subscription_id.into_inner(), get_sub_pool).await?;

    if !verify_owner(&user, &subscription.organization_id) {
        return Err(ServiceError::Forbidden.into());
    };

    cancel_stripe_subscription(subscription.stripe_id).await?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateSubscriptionData {
    pub subscription_id: uuid::Uuid,
    pub plan_id: uuid::Uuid,
}

/// Update Subscription Plan
///
/// Update a subscription to a new plan
#[utoipa::path(
    patch,
    path = "/stripe/subscription_plan/{subscription_id}/{plan_id}",
    context_path = "/api",
    tag = "Stripe",
    responses(
        (status = 200, description = "Confirmation that the subscription was updated to the new plan"),
        (status = 400, description = "Service error relating to updating the subscription to the new plan", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
        ("subscription_id" = uuid::Uuid, Path, description = "id of the subscription you want to update"),
        ("plan_id" = uuid::Uuid, Path, description = "id of the plan you want to subscribe to"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn update_subscription_plan(
    path_data: web::Path<UpdateSubscriptionData>,
    user: OwnerOnly,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let get_subscription_pool = pool.clone();
    let get_plan_pool = pool.clone();
    let update_subscription_plan_pool = pool.clone();

    let subscription_id = path_data.subscription_id;
    let subscription = get_subscription_by_id_query(subscription_id, get_subscription_pool).await?;

    if !verify_owner(&user, &subscription.organization_id) {
        return Err(ServiceError::Forbidden.into());
    };

    let plan_id = path_data.plan_id;
    let plan = get_plan_by_id_query(plan_id, get_plan_pool).await?;

    update_stripe_subscription(subscription.stripe_id, plan.stripe_id).await?;

    update_stripe_subscription_plan_query(subscription.id, plan.id, update_subscription_plan_pool)
        .await?;

    Ok(HttpResponse::Ok().finish())
}

/// Get All Plans
///
/// Get a list of all plans
#[utoipa::path(
    get,
    path = "/stripe/plans",
    context_path = "/api",
    tag = "Stripe",
    responses(
        (status = 200, description = "List of all plans", body = Vec<StripePlan>),
        (status = 400, description = "Service error relating to getting all plans", body = ErrorResponseBody),
    ),
)]
#[tracing::instrument(skip(pool))]
pub async fn get_all_plans(pool: web::Data<Pool>) -> Result<HttpResponse, actix_web::Error> {
    let stripe_plans = get_all_plans_query(pool)
        .await
        .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    Ok(HttpResponse::Ok().json(stripe_plans))
}

/// Get All Invoices
///
/// Get a list of all invoices
#[utoipa::path(
    get,
    path = "/stripe/invoices/{organization_id}",
    context_path = "/api",
    tag = "Stripe",
    responses (
        (status = 200, description ="List of all invoices", body = Vec<StripeInvoice>),
        (status = 400, description = "Service error relating to getting all invoices", body = ErrorResponseBody),
    ),
)]
#[tracing::instrument(skip(pool))]
pub async fn get_all_invoices(
    pool: web::Data<Pool>,
    user: OwnerOnly,
    path_data: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let org_id = path_data.into_inner();
    let invoices = get_invoices_for_org_query(org_id, pool).await?;
    Ok(HttpResponse::Ok().json(invoices))
}
