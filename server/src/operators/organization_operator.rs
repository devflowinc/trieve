use crate::{
    data::models::{
        Dataset, Organization, OrganizationUsageCount, OrganizationWithSubAndPlan, Pool,
        ServerDatasetConfiguration, SlimUser, StripePlan, StripeSubscription, User,
        UserOrganization,
    },
    errors::ServiceError,
    operators::{
        dataset_operator::delete_dataset_by_id_query, user_operator::get_user_by_id_query,
    },
    randutil,
};
use actix_identity::Identity;
use actix_web::{web, HttpMessage, HttpRequest};
use diesel::prelude::*;
use diesel::{
    result::DatabaseErrorKind, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    SelectableHelper, Table,
};
use diesel_async::RunQueryDsl;
use itertools::Itertools;

/// Creates a dataset from Name if it doesn't conflict. If it does, then it creates a random name
/// for the user
#[tracing::instrument(skip(pool))]
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

#[tracing::instrument(skip(pool))]
pub async fn update_organization_query(
    id: uuid::Uuid,
    name: &str,
    pool: web::Data<Pool>,
) -> Result<Organization, ServiceError> {
    use crate::data::schema::organizations::dsl as organizations_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

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
                ServiceError::BadRequest("Organization name already exists".to_string())
            }
            _ => ServiceError::BadRequest("Failed to update organization, try again".to_string()),
        })?;

    Ok(updated_organization)
}

#[tracing::instrument(skip(pool))]
pub async fn delete_organization_query(
    req: Option<&HttpRequest>,
    calling_user_id: Option<uuid::Uuid>,
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Organization, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;
    use crate::data::schema::organizations::dsl as organizations_columns;
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

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
        let config = ServerDatasetConfiguration::from_json(dataset.server_configuration);

        delete_dataset_by_id_query(dataset.id, pool.clone(), config.clone())
            .await
            .map_err(|e| {
                log::error!(
                    "Error deleting dataset in delete_organization_query: {:?}",
                    e
                );
                ServiceError::BadRequest("Error deleting dataset".to_string())
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
        ServiceError::BadRequest("Could not delete organization, try again".to_string())
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
            ServiceError::BadRequest("Could not serialize user".to_string())
        })?;

        Identity::login(
            &req.expect("Cannot be none at this point").extensions(),
            user_string,
        )
        .expect("Failed to refresh login for user");
    }

    Ok(deleted_organization)
}

#[tracing::instrument(skip(pool))]
pub async fn get_org_from_id_query(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<OrganizationWithSubAndPlan, ServiceError> {
    use crate::data::schema::organizations::dsl as organizations_columns;
    use crate::data::schema::stripe_plans::dsl as stripe_plans_columns;
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

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

    let org_plan_sub: (Organization, Option<StripePlan>, Option<StripeSubscription>) = query
        .filter(organizations_columns::id.eq(organization_id))
        .first::<(Organization, Option<StripePlan>, Option<StripeSubscription>)>(&mut conn)
        .await
        .map_err(|e| {
            log::error!(
                "Error getting org from id in get_org_from_id_query: {:?}",
                e
            );

            ServiceError::NotFound("Organization not found".to_string())
        })?;

    let org_with_plan_sub: OrganizationWithSubAndPlan =
        OrganizationWithSubAndPlan::from_components(org_plan_sub.0, org_plan_sub.1, org_plan_sub.2);

    Ok(org_with_plan_sub)
}

#[tracing::instrument(skip(pool))]
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
        .map_err(|_| ServiceError::BadRequest("Error loading org dataset count".to_string()))?;

    Ok(dataset_count)
}

#[tracing::instrument(skip(pool))]
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

#[tracing::instrument(skip(pool))]
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

#[tracing::instrument(skip(pool))]
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

#[tracing::instrument(skip(pool))]
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
            .first(&mut conn)
            .await
            .map_err(|_| {
                ServiceError::BadRequest("Could not find organization usage count".to_string())
            })?;

    Ok(org_usage_count)
}

#[tracing::instrument(skip(pool))]
pub async fn get_org_users_by_id_query(
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<SlimUser>, ServiceError> {
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
        .map_err(|_| ServiceError::BadRequest("Error loading user".to_string()))?;

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

#[tracing::instrument(skip(pool))]
pub async fn get_arbitrary_org_owner_from_org_id(
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<SlimUser, ServiceError> {
    use crate::data::schema::organizations::dsl as organization_columns;
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().await.unwrap();

    let user_orgs_orgs: (User, UserOrganization, Organization) = users_columns::users
        .inner_join(user_organizations_columns::user_organizations)
        .inner_join(
            organization_columns::organizations
                .on(organization_columns::id.eq(user_organizations_columns::organization_id)),
        )
        .filter(user_organizations_columns::organization_id.eq(org_id))
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
                "Error getting arbitrary org owner from org id in get_arbitrary_org_owner_from_org_id: {:?}",
                e
            );

            ServiceError::BadRequest(
                "Relevant organization had no owner level users".to_string(),
            )
        }
    )?;

    Ok(SlimUser::from_details(
        user_orgs_orgs.0,
        vec![user_orgs_orgs.1],
        vec![user_orgs_orgs.2],
    ))
}

#[tracing::instrument(skip(pool))]
pub async fn get_arbitrary_org_owner_from_dataset_id(
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<SlimUser, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;
    use crate::data::schema::organizations::dsl as organization_columns;
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().await.unwrap();

    let user_orgs_orgs: (User, UserOrganization, Organization) = users_columns::users
        .inner_join(user_organizations_columns::user_organizations)
        .inner_join(
            organization_columns::organizations
                .on(organization_columns::id.eq(user_organizations_columns::organization_id)),
        )
        .inner_join(
            datasets_columns::datasets
                .on(datasets_columns::organization_id.eq(organization_columns::id)),
        )
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
        user_orgs_orgs.0,
        vec![user_orgs_orgs.1],
        vec![user_orgs_orgs.2],
    ))
}
