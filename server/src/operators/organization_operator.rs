use crate::{
    data::models::{
        Organization, OrganizationWithSubAndPlan, Pool, StripePlan, StripeSubscription,
    },
    errors::DefaultError,
    operators::stripe_operator::refresh_redis_org_plan_sub,
};
use actix_web::web;
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper, Table,
};

pub async fn create_organization_query(
    name: &str,
    configuration: serde_json::Value,
    pool: web::Data<Pool>,
) -> Result<Organization, DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let new_organization = Organization::from_details(name.to_string(), configuration);

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let created_organization: Organization =
        diesel::insert_into(organizations_columns::organizations)
            .values(new_organization)
            .get_result(&mut conn)
            .map_err(|_| DefaultError {
                message: "Could not create organization, try again",
            })?;

    refresh_redis_org_plan_sub(created_organization.id, pool).await?;

    Ok(created_organization)
}

pub async fn update_organization_query(
    id: uuid::Uuid,
    name: &str,
    configuration: serde_json::Value,
    pool: web::Data<Pool>,
) -> Result<Organization, DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let updated_organization: Organization = diesel::update(organizations_columns::organizations)
        .filter(organizations_columns::id.eq(id))
        .set((
            organizations_columns::name.eq(name),
            organizations_columns::configuration.eq(configuration),
            organizations_columns::updated_at.eq(chrono::Utc::now().naive_local()),
        ))
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to update organization, try again",
        })?;

    refresh_redis_org_plan_sub(updated_organization.id, pool).await?;

    Ok(updated_organization)
}

pub async fn get_organization_by_id_query(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<OrganizationWithSubAndPlan, DefaultError> {
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let redis_client = redis::Client::open(redis_url).map_err(|_| DefaultError {
        message: "Could not create redis client",
    })?;
    let mut redis_conn = redis_client
        .get_async_connection()
        .await
        .map_err(|_| DefaultError {
            message: "Could not connect to redis",
        })?;

    let redis_organization: Result<String, DefaultError> = redis::cmd("GET")
        .arg(format!("dataset:{}", id))
        .query_async(&mut redis_conn)
        .await
        .map_err(|_| DefaultError {
            message: "Could not get dataset from redis",
        });

    let org_plan_sub = match redis_organization {
        Ok(organization_str) => {
            let org_with_plan_sub =
                serde_json::from_str::<OrganizationWithSubAndPlan>(&organization_str)
                    .expect("Could not deserialize org with sub and plan from redis");

            org_with_plan_sub
        }
        Err(_) => {
            use crate::data::schema::organizations::dsl as organizations_columns;
            use crate::data::schema::stripe_plans::dsl as stripe_plans_columns;
            use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

            let mut conn = pool.get().map_err(|_| DefaultError {
                message: "Could not get database connection",
            })?;

            let org_plan_sub: (Organization, Option<StripePlan>, Option<StripeSubscription>) =
                organizations_columns::organizations
                    .left_outer_join(stripe_subscriptions_columns::stripe_subscriptions)
                    .left_outer_join(
                        stripe_plans_columns::stripe_plans
                            .on(stripe_plans_columns::id.eq(stripe_subscriptions_columns::plan_id)),
                    )
                    .select((
                        organizations_columns::organizations::all_columns(),
                        stripe_plans_columns::stripe_plans::all_columns().nullable(),
                        stripe_subscriptions_columns::stripe_subscriptions::all_columns()
                            .nullable(),
                    ))
                    .filter(organizations_columns::id.eq(id))
                    .first::<(Organization, Option<StripePlan>, Option<StripeSubscription>)>(
                        &mut conn,
                    )
                    .map_err(|_| DefaultError {
                        message: "Could not find organizations",
                    })?;

            let org_with_plan_sub: OrganizationWithSubAndPlan =
                OrganizationWithSubAndPlan::from_components(
                    org_plan_sub.0,
                    org_plan_sub.1,
                    org_plan_sub.2,
                );

            let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
            let client = redis::Client::open(redis_url).map_err(|_| DefaultError {
                message: "Could not create redis client",
            })?;
            let mut redis_conn = client
                .get_async_connection()
                .await
                .map_err(|_| DefaultError {
                    message: "Could not create redis client",
                })?;

            redis::cmd("SET")
                .arg(format!("organization:{}", org_with_plan_sub.id))
                .arg(
                    serde_json::to_string(&org_with_plan_sub).map_err(|_| DefaultError {
                        message: "Could not stringify organization",
                    })?,
                )
                .query_async(&mut redis_conn)
                .await
                .map_err(|_| DefaultError {
                    message: "Could not set organization in redis",
                })?;

            org_with_plan_sub
        }
    };

    Ok(org_plan_sub)
}

pub async fn get_org_from_dataset_id_query(
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Organization, DefaultError> {
    use crate::data::schema::datasets::dsl as datasets_columns;
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let organization: Organization = datasets_columns::datasets
        .inner_join(organizations_columns::organizations)
        .filter(datasets_columns::id.eq(dataset_id))
        .select(Organization::as_select())
        .first(&mut conn)
        .map_err(|_| DefaultError {
            message: "Could not find organization, try again with a different id",
        })?;

    Ok(organization)
}

pub fn get_org_dataset_count(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i64, DefaultError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let dataset_count = datasets_columns::datasets
        .filter(datasets_columns::organization_id.eq(organization_id))
        .count()
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error loading org datasets count",
        })?;

    Ok(dataset_count)
}
