use crate::{
    data::models::{
        Organization, OrganizationWithSubscriptionAndPlan, Pool, StripePlan, StripeSubscription,
    },
    errors::DefaultError,
};
use actix_web::web;
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper, Table,
};

pub async fn load_organization_with_subscription_and_plans_redis_query(
    pool: Pool,
) -> Result<(), DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;
    use crate::data::schema::stripe_plans::dsl as stripe_plans_columns;
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let organizations: Vec<(Organization, Option<StripePlan>, Option<StripeSubscription>)> =
        organizations_columns::organizations
            .left_outer_join(stripe_subscriptions_columns::stripe_subscriptions)
            .left_outer_join(
                stripe_plans_columns::stripe_plans
                    .on(stripe_plans_columns::id.eq(stripe_subscriptions_columns::plan_id)),
            )
            .select((
                organizations_columns::organizations::all_columns(),
                stripe_plans_columns::stripe_plans::all_columns().nullable(),
                stripe_subscriptions_columns::stripe_subscriptions::all_columns().nullable(),
            ))
            .load::<(Organization, Option<StripePlan>, Option<StripeSubscription>)>(&mut conn)
            .map_err(|_| DefaultError {
                message: "Could not find organizations",
            })?;

    let orgs_with_subs_and_plans: Vec<OrganizationWithSubscriptionAndPlan> = organizations
        .into_iter()
        .map(|(org, plan, sub)| {
            OrganizationWithSubscriptionAndPlan::from_components(org, plan, sub)
        })
        .collect();

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

    for org in orgs_with_subs_and_plans {
        redis::cmd("SET")
            .arg(format!("organization:{}", org.id))
            .arg(serde_json::to_string(&org).map_err(|_| DefaultError {
                message: "Could not stringify organization",
            })?)
            .query_async(&mut redis_conn)
            .await
            .map_err(|_| DefaultError {
                message: "Could not set organization in redis",
            })?;
    }

    Ok(())
}

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

    let org_with_plan_and_sub = OrganizationWithSubscriptionAndPlan::from_components(
        created_organization.clone(),
        None,
        None,
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
        .arg(format!("organization:{}", org_with_plan_and_sub.id))
        .arg(
            serde_json::to_string(&org_with_plan_and_sub).map_err(|_| DefaultError {
                message: "Could not stringify organization",
            })?,
        )
        .query_async(&mut redis_conn)
        .await
        .map_err(|_| DefaultError {
            message: "Could not set organization in redis",
        })?;

    Ok(created_organization)
}

pub async fn update_organization_query(
    id: uuid::Uuid,
    name: &str,
    configuration: serde_json::Value,
    pool: web::Data<Pool>,
) -> Result<Organization, DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;
    use crate::data::schema::stripe_plans::dsl as stripe_plans_columns;
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

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

    let stripe_subscription_and_plan: (Option<StripeSubscription>, Option<StripePlan>) =
        stripe_subscriptions_columns::stripe_subscriptions
            .inner_join(stripe_plans_columns::stripe_plans)
            .filter(stripe_subscriptions_columns::organization_id.eq(id))
            .select((
                stripe_subscriptions_columns::stripe_subscriptions::all_columns().nullable(),
                stripe_plans_columns::stripe_plans::all_columns().nullable(),
            ))
            .first(&mut conn)
            .map_err(|_| DefaultError {
                message: "Could not find stripe subscription and plan, try again",
            })?;

    let org_with_plan_and_sub = OrganizationWithSubscriptionAndPlan::from_components(
        updated_organization.clone(),
        stripe_subscription_and_plan.1,
        stripe_subscription_and_plan.0,
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
        .arg(format!("organization:{}", org_with_plan_and_sub.id))
        .arg(
            serde_json::to_string(&org_with_plan_and_sub).map_err(|_| DefaultError {
                message: "Could not stringify organization",
            })?,
        )
        .query_async(&mut redis_conn)
        .await
        .map_err(|_| DefaultError {
            message: "Could not set organization in redis",
        })?;

    Ok(updated_organization)
}

pub async fn delete_organization_by_id_query(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    diesel::delete(organizations_columns::organizations)
        .filter(organizations_columns::id.eq(id))
        .execute(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to delete organization, try again",
        })?;

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
    redis::cmd("DEL")
        .arg(format!("organization:{}", id))
        .query_async(&mut redis_conn)
        .await
        .map_err(|_| DefaultError {
            message: "Could not delete organization in redis",
        })?;

    Ok(())
}

pub fn get_organization_by_id_query(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Organization, DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let organization: Organization = organizations_columns::organizations
        .filter(organizations_columns::id.eq(id))
        .select(Organization::as_select())
        .first(&mut conn)
        .map_err(|_| DefaultError {
            message: "Could not find organization, try again with a different id",
        })?;

    Ok(organization)
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
