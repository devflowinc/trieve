use crate::data::models::{
    ApiKeyDTO, ApiKeyRole, Organization, SlimUser, UserApiKey, UserOrganization, UserRole,
};
use crate::diesel::prelude::*;
use crate::errors::ServiceError;
use crate::handlers::auth_handler::LoggedUser;
use crate::operators::stripe_operator::refresh_redis_org_plan_sub;
use crate::{
    data::models::{Pool, User},
    errors::DefaultError,
};
use actix_identity::Identity;
use actix_web::{web, HttpMessage, HttpRequest};
use argon2::Config;
use once_cell::sync::Lazy;
use rand::distributions::Alphanumeric;
use rand::Rng;

pub fn get_user_by_username_query(
    user_name: &String,
    pool: web::Data<Pool>,
) -> Result<User, DefaultError> {
    use crate::data::schema::users::dsl::*;

    let mut conn = pool.get().unwrap();

    let user: Option<User> = users
        .filter(username.eq(user_name))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| DefaultError {
            message: "Error loading user",
        })?;
    match user {
        Some(user) => Ok(user),
        None => Err(DefaultError {
            message: "User not found",
        }),
    }
}

pub fn get_user_by_id_query(
    user_id: &uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(User, Vec<UserOrganization>, Vec<Organization>), DefaultError> {
    use crate::data::schema::organizations::dsl as organization_columns;
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().unwrap();

    let user_orgs_orgs: Vec<(User, UserOrganization, Organization)> = users_columns::users
        .inner_join(user_organizations_columns::user_organizations)
        .inner_join(
            organization_columns::organizations
                .on(organization_columns::id.eq(user_organizations_columns::organization_id)),
        )
        .filter(users_columns::id.eq(user_id))
        .select((
            User::as_select(),
            UserOrganization::as_select(),
            Organization::as_select(),
        ))
        .load::<(User, UserOrganization, Organization)>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error loading user with organizations and roles for get_user_by_id_query",
        })?;

    match user_orgs_orgs.first() {
        Some(first_user_org) => {
            let user = first_user_org.0.clone();
            let user_orgs = user_orgs_orgs
                .iter()
                .map(|user_org_org: &(User, UserOrganization, Organization)| user_org_org.1.clone())
                .collect::<Vec<UserOrganization>>();
            let orgs = user_orgs_orgs
                .iter()
                .map(|user_org_org| user_org_org.2.clone())
                .collect::<Vec<Organization>>();
            Ok((user, user_orgs, orgs))
        }
        None => {
            let user = users_columns::users
                .filter(users_columns::id.eq(user_id))
                .select(User::as_select())
                .first::<User>(&mut conn)
                .map_err(|_| DefaultError {
                    message: "Error loading user by itself for get_user_by_id_query",
                })?;
            
            Ok((user, vec![], vec![]))
        }
    }
}

pub async fn add_existing_user_to_org(
    email: String,
    organization_id: uuid::Uuid,
    user_role: UserRole,
    pool: web::Data<Pool>,
) -> Result<bool, DefaultError> {
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().unwrap();

    let user: Vec<User> = users_columns::users
        .filter(users_columns::email.eq(email))
        .select(User::as_select())
        .load::<User>(&mut conn)
        .map_err(|e| {
            log::error!("Error loading users: {:?}", e);
            DefaultError {
                message: "Error loading users",
            }
        })?;

    match user.first() {
        Some(user) => Ok(add_user_to_organization(
            None,
            None,
            UserOrganization::from_details(user.id, organization_id, user_role),
            pool,
        )
        .await
        .is_ok()),
        None => Ok(false),
    }
}

pub fn update_user_query(
    user: &LoggedUser,
    username: &Option<String>,
    name: &Option<String>,
    website: &Option<String>,
    role: Option<UserRole>,
    visible_email: bool,
    pool: web::Data<Pool>,
) -> Result<User, DefaultError> {
    use crate::data::schema::users::dsl as user_columns;

    let mut conn = pool.get().unwrap();

    if username.clone().is_some()
        && username.clone().unwrap() != user.username.clone().unwrap_or("".to_string())
    {
        let user_by_username = get_user_by_username_query(&username.clone().unwrap(), pool);

        if let Ok(old_user) = user_by_username {
            if !(old_user.username.is_some()
                && old_user.username.unwrap() == username.clone().unwrap())
            {
                return Err(DefaultError {
                    message: "That username is already taken",
                });
            }
        }
    }

    let user: User = diesel::update(user_columns::users.filter(user_columns::id.eq(user.id)))
        .set((
            user_columns::username.eq(username),
            user_columns::website.eq(website),
            user_columns::visible_email.eq(&visible_email),
            user_columns::name.eq(name),
        ))
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error updating user",
        })?;
    if let Some(role) = role {
        diesel::update(
            crate::data::schema::user_organizations::dsl::user_organizations
                .filter(crate::data::schema::user_organizations::dsl::user_id.eq(user.id)),
        )
        .set(crate::data::schema::user_organizations::dsl::role.eq(Into::<i32>::into(role)))
        .execute(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error updating user",
        })?;
    }

    Ok(user)
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

pub static SECRET_KEY: Lazy<String> =
    Lazy::new(|| std::env::var("SECRET_KEY").unwrap_or_else(|_| "0123".repeat(16)));

pub static SALT: Lazy<String> =
    Lazy::new(|| std::env::var("SALT").unwrap_or_else(|_| "supersecuresalt".to_string()));

pub fn hash_password(password: &str) -> Result<String, DefaultError> {
    let config = Config {
        secret: SECRET_KEY.as_bytes(),
        ..Config::original()
    };
    argon2::hash_encoded(password.as_bytes(), SALT.as_bytes(), &config).map_err(|_err| {
        DefaultError {
            message: "Error processing password, try again",
        }
    })
}

pub fn set_user_api_key_query(
    user_id: uuid::Uuid,
    name: String,
    role: ApiKeyRole,
    pool: web::Data<Pool>,
) -> Result<String, DefaultError> {
    let raw_api_key = generate_api_key();
    let hashed_api_key = hash_password(&raw_api_key)?;

    let mut conn = pool.get().unwrap();

    let api_key_struct = UserApiKey::from_details(user_id, hashed_api_key.clone(), name, role);

    diesel::insert_into(crate::data::schema::user_api_key::dsl::user_api_key)
        .values(&api_key_struct)
        .execute(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error setting api key",
        })?;

    Ok(raw_api_key)
}

pub fn get_user_from_api_key_query(
    api_key: &str,
    pool: &web::Data<Pool>,
) -> Result<SlimUser, DefaultError> {
    use crate::data::schema::organizations::dsl as organization_columns;
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let api_key_hash = hash_password(api_key)?;

    let mut conn = pool.get().unwrap();

    let user_orgs_orgs: Vec<(User, UserOrganization, Organization, UserApiKey)> =
        users_columns::users
            .inner_join(user_organizations_columns::user_organizations)
            .inner_join(
                organization_columns::organizations
                    .on(organization_columns::id.eq(user_organizations_columns::organization_id)),
            )
            .inner_join(crate::data::schema::user_api_key::dsl::user_api_key)
            .filter(crate::data::schema::user_api_key::dsl::api_key_hash.eq(api_key_hash))
            .select((
                User::as_select(),
                UserOrganization::as_select(),
                Organization::as_select(),
                UserApiKey::as_select(),
            ))
            .load::<(User, UserOrganization, Organization, UserApiKey)>(&mut conn)
            .map_err(|_| DefaultError {
                message: "Error loading user",
            })?;

    match user_orgs_orgs.first() {
        Some(first_user_org) => {
            let user = first_user_org.0.clone();
            let mut user_orgs = user_orgs_orgs
                .iter()
                .map(|user_org| user_org.1.clone())
                .collect::<Vec<UserOrganization>>();

            //TODO: change this so that it is not above user current role
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
            Ok(SlimUser::from_details(user, user_orgs, orgs))
        }
        None => Err(DefaultError {
            message: "User not found",
        }),
    }
}

pub fn get_user_api_keys_query(
    user_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<ApiKeyDTO>, DefaultError> {
    use crate::data::schema::user_api_key::dsl as user_api_key_columns;

    let mut conn = pool.get().unwrap();

    let api_keys = user_api_key_columns::user_api_key
        .filter(user_api_key_columns::user_id.eq(user_id))
        .select(UserApiKey::as_select())
        .load::<UserApiKey>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error loading user api keys",
        })?;

    let api_keys = api_keys
        .into_iter()
        .map(|api_key| api_key.into())
        .collect::<Vec<ApiKeyDTO>>();
    Ok(api_keys)
}

pub fn delete_user_api_keys_query(
    user_id: uuid::Uuid,
    api_key_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::user_api_key::dsl as user_api_key_columns;

    let mut conn = pool.get().unwrap();

    diesel::delete(
        user_api_key_columns::user_api_key
            .filter(user_api_key_columns::user_id.eq(user_id))
            .filter(user_api_key_columns::id.eq(api_key_id)),
    )
    .execute(&mut conn)
    .map_err(|_| DefaultError {
        message: "Error deleting user api key",
    })?;

    Ok(())
}

pub fn create_user_query(
    user_id: uuid::Uuid,
    email: String,
    name: Option<String>,
    role: UserRole,
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(User, Vec<UserOrganization>, Vec<Organization>), DefaultError> {
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().unwrap();

    let old_user: Option<User> = users_columns::users
        .select(User::as_select())
        .filter(users_columns::email.eq(&email))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| DefaultError {
            message: "Error loading user",
        })?;

    if let Some(old_user) = old_user {
        let mut conn = pool.get().unwrap();
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            //users
            diesel::update(users_columns::users.filter(users_columns::id.eq(old_user.id)))
                .set(users_columns::id.eq(user_id))
                .execute(conn)?;

            Ok(())
        })
        .map_err(|e| {
            log::error!("Error updating ids: {:?}", e);

            DefaultError {
                message: "Error creating user",
            }
        })?;

        let user = get_user_by_id_query(&user_id, pool)?;

        return Ok(user);
    }

    let user = User::from_details_with_id(user_id, email, name);
    let user_org = UserOrganization::from_details(user_id, org_id, role);

    let user_org = conn
        .transaction::<_, diesel::result::Error, _>(|conn| {
            let user = diesel::insert_into(users_columns::users)
                .values(&user)
                .get_result::<User>(conn)?;

            let user_org = diesel::insert_into(user_organizations_columns::user_organizations)
                .values(&user_org)
                .get_result::<UserOrganization>(conn)?;

            Ok((user, vec![user_org]))
        })
        .map_err(|_| DefaultError {
            message: "Failed to create user, likely that organization_id is invalid",
        })?;

    let user_org = get_user_by_id_query(&user_org.0.id, pool)?;

    Ok(user_org)
}

pub async fn add_user_to_organization(
    req: Option<&HttpRequest>,
    calling_user_id: Option<uuid::Uuid>,
    user_org: UserOrganization,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;

    let org_id_refresh = user_org.organization_id.clone();
    let user_id_refresh = user_org.user_id.clone();
    let user_id_refresh_pool = pool.clone();

    let mut conn = pool.get().unwrap();

    diesel::insert_into(user_organizations_columns::user_organizations)
        .values(user_org)
        .execute(&mut conn)
        .map_err(|_| {
            ServiceError::InternalServerError("Failed to add user to organization".to_string())
        })?;

    refresh_redis_org_plan_sub(org_id_refresh, pool)
        .await
        .map_err(|e| {
            log::error!("Error refreshing redis org plan sub: {:?}", e);
            ServiceError::InternalServerError("Failed to refresh redis org plan sub".to_string())
        })?;

    if req.is_some() && calling_user_id.is_some_and(|id| id == user_id_refresh) {
        let user = get_user_by_id_query(&user_id_refresh, user_id_refresh_pool).map_err(|e| {
            log::error!("Error getting user by id: {:?}", e);
            ServiceError::InternalServerError("Failed to get user by id".to_string())
        })?;

        let slim_user: SlimUser = SlimUser::from_details(user.0, user.1, user.2);

        let user_string = serde_json::to_string(&slim_user).map_err(|_| {
            ServiceError::InternalServerError("Failed to serialize user to JSON".into())
        })?;

        Identity::login(
            &req.expect("Cannot be none at this point").extensions(),
            user_string,
        )
        .expect("Failed to refresh login for user");
    }

    Ok(())
}

pub fn create_default_user(api_key: &str, pool: web::Data<Pool>) -> Result<(), DefaultError> {
    use crate::data::schema::organizations::dsl as organization_columns;
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let api_key_hash = hash_password(api_key)?;

    let mut conn = pool.get().unwrap();

    let user = User::from_details_with_id(
        uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
        "default".to_string(),
        None,
    );

    let user = diesel::insert_into(users_columns::users)
        .values(&user)
        .get_result::<User>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to create default user",
        })?;

    let org = Organization::from_details("default".to_string());

    let org = diesel::insert_into(organization_columns::organizations)
        .values(&org)
        .get_result::<Organization>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to create default organization",
        })?;

    let user_org = UserOrganization::from_details(user.id, org.id, UserRole::Owner);

    diesel::insert_into(user_organizations_columns::user_organizations)
        .values(&user_org)
        .execute(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to create default user organization",
        })?;

    let api_key_struct = UserApiKey::from_details(
        user.id,
        api_key_hash,
        "default".to_string(),
        ApiKeyRole::ReadAndWrite,
    );

    diesel::insert_into(crate::data::schema::user_api_key::dsl::user_api_key)
        .values(&api_key_struct)
        .execute(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error setting api key",
        })?;

    Ok(())
}
