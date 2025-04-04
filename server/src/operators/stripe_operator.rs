use crate::handlers::stripe_handler::{BillItem, BillingEstimate};
use std::{cmp::max, collections::HashMap, str::FromStr};

use crate::{
    data::{
        models::{
            DateRange, OrganizationUsageCount, Pool, StripeInvoice, StripePlan, StripeSubscription,
            StripeUsageBasedPlan, StripeUsageBasedSubscription, TrievePlan, TrieveSubscription,
        },
        schema::stripe_usage_based_subscriptions,
    },
    errors::ServiceError,
    get_env,
    operators::chunk_operator::get_storage_mb_from_chunk_count,
};
use actix_web::web;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::organization_operator::{
    get_extended_org_usage_by_id_query, get_org_usage_by_id_query, hash_function,
};

pub fn get_stripe_client() -> stripe::Client {
    let stripe_secret = get_env!("STRIPE_SECRET", "STRIPE_SECRET must be set");

    stripe::Client::new(stripe_secret)
}

pub async fn create_usage_stripe_subscription_query(
    stripe_subscription_id: String,
    current_usage: OrganizationUsageCount,
    usage_based_plan_id: uuid::Uuid,
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::stripe_usage_based_subscriptions::dsl as stripe_usage_based_subscriptions_columns;

    let stripe_usage_based_subscription = StripeUsageBasedSubscription {
        id: uuid::Uuid::new_v4(),
        organization_id,
        stripe_subscription_id,
        last_recorded_meter: (chrono::Utc::now() - chrono::Duration::days(30)).naive_utc(),
        usage_based_plan_id,
        created_at: chrono::Utc::now().naive_utc(),

        last_cycle_timestamp: chrono::Utc::now().naive_utc(),
        last_cycle_dataset_count: current_usage.dataset_count as i64,
        last_cycle_users_count: current_usage.user_count,
        last_cycle_chunks_stored_mb: get_storage_mb_from_chunk_count(current_usage.chunk_count),
        last_cycle_files_storage_mb: current_usage.file_storage * 1024,

        current_period_end: None,
    };

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");
    diesel::insert_into(stripe_usage_based_subscriptions_columns::stripe_usage_based_subscriptions)
        .values(&stripe_usage_based_subscription)
        .execute(&mut conn)
        .await
        .map_err(|e| {
            log::error!("Failed to insert stripe subscription: {}", e);
            ServiceError::BadRequest("Failed to insert stripe subscription".to_string())
        })?;

    Ok(())
}

pub async fn update_static_stripe_meters(
    usage_based_subscription: StripeUsageBasedSubscription,
    pool: web::Data<Pool>,
) -> Result<Vec<(&'static str, String)>, ServiceError> {
    let timestamp_now = chrono::Utc::now();

    // Check if within 4 days
    if usage_based_subscription
        .last_cycle_timestamp
        .date()
        .signed_duration_since(timestamp_now.naive_utc().date())
        .num_days()
        > 4
    {
        // Prevent double metering
        let static_events: Vec<(&str, String)> = vec![
            ("chunk_storage_mb", "None, already sent".to_string()),
            ("file_storage_mb", "None, already sent".to_string()),
            ("users", "None, already sent".to_string()),
            ("dataset_count", "None, already sent".to_string()),
        ];
        return Ok(static_events);
    }

    let usage =
        get_org_usage_by_id_query(usage_based_subscription.organization_id, pool.clone()).await?;

    let static_events: Vec<(&str, String)> = vec![
        (
            "chunk_storage_mb",
            get_storage_mb_from_chunk_count(usage.chunk_count).to_string(),
        ),
        ("file_storage_mb", (usage.file_storage * 1024).to_string()),
        ("users", usage.user_count.to_string()),
        ("dataset_count", usage.dataset_count.to_string()),
    ];

    let stripe_secret = get_env!("STRIPE_SECRET", "STRIPE_SECRET must be set");
    let reqwest_client = reqwest::Client::new();

    let stripe_customer_id = get_stripe_customer_id_from_subscription(
        usage_based_subscription.stripe_subscription_id.clone(),
    )
    .await?;

    for (event_name, event_value) in static_events.clone().into_iter() {
        let identifier = hash_function(&format!(
            "{:?}||{:?}||{:?}||{:?}",
            stripe_customer_id,
            timestamp_now.to_string(),
            event_name,
            usage_based_subscription.organization_id
        ));

        let response = reqwest_client
            .post("https://api.stripe.com/v1/billing/meter_events")
            .basic_auth(stripe_secret, Option::<&str>::None)
            .form(&[
                ("event_name", event_name),
                ("timestamp", &timestamp_now.timestamp().to_string()),
                ("identifier", &identifier),
                ("payload[stripe_customer_id]", &stripe_customer_id),
                ("payload[value]", &event_value),
            ])
            .send()
            .await
            .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

        let stripe_billing_response = response
            .json::<StripeBillingMetricsResponse>()
            .await
            .unwrap();

        if let Some(error) = stripe_billing_response.error {
            log::error!("Failed to send stripe billing: {}", error.message);
        }
    }

    use crate::data::schema::stripe_usage_based_subscriptions::dsl as stripe_usage_based_subscriptions_columns;
    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    diesel::update(
        stripe_usage_based_subscriptions_columns::stripe_usage_based_subscriptions.filter(
            stripe_usage_based_subscriptions_columns::stripe_subscription_id
                .eq(usage_based_subscription.stripe_subscription_id),
        ),
    )
    .set(
        stripe_usage_based_subscriptions_columns::last_cycle_timestamp
            .eq(timestamp_now.naive_utc()),
    )
    .execute(&mut conn)
    .await
    .map_err(|e| {
        log::error!("Failed to update usage based stripe subscription: {}", e);
        ServiceError::BadRequest("Failed to update usage based stripe subscription".to_string())
    })?;

    Ok(static_events)
}

pub async fn create_stripe_subscription_query(
    stripe_id: String,
    plan_id: uuid::Uuid,
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

    let stripe_subscription =
        StripeSubscription::from_details(stripe_id, plan_id, organization_id, None);

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");
    diesel::insert_into(stripe_subscriptions_columns::stripe_subscriptions)
        .values(&stripe_subscription)
        .execute(&mut conn)
        .await
        .map_err(|e| {
            log::error!("Failed to insert stripe subscription: {}", e);
            ServiceError::BadRequest("Failed to insert stripe subscription".to_string())
        })?;

    Ok(())
}

pub async fn create_stripe_plan_query(
    stripe_id: String,
    amount: i64,
    pool: web::Data<Pool>,
) -> Result<StripePlan, ServiceError> {
    use crate::data::schema::stripe_plans::dsl as stripe_plans_columns;

    // TODO: Make this configurable
    let stripe_plan = StripePlan::from_details(
        stripe_id,
        10000,
        1000000000,
        100,
        1,
        10000,
        amount,
        "Project".to_string(),
        false,
    );

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");
    let created_stripe_plan: StripePlan = diesel::insert_into(stripe_plans_columns::stripe_plans)
        .values(&stripe_plan)
        .get_result(&mut conn)
        .await
        .map_err(|e| {
            log::error!("Failed to insert stripe plan: {}", e);
            ServiceError::BadRequest("Failed to insert stripe plan".to_string())
        })?;

    Ok(created_stripe_plan)
}

pub async fn get_trieve_plan_by_id_query(
    plan_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<TrievePlan, ServiceError> {
    let stripe_plan = get_plan_by_id_query(plan_id, pool.clone()).await.ok();
    let stripe_usage_based_plan = get_usage_based_plan_query(plan_id, pool).await.ok();

    TrievePlan::from_flat(stripe_plan, stripe_usage_based_plan)
        .ok_or(ServiceError::NotFound("TrievePlan not found".to_string()))
}

pub async fn get_plan_by_id_query(
    plan_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<StripePlan, ServiceError> {
    use crate::data::schema::stripe_plans::dsl as stripe_plans_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    let stripe_plan: StripePlan = stripe_plans_columns::stripe_plans
        .filter(stripe_plans_columns::id.eq(plan_id))
        .first(&mut conn)
        .await
        .map_err(|e| {
            log::error!("Failed to get stripe plan: {}", e);
            ServiceError::BadRequest("Failed to get stripe plan".to_string())
        })?;

    Ok(stripe_plan)
}

pub async fn get_usage_based_plan_query(
    plan_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<StripeUsageBasedPlan, ServiceError> {
    use crate::data::schema::stripe_usage_based_plans::dsl as stripe_usage_based_plans_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    let stripe_plan: StripeUsageBasedPlan =
        stripe_usage_based_plans_columns::stripe_usage_based_plans
            .filter(stripe_usage_based_plans_columns::id.eq(plan_id))
            .first(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to get stripe plan: {}", e);
                ServiceError::BadRequest("Failed to get stripe plan".to_string())
            })?;

    Ok(stripe_plan)
}

pub async fn get_all_usage_plans_query(
    pool: web::Data<Pool>,
) -> Result<Vec<StripeUsageBasedPlan>, ServiceError> {
    use crate::data::schema::stripe_usage_based_plans::dsl as stripe_usage_based_plans_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    let stripe_plans: Vec<StripeUsageBasedPlan> =
        stripe_usage_based_plans_columns::stripe_usage_based_plans
            .load(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to get stripe plans: {}", e);
                ServiceError::BadRequest("Failed to get stripe plans".to_string())
            })?;

    Ok(stripe_plans)
}

pub async fn get_all_plans_query(pool: web::Data<Pool>) -> Result<Vec<StripePlan>, ServiceError> {
    use crate::data::schema::stripe_plans::dsl as stripe_plans_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");
    let stripe_plans: Vec<StripePlan> = stripe_plans_columns::stripe_plans
        .load(&mut conn)
        .await
        .map_err(|e| {
            log::error!("Failed to get stripe plans: {}", e);
            ServiceError::BadRequest("Failed to get stripe plans".to_string())
        })?;

    Ok(stripe_plans)
}

pub async fn create_stripe_payment_link(
    plan: StripePlan,
    organization_id: uuid::Uuid,
) -> Result<String, ServiceError> {
    let admin_dashboard_url = get_env!("ADMIN_DASHBOARD_URL", "ADMIN_DASHBOARD_URL must be set");

    let stripe_secret = get_env!("STRIPE_SECRET", "STRIPE_SECRET must be set");
    let payment_link_create_request = reqwest::Client::new()
        .post("https://api.stripe.com/v1/payment_links")
        .header("Authorization", format!("Bearer {}", stripe_secret));

    let payment_link_form_url_encoded = json!({
        "line_items[0][price]": plan.stripe_id,
        "line_items[0][quantity]": 1,
        "allow_promotion_codes": true,
        "after_completion[redirect][url]": format!("{}/dashboard/{}/billing", admin_dashboard_url, organization_id),
        "after_completion[type]": "redirect",
        "metadata[organization_id]": organization_id.to_string(),
        "metadata[plan_type]": "flat",
        "metadata[plan_id]": plan.id.to_string()
    });

    let payment_link_response = payment_link_create_request
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&payment_link_form_url_encoded)
        .send()
        .await
        .map_err(|e| {
            log::error!("Failed to create stripe payment link: {}", e);
            ServiceError::BadRequest("Failed to create stripe payment link".to_string())
        })?;

    let payment_link_response_json: serde_json::Value =
        payment_link_response.json().await.map_err(|e| {
            log::error!("Failed to get stripe payment link json: {}", e);
            ServiceError::BadRequest("Failed to get stripe payment link json".to_string())
        })?;

    let payment_link =
        payment_link_response_json["url"]
            .as_str()
            .ok_or(ServiceError::BadRequest(
                "Failed to get stripe payment link url".to_string(),
            ))?;

    Ok(payment_link.to_string())
}

pub async fn create_usage_based_stripe_payment_link(
    usage_based_plan: StripeUsageBasedPlan,
    organization_id: uuid::Uuid,
) -> Result<String, ServiceError> {
    let admin_dashboard_url = get_env!("ADMIN_DASHBOARD_URL", "ADMIN_DASHBOARD_URL must be set");
    let stripe_client = get_stripe_client();

    let line_item_ids = usage_based_plan.line_item_ids();

    let mut checkout_line_items: Vec<stripe::CreateCheckoutSessionLineItems> = line_item_ids
        .iter()
        .map(|id| stripe::CreateCheckoutSessionLineItems {
            price: Some(id.to_string()),
            ..Default::default()
        })
        .collect();

    if let Some(platform_price_id) = usage_based_plan.platform_price_id {
        checkout_line_items.insert(
            0,
            stripe::CreateCheckoutSessionLineItems {
                price: Some(platform_price_id.to_string()),
                quantity: Some(1),
                ..Default::default()
            },
        );
    }

    let session = stripe::CheckoutSession::create(
        &stripe_client,
        stripe::CreateCheckoutSession {
            mode: Some(stripe::CheckoutSessionMode::Subscription),
            setup_intent_data: None,
            currency: Some(stripe::Currency::USD),
            success_url: Some(
                format!(
                    "{}/dashboard/{}/billing?session_id={{CHECKOUT_SESSION_ID}}",
                    admin_dashboard_url, organization_id
                )
                .as_str(),
            ),
            cancel_url: Some(
                format!(
                    "{}/dashboard/{}/billing?session_id={{CHECKOUT_SESSION_ID}}",
                    admin_dashboard_url, organization_id
                )
                .as_str(),
            ),
            line_items: Some(checkout_line_items),
            allow_promotion_codes: Some(true),
            metadata: Some(HashMap::from([
                ("organization_id".to_string(), organization_id.to_string()),
                ("plan_type".to_string(), "usage-based".to_string()),
                ("plan_id".to_string(), usage_based_plan.id.to_string()),
            ])),
            ..Default::default()
        },
    )
    .await
    .map_err(|err| {
        ServiceError::BadRequest(format!("Failed to create setup checkout session {:?}", err))
    })?;
    if session.url.is_none() {
        return Err(ServiceError::BadRequest(
            "Failed to get setup checkout session url".to_string(),
        ));
    }
    Ok(session.url.unwrap().to_string())
}

pub async fn get_trieve_subscription_by_id_query(
    subscription_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<TrieveSubscription, ServiceError> {
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;
    use crate::data::schema::stripe_usage_based_subscriptions::dsl as stripe_usage_based_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    let stripe_subscription: Option<StripeSubscription> =
        stripe_subscriptions_columns::stripe_subscriptions
            .filter(stripe_subscriptions_columns::id.eq(subscription_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                log::error!("Failed to get stripe subscription: {}", e);
                ServiceError::BadRequest("Failed to get stripe subscription".to_string())
            })?;

    let stripe_usage_based_subscription: Option<StripeUsageBasedSubscription> =
        stripe_usage_based_subscriptions_columns::stripe_usage_based_subscriptions
            .filter(stripe_usage_based_subscriptions_columns::id.eq(subscription_id))
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| {
                log::error!("Failed to get stripe usage based subscription: {}", e);
                ServiceError::BadRequest(
                    "Failed to get stripe usage based subscription".to_string(),
                )
            })?;

    TrieveSubscription::from_flat(stripe_subscription, stripe_usage_based_subscription)
        .ok_or(ServiceError::NotFound("Subscription not found".to_string()))
}

pub async fn delete_usage_subscription_by_id_query(
    subscription_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::stripe_usage_based_subscriptions::dsl as stripe_usage_based_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");
    diesel::delete(
        stripe_usage_based_subscriptions_columns::stripe_usage_based_subscriptions
            .filter(stripe_usage_based_subscriptions::id.eq(subscription_id)),
    )
    .execute(&mut conn)
    .await
    .map_err(|e| {
        log::error!("Failed to delete stripe subscription: {}", e);
        ServiceError::BadRequest("Failed to delete stripe subscription".to_string())
    })?;

    Ok(())
}

pub async fn delete_subscription_by_id_query(
    subscription_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");
    diesel::delete(
        stripe_subscriptions_columns::stripe_subscriptions
            .filter(stripe_subscriptions_columns::id.eq(subscription_id)),
    )
    .get_result::<StripeSubscription>(&mut conn)
    .await
    .map_err(|e| {
        log::error!("Failed to delete stripe subscription: {}", e);
        ServiceError::BadRequest("Failed to delete stripe subscription".to_string())
    })?;

    Ok(())
}

pub async fn get_option_usage_based_subscription_by_organization_id_query(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Option<StripeUsageBasedSubscription>, ServiceError> {
    use crate::data::schema::stripe_usage_based_subscriptions::dsl as stripe_usage_based_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    let stripe_usage_based_subscriptions =
        stripe_usage_based_subscriptions_columns::stripe_usage_based_subscriptions
            .select(StripeUsageBasedSubscription::as_select())
            .filter(stripe_usage_based_subscriptions_columns::organization_id.eq(organization_id))
            .first::<StripeUsageBasedSubscription>(&mut conn)
            .await
            .optional()?;

    Ok(stripe_usage_based_subscriptions)
}

pub async fn get_option_usage_based_subscription_by_subscription_id_query(
    stripe_subscription_id: String,
    pool: web::Data<Pool>,
) -> Result<Option<StripeUsageBasedSubscription>, ServiceError> {
    use crate::data::schema::stripe_usage_based_subscriptions::dsl as stripe_usage_based_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    let stripe_usage_based_subscriptions =
        stripe_usage_based_subscriptions_columns::stripe_usage_based_subscriptions
            .select(StripeUsageBasedSubscription::as_select())
            .filter(
                stripe_usage_based_subscriptions_columns::stripe_subscription_id
                    .eq(stripe_subscription_id),
            )
            .first::<StripeUsageBasedSubscription>(&mut conn)
            .await
            .optional()?;

    Ok(stripe_usage_based_subscriptions)
}

pub async fn get_option_subscription_by_organization_id_query(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Option<StripeSubscription>, ServiceError> {
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");
    let stripe_subscriptions: Vec<StripeSubscription> =
        stripe_subscriptions_columns::stripe_subscriptions
            .filter(stripe_subscriptions_columns::organization_id.eq(organization_id))
            .load(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to get stripe subscription: {}", e);
                ServiceError::BadRequest("Failed to get stripe subscription".to_string())
            })?;

    Ok(stripe_subscriptions.into_iter().next())
}

pub async fn set_stripe_subscription_current_period_end(
    stripe_subscription_id: String,
    current_period_end: chrono::NaiveDateTime,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;
    // usage based
    use crate::data::schema::stripe_usage_based_subscriptions::dsl as stripe_usage_based_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    diesel::update(
        stripe_subscriptions_columns::stripe_subscriptions
            .filter(stripe_subscriptions_columns::stripe_id.eq(stripe_subscription_id.clone())),
    )
    .set(stripe_subscriptions_columns::current_period_end.eq(current_period_end))
    .execute(&mut conn)
    .await
    .optional()
    .map_err(|e| {
        log::error!(
            "Failed to update flat stripe subscription in postgres: {}",
            e
        );
        ServiceError::BadRequest("Failed to update stripe subscription".to_string())
    })?;

    diesel::update(
        stripe_usage_based_subscriptions_columns::stripe_usage_based_subscriptions.filter(
            stripe_usage_based_subscriptions_columns::stripe_subscription_id
                .eq(stripe_subscription_id),
        ),
    )
    .set(stripe_usage_based_subscriptions_columns::current_period_end.eq(current_period_end))
    .execute(&mut conn)
    .await
    .optional()
    .map_err(|e| {
        log::error!(
            "Failed to update usage stripe subscription period end in postgres: {}",
            e
        );
        ServiceError::BadRequest("Failed to update stripe subscription".to_string())
    })?;

    Ok(())
}

pub async fn cancel_stripe_subscription(
    subscription_stripe_id: String,
) -> Result<(), ServiceError> {
    let stripe_client = get_stripe_client();
    let stripe_subscription_id: stripe::SubscriptionId =
        subscription_stripe_id.parse().map_err(|_| {
            ServiceError::BadRequest("Failed to parse stripe subscription id".to_string())
        })?;
    stripe::Subscription::cancel(
        &stripe_client,
        &stripe_subscription_id,
        stripe::CancelSubscription::default(),
    )
    .await
    .map_err(|e| {
        log::error!("Failed to cancel stripe subscription: {}", e);
        ServiceError::BadRequest("Request to stripe failed".to_string())
    })?;

    Ok(())
}

pub async fn update_stripe_subscription_plan_query(
    subscription_id: uuid::Uuid,
    plan_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");
    diesel::update(
        stripe_subscriptions_columns::stripe_subscriptions
            .filter(stripe_subscriptions_columns::id.eq(subscription_id)),
    )
    .set(stripe_subscriptions_columns::plan_id.eq(plan_id))
    .get_result::<StripeSubscription>(&mut conn)
    .await
    .map_err(|e| {
        log::error!("Failed to update stripe subscription in postgres: {}", e);
        ServiceError::BadRequest("Failed to update stripe subscription".to_string())
    })?;

    Ok(())
}

pub async fn update_to_usage_based_stripe_subscription(
    subscription_stripe_id: String,
    usage_plan: StripeUsageBasedPlan,
) -> Result<(), ServiceError> {
    let stripe_client = get_stripe_client();

    let stripe_subscription_id: stripe::SubscriptionId =
        subscription_stripe_id.parse().map_err(|_| {
            ServiceError::BadRequest("Failed to parse stripe subscription id".to_string())
        })?;
    let list_sub_items = stripe::generated::billing::subscription_item::ListSubscriptionItems::new(
        stripe_subscription_id.clone(),
    );
    let subscription_items = stripe::SubscriptionItem::list(&stripe_client, &list_sub_items)
        .await
        .map_err(|e| {
            log::error!("Failed to list stripe subscription items: {}", e);
            ServiceError::BadRequest("Failed to list stripe subscription items".to_string())
        })?;

    let mut update_subscription_items: Vec<stripe::UpdateSubscriptionItems> = vec![];
    let mut deleted_item = stripe::UpdateSubscriptionItems::default();
    for stripe_item in subscription_items.data.iter() {
        deleted_item.id = Some(stripe_item.id.to_string());
        deleted_item.deleted = Some(true);
        update_subscription_items.push(deleted_item.clone());
    }

    if let Some(ref platform_price_id) = usage_plan.platform_price_id {
        update_subscription_items.push(stripe::UpdateSubscriptionItems {
            price: Some(platform_price_id.to_string()),
            quantity: Some(1),
            ..Default::default()
        })
    }

    let mut new_items = usage_plan
        .line_item_ids()
        .iter()
        .map(|id| stripe::UpdateSubscriptionItems {
            price: Some(id.to_string()),
            ..Default::default()
        })
        .collect::<Vec<stripe::UpdateSubscriptionItems>>();

    update_subscription_items.append(&mut new_items);

    let update_subscription = stripe::UpdateSubscription::<'_> {
        items: Some(update_subscription_items),
        ..Default::default()
    };

    stripe::Subscription::update(
        &stripe_client,
        &stripe_subscription_id.clone(),
        update_subscription,
    )
    .await
    .map_err(|e| {
        log::error!("Failed to update usage stripe subscription: {}", e);
        ServiceError::BadRequest("Failed to update stripe subscription".to_string())
    })?;

    Ok(())
}

pub async fn update_to_flat_stripe_subscription(
    subscription_stripe_id: String,
    plan_stripe_id: String,
) -> Result<(), ServiceError> {
    let stripe_client = get_stripe_client();

    let stripe_subscription_id: stripe::SubscriptionId =
        subscription_stripe_id.parse().map_err(|_| {
            ServiceError::BadRequest("Failed to parse stripe subscription id".to_string())
        })?;
    let list_sub_items = stripe::generated::billing::subscription_item::ListSubscriptionItems::new(
        stripe_subscription_id.clone(),
    );
    let subscription_items = stripe::SubscriptionItem::list(&stripe_client, &list_sub_items)
        .await
        .map_err(|e| {
            log::error!("Failed to list stripe subscription items: {}", e);
            ServiceError::BadRequest("Failed to list stripe subscription items".to_string())
        })?;

    let mut update_subscription_items: Vec<stripe::UpdateSubscriptionItems> = vec![];
    let mut deleted_item = stripe::UpdateSubscriptionItems::default();
    for stripe_item in subscription_items.data.iter() {
        deleted_item.id = Some(stripe_item.id.to_string());
        deleted_item.deleted = Some(true);
        update_subscription_items.push(deleted_item.clone());
    }

    let new_stripe_item = stripe::UpdateSubscriptionItems {
        plan: Some(plan_stripe_id),
        quantity: Some(1),
        ..Default::default()
    };
    update_subscription_items.push(new_stripe_item);

    let update_subscription = stripe::UpdateSubscription::<'_> {
        items: Some(update_subscription_items),
        ..Default::default()
    };

    stripe::Subscription::update(
        &stripe_client,
        &stripe_subscription_id.clone(),
        update_subscription,
    )
    .await
    .map_err(|e| {
        log::error!("Failed to update to flat stripe subscription: {}", e);
        ServiceError::BadRequest("Failed to update stripe subscription".to_string())
    })?;

    Ok(())
}

pub async fn create_invoice_query(
    org_id: uuid::Uuid,
    invoice_id: stripe::InvoiceId,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    let stripe_client = get_stripe_client();
    let invoice = stripe::Invoice::retrieve(&stripe_client, &invoice_id, &[])
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to get invoice".to_string()))?;
    let created_at = chrono::NaiveDateTime::from_timestamp(invoice.created.unwrap_or(0), 0);
    let total = invoice.total.unwrap_or(0);
    let status = invoice
        .status
        .unwrap_or(stripe::InvoiceStatus::Draft)
        .to_string();
    let url = invoice.hosted_invoice_url.unwrap_or("".to_string());
    let stripe_invoice = StripeInvoice::from_details(
        org_id,
        total,
        created_at,
        status,
        url,
        invoice_id.to_string(),
    );

    use crate::data::schema::stripe_invoices::dsl as stripe_invoices_columns;

    let invoice_exists = invoice_exists_query(invoice_id.clone().to_string(), pool.clone()).await?;

    if invoice_exists {
        return Err(ServiceError::BadRequest(
            "Invoice already exists".to_string(),
        ));
    }

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    diesel::insert_into(stripe_invoices_columns::stripe_invoices)
        .values(stripe_invoice)
        .execute(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to create invoice".to_string()))?;

    Ok(())
}

pub async fn invoice_exists_query(
    invoice_id: String,
    pool: web::Data<Pool>,
) -> Result<bool, ServiceError> {
    use crate::data::schema::stripe_invoices::dsl as stripe_invoices_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    let invoice: Option<String> = stripe_invoices_columns::stripe_invoices
        .filter(stripe_invoices_columns::stripe_id.eq(invoice_id))
        .select(stripe_invoices_columns::hosted_invoice_url)
        .first::<String>(&mut conn)
        .await
        .optional()
        .map_err(|_| ServiceError::BadRequest("Failed to get stripe invoice".to_string()))?;

    Ok(invoice.is_some())
}

pub async fn get_invoices_for_org_query(
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<StripeInvoice>, ServiceError> {
    use crate::data::schema::stripe_invoices::dsl as stripe_invoices_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    let invoices = stripe_invoices_columns::stripe_invoices
        .filter(stripe_invoices_columns::org_id.eq(org_id))
        .load::<StripeInvoice>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to get stripe invoices".to_string()))?;

    Ok(invoices)
}

pub async fn create_stripe_setup_checkout_session(
    subscription_id: String,
    organization_id: uuid::Uuid,
) -> Result<String, ServiceError> {
    let stripe_client = get_stripe_client();
    let admin_dashboard_url = format!(
        "{}/dashboard/{}/billing",
        get_env!("ADMIN_DASHBOARD_URL", "ADMIN_DASHBOARD_URL must be set"),
        organization_id
    );

    let session = stripe::CheckoutSession::create(
        &stripe_client,
        stripe::CreateCheckoutSession {
            mode: Some(stripe::CheckoutSessionMode::Setup),
            setup_intent_data: Some(stripe::CreateCheckoutSessionSetupIntentData {
                metadata: Some(HashMap::from([(
                    "subscription_id".to_string(),
                    subscription_id,
                )])),
                ..Default::default()
            }),
            currency: Some(stripe::Currency::USD),
            success_url: Some(
                format!("{}?session_id={{CHECKOUT_SESSION_ID}}", admin_dashboard_url).as_str(),
            ),
            cancel_url: Some(admin_dashboard_url.as_str()),
            ..Default::default()
        },
    )
    .await
    .map_err(|_| ServiceError::BadRequest("Failed to create setup checkout session".to_string()))?;
    if session.url.is_none() {
        return Err(ServiceError::BadRequest(
            "Failed to get setup checkout session url".to_string(),
        ));
    }
    Ok(session.url.unwrap().to_string())
}

pub async fn set_subscription_payment_method(
    setup_intent: stripe::SetupIntent,
    subscription_id: String,
) -> Result<(), ServiceError> {
    let client = get_stripe_client();
    let subscription_id = stripe::SubscriptionId::from_str(subscription_id.as_str())
        .map_err(|_| ServiceError::BadRequest("Invalid subscription id".to_string()))?;

    let subscription = stripe::Subscription::retrieve(&client, &subscription_id, &[])
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to get subscription".to_string()))?;

    let customer_id = subscription.customer.id();

    let payment_method = setup_intent.payment_method.ok_or(ServiceError::BadRequest(
        "Payment method must be present".to_string(),
    ))?;

    stripe::PaymentMethod::attach(
        &client,
        &payment_method.id(),
        stripe::AttachPaymentMethod {
            customer: customer_id,
        },
    )
    .await
    .map_err(|_| {
        ServiceError::BadRequest("Failed to attach payment method to customer".to_string())
    })?;

    stripe::Subscription::update(
        &client,
        &subscription_id,
        stripe::UpdateSubscription {
            default_payment_method: Some(payment_method.id().as_str()),
            ..Default::default()
        },
    )
    .await
    .map_err(|_| ServiceError::BadRequest("Failed to update payment method".to_string()))?;

    Ok(())
}

pub async fn get_stripe_customer_id_from_subscription(
    stripe_subscription_id: String,
) -> Result<String, ServiceError> {
    let stripe_client = get_stripe_client();

    let response = stripe::Subscription::retrieve(
        &stripe_client,
        &stripe::SubscriptionId::from_str(stripe_subscription_id.as_str())
            .map_err(|_| ServiceError::BadRequest("Invalid subscription id".to_string()))?,
        &[],
    )
    .await
    .map_err(|_| ServiceError::BadRequest("Failed to get subscription".to_string()))?;

    let stripe_customer_id = response.customer.id();
    Ok(stripe_customer_id.as_str().to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StripeBillingMetricsResponse {
    pub error: Option<StripeErrorMessage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StripeErrorMessage {
    pub message: String,
}

pub async fn send_stripe_billing(
    usage_based_subscription: StripeUsageBasedSubscription,
    clickhouse_client: &clickhouse::Client,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    let now_minus_1_hour = chrono::Utc::now() - chrono::Duration::hours(1);

    let last_recorded = chrono::DateTime::<chrono::Utc>::from_utc(
        usage_based_subscription.last_recorded_meter,
        chrono::Utc,
    );

    let date_range = Some(DateRange {
        gt: Some(last_recorded.format("%Y-%m-%d %H:%M:%S").to_string()),
        lt: Some(now_minus_1_hour.format("%Y-%m-%d %H:%M:%S").to_string()),
        ..Default::default()
    });

    let usage = get_extended_org_usage_by_id_query(
        usage_based_subscription.organization_id,
        date_range,
        clickhouse_client,
        pool.clone(),
        &mut None,
    )
    .await?;

    let stripe_customer_id =
        get_stripe_customer_id_from_subscription(usage_based_subscription.stripe_subscription_id)
            .await?;

    let events: Vec<(&str, String)> = vec![
        ("search_tokens", usage.search_tokens.to_string()),
        ("message_tokens", usage.message_tokens.to_string()),
        ("bytes_ingested", usage.bytes_ingested.to_string()),
        ("tokens_ingested", usage.tokens_ingested.to_string()),
        ("pages_crawled", usage.website_pages_scraped.to_string()),
        ("ocr_pages", usage.ocr_pages_ingested.to_string()),
        ("analytics_events", usage.events_ingested.to_string()),
    ];

    // Send to stripe meters
    let stripe_secret = get_env!("STRIPE_SECRET", "STRIPE_SECRET must be set");
    let reqwest_client = reqwest::Client::new();

    log::info!("Sending events for customer {:?}", stripe_customer_id);
    for (event_name, event_value) in events {
        log::info!("Sending event: {} -> {}", event_name, event_value);

        let identifier = hash_function(&format!(
            "{:?}||{:?}||{:?}||{:?}",
            event_name,
            now_minus_1_hour.to_rfc3339(),
            stripe_customer_id,
            usage_based_subscription.organization_id
        ));

        let response = reqwest_client
            .post("https://api.stripe.com/v1/billing/meter_events")
            .basic_auth(stripe_secret, Option::<&str>::None)
            .form(&[
                ("event_name", event_name),
                ("timestamp", &now_minus_1_hour.timestamp().to_string()),
                ("identifier", &identifier),
                ("payload[stripe_customer_id]", &stripe_customer_id),
                ("payload[value]", &event_value),
            ])
            .send()
            .await
            .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

        let stripe_billing_response = response
            .json::<StripeBillingMetricsResponse>()
            .await
            .unwrap();

        if let Some(error) = stripe_billing_response.error {
            log::error!("Failed to send stripe billing: {}", error.message);
        }
    }

    update_stripe_usage_based_subscription(
        usage_based_subscription.id,
        now_minus_1_hour.naive_utc(),
        pool.clone(),
    )
    .await
    .map_err(|e| {
        log::error!("Failed to update usage based stripe subscription: {}", e);
        ServiceError::BadRequest("Failed to update usage based stripe subscription".to_string())
    })?;

    log::info!(
        "Updated usage based stripe subscription {:?} last recorded meter to {:?}",
        usage_based_subscription.id,
        now_minus_1_hour.naive_utc()
    );

    Ok(())
}

pub async fn update_stripe_usage_based_subscription_plan_query(
    subscription_id: uuid::Uuid,
    plan_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::stripe_usage_based_subscriptions::dsl as stripe_usage_based_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    diesel::update(
        stripe_usage_based_subscriptions_columns::stripe_usage_based_subscriptions
            .filter(stripe_usage_based_subscriptions_columns::id.eq(subscription_id)),
    )
    .set(stripe_usage_based_subscriptions_columns::usage_based_plan_id.eq(plan_id))
    .execute(&mut conn)
    .await
    .map_err(|e| {
        log::error!("Failed to update usage based stripe subscription: {}", e);
        ServiceError::BadRequest("Failed to update usage based stripe subscription".to_string())
    })?;

    Ok(())
}

pub async fn update_stripe_usage_based_subscription(
    usage_based_subscription_id: uuid::Uuid,
    last_recorded_meter: chrono::NaiveDateTime,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::stripe_usage_based_subscriptions::dsl as stripe_usage_based_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    diesel::update(
        stripe_usage_based_subscriptions_columns::stripe_usage_based_subscriptions
            .filter(stripe_usage_based_subscriptions_columns::id.eq(usage_based_subscription_id)),
    )
    .set(stripe_usage_based_subscriptions_columns::last_recorded_meter.eq(last_recorded_meter))
    .execute(&mut conn)
    .await
    .map_err(|e| {
        log::error!("Failed to update usage based stripe subscription: {}", e);
        ServiceError::BadRequest("Failed to update usage based stripe subscription".to_string())
    })?;

    Ok(())
}

pub struct BillingPrice {
    pub free_tier: u64,
    pub past_free_tier_charge: f64,
    pub guage_name: String,
}

pub async fn get_bill_from_range(
    organization_id: uuid::Uuid,
    usage_plan: StripeUsageBasedPlan,
    date_range: DateRange,
    clickhouse_client: &clickhouse::Client,
    pool: web::Data<Pool>,
) -> Result<BillingEstimate, ServiceError> {
    let usage = get_extended_org_usage_by_id_query(
        organization_id,
        Some(date_range),
        clickhouse_client,
        pool.clone(),
        &mut None,
    )
    .await?;

    let stripe_secret = get_env!("STRIPE_SECRET", "STRIPE_SECRET must be set");

    let guage_to_price = usage_plan.guage_line_item_map();

    let req_client = reqwest::Client::new();

    let futures: Vec<_> = guage_to_price
        .iter()
        .map(|(guage, price_id)| {
            let req_client = req_client.clone();

            async move {
                // Get stripe price
                let price = req_client
                    .post(format!("https://api.stripe.com/v1/prices/{}", price_id))
                    .basic_auth(stripe_secret, Option::<&str>::None)
                    .form(&[("expand[]", "tiers")])
                    .send()
                    .await
                    .map_err(|e| {
                        ServiceError::BadRequest(format!("Failed to get stripe price: {}", e))
                    })?
                    .json::<stripe::Price>()
                    .await
                    .map_err(|e| {
                        ServiceError::BadRequest(format!("Failed to get stripe price text: {}", e))
                    })?;

                if let Some(tiers) = &price.tiers {
                    let mut iter_tiers = tiers.iter();

                    let free_tier = iter_tiers
                        .next()
                        .ok_or(ServiceError::BadRequest(
                            "Price must have tier [0]".to_string(),
                        ))?
                        .up_to
                        .expect("price must have up to");

                    let per_unit_price = iter_tiers.next().ok_or(ServiceError::BadRequest(
                        "Price must have tier [1]".to_string(),
                    ))?;
                    let per_unit_price = per_unit_price
                        .clone()
                        .unit_amount_decimal
                        .ok_or(ServiceError::BadRequest(
                            "Price must have unit amount".to_string(),
                        ))?
                        .clone();

                    Ok(BillingPrice {
                        free_tier: free_tier as u64,
                        past_free_tier_charge: per_unit_price.parse::<f64>().map_err(|_| {
                            ServiceError::BadRequest("Failed to format unit price".to_string())
                        })? / 100.0f64,
                        guage_name: guage.clone(),
                    })
                } else {
                    Err(ServiceError::BadRequest(
                        "Price must have tiers".to_string(),
                    ))
                }
            }
        })
        .collect();

    let prices: Vec<BillingPrice> = futures::future::join_all(futures)
        .await
        .into_iter()
        .collect::<Result<Vec<BillingPrice>, ServiceError>>()?;

    let all_events: Vec<(&str, String, u64)> = vec![
        (
            "chunk_storage_mb",
            "Chunk Storage (MB)".to_string(),
            get_storage_mb_from_chunk_count(usage.chunk_count) as u64,
        ),
        (
            "file_storage_mb",
            "File Storage (MB)".to_string(),
            (usage.file_storage * 1024) as u64,
        ),
        ("users", "Users".to_string(), usage.user_count as u64),
        (
            "dataset_count",
            "Datasets".to_string(),
            usage.dataset_count as u64,
        ),
        (
            "search_tokens",
            "Search Tokens".to_string(),
            usage.search_tokens as u64,
        ),
        (
            "message_tokens",
            "Message Tokens".to_string(),
            usage.message_tokens as u64,
        ),
        (
            "bytes_ingested",
            "Bytes Ingested".to_string(),
            usage.bytes_ingested as u64,
        ),
        (
            "tokens_ingested",
            "Tokens Ingested".to_string(),
            usage.tokens_ingested as u64,
        ),
        (
            "pages_crawled",
            "Pages Crawled".to_string(),
            usage.website_pages_scraped as u64,
        ),
        (
            "ocr_pages",
            "OCR Pages".to_string(),
            usage.ocr_pages_ingested as u64,
        ),
        (
            "analytics_events",
            "Analytics Events".to_string(),
            usage.events_ingested as u64,
        ),
    ];

    let mut total_cost = 0.0;
    let mut breakdown = vec![];
    for (guage, clean_name, usage_amount) in all_events.iter() {
        let billing_price = prices
            .iter()
            .find(|p| p.guage_name == *guage)
            .expect("Billing price will exist");

        let cost = max(usage_amount - billing_price.free_tier, 0) as f64
            * billing_price.past_free_tier_charge;

        breakdown.push(BillItem {
            name: billing_price.guage_name.clone(),
            usage_amount: *usage_amount,
            clean_name: clean_name.clone(),
            amount: cost,
        });

        total_cost += cost;
    }

    if let Some(platform_price_amount) = usage_plan.platform_price_amount {
        total_cost += platform_price_amount as f64;
        breakdown.push(BillItem {
            name: "Platform".to_string(),
            usage_amount: 0,
            clean_name: "Platform".to_string(),
            amount: platform_price_amount as f64,
        });
    }

    Ok(BillingEstimate {
        items: breakdown,
        total: total_cost,
    })
}
