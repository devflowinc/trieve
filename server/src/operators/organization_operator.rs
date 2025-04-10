use crate::{
    data::models::{
        ApiKeyRespBody, Dataset, DatasetConfiguration, DateRange, Organization, OrganizationApiKey,
        OrganizationUsageCount, OrganizationWithSubAndPlan, Pool, RedisPool, SlimUser, StripePlan,
        StripeSubscription, StripeUsageBasedPlan, StripeUsageBasedSubscription, TrievePlan,
        TrieveSubscription, User, UserApiKey, UserOrganization,
    },
    errors::ServiceError,
    handlers::organization_handler::{CreateApiKeyReqPayload, ExtendedOrganizationUsageCount},
    operators::dataset_operator::soft_delete_dataset_by_id_query,
    randutil,
};
use actix_web::{web, HttpRequest};
use chrono::NaiveDateTime;
use clickhouse::Row;
use dateparser::DateTimeUtc;
use diesel::{
    prelude::*, result::DatabaseErrorKind, sql_query, ExpressionMethods, JoinOnDsl,
    NullableExpressionMethods, SelectableHelper, Table,
};
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use rand::{distributions::Alphanumeric, Rng};
use redis::AsyncCommands;
use serde::Deserialize;

/// Creates a dataset from Name if it doesn't conflict. If it does, then it creates a random name
/// for the user

pub async fn create_organization_query(
    name: &str,
    pool: web::Data<Pool>,
) -> Result<Organization, ServiceError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut new_organization = Organization::from_details(name.to_string());

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let mut number: usize = diesel::insert_into(organizations_columns::organizations)
        .values(new_organization.clone())
        .execute(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::BadRequest("Could not create organization, try again".to_string())
        })?;

    while number == 0 {
        // Get random name
        new_organization = Organization::from_details(randutil::random_organization_name());

        number = diesel::insert_into(organizations_columns::organizations)
            .values(new_organization.clone())
            .execute(&mut conn)
            .await
            .map_err(|_| {
                ServiceError::BadRequest("Could not create organization, try again".to_string())
            })?;
    }

    Ok(new_organization)
}

pub async fn update_organization_query(
    id: uuid::Uuid,
    name: &str,
    partner_configuration: serde_json::Value,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<Organization, ServiceError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let updated_organization: Organization = diesel::update(organizations_columns::organizations)
        .filter(organizations_columns::id.eq(id))
        .filter(organizations_columns::deleted.eq(0))
        .set((
            organizations_columns::name.eq(name),
            organizations_columns::partner_configuration.eq(partner_configuration),
            organizations_columns::updated_at.eq(chrono::Utc::now().naive_local()),
        ))
        .get_result(&mut conn)
        .await
        .map_err(|err| match err {
            diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                ServiceError::BadRequest("Organization name already exists".to_string())
            }
            _ => ServiceError::BadRequest("Failed to update organization, try again".to_string()),
        })?;

    let users: Vec<String> = crate::data::schema::user_organizations::dsl::user_organizations
        .filter(crate::data::schema::user_organizations::dsl::organization_id.eq(id))
        .select(crate::data::schema::user_organizations::dsl::user_id)
        .load::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|e| {
            log::error!("Error loading users in delete_organization_query: {:?}", e);
            ServiceError::BadRequest("Error loading users".to_string())
        })?
        .into_iter()
        .map(|id| id.to_string())
        .collect();

    let mut redis_conn = redis_pool.get().await.map_err(|_| {
        ServiceError::InternalServerError("Failed to get redis connection".to_string())
    })?;

    redis_conn.del::<_, ()>(users).await.map_err(|_| {
        ServiceError::InternalServerError("Failed to delete user from redis".to_string())
    })?;

    Ok(updated_organization)
}

pub async fn delete_organization_query(
    _req: Option<&HttpRequest>,
    _calling_user_id: Option<uuid::Uuid>,
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;
    use crate::data::schema::organizations::dsl as organizations_columns;
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let users: Vec<String> = crate::data::schema::user_organizations::dsl::user_organizations
        .filter(crate::data::schema::user_organizations::dsl::organization_id.eq(org_id))
        .select(crate::data::schema::user_organizations::dsl::user_id)
        .load::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|e| {
            log::error!("Error loading users in delete_organization_query: {:?}", e);
            ServiceError::BadRequest("Error loading users".to_string())
        })?
        .into_iter()
        .map(|id| id.to_string())
        .collect();

    let mut redis_conn = redis_pool.get().await.map_err(|_| {
        ServiceError::InternalServerError("Failed to get redis connection".to_string())
    })?;

    redis_conn.del::<_, ()>(users).await.map_err(|_| {
        ServiceError::InternalServerError("Failed to delete user from redis".to_string())
    })?;

    let existing_subscription: Option<StripeSubscription> =
        stripe_subscriptions_columns::stripe_subscriptions
            .filter(stripe_subscriptions_columns::organization_id.eq(org_id))
            .first(&mut conn)
            .await
            .ok();

    if let Some(subscription) = existing_subscription {
        if subscription.current_period_end.is_none() {
            return Err(ServiceError::BadRequest(
                "Cannot delete organization with active subscription".to_string(),
            ));
        };

        diesel::delete(
            stripe_subscriptions_columns::stripe_subscriptions
                .filter(stripe_subscriptions_columns::organization_id.eq(org_id)),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| {
            log::error!(
                "Error deleting subscription in delete_organization_query: {:?}",
                e
            );
            ServiceError::BadRequest("Could not delete subscription, try again".to_string())
        })?;
    }

    let datasets: Vec<Dataset> = datasets_columns::datasets
        .filter(datasets_columns::organization_id.eq(org_id))
        .select(Dataset::as_select())
        .load::<Dataset>(&mut conn)
        .await
        .map_err(|e| {
            log::error!(
                "Error loading datasets in delete_organization_query: {:?}",
                e
            );
            ServiceError::BadRequest("Error loading datasets".to_string())
        })?;

    for dataset in datasets {
        let config = DatasetConfiguration::from_json(dataset.server_configuration);

        soft_delete_dataset_by_id_query(
            dataset.id,
            config.clone(),
            pool.clone(),
            redis_pool.clone(),
        )
        .await
        .map_err(|e| {
            log::error!(
                "Error deleting dataset in delete_organization_query: {:?}",
                e
            );
            ServiceError::BadRequest("Error deleting dataset".to_string())
        })?;
    }

    diesel::update(
        organizations_columns::organizations.filter(organizations_columns::id.eq(org_id)),
    )
    .set(organizations_columns::deleted.eq(1))
    .execute(&mut conn)
    .await
    .map_err(|e| {
        log::error!(
            "Error deleting organization in delete_organization_query: {:?}",
            e
        );
        ServiceError::BadRequest("Could not delete organization, try again".to_string())
    })?;

    redis_conn
        .sadd::<_, _, ()>("deleted_organizations", org_id.to_string())
        .await
        .map_err(|_| {
            ServiceError::InternalServerError(
                "Failed to add organization to deleted_organizations".to_string(),
            )
        })?;

    Ok(())
}

pub async fn get_org_id_from_subscription_id_query(
    subscription_id: String,
    pool: web::Data<Pool>,
) -> Result<uuid::Uuid, ServiceError> {
    use crate::data::schema::organizations::dsl as organizations_columns;
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;
    use crate::data::schema::stripe_usage_based_subscriptions::dsl as stripe_usage_based_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let org_from_flat_subscription: Option<uuid::Uuid> = organizations_columns::organizations
        .inner_join(
            stripe_subscriptions_columns::stripe_subscriptions
                .on(stripe_subscriptions_columns::organization_id.eq(organizations_columns::id)),
        )
        .filter(stripe_subscriptions_columns::stripe_id.eq(subscription_id.clone()))
        .select(organizations_columns::id)
        .first::<uuid::Uuid>(&mut conn)
        .await
        .optional()?;

    let org_from_usage_based_subscription: Option<uuid::Uuid> =
        organizations_columns::organizations
            .inner_join(
                stripe_usage_based_subscriptions_columns::stripe_usage_based_subscriptions
                    .on(stripe_usage_based_subscriptions_columns::organization_id
                        .eq(organizations_columns::id)),
            )
            .filter(
                stripe_usage_based_subscriptions_columns::stripe_subscription_id
                    .eq(subscription_id),
            )
            .select(organizations_columns::id)
            .first::<uuid::Uuid>(&mut conn)
            .await
            .optional()?;

    let org_id = org_from_flat_subscription
        .or(org_from_usage_based_subscription)
        .ok_or(ServiceError::NotFound("Organization not found".to_string()))?;

    Ok(org_id)
}

pub async fn get_org_from_id_query(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<OrganizationWithSubAndPlan, ServiceError> {
    use crate::data::schema::organizations::dsl as organizations_columns;
    use crate::data::schema::stripe_plans::dsl as stripe_plans_columns;
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;
    use crate::data::schema::stripe_usage_based_plans::dsl as stripe_usage_based_plans_columns;
    use crate::data::schema::stripe_usage_based_subscriptions::dsl as stripe_usage_based_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let query = organizations_columns::organizations
        .left_outer_join(stripe_subscriptions_columns::stripe_subscriptions.on(
            stripe_subscriptions_columns::organization_id
                .eq(organizations_columns::id)
                .and(
                    stripe_subscriptions_columns::current_period_end
                        .is_null()
                        .or(stripe_subscriptions_columns::current_period_end
                            .gt(chrono::Utc::now().naive_utc())),
                ),
        ))
        .left_outer_join(
            stripe_plans_columns::stripe_plans
                .on(stripe_plans_columns::id.eq(stripe_subscriptions_columns::plan_id)),
        )
        .left_outer_join(
            stripe_usage_based_subscriptions_columns::stripe_usage_based_subscriptions
                .on(stripe_usage_based_subscriptions_columns::organization_id
                    .eq(organizations_columns::id)
                    .and(
                        stripe_usage_based_subscriptions_columns::current_period_end
                            .is_null()
                            .or(stripe_usage_based_subscriptions_columns::current_period_end
                                .gt(chrono::Utc::now().naive_utc())),
                    ),
                )
        )
        .left_outer_join(
            stripe_usage_based_plans_columns::stripe_usage_based_plans
                .on(stripe_usage_based_plans_columns::id
                    .eq(stripe_usage_based_subscriptions_columns::usage_based_plan_id)),
        )
        .select((
            organizations_columns::organizations::all_columns(),
            stripe_plans_columns::stripe_plans::all_columns().nullable(),
            stripe_subscriptions_columns::stripe_subscriptions::all_columns().nullable(),
            stripe_usage_based_plans_columns::stripe_usage_based_plans::all_columns().nullable(),
            stripe_usage_based_subscriptions_columns::stripe_usage_based_subscriptions::all_columns().nullable()
        ))
        .into_boxed();

    let (organization, stripe_plan, stripe_subscription, usage_plan, usage_subscription): (
        Organization,
        Option<StripePlan>,
        Option<StripeSubscription>,
        Option<StripeUsageBasedPlan>,
        Option<StripeUsageBasedSubscription>,
    ) = query
        .filter(organizations_columns::id.eq(organization_id))
        .filter(organizations_columns::deleted.eq(0))
        .first::<(
            Organization,
            Option<StripePlan>,
            Option<StripeSubscription>,
            Option<StripeUsageBasedPlan>,
            Option<StripeUsageBasedSubscription>,
        )>(&mut conn)
        .await
        .map_err(|e| {
            log::error!(
                "Error getting org from id in get_org_from_id_query: {:?}",
                e
            );

            ServiceError::NotFound("Organization not found".to_string())
        })?;

    let mut subscription = TrieveSubscription::from_flat(stripe_subscription, usage_subscription);
    let mut plan = TrievePlan::from_flat(stripe_plan, usage_plan);

    if let Some(sub) = &subscription {
        let end_date = sub.current_period_end();
        if end_date.is_some()
            && chrono::Utc::now().naive_utc() > end_date.expect("End date should be set")
        {
            log::info!("Gotcha!");
            subscription = None;
            plan = None;
        }
    }

    let org_with_plan_sub: OrganizationWithSubAndPlan =
        OrganizationWithSubAndPlan::from_components(organization, plan, subscription);

    Ok(org_with_plan_sub)
}

pub async fn get_org_dataset_count(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i32, ServiceError> {
    use crate::data::schema::organization_usage_counts::dsl as organization_usage_counts_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let dataset_count = organization_usage_counts_columns::organization_usage_counts
        .filter(organization_usage_counts_columns::org_id.eq(organization_id))
        .select(organization_usage_counts_columns::dataset_count)
        .first(&mut conn)
        .await
        .map_err(|e| {
            log::error!(
                "Error loading org dataset count in get_org_dataset_count: {:?}",
                e
            );

            ServiceError::BadRequest("Error loading org dataset count".to_string())
        })?;

    Ok(dataset_count)
}

pub async fn get_user_org_count(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i32, ServiceError> {
    use crate::data::schema::organization_usage_counts::dsl as organization_usage_counts_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let user_count = organization_usage_counts_columns::organization_usage_counts
        .filter(organization_usage_counts_columns::org_id.eq(organization_id))
        .select(organization_usage_counts_columns::user_count)
        .get_result(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Error loading org user count".to_string()))?;

    Ok(user_count)
}

pub async fn get_message_org_count(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i32, ServiceError> {
    use crate::data::schema::organization_usage_counts::dsl as organization_usage_counts_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let messages_count = organization_usage_counts_columns::organization_usage_counts
        .filter(organization_usage_counts_columns::org_id.eq(organization_id))
        .select(organization_usage_counts_columns::message_count)
        .get_result(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::BadRequest("Error loading message organization count".to_string())
        })?;

    Ok(messages_count)
}

pub async fn get_file_size_sum_org(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i64, ServiceError> {
    use crate::data::schema::organization_usage_counts::dsl as organization_usage_counts_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let file_size_sums: i64 = organization_usage_counts_columns::organization_usage_counts
        .filter(organization_usage_counts_columns::org_id.eq(organization_id))
        .select(organization_usage_counts_columns::file_storage)
        .get_result(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::BadRequest("Error loading file size sum organization count".to_string())
        })?;

    Ok(file_size_sums)
}

pub async fn get_org_usage_by_id_query(
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<OrganizationUsageCount, ServiceError> {
    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let org_usage_count: OrganizationUsageCount =
        crate::data::schema::organization_usage_counts::dsl::organization_usage_counts
            .filter(crate::data::schema::organization_usage_counts::dsl::org_id.eq(org_id))
            .select(OrganizationUsageCount::as_select())
            .first(&mut conn)
            .await
            .map_err(|_| {
                ServiceError::BadRequest("Could not find organization usage count".to_string())
            })?;

    Ok(org_usage_count)
}

#[derive(Row, Deserialize)]
pub struct SearchTokensRow {
    search_tokens: u64,
    search_count: u64,
}

#[derive(Row, Deserialize)]
pub struct MessageTokensRow {
    message_tokens: u64,
    message_count: u64,
}

#[derive(Row, Deserialize)]
pub struct IngestedTokensRow {
    bytes_ingested: u64,
    tokens_ingested: u64,
}

#[derive(Row, Deserialize)]
pub struct OCRPageCountRow {
    page_count: u64,
}

#[derive(Row, Deserialize)]
pub struct WebpagesScrapedRow {
    pages_scraped: u64,
}

#[derive(Row, Deserialize)]
pub struct EventsIngestedRow {
    event_count: u64,
}

#[derive(Row, Deserialize)]
pub struct CurrentMonthMessageCountRow {
    query_count: u64,
}

pub fn format_with_daterange(query_string: String, date_range: &Option<DateRange>) -> String {
    let mut query_string = query_string;
    if let Some(date_range) = date_range {
        if let Some(gt) = &date_range.gt {
            query_string.push_str(&format!(" AND created_at > '{}'", gt));
        }
        if let Some(lt) = &date_range.lt {
            query_string.push_str(&format!(" AND created_at < '{}'", lt));
        }
        if let Some(gte) = &date_range.gte {
            query_string.push_str(&format!(" AND created_at >= '{}'", gte));
        }
        if let Some(lte) = &date_range.lte {
            query_string.push_str(&format!(" AND created_at <= '{}'", lte));
        }
    }

    query_string
}

pub async fn get_extended_org_usage_by_id_query(
    organization_id: uuid::Uuid,
    date_range: Option<DateRange>,
    clickhouse_client: &clickhouse::Client,
    pool: web::Data<Pool>,
    timer: &mut Option<simple_server_timing_header::Timer>,
) -> Result<ExtendedOrganizationUsageCount, ServiceError> {
    let usage = get_org_usage_by_id_query(organization_id, pool).await?;

    let search_tokens_fut = clickhouse_client
        .query(&format_with_daterange(
            "
        SELECT SUM(tokens) as search_tokens, COUNT(*) as search_count
        FROM search_queries
        WHERE organization_id = ?
        "
            .to_string(),
            &date_range,
        ))
        .bind(organization_id)
        .fetch_one::<SearchTokensRow>();

    let message_tokens_fut = clickhouse_client
        .query(&format_with_daterange(
            "
        SELECT SUM(tokens) as message_tokens, COUNT(*) as message_count
        FROM rag_queries
        WHERE organization_id = ?
        "
            .to_string(),
            &date_range,
        ))
        .bind(organization_id)
        .fetch_one::<MessageTokensRow>();

    let bytes_and_tokens_ingested_fut = clickhouse_client
        .query(&format_with_daterange(
            "
        SELECT SUM(JSONExtractUInt(event_data, 'bytes_ingested')) as bytes_ingested,
               SUM(JSONExtractUInt(event_data, 'tokens_ingested')) as tokens_ingested
        FROM dataset_events
        WHERE event_type  = 'chunks_uploaded' and organization_id = ?"
                .to_string(),
            &date_range,
        ))
        .bind(organization_id)
        .fetch_one::<IngestedTokensRow>();

    let ocr_pages_fut = clickhouse_client
        .query(&format_with_daterange(
            "
        SELECT SUM(JSONExtractUInt(event_data, 'pages')) as page_count
        FROM dataset_events
        WHERE event_type  = 'file_uploaded' and organization_id = ?"
                .to_string(),
            &date_range,
        ))
        .bind(organization_id)
        .fetch_one::<OCRPageCountRow>();

    let website_pages_scraped_fut = clickhouse_client
        .query(&format_with_daterange(
            "
        SELECT SUM(JSONExtractUInt(event_data, 'pages_crawled')) as pages_scraped
        FROM dataset_events
        WHERE event_type  = 'crawl_completed' and organization_id = ?"
                .to_string(),
            &date_range,
        ))
        .bind(organization_id)
        .fetch_one::<WebpagesScrapedRow>();

    let rag_events_fut = clickhouse_client
        .query(&format_with_daterange(
            "
            SELECT COUNT(*) as event_count
            FROM rag_queries
            WHERE organization_id = ?
        "
            .to_string(),
            &date_range,
        ))
        .bind(organization_id)
        .fetch_one::<EventsIngestedRow>();

    let search_events_fut = clickhouse_client
        .query(&format_with_daterange(
            "
            SELECT COUNT(*) as event_count
            FROM search_queries
            WHERE organization_id = ?
        "
            .to_string(),
            &date_range,
        ))
        .bind(organization_id)
        .fetch_one::<EventsIngestedRow>();

    let current_months_message_count_fut = clickhouse_client
        .query(&format_with_daterange(
            "
            SELECT query_count
            FROM mv_rag_queries_monthly_counts
            WHERE organization_id = ?
            AND month_year = formatDateTime(now(), '%Y-%m')
        "
            .to_string(),
            &None,
        ))
        .bind(organization_id)
        .fetch_one::<CurrentMonthMessageCountRow>();

    // Execute all queries in parallel
    let (
        search_tokens_result,
        message_tokens_result,
        bytes_and_tokens_ingested_result,
        ocr_pages_result,
        website_pages_scraped_result,
        rag_events_result,
        search_events_result,
        current_months_message_count_result,
    ) = futures::join!(
        search_tokens_fut,
        message_tokens_fut,
        bytes_and_tokens_ingested_fut,
        ocr_pages_fut,
        website_pages_scraped_fut,
        rag_events_fut,
        search_events_fut,
        current_months_message_count_fut
    );

    // Handle the results
    let search_tokens = search_tokens_result.map_err(|e| {
        ServiceError::InternalServerError(format!("Error fetching search queries {:?}", e))
    })?;

    let message_tokens = message_tokens_result.map_err(|e| {
        ServiceError::InternalServerError(format!("Error fetching message queries {:?}", e))
    })?;

    let bytes_and_tokens_ingested = bytes_and_tokens_ingested_result.map_err(|e| {
        ServiceError::InternalServerError(format!("Error fetching ingestion data {:?}", e))
    })?;

    let ocr_pages = ocr_pages_result.map_err(|e| {
        ServiceError::InternalServerError(format!("Error fetching ingestion data {:?}", e))
    })?;

    let website_pages_scraped = website_pages_scraped_result.map_err(|e| {
        ServiceError::InternalServerError(format!("Error fetching ingestion data {:?}", e))
    })?;

    let rag_events = rag_events_result.map_err(|e| {
        ServiceError::InternalServerError(format!("Error fetching ingestion data {:?}", e))
    })?;

    let search_events = search_events_result.map_err(|e| {
        ServiceError::InternalServerError(format!("Error fetching ingestion data {:?}", e))
    })?;

    let current_months_message_count = current_months_message_count_result.map_err(|e| {
        ServiceError::InternalServerError(format!("Error fetching ingestion data {:?}", e))
    })?;

    if let Some(timer) = timer {
        timer.add("fetched all clickhouse data in parallel");
    }

    Ok(ExtendedOrganizationUsageCount {
        dataset_count: usage.dataset_count,
        user_count: usage.user_count,
        chunk_count: usage.chunk_count,
        file_storage: usage.file_storage,
        search_tokens: search_tokens.search_tokens,
        message_tokens: message_tokens.message_tokens,
        bytes_ingested: bytes_and_tokens_ingested.bytes_ingested,
        tokens_ingested: bytes_and_tokens_ingested.tokens_ingested,
        message_count: message_tokens.message_count,
        search_count: search_tokens.search_count,
        ocr_pages_ingested: ocr_pages.page_count,
        website_pages_scraped: website_pages_scraped.pages_scraped,
        events_ingested: rag_events.event_count + search_events.event_count,
        current_months_message_count: current_months_message_count.query_count,
    })
}

pub async fn get_org_users_by_id_query(
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<SlimUser>, ServiceError> {
    use crate::data::schema::organizations::dsl as organization_columns;
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let user_orgs_orgs: Vec<(User, UserOrganization, Organization)> = users_columns::users
        .inner_join(user_organizations_columns::user_organizations)
        .inner_join(
            organization_columns::organizations
                .on(organization_columns::id.eq(user_organizations_columns::organization_id)),
        )
        .filter(organization_columns::deleted.eq(0))
        .filter(user_organizations_columns::organization_id.eq(org_id))
        .select((
            User::as_select(),
            UserOrganization::as_select(),
            Organization::as_select(),
        ))
        .load::<(User, UserOrganization, Organization)>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Error loading user".to_string()))?;

    Ok(user_orgs_orgs
        .into_iter()
        .map(|user_orgs_orgs| {
            SlimUser::from_details(
                user_orgs_orgs.0,
                vec![user_orgs_orgs.1],
                vec![user_orgs_orgs.2.with_complete_partner_config()],
            )
        })
        .collect_vec())
}

pub async fn get_arbitrary_org_owner_from_org_id(
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<SlimUser, ServiceError> {
    use crate::data::schema::organizations::dsl as organization_columns;
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let (user, user_organization, organization): (User, UserOrganization, Organization) = users_columns::users
        .inner_join(user_organizations_columns::user_organizations)
        .inner_join(
            organization_columns::organizations
                .on(organization_columns::id.eq(user_organizations_columns::organization_id)),
        )
        .filter(user_organizations_columns::organization_id.eq(org_id))
        .filter(user_organizations_columns::role.eq(2))
                    .filter(organization_columns::deleted.eq(0))

        .select((
            User::as_select(),
            UserOrganization::as_select(),
            Organization::as_select(),
        ))
        .first::<(User, UserOrganization, Organization)>(&mut conn)
        .await
        .map_err(|e| {
            log::error!(
                "Error getting arbitrary org owner from org id in get_arbitrary_org_owner_from_org_id: {:?}",
                e
            );

            ServiceError::BadRequest(
                "Relevant organization had no owner level users".to_string(),
            )
        }
    )?;

    Ok(SlimUser::from_details(
        user,
        vec![user_organization],
        vec![organization.with_complete_partner_config()],
    ))
}

pub async fn get_organization_from_dataset_id(
    dataset_id: uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<Organization, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;
    use crate::data::schema::organizations::dsl as organization_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let organization: Organization = datasets_columns::datasets
        .inner_join(
            organization_columns::organizations
                .on(organization_columns::id.eq(datasets_columns::organization_id)),
        )
        .filter(organization_columns::deleted.eq(0))
        .filter(datasets_columns::id.eq(dataset_id))
        .select(Organization::as_select())
        .first::<Organization>(&mut conn)
        .await
        .map_err(|e| {
            log::error!(
                "Could not get organization from dataset id in get_organization_from_dataset_id: {:?}",
                e
            );
            ServiceError::BadRequest(
                "Could not get organization from dataset id".to_string(),
            )
        })?;

    Ok(organization)
}

pub async fn get_arbitrary_org_owner_from_dataset_id(
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<SlimUser, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;
    use crate::data::schema::organizations::dsl as organization_columns;
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let (user, user_organization, organization): (User, UserOrganization, Organization) = users_columns::users
        .inner_join(user_organizations_columns::user_organizations)
        .inner_join(
            organization_columns::organizations
                .on(organization_columns::id.eq(user_organizations_columns::organization_id)),
        )
        .inner_join(
            datasets_columns::datasets
                .on(datasets_columns::organization_id.eq(organization_columns::id)),
        )
        .filter(organization_columns::deleted.eq(0))
        .filter(datasets_columns::id.eq(dataset_id))
        .filter(user_organizations_columns::role.eq(2))
        .select((
            User::as_select(),
            UserOrganization::as_select(),
            Organization::as_select(),
        ))
        .first::<(User, UserOrganization, Organization)>(&mut conn)
        .await
        .map_err(|e| {
            log::error!(
                "Error getting arbitrary org owner from dataset id in get_arbitrary_org_owner_from_dataset_id: {:?}",
                e
            );
            ServiceError::BadRequest(
                "Relevant organization for dataset had no owner level users".to_string(),
            )
        }
    )?;

    Ok(SlimUser::from_details(
        user,
        vec![user_organization],
        vec![organization.with_complete_partner_config()],
    ))
}

pub async fn get_soft_deleted_datasets_for_organization(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<Dataset>, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let datasets: Vec<Dataset> = datasets_columns::datasets
        .filter(datasets_columns::organization_id.eq(organization_id))
        .filter(datasets_columns::deleted.eq(1))
        .load::<Dataset>(&mut conn)
        .await
        .map_err(|e| {
            log::error!(
                "Error getting soft deleted datasets for organization in get_soft_deleted_datasets_for_organization: {:?}",
                e
            );
            ServiceError::BadRequest("Error loading soft deleted datasets".to_string())
        })?;

    Ok(datasets)
}

pub async fn delete_actual_organization_query(
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    diesel::delete(
        organizations_columns::organizations.filter(organizations_columns::id.eq(org_id)),
    )
    .execute(&mut conn)
    .await
    .map_err(|e| {
        log::error!(
            "Error deleting organization in delete_actual_organization_query: {:?}",
            e
        );
        ServiceError::BadRequest("Error deleting organization".to_string())
    })?;

    Ok(())
}

pub async fn update_all_org_dataset_configs_query(
    org_id: uuid::Uuid,
    new_config: serde_json::Value,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    let concat_configs_raw_query = sql_query(format!(
        "UPDATE datasets SET server_configuration = server_configuration || '{}' WHERE organization_id = '{}';",
        new_config.to_string().replace('\'', "''"), org_id
    ));

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    concat_configs_raw_query
        .execute(&mut conn)
        .await
        .map_err(|e| {
            log::error!(
                "Error updating datasets in update_all_org_dataset_server_configs: {:?}",
                e
            );
            ServiceError::BadRequest("Error updating datasets".to_string())
        })?;

    Ok(())
}

pub fn generate_api_key() -> String {
    let rng = rand::thread_rng();
    let api_key: String = format!(
        "tr-{}",
        rng.sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>()
    );

    api_key
}

pub fn hash_function(password: &str) -> String {
    blake3::hash(password.as_bytes()).to_string()
}

pub async fn create_organization_api_key_query(
    organization_id: uuid::Uuid,
    data: CreateApiKeyReqPayload,
    pool: web::Data<Pool>,
) -> Result<String, ServiceError> {
    let raw_api_key = generate_api_key();
    let hashed_api_key = hash_function(&raw_api_key);

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let expiry = {
        data.expires_at
            .clone()
            .and_then(|ts| -> Option<NaiveDateTime> {
                ts.parse::<DateTimeUtc>()
                    .ok()
                    .map(|date| date.0.naive_utc())
            })
    };

    let api_key_struct = OrganizationApiKey::from_details(
        organization_id,
        hashed_api_key.clone(),
        data.name,
        data.role.into(),
        data.dataset_ids,
        data.scopes,
        data.default_params,
        expiry,
    );

    diesel::insert_into(crate::data::schema::organization_api_key::dsl::organization_api_key)
        .values(&api_key_struct)
        .execute(&mut conn)
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Error setting api key {}", err)))?;

    Ok(raw_api_key)
}

pub async fn get_organization_api_keys_query(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<ApiKeyRespBody>, ServiceError> {
    use crate::data::schema::organization_api_key::dsl as organization_api_key_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let api_keys = organization_api_key_columns::organization_api_key
        .filter(organization_api_key_columns::organization_id.eq(organization_id))
        .select(OrganizationApiKey::as_select())
        .load::<OrganizationApiKey>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Error loading organization api keys".to_string()))?;

    let api_keys = api_keys
        .into_iter()
        .map(|api_key| api_key.into())
        .collect::<Vec<ApiKeyRespBody>>();
    Ok(api_keys)
}

pub async fn delete_organization_api_keys_query(
    organization_id: uuid::Uuid,
    api_key_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::organization_api_key::dsl as organization_api_key_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    diesel::delete(
        organization_api_key_columns::organization_api_key
            .filter(organization_api_key_columns::organization_id.eq(organization_id))
            .filter(organization_api_key_columns::id.eq(api_key_id)),
    )
    .execute(&mut conn)
    .await
    .map_err(|_| ServiceError::BadRequest("Error deleting organization api key".to_string()))?;

    Ok(())
}

pub async fn get_assumed_user_by_organization_api_key(
    api_key: &str,
    pool: web::Data<Pool>,
) -> Result<(SlimUser, UserApiKey), ServiceError> {
    use crate::data::schema::organization_api_key::dsl as organization_api_key_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let api_key: OrganizationApiKey = organization_api_key_columns::organization_api_key
        .filter(organization_api_key_columns::api_key_hash.eq(hash_function(api_key)))
        .filter(
            organization_api_key_columns::expires_at
                .is_null()
                .or(organization_api_key_columns::expires_at.ge(diesel::dsl::now.nullable())),
        )
        .select(OrganizationApiKey::as_select())
        .first::<OrganizationApiKey>(&mut conn)
        .await
        .map_err(|_| ServiceError::Unauthorized)?;

    let org_sub_plan = get_org_from_id_query(api_key.organization_id, pool.clone()).await?;

    let fake_user_id = uuid::Uuid::new_v4();

    let user = SlimUser {
        id: fake_user_id,
        email: "".to_string(),
        name: Some("".to_string()),
        created_at: chrono::Utc::now().naive_utc(),
        user_orgs: vec![UserOrganization {
            id: uuid::Uuid::new_v4(),
            user_id: fake_user_id,
            organization_id: org_sub_plan.organization.id,
            role: api_key.role,
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        }],
        orgs: vec![org_sub_plan.organization],
    };

    Ok((user, api_key.into()))
}

pub async fn get_all_organization_ids(
    pool: web::Data<Pool>,
) -> Result<Vec<uuid::Uuid>, ServiceError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let organizations = organizations_columns::organizations
        .select(organizations_columns::id)
        .filter(organizations_columns::deleted.eq(0))
        .load::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|_| ServiceError::NotFound("Could not get organizations".to_string()))?;

    Ok(organizations)
}
