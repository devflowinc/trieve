use crate::{
    data::models::{
        Organization, OrganizationUsageCount, OrganizationWithSubAndPlan, Pool, StripePlan,
        StripeSubscription,
    },
    errors::DefaultError,
    operators::stripe_operator::refresh_redis_org_plan_sub,
    randutil,
};
use actix_web::web;
use diesel::{
    result::DatabaseErrorKind, upsert::on_constraint, ExpressionMethods, JoinOnDsl,
    NullableExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper, Table,
};

/// Creates a dataset from Name if it doesn't conflict. If it does, then it creates a random name
/// for the user
pub async fn create_organization_query(
    name: &str,
    configuration: serde_json::Value,
    pool: web::Data<Pool>,
) -> Result<Organization, DefaultError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut new_organization = Organization::from_details(name.to_string(), configuration.clone());

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let mut number: usize = diesel::insert_into(organizations_columns::organizations)
        .values(new_organization.clone())
        .on_conflict(on_constraint("organizations_name_key"))
        .do_nothing()
        .execute(&mut conn)
        .map_err(|_| DefaultError {
            message: "Could not create organization, try again",
        })?;

    while number == 0 {
        // Get random name
        new_organization =
            Organization::from_details(randutil::random_organization_name(), configuration.clone());

        number = diesel::insert_into(organizations_columns::organizations)
            .values(new_organization.clone())
            .on_conflict(on_constraint("organizations_name_key"))
            .do_nothing()
            .execute(&mut conn)
            .map_err(|_| DefaultError {
                message: "Could not create organization, try again",
            })?;
    }

    refresh_redis_org_plan_sub(new_organization.id, pool).await?;

    Ok(new_organization)
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

    refresh_redis_org_plan_sub(updated_organization.id, pool).await?;

    Ok(updated_organization)
}

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
pub async fn get_organization_by_key_query(
    key: OrganizationKey,
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

            let mut conn = pool.get().map_err(|_| DefaultError {
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
                        .map_err(|_| DefaultError {
                            message: "Could not find organizations",
                        })?,
                    OrganizationKey::Name(name) => query
                        .filter(organizations_columns::name.eq(name))
                        .first::<(Organization, Option<StripePlan>, Option<StripeSubscription>)>(
                            &mut conn,
                        )
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
) -> Result<i32, DefaultError> {
    use crate::data::schema::organization_usage_counts::dsl as organization_usage_counts_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let dataset_count = organization_usage_counts_columns::organization_usage_counts
        .filter(organization_usage_counts_columns::org_id.eq(organization_id))
        .select(organization_usage_counts_columns::dataset_count)
        .first(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error loading org dataset count",
        })?;

    Ok(dataset_count)
}

pub fn get_user_org_count(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i32, DefaultError> {
    use crate::data::schema::organization_usage_counts::dsl as organization_usage_counts_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let user_count = organization_usage_counts_columns::organization_usage_counts
        .filter(organization_usage_counts_columns::org_id.eq(organization_id))
        .select(organization_usage_counts_columns::user_count)
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error loading org user count",
        })?;

    Ok(user_count)
}

pub fn get_message_org_count(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i32, DefaultError> {
    use crate::data::schema::organization_usage_counts::dsl as organization_usage_counts_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let messages_count = organization_usage_counts_columns::organization_usage_counts
        .filter(organization_usage_counts_columns::org_id.eq(organization_id))
        .select(organization_usage_counts_columns::message_count)
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error loading message organization count",
        })?;

    Ok(messages_count)
}

pub fn get_file_size_sum_org(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i32, DefaultError> {
    use crate::data::schema::organization_usage_counts::dsl as organization_usage_counts_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let file_size_sums: i32 = organization_usage_counts_columns::organization_usage_counts
        .filter(organization_usage_counts_columns::org_id.eq(organization_id))
        .select(organization_usage_counts_columns::file_storage)
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error loading file size sum organization count",
        })?;

    Ok(file_size_sums)
}

pub async fn get_org_usage_by_id_query(
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<OrganizationUsageCount, DefaultError> {
    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let org_usage_count: OrganizationUsageCount =
        crate::data::schema::organization_usage_counts::dsl::organization_usage_counts
            .filter(crate::data::schema::organization_usage_counts::dsl::org_id.eq(org_id))
            .first(&mut conn)
            .map_err(|_| DefaultError {
                message: "Could not find organization usage count",
            })?;

    Ok(org_usage_count)
}
