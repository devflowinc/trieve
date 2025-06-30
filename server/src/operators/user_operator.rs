use crate::data::models::{
    ApiKeyRespBody, ApiKeyRole, Organization, RedisPool, SlimUser, UserApiKey, UserOrganization,
    UserRole,
};
use crate::operators::organization_operator::hash_function;
use crate::{
    data::models::{Pool, User},
    errors::ServiceError,
};
use actix_web::{web, HttpRequest};
use diesel::prelude::*;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use redis::AsyncCommands;

#[tracing::instrument(skip_all)]
pub async fn get_user_by_id_query(
    user_id: &uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(User, Vec<UserOrganization>, Vec<Organization>), ServiceError> {
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
        .filter(users_columns::id.eq(user_id))
        .filter(organization_columns::deleted.eq(0))
        .select((
            User::as_select(),
            UserOrganization::as_select(),
            Organization::as_select(),
        ))
        .load::<(User, UserOrganization, Organization)>(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::BadRequest(
                "Error loading user with organizations and roles for get_user_by_id_query"
                    .to_string(),
            )
        })?;

    match user_orgs_orgs.get(0) {
        Some(first_user_org) => {
            let user = first_user_org.0.clone();
            let user_orgs = user_orgs_orgs
                .iter()
                .map(|user_org_org: &(User, UserOrganization, Organization)| user_org_org.1.clone())
                .collect::<Vec<UserOrganization>>();
            let orgs = user_orgs_orgs
                .iter()
                .map(|user_org_org| user_org_org.2.with_complete_partner_config().clone())
                .collect::<Vec<Organization>>();
            Ok((user, user_orgs, orgs))
        }
        None => {
            let user = users_columns::users
                .filter(users_columns::id.eq(user_id))
                .select(User::as_select())
                .first::<User>(&mut conn)
                .await
                .map_err(|_| {
                    ServiceError::BadRequest(
                        "Error loading user by itself for get_user_by_id_query".to_string(),
                    )
                })?;

            Ok((user, vec![], vec![]))
        }
    }
}

#[tracing::instrument(skip_all)]
pub async fn get_user_by_oidc_subject_query(
    oidc_subject: &str,
    pool: web::Data<Pool>,
) -> Result<(User, Vec<UserOrganization>, Vec<Organization>), ServiceError> {
    use crate::data::schema::organizations::dsl as organization_columns;
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_owned())
    })?;

    let user = users_columns::users
        .filter(users_columns::oidc_subject.eq(oidc_subject))
        .select(User::as_select())
        .first::<User>(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::BadRequest(
                "Error loading user by itself for get_user_by_oidc_subject_query".to_owned(),
            )
        })?;

    let orgs = user_organizations_columns::user_organizations
        .inner_join(organization_columns::organizations)
        .filter(user_organizations_columns::user_id.eq(user.id))
        .filter(organization_columns::deleted.eq(0))
        .select((UserOrganization::as_select(), Organization::as_select()))
        .load::<(UserOrganization, Organization)>(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::BadRequest(
                "Error loading user organizations for get_user_by_oidc_subject_query".to_owned(),
            )
        })?;

    let (user_orgs, orgs) = orgs.into_iter().unzip();

    Ok((user, user_orgs, orgs))
}

#[tracing::instrument(skip_all)]
pub async fn get_option_user_by_email_query(
    email: &str,
    pool: web::Data<Pool>,
) -> Result<Option<User>, ServiceError> {
    use crate::data::schema::users::dsl as users_columns;
    let mut conn = pool.get().await.unwrap();

    users_columns::users
        .filter(users_columns::email.eq(email.to_lowercase()))
        .select(User::as_select())
        .first::<User>(&mut conn)
        .await
        .optional()
        .map_err(|_| ServiceError::BadRequest("Error checking user by id".to_string()))
}

#[tracing::instrument(skip_all)]
pub async fn add_existing_user_to_org(
    email: String,
    organization_id: uuid::Uuid,
    user_role: UserRole,
    scopes: Option<Vec<String>>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<bool, ServiceError> {
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let user: Vec<User> = users_columns::users
        .filter(users_columns::email.eq(email))
        .select(User::as_select())
        .load::<User>(&mut conn)
        .await
        .map_err(|e| {
            log::error!("Error loading users: {:?}", e);
            ServiceError::BadRequest("Error loading users".to_string())
        })?;

    match user.get(0) {
        Some(user) => Ok(add_user_to_organization(
            None,
            None,
            UserOrganization::from_details(user.id, organization_id, user_role, scopes),
            pool,
            redis_pool,
        )
        .await
        .is_ok()),
        None => Ok(false),
    }
}

#[tracing::instrument(skip_all)]
pub async fn update_user_query(
    user_id: uuid::Uuid,
    organization_id: uuid::Uuid,
    role: Option<UserRole>,
    scopes: Option<Vec<String>>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    match (role, scopes) {
        (Some(role_val), Some(scopes_val)) => {
            diesel::update(
                user_organizations_columns::user_organizations
                    .filter(user_organizations_columns::user_id.eq(user_id))
                    .filter(user_organizations_columns::organization_id.eq(organization_id)),
            )
            .set((
                user_organizations_columns::role.eq(Into::<i32>::into(role_val)),
                user_organizations_columns::scopes.eq(scopes_val),
            ))
            .execute(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Error updating user".to_string()))?;
        }
        (Some(role_val), None) => {
            diesel::update(
                user_organizations_columns::user_organizations
                    .filter(user_organizations_columns::user_id.eq(user_id))
                    .filter(user_organizations_columns::organization_id.eq(organization_id)),
            )
            .set(user_organizations_columns::role.eq(Into::<i32>::into(role_val)))
            .execute(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Error updating user".to_string()))?;
        }
        (None, Some(scopes_val)) => {
            diesel::update(
                user_organizations_columns::user_organizations
                    .filter(user_organizations_columns::user_id.eq(user_id))
                    .filter(user_organizations_columns::organization_id.eq(organization_id)),
            )
            .set(user_organizations_columns::scopes.eq(scopes_val))
            .execute(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Error updating user".to_string()))?;
        }
        (None, None) => {
            // Nothing to update
            return Ok(());
        }
    }

    let mut redis_conn = redis_pool.get().await.map_err(|_| {
        ServiceError::InternalServerError("Failed to get redis connection".to_string())
    })?;

    redis_conn
        .del::<_, ()>(user_id.to_string())
        .await
        .map_err(|_| {
            ServiceError::InternalServerError("Failed to delete user from redis".to_string())
        })?;

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn associate_user_to_oidc_subject_query(
    user_id: uuid::Uuid,
    oidc_subject: String,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    diesel::update(users_columns::users.filter(users_columns::id.eq(user_id)))
        .set(users_columns::oidc_subject.eq(oidc_subject))
        .execute(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Error updating user".to_string()))?;

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn create_user_query(
    user_oidc_subject: Option<String>,
    email: String,
    name: Option<String>,
    role: UserRole,
    scopes: Option<Vec<String>>,
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(User, Vec<UserOrganization>, Vec<Organization>), ServiceError> {
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let user = User::from_details_with_oidc_subject(user_oidc_subject, email, name);
    let user_org = UserOrganization::from_details(user.id, org_id, role, scopes);

    let user_org = conn
        .transaction::<_, diesel::result::Error, _>(|conn| {
            async move {
                let user = diesel::insert_into(users_columns::users)
                    .values(&user)
                    .get_result::<User>(conn)
                    .await?;

                let user_org = diesel::insert_into(user_organizations_columns::user_organizations)
                    .values(&user_org)
                    .get_result::<UserOrganization>(conn)
                    .await?;

                Ok((user, vec![user_org]))
            }
            .scope_boxed()
        })
        .await
        .map_err(|err| {
            ServiceError::InternalServerError(format!("Failed to create user {:?}", err))
        })?;

    let user_org = get_user_by_id_query(&user_org.0.id, pool).await?;

    Ok(user_org)
}

#[tracing::instrument(skip_all)]
pub async fn get_user_from_api_key_query(
    api_key: &str,
    pool: web::Data<Pool>,
) -> Result<(SlimUser, UserApiKey), ServiceError> {
    use crate::data::schema::organizations::dsl as organization_columns;
    use crate::data::schema::user_api_key::dsl as user_api_key_columns;
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let api_key_hash = hash_function(api_key);

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let user_orgs_orgs: Vec<(User, UserOrganization, Organization, UserApiKey)> =
        users_columns::users
            .inner_join(user_organizations_columns::user_organizations)
            .inner_join(
                organization_columns::organizations
                    .on(organization_columns::id.eq(user_organizations_columns::organization_id)),
            )
            .inner_join(user_api_key_columns::user_api_key)
            .filter(user_api_key_columns::blake3_hash.eq(api_key_hash.clone()))
            .filter(
                user_api_key_columns::expires_at
                    .is_null()
                    .or(user_api_key_columns::expires_at.ge(diesel::dsl::now.nullable())),
            )
            .filter(organization_columns::deleted.eq(0))
            .select((
                User::as_select(),
                UserOrganization::as_select(),
                Organization::as_select(),
                UserApiKey::as_select(),
            ))
            .load::<(User, UserOrganization, Organization, UserApiKey)>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Error loading user".to_string()))?;

    match user_orgs_orgs.get(0) {
        Some(first_user_org) => {
            let user = first_user_org.0.clone();
            let mut user_orgs = user_orgs_orgs
                .iter()
                .map(|user_org| user_org.1.clone())
                .collect::<Vec<UserOrganization>>();

            user_orgs.iter_mut().for_each(|user_org| {
                if user_orgs_orgs
                    .iter()
                    .find(|user_org_org| user_org_org.1.id == user_org.id)
                    .unwrap()
                    .3
                    .role
                    == 0
                {
                    user_org.role = 0;
                }
            });

            let orgs = user_orgs_orgs
                .iter()
                .map(|user_org_org| user_org_org.2.clone())
                .collect::<Vec<Organization>>();

            Ok((
                SlimUser::from_details(user, user_orgs, orgs),
                first_user_org.3.clone(),
            ))
        }
        None => Err(ServiceError::BadRequest("API Key Incorrect".to_string())),
    }
}

#[tracing::instrument(skip_all)]
pub async fn add_user_to_organization(
    _req: Option<&HttpRequest>,
    _calling_user_id: Option<uuid::Uuid>,
    user_org: UserOrganization,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;

    let user_id_refresh = user_org.user_id;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    diesel::insert_into(user_organizations_columns::user_organizations)
        .values(user_org)
        .execute(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::InternalServerError("Failed to add user to organization".to_string())
        })?;

    let mut redis_conn = redis_pool.get().await.map_err(|_| {
        ServiceError::InternalServerError("Failed to get redis connection".to_string())
    })?;

    redis_conn
        .del::<_, ()>(user_id_refresh.to_string())
        .await
        .map_err(|_| {
            ServiceError::InternalServerError("Failed to delete user from redis".to_string())
        })?;

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn create_default_user(api_key: &str, pool: web::Data<Pool>) -> Result<(), ServiceError> {
    use crate::data::schema::organizations::dsl as organization_columns;
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let api_key_hash = hash_function(api_key);

    let mut conn = pool.get_ref().get().await.unwrap();

    let user = User::from_details_with_oidc_subject(
        uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").ok(),
        "default".to_string(),
        None,
    );

    let option_user = match diesel::insert_into(users_columns::users)
        .values(&user)
        .get_result::<User>(&mut conn)
        .await
    {
        Ok(user) => Some(user),
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                log::info!("Skipped creating default user as it already exists");
            }

            None
        }
    };

    if option_user.is_none() {
        return Ok(());
    }

    let user = option_user.expect("User must be present");

    let org = Organization::from_details("default".to_string());

    let org = diesel::insert_into(organization_columns::organizations)
        .values(&org)
        .get_result::<Organization>(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::BadRequest("Failed to create default organization".to_string())
        })?;

    let user_org = UserOrganization::from_details(user.id, org.id, UserRole::Owner, None);

    diesel::insert_into(user_organizations_columns::user_organizations)
        .values(&user_org)
        .execute(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::BadRequest("Failed to create default user organization".to_string())
        })?;

    let api_key_struct = UserApiKey::from_details(
        user.id,
        api_key_hash,
        "default".to_string(),
        ApiKeyRole::Admin,
        None,
        None,
        None,
        None,
        None,
    );

    diesel::insert_into(crate::data::schema::user_api_key::dsl::user_api_key)
        .values(&api_key_struct)
        .execute(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Error setting api key".to_string()))?;

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn remove_user_from_org_query(
    user_id: uuid::Uuid,
    max_allowed_role: UserRole,
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let user_role_being_removed: i32 = user_organizations_columns::user_organizations
        .filter(user_organizations_columns::user_id.eq(user_id))
        .filter(user_organizations_columns::organization_id.eq(organization_id))
        .select(user_organizations_columns::role)
        .first::<i32>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Error loading user role".to_string()))?;

    if user_role_being_removed > max_allowed_role.into() {
        return Err(ServiceError::Forbidden);
    }

    diesel::delete(
        user_organizations_columns::user_organizations
            .filter(user_organizations_columns::user_id.eq(user_id))
            .filter(user_organizations_columns::organization_id.eq(organization_id)),
    )
    .execute(&mut conn)
    .await
    .map_err(|_| ServiceError::BadRequest("Error removing user from org".to_string()))?;

    let mut redis_conn = redis_pool.get().await.map_err(|_| {
        ServiceError::InternalServerError("Failed to get redis connection".to_string())
    })?;

    redis_conn
        .del::<_, ()>(user_id.to_string())
        .await
        .map_err(|_| {
            ServiceError::InternalServerError("Failed to delete user from redis".to_string())
        })?;

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn get_all_users_query(pool: web::Data<Pool>) -> Result<Vec<SlimUser>, ServiceError> {
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
        .select((
            User::as_select(),
            UserOrganization::as_select(),
            Organization::as_select(),
        ))
        .load::<(User, UserOrganization, Organization)>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Error loading user".to_string()))?;

    let mut slim_users: Vec<SlimUser> = Vec::new();
    let mut current_user: Option<User> = None;
    let mut user_orgs: Vec<UserOrganization> = Vec::new();
    let mut orgs: Vec<Organization> = Vec::new();

    for (user, user_org, org) in user_orgs_orgs {
        if let Some(current_user) = current_user {
            if current_user.id != user.id {
                slim_users.push(SlimUser::from_details(
                    current_user.clone(),
                    user_orgs.clone(),
                    orgs.clone(),
                ));
                user_orgs.clear();
                orgs.clear();
            }
        }
        current_user = Some(user);
        user_orgs.push(user_org);
        orgs.push(org);
    }

    if let Some(user) = current_user {
        slim_users.push(SlimUser::from_details(
            user.clone(),
            user_orgs.clone(),
            orgs.clone(),
        ));
    } else {
        return Err(ServiceError::BadRequest("No users found".to_string()));
    }

    let mut unique_slim_users: Vec<SlimUser> = Vec::new();
    for slim_user in slim_users {
        if let Some(existing_user) = unique_slim_users.iter_mut().find(|u| u.id == slim_user.id) {
            // Merge user_orgs
            existing_user.user_orgs.extend(slim_user.user_orgs);
            existing_user.user_orgs.sort_by_key(|uo| uo.id);
            existing_user.user_orgs.dedup_by(|a, b| a.id == b.id);

            // Merge orgs
            existing_user.orgs.extend(slim_user.orgs);
            existing_user.orgs.sort_by_key(|o| o.id);
            existing_user.orgs.dedup_by(|a, b| a.id == b.id);
        } else {
            unique_slim_users.push(slim_user);
        }
    }

    Ok(unique_slim_users)
}

#[tracing::instrument(skip_all)]
pub async fn get_user_api_keys_query(
    user_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<ApiKeyRespBody>, ServiceError> {
    use crate::data::schema::user_api_key::dsl as user_api_key_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let api_keys = user_api_key_columns::user_api_key
        .filter(user_api_key_columns::user_id.eq(user_id))
        .select(UserApiKey::as_select())
        .load::<UserApiKey>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Error loading user api keys".to_string()))?;

    let api_keys = api_keys
        .into_iter()
        .map(|api_key| api_key.into())
        .collect::<Vec<ApiKeyRespBody>>();
    Ok(api_keys)
}

#[tracing::instrument(skip_all)]
pub async fn delete_user_api_keys_query(
    user_id: uuid::Uuid,
    api_key_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::user_api_key::dsl as user_api_key_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    diesel::delete(
        user_api_key_columns::user_api_key
            .filter(user_api_key_columns::user_id.eq(user_id))
            .filter(user_api_key_columns::id.eq(api_key_id)),
    )
    .execute(&mut conn)
    .await
    .map_err(|_| ServiceError::BadRequest("Error deleting user api key".to_string()))?;

    Ok(())
}
