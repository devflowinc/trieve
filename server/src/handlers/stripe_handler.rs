use crate::{
    data::models::{DateRange, OrganizationWithSubAndPlan, Pool, TrievePlan, TrieveSubscription},
    errors::ServiceError,
    get_env,
    middleware::auth_middleware::verify_owner,
    operators::{
        email_operator::send_email,
        organization_operator::{
            get_org_from_id_query, get_org_id_from_subscription_id_query, get_org_usage_by_id_query,
        },
        stripe_operator::{
            cancel_stripe_subscription, create_invoice_query, create_stripe_payment_link,
            create_stripe_plan_query, create_stripe_setup_checkout_session,
            create_stripe_subscription_query, create_usage_based_stripe_payment_link,
            create_usage_stripe_subscription_query, delete_subscription_by_id_query,
            delete_usage_subscription_by_id_query, get_all_plans_query, get_all_usage_plans_query,
            get_bill_from_range, get_invoices_for_org_query,
            get_option_subscription_by_organization_id_query,
            get_option_usage_based_subscription_by_organization_id_query,
            get_option_usage_based_subscription_by_subscription_id_query, get_stripe_client,
            get_trieve_plan_by_id_query, get_trieve_subscription_by_id_query,
            set_stripe_subscription_current_period_end, set_subscription_payment_method,
            update_static_stripe_meters, update_stripe_subscription_plan_query,
            update_stripe_usage_based_subscription_plan_query, update_to_flat_stripe_subscription,
            update_to_usage_based_stripe_subscription,
        },
    },
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use stripe::{EventObject, EventType, Object, Webhook};
use utoipa::ToSchema;

use super::auth_handler::OwnerOnly;

pub async fn delete_existing_subscriptions_if_exists(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    let optional_existing_flat_subscription =
        get_option_subscription_by_organization_id_query(organization_id, pool.clone()).await?;

    if let Some(existing_subscription) = optional_existing_flat_subscription {
        let delete_subscription_pool = pool.clone();

        delete_subscription_by_id_query(existing_subscription.id, delete_subscription_pool).await?;
    }

    let optional_usage_existing_subscription =
        get_option_usage_based_subscription_by_organization_id_query(organization_id, pool.clone())
            .await?;

    if let Some(existing_subscription) = optional_usage_existing_subscription {
        delete_usage_subscription_by_id_query(existing_subscription.id, pool.clone()).await?;
    }

    Ok(())
}

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
                    let checkout_type = checkout_session.mode;
                    match checkout_type {
                        stripe::CheckoutSessionMode::Setup => {
                            let client = get_stripe_client();
                            let setup_intent_id = checkout_session
                                .setup_intent
                                .ok_or(ServiceError::BadRequest(
                                    "Setup checkout session must have setup intent id".to_string(),
                                ))?
                                .id();

                            let setup_intent =
                                stripe::SetupIntent::retrieve(&client, &setup_intent_id, &[])
                                    .await
                                    .map_err(|_| {
                                        ServiceError::BadRequest(
                                            "failed to get setup intent".to_string(),
                                        )
                                    })?;

                            let metadata =
                                setup_intent
                                    .clone()
                                    .metadata
                                    .ok_or(ServiceError::BadRequest(
                                        "Checkout session must have metadata".to_string(),
                                    ))?;

                            let subscription_id =
                                metadata
                                    .get("subscription_id")
                                    .ok_or(ServiceError::BadRequest(
                                        "Checkout session must have an subscription_id metadata"
                                            .to_string(),
                                    ))?;

                            set_subscription_payment_method(
                                setup_intent,
                                subscription_id.to_string(),
                            )
                            .await?;
                        }
                        _ => {
                            let subscription_stripe_id = checkout_session
                                .clone()
                                .subscription
                                .ok_or(ServiceError::BadRequest(
                                    "Checkout session must have a subscription".to_string(),
                                ))?
                                .id()
                                .to_string();

                            let metadata = checkout_session.clone().metadata.ok_or(
                                ServiceError::BadRequest(
                                    "Checkout session must have metadata".to_string(),
                                ),
                            )?;

                            let organization_id = metadata
                                .get("organization_id")
                                .ok_or(ServiceError::BadRequest(
                                    "Checkout session must have an organization_id metadata"
                                        .to_string(),
                                ))?
                                .parse::<uuid::Uuid>()
                                .map_err(|_| {
                                    ServiceError::BadRequest(
                                        "organization_id metadata must be a uuid".to_string(),
                                    )
                                })?;

                            let plan_type =
                                metadata.get("plan_type").ok_or(ServiceError::BadRequest(
                                    "Checkout session must have an plan_type metadata".to_string(),
                                ))?;

                            let plan_id = metadata
                                .get("plan_id")
                                .ok_or(ServiceError::BadRequest(
                                    "Checkout session must have an organization_id metadata"
                                        .to_string(),
                                ))?
                                .parse::<uuid::Uuid>()
                                .map_err(|_| {
                                    ServiceError::BadRequest(
                                        "organization_id metadata must be a uuid".to_string(),
                                    )
                                })?;

                            if plan_type == "usage-based" {
                                let _ = delete_existing_subscriptions_if_exists(
                                    organization_id,
                                    pool.clone(),
                                )
                                .await;

                                // record current usage
                                let usage =
                                    get_org_usage_by_id_query(organization_id, pool.clone())
                                        .await?;
                                // This is a usage based query
                                create_usage_stripe_subscription_query(
                                    subscription_stripe_id,
                                    usage,
                                    plan_id,
                                    organization_id,
                                    pool.clone(),
                                )
                                .await?;
                            } else if plan_type == "flat" {
                                let _ = delete_existing_subscriptions_if_exists(
                                    organization_id,
                                    pool.clone(),
                                )
                                .await;

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
            EventType::InvoiceUpcoming => {
                if let EventObject::Invoice(invoice) = event.data.object {
                    let subscription_stripe_id = invoice.subscription.unwrap().id().to_string();

                    let usage_based_subscription =
                        get_option_usage_based_subscription_by_subscription_id_query(
                            subscription_stripe_id.clone(),
                            pool.clone(),
                        )
                        .await?;

                    if let Some(usage_based_subscription) = usage_based_subscription {
                        let organization = get_org_from_id_query(
                            usage_based_subscription.organization_id,
                            pool.clone(),
                        )
                        .await?;
                        let metrics_sent =
                            update_static_stripe_meters(usage_based_subscription.clone(), pool)
                                .await?;

                        let subject = format!(
                            "Send static stripe billing for organization: '{}', id: {}",
                            organization.organization.name, organization.organization.id
                        );

                        let body = format!(
                            "{:?}<br/>{:?}<br/>{:?}",
                            subject.clone(),
                            metrics_sent
                                .iter()
                                .map(|(k, v)| format!("{}: {}", k, v))
                                .collect::<Vec<String>>()
                                .join("<br/>"),
                            format!("View their bill here: https://dashboard.stripe.com/subcriptions/{}", usage_based_subscription.stripe_subscription_id)
                        );

                        send_email(body, "webmaster@trieve.ai".to_string(), Some(subject))?;
                    }
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
            EventType::InvoicePaid => {
                if let EventObject::Invoice(invoice) = event.data.object {
                    let subscription_stripe_id = invoice
                        .clone()
                        .subscription
                        .ok_or(ServiceError::BadRequest(
                            "Failed to get subscription from invoice".to_string(),
                        ))?
                        .id();

                    let org_id = get_org_id_from_subscription_id_query(
                        subscription_stripe_id.to_string(),
                        pool.clone(),
                    )
                    .await?;

                    create_invoice_query(org_id, invoice.id(), pool.clone()).await?;
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
/// Get a 303 SeeOther redirect link to the stripe checkout page for the plan and organization
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
pub async fn direct_to_payment_link(
    path_data: web::Path<GetDirectPaymentLinkData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let subscription_org_id = path_data.organization_id;

    let current_subscription =
        get_trieve_subscription_by_id_query(subscription_org_id, pool.clone())
            .await
            .ok();

    if current_subscription.is_some_and(|s| s.current_period_end().is_none()) {
        return Ok(HttpResponse::Conflict().finish());
    }

    let organization_id = path_data.organization_id;
    let organization_id_clone = path_data.organization_id;
    let _org_plan_sub = get_org_from_id_query(organization_id_clone, pool.clone()).await?;
    let plan_id = path_data.plan_id;

    let plan = get_trieve_plan_by_id_query(plan_id, pool.clone()).await?;

    let payment_link = match plan {
        TrievePlan::UsageBased(usage_based_plan) => {
            create_usage_based_stripe_payment_link(usage_based_plan, organization_id).await?
        }
        TrievePlan::Flat(flat_plan) => {
            create_stripe_payment_link(flat_plan, organization_id).await?
        }
    };

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
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
        ("subscription_id" = uuid, Path, description = "id of the subscription you want to cancel"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn cancel_subscription(
    subscription_id: web::Path<uuid::Uuid>,
    user: OwnerOnly,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let get_sub_pool = pool.clone();
    let subscription =
        get_trieve_subscription_by_id_query(subscription_id.into_inner(), get_sub_pool).await?;

    if !verify_owner(&user, &subscription.organization_id()) {
        return Err(ServiceError::Forbidden.into());
    };

    cancel_stripe_subscription(subscription.stripe_subscription_id()).await?;

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
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
        ("subscription_id" = uuid::Uuid, Path, description = "id of the subscription you want to update"),
        ("plan_id" = uuid::Uuid, Path, description = "id of the plan you want to subscribe to"),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn update_subscription_plan(
    path_data: web::Path<UpdateSubscriptionData>,
    user: OwnerOnly,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let current_subscription_id = path_data.subscription_id;
    let new_plan_id = path_data.plan_id;

    let current_trieve_subscription =
        get_trieve_subscription_by_id_query(current_subscription_id, pool.clone()).await?;

    let new_trieve_plan = get_trieve_plan_by_id_query(new_plan_id, pool.clone()).await?;

    if !verify_owner(&user, &current_trieve_subscription.organization_id()) {
        return Err(ServiceError::Forbidden.into());
    };

    match new_trieve_plan {
        TrievePlan::Flat(flat_plan) => {
            update_to_flat_stripe_subscription(
                current_trieve_subscription.stripe_subscription_id(),
                flat_plan.stripe_id.clone(),
            )
            .await?;

            match current_trieve_subscription {
                TrieveSubscription::Flat(_) => {
                    update_stripe_subscription_plan_query(
                        current_trieve_subscription.id(),
                        flat_plan.id,
                        pool.clone(),
                    )
                    .await?;
                }
                TrieveSubscription::UsageBased(_) => {
                    // Old was usage based, create a new entry for flat
                    create_stripe_subscription_query(
                        current_trieve_subscription.stripe_subscription_id(),
                        flat_plan.id,
                        current_trieve_subscription.organization_id(),
                        pool.clone(),
                    )
                    .await?;

                    // delete old subscription
                    delete_usage_subscription_by_id_query(
                        current_trieve_subscription.id(),
                        pool.clone(),
                    )
                    .await?;
                }
            }
        }
        TrievePlan::UsageBased(stripe_usage_based_plan) => {
            update_to_usage_based_stripe_subscription(
                current_trieve_subscription.stripe_subscription_id(),
                stripe_usage_based_plan.clone(),
            )
            .await?;

            match current_trieve_subscription {
                TrieveSubscription::UsageBased(_) => {
                    update_stripe_usage_based_subscription_plan_query(
                        current_trieve_subscription.id(),
                        stripe_usage_based_plan.id,
                        pool.clone(),
                    )
                    .await?;
                }
                TrieveSubscription::Flat(_) => {
                    // Old was flat, create a new one
                    let current_usage = get_org_usage_by_id_query(
                        current_trieve_subscription.organization_id(),
                        pool.clone(),
                    )
                    .await?;

                    create_usage_stripe_subscription_query(
                        current_trieve_subscription.stripe_subscription_id(),
                        current_usage,
                        stripe_usage_based_plan.id,
                        current_trieve_subscription.organization_id(),
                        pool.clone(),
                    )
                    .await?;

                    // delete old subscription
                    delete_subscription_by_id_query(current_trieve_subscription.id(), pool.clone())
                        .await?
                }
            }
        }
    }
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
pub async fn get_all_plans(pool: web::Data<Pool>) -> Result<HttpResponse, actix_web::Error> {
    let stripe_plans = get_all_plans_query(pool)
        .await
        .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    Ok(HttpResponse::Ok().json(stripe_plans))
}

/// Get All Usage Plans
///
/// Get a list of all usage_based plans
#[utoipa::path(
    get,
    path = "/stripe/usage_plans",
    context_path = "/api",
    tag = "Stripe",
    responses(
        (status = 200, description = "List of all plans", body = Vec<StripeUsageBasedPlan>),
        (status = 400, description = "Service error relating to getting all plans", body = ErrorResponseBody),
    ),
)]
pub async fn get_all_usage_plans(pool: web::Data<Pool>) -> Result<HttpResponse, actix_web::Error> {
    let stripe_plans = get_all_usage_plans_query(pool)
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
    params (
        ("organization_id" = uuid::Uuid, Path, description = "The id of the organization to get invoices for."),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn get_all_invoices(
    pool: web::Data<Pool>,
    _user: OwnerOnly,
    path_data: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let org_id = path_data.into_inner();
    let invoices = get_invoices_for_org_query(org_id, pool).await?;
    Ok(HttpResponse::Ok().json(invoices))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSetupCheckoutSessionResPayload {
    pub url: String,
}

/// Update Payment Method
///
/// Update a your payment method to a new one
#[utoipa::path(
    post,
    path = "/stripe/checkout/setup/{organization_id}",
    context_path = "/api",
    tag = "Stripe",
    responses (
        (status = 200, description ="Checkout session (setup) response", body = CreateSetupCheckoutSessionResPayload),
        (status = 400, description = "Service error relating to creating setup checkout session", body = ErrorResponseBody),
    ),
    params (
        ("organization_id" = uuid::Uuid, Path, description = "The id of the organization to create setup checkout session for."),
    ),
    security(
        ("ApiKey" = ["owner"]),
    )
)]
pub async fn update_payment_method(
    pool: web::Data<Pool>,
    _user: OwnerOnly,
    path_data: web::Path<uuid::Uuid>,
) -> Result<HttpResponse, actix_web::Error> {
    let org_id = path_data.into_inner();
    let subscription_id =
        match get_option_subscription_by_organization_id_query(org_id, pool.clone()).await? {
            Some(subscription) => subscription.stripe_id,
            None => {
                match get_option_usage_based_subscription_by_organization_id_query(org_id, pool)
                    .await?
                {
                    Some(usage_subscription) => usage_subscription.stripe_subscription_id,
                    None => {
                        return Err(ServiceError::BadRequest(
                            "Organization does not have an active subscription".to_string(),
                        )
                        .into())
                    }
                }
            }
        };

    let checkout_link = create_stripe_setup_checkout_session(subscription_id, org_id).await?;

    Ok(HttpResponse::Ok().json(CreateSetupCheckoutSessionResPayload { url: checkout_link }))
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BillingEstimate {
    pub items: Vec<BillItem>,
    pub total: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BillItem {
    pub name: String,
    pub clean_name: String,
    pub usage_amount: u64,
    pub amount: f64,
}

/// Estimate Bill From Range
///
/// Return the amount you will be billed from a date range if you were on usage based pricing
#[utoipa::path(
    get,
    path = "/stripe/estimate_bill/{plan_id}",
    context_path = "/api",
    tag = "Stripe",
    responses (
        (status = 200, description ="Billing estimate", body = BillingEstimate ),
        (status = 400, description = "Service error relating to calculating bill", body = ErrorResponseBody),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn estimate_bill_from_range(
    _user: OwnerOnly,
    org_with_plan_and_sub: OrganizationWithSubAndPlan,
    plan_id: web::Path<uuid::Uuid>,
    date_range: web::Json<DateRange>,
    clickhouse_client: web::Data<clickhouse::Client>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let usage_plan = get_trieve_plan_by_id_query(plan_id.into_inner(), pool.clone())
        .await
        .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    match usage_plan {
        TrievePlan::UsageBased(usage_plan) => {
            let billing_estimate = get_bill_from_range(
                org_with_plan_and_sub.organization.id,
                usage_plan,
                date_range.into_inner(),
                &clickhouse_client,
                pool,
            )
            .await?;

            Ok(HttpResponse::Ok().json(billing_estimate))
        }
        _ => Err(ServiceError::BadRequest("Plan is not usage based".to_string()).into()),
    }
}
