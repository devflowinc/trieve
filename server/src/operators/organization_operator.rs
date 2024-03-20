use crate::{
    data::models::{
        Dataset, Organization, OrganizationUsageCount, OrganizationWithSubAndPlan, Pool, RedisPool,
        ServerDatasetConfiguration, SlimUser, StripePlan, StripeSubscription, User,
        UserOrganization,
    },
    errors::DefaultError,
    operators::{
        dataset_operator::delete_dataset_by_id_query, stripe_operator::refresh_redis_org_plan_sub,
        user_operator::get_user_by_id_query,
    },
    randutil,
};
use actix_identity::Identity;
use actix_web::{web, HttpMessage, HttpRequest};
use diesel::prelude::*;
use diesel::{
    result::DatabaseErrorKind, upsert::on_constraint, ExpressionMethods, JoinOnDsl,
    NullableExpressionMethods, SelectableHelper, Table,
};
use diesel_async::RunQueryDsl;
use itertools::Itertools;

/// Creates a dataset from Name if it doesn't conflict. If it does, then it creates a random name
/// for the user
#[tracing::instrument(skip(pool, redis_pool))]
pub async fn create_organization_query(
    name: &str,
    redis_pool: web::Data<RedisPool>,
    pool: web::Data<Pool>,
) -> Result<Organization, DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut new_organization = Organization::from_details(name.to_string());

    let mut conn = pool.get().await.map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let mut number: usize = diesel::insert_into(organizations_columns::organizations)
        .values(new_organization.clone())
        .on_conflict(on_constraint("organizations_name_key"))
        .do_nothing()
        .execute(&mut conn)
        .await
        .map_err(|_| DefaultError {
            message: "Could not create organization, try again",
        })?;

    while number == 0 {
        // Get random name
        new_organization = Organization::from_details(randutil::random_organization_name());

        number = diesel::insert_into(organizations_columns::organizations)
            .values(new_organization.clone())
            .on_conflict(on_constraint("organizations_name_key"))
            .do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|_| DefaultError {
                message: "Could not create organization, try again",
            })?;
    }

    refresh_redis_org_plan_sub(new_organization.id, redis_pool, pool).await?;

    Ok(new_organization)
}

#[tracing::instrument(skip(pool, redis_pool))]
pub async fn update_organization_query(
    id: uuid::Uuid,
    name: &str,
    redis_pool: web::Data<RedisPool>,
    pool: web::Data<Pool>,
) -> Result<Organization, DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool.get().await.map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let updated_organization: Organization = diesel::update(organizations_columns::organizations)
        .filter(organizations_columns::id.eq(id))
        .set((
            organizations_columns::name.eq(name),
            organizations_columns::updated_at.eq(chrono::Utc::now().naive_local()),
        ))
        .get_result(&mut conn)
        .await
        .map_err(|err| match err {
            diesel::result::Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _) => {
                DefaultError {
                    message: "Organization name already exists",
                }
            }
            _ => DefaultError {
                message: "Failed to update organization, try again",
            },
        })?;

    refresh_redis_org_plan_sub(updated_organization.id, redis_pool, pool).await?;

    Ok(updated_organization)
}

#[tracing::instrument(skip(redis_pool, pool))]
pub async fn delete_organization_query(
    req: Option<&HttpRequest>,
    calling_user_id: Option<uuid::Uuid>,
    org_id: uuid::Uuid,
    redis_pool: web::Data<RedisPool>,
    pool: web::Data<Pool>,
) -> Result<Organization, DefaultError> {
    use crate::data::schema::datasets::dsl as datasets_columns;
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool.get().await.map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

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
            DefaultError {
                message: "Error loading datasets",
            }
        })?;

    for dataset in datasets {
        let config = ServerDatasetConfiguration::from_json(dataset.server_configuration);

        delete_dataset_by_id_query(dataset.id, pool.clone(), redis_pool.clone(), config.clone())
            .await
            .map_err(|e| {
                log::error!(
                    "Error deleting dataset in delete_organization_query: {:?}",
                    e
                );
                DefaultError {
                    message: "Error deleting dataset",
                }
            })?;
    }

    let deleted_organization: Organization = diesel::delete(
        organizations_columns::organizations.filter(organizations_columns::id.eq(org_id)),
    )
    .get_result(&mut conn)
    .await
    .map_err(|e| {
        log::error!(
            "Error deleting organization in delete_organization_query: {:?}",
            e
        );
        DefaultError {
            message: "Could not delete organization, try again",
        }
    })?;

    if req.is_some() && calling_user_id.is_some() {
        let user = get_user_by_id_query(
            &calling_user_id.expect("calling_user_id cannot be null here"),
            pool,
        )
        .await?;

        let slim_user: SlimUser = SlimUser::from_details(user.0, user.1, user.2);

        let user_string = serde_json::to_string(&slim_user).map_err(|e| {
            log::error!(
                "Error serializing user in delete_organization_query: {:?}",
                e
            );
            DefaultError {
                message: "Could not serialize user",
            }
        })?;

        Identity::login(
            &req.expect("Cannot be none at this point").extensions(),
            user_string,
        )
        .expect("Failed to refresh login for user");
    }

    Ok(deleted_organization)
}

#[derive(Debug)]
pub enum OrganizationKey {
    Id(uuid::Uuid),
    Name(String),
}

impl OrganizationKey {
    pub fn display(&self) -> String {
        match self {
            OrganizationKey::Id(id) => id.to_string(),
            OrganizationKey::Name(name) => name.to_string(),
        }
    }
}

impl From<uuid::Uuid> for OrganizationKey {
    fn from(id: uuid::Uuid) -> Self {
        OrganizationKey::Id(id)
    }
}

impl From<String> for OrganizationKey {
    fn from(name: String) -> Self {
        OrganizationKey::Name(name)
    }
}

/// Gets organization by id or name
#[tracing::instrument(skip(redis_pool, pool))]
pub async fn get_organization_by_key_query(
    key: OrganizationKey,
    redis_pool: web::Data<RedisPool>,
    pool: web::Data<Pool>,
) -> Result<OrganizationWithSubAndPlan, DefaultError> {
    let mut redis_conn = redis_pool.get().await.map_err(|_| DefaultError {
        message: "Failed to parse error",
    })?;

    let redis_organization: Result<String, DefaultError> = deadpool_redis::redis::cmd("GET")
        .arg(format!("organization:{}", key.display()))
        .query_async(&mut redis_conn)
        .await
        .map_err(|_| DefaultError {
            message: "Could not get dataset from redis",
        });

    let org_plan_sub = match redis_organization {
        Ok(organization_str) => {
            serde_json::from_str::<OrganizationWithSubAndPlan>(&organization_str)
                .expect("Could not deserialize org with sub and plan from redis")
        }
        Err(_) => {
            use crate::data::schema::organizations::dsl as organizations_columns;
            use crate::data::schema::stripe_plans::dsl as stripe_plans_columns;
            use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

            let mut conn = pool.get().await.map_err(|_| DefaultError {
                message: "Could not get database connection",
            })?;

            let query = organizations_columns::organizations
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
                .into_boxed();

            let org_plan_sub: (Organization, Option<StripePlan>, Option<StripeSubscription>) =
                match key {
                    OrganizationKey::Id(id) => query
                        .filter(organizations_columns::id.eq(id))
                        .first::<(Organization, Option<StripePlan>, Option<StripeSubscription>)>(
                            &mut conn,
                        )
                        .await
                        .map_err(|_| DefaultError {
                            message: "Could not find organizations",
                        })?,
                    OrganizationKey::Name(name) => query
                        .filter(organizations_columns::name.eq(name))
                        .first::<(Organization, Option<StripePlan>, Option<StripeSubscription>)>(
                            &mut conn,
                        )
                        .await
                        .map_err(|_| DefaultError {
                            message: "Could not find organizations",
                        })?,
                };

            let org_with_plan_sub: OrganizationWithSubAndPlan =
                OrganizationWithSubAndPlan::from_components(
                    org_plan_sub.0,
                    org_plan_sub.1,
                    org_plan_sub.2,
                );

            let mut redis_conn = redis_pool.get().await.map_err(|_| DefaultError {
                message: "Could not create redis client",
            })?;

            deadpool_redis::redis::cmd("SET")
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

            deadpool_redis::redis::cmd("SET")
                .arg(format!("organization:{}", org_with_plan_sub.name))
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

#[tracing::instrument(skip(pool))]
pub async fn get_org_from_id_query(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Organization, DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool.get().await.map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let organization: Organization = organizations_columns::organizations
        .filter(organizations_columns::id.eq(organization_id))
        .select(Organization::as_select())
        .first(&mut conn)
        .await
        .map_err(|_| DefaultError {
            message: "Could not find organization, try again with a different id",
        })?;

    Ok(organization)
}

#[tracing::instrument(skip(pool))]
pub async fn get_org_dataset_count(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i32, DefaultError> {
    use crate::data::schema::organization_usage_counts::dsl as organization_usage_counts_columns;

    let mut conn = pool.get().await.map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let dataset_count = organization_usage_counts_columns::organization_usage_counts
        .filter(organization_usage_counts_columns::org_id.eq(organization_id))
        .select(organization_usage_counts_columns::dataset_count)
        .first(&mut conn)
        .await
        .map_err(|_| DefaultError {
            message: "Error loading org dataset count",
        })?;

    Ok(dataset_count)
}

#[tracing::instrument(skip(pool))]
pub async fn get_user_org_count(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i32, DefaultError> {
    use crate::data::schema::organization_usage_counts::dsl as organization_usage_counts_columns;

    let mut conn = pool.get().await.map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let user_count = organization_usage_counts_columns::organization_usage_counts
        .filter(organization_usage_counts_columns::org_id.eq(organization_id))
        .select(organization_usage_counts_columns::user_count)
        .get_result(&mut conn)
        .await
        .map_err(|_| DefaultError {
            message: "Error loading org user count",
        })?;

    Ok(user_count)
}

#[tracing::instrument(skip(pool))]
pub async fn get_message_org_count(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i32, DefaultError> {
    use crate::data::schema::organization_usage_counts::dsl as organization_usage_counts_columns;

    let mut conn = pool.get().await.map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let messages_count = organization_usage_counts_columns::organization_usage_counts
        .filter(organization_usage_counts_columns::org_id.eq(organization_id))
        .select(organization_usage_counts_columns::message_count)
        .get_result(&mut conn)
        .await
        .map_err(|_| DefaultError {
            message: "Error loading message organization count",
        })?;

    Ok(messages_count)
}

#[tracing::instrument(skip(pool))]
pub async fn get_file_size_sum_org(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i64, DefaultError> {
    use crate::data::schema::organization_usage_counts::dsl as organization_usage_counts_columns;

    let mut conn = pool.get().await.map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let file_size_sums: i64 = organization_usage_counts_columns::organization_usage_counts
        .filter(organization_usage_counts_columns::org_id.eq(organization_id))
        .select(organization_usage_counts_columns::file_storage)
        .get_result(&mut conn)
        .await
        .map_err(|_| DefaultError {
            message: "Error loading file size sum organization count",
        })?;

    Ok(file_size_sums)
}

#[tracing::instrument(skip(pool))]
pub async fn get_org_usage_by_id_query(
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<OrganizationUsageCount, DefaultError> {
    let mut conn = pool.get().await.map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let org_usage_count: OrganizationUsageCount =
        crate::data::schema::organization_usage_counts::dsl::organization_usage_counts
            .filter(crate::data::schema::organization_usage_counts::dsl::org_id.eq(org_id))
            .first(&mut conn)
            .await
            .map_err(|_| DefaultError {
                message: "Could not find organization usage count",
            })?;

    Ok(org_usage_count)
}

#[tracing::instrument(skip(pool))]
pub async fn get_org_users_by_id_query(
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<SlimUser>, DefaultError> {
    use crate::data::schema::organizations::dsl as organization_columns;
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().await.unwrap();

    let user_orgs_orgs: Vec<(User, UserOrganization, Organization)> = users_columns::users
        .inner_join(user_organizations_columns::user_organizations)
        .inner_join(
            organization_columns::organizations
                .on(organization_columns::id.eq(user_organizations_columns::organization_id)),
        )
        .filter(user_organizations_columns::organization_id.eq(org_id))
        .select((
            User::as_select(),
            UserOrganization::as_select(),
            Organization::as_select(),
        ))
        .load::<(User, UserOrganization, Organization)>(&mut conn)
        .await
        .map_err(|_| DefaultError {
            message: "Error loading user",
        })?;

    Ok(user_orgs_orgs
        .into_iter()
        .map(|user_orgs_orgs| {
            SlimUser::from_details(
                user_orgs_orgs.0,
                vec![user_orgs_orgs.1],
                vec![user_orgs_orgs.2],
            )
        })
        .collect_vec())
}
