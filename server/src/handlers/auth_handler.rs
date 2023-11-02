use crate::handlers::register_handler;
use crate::{
    data::models::{Pool, SlimUser, User},
    errors::{DefaultError, ServiceError},
    handlers::register_handler::hash_password,
    operators::{
        self,
        user_operator::{get_user_by_id_query, get_user_from_api_key_query},
    },
};
use actix_identity::Identity;
use actix_web::{
    dev::Payload, web, Error, FromRequest, HttpMessage as _, HttpRequest, HttpResponse,
};
use diesel::prelude::*;
use serde::Deserialize;
use std::future::{ready, Ready};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AuthData {
    pub email: String,
    pub password: String,
}
pub type LoggedUser = SlimUser;

impl FromRequest for LoggedUser {
    type Error = Error;
    type Future = Ready<Result<LoggedUser, Error>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        if let Ok(identity) = Identity::from_request(req, pl).into_inner() {
            if let Ok(user_json) = identity.id() {
                if let Ok(user) = serde_json::from_str(&user_json) {
                    return ready(Ok(user));
                }
            }
        }

        if let Some(authen_header) = req.headers().get("Authorization") {
            if let Ok(authen_header) = authen_header.to_str() {
                if let Some(pool) = req.app_data::<web::Data<Pool>>() {
                    if let Ok(user) = get_user_from_api_key_query(authen_header, pool) {
                        return ready(Ok(user));
                    }
                }
            }
        }

        ready(Err(ServiceError::Unauthorized.into()))
    }
}

pub struct RequireAuth {}

impl FromRequest for RequireAuth {
    type Error = Error;
    type Future = Ready<Result<RequireAuth, Error>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        let always_require_auth = std::env::var("ALWAYS_REQUIRE_AUTH").unwrap_or_default();
        if always_require_auth != "on" {
            return ready(Ok(RequireAuth {}));
        }

        if let Ok(identity) = Identity::from_request(req, pl).into_inner() {
            if let Ok(user_json) = identity.id() {
                if let Ok(_user) = serde_json::from_str::<LoggedUser>(&user_json) {
                    return ready(Ok(RequireAuth {}));
                }
            }
        }

        if let Some(authen_header) = req.headers().get("Authorization") {
            if let Ok(authen_header) = authen_header.to_str() {
                if let Some(pool) = req.app_data::<web::Data<Pool>>() {
                    if let Ok(_user) = get_user_from_api_key_query(authen_header, pool) {
                        return ready(Ok(RequireAuth {}));
                    }
                }
            }
        }

        ready(Err(ServiceError::Unauthorized.into()))
    }
}

pub fn verify(hash: &str, password: &str) -> Result<bool, ServiceError> {
    argon2::verify_encoded_ext(
        hash,
        password.as_bytes(),
        register_handler::SECRET_KEY.as_bytes(),
        &[],
    )
    .map_err(|err| {
        dbg!(err);
        ServiceError::Unauthorized
    })
}

pub async fn create_admin_account(email: String, password: String, pool: Pool) {
    // see if account exists
    let user = {
        let email = email.clone();
        let pool = pool.clone();
        let password = password.clone();
        web::block(move || find_user_match(AuthData { email, password }, web::Data::new(pool)))
            .await
            .unwrap()
    };

    if user.is_ok() {
        log::info!("Admin account already exists");
        return;
    }

    log::info!("Creating admin account for {}", email);

    let password: String = hash_password(&password).expect("Failed to hash password");

    use crate::data::schema::users::dsl as users_columns;

    let user = User::from_details(email, password);
    let mut conn = pool.get().unwrap();
    match diesel::insert_into(users_columns::users)
        .values(&user)
        .execute(&mut conn)
    {
        Ok(_) => log::info!("Admin account created"),
        Err(e) => log::error!("Failed to create admin account: {}", e),
    }
}

#[utoipa::path(
    delete,
    path = "/auth",
    context_path = "/api",
    tag = "auth",
    responses(
        (status = 204, description = "Confirmation that your current auth credentials have been cleared"),
    )
)]
pub async fn logout(id: Identity) -> HttpResponse {
    id.logout();
    HttpResponse::NoContent().finish()
}

#[utoipa::path(
    post,
    path = "/auth",
    context_path = "/api",
    tag = "auth",
    request_body(content = AuthData, description = "JSON request payload to sign in", content_type = "application/json"),
    responses(
        (status = 200, description = "Response that returns with set-cookie header", body = [SlimUser]),
        (status = 400, description = "Email or password empty or incorrect", body = [DefaultError]),
    )
)]
pub async fn login(
    req: HttpRequest,
    auth_data: web::Json<AuthData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let auth_data_inner = auth_data.into_inner();
    let email = auth_data_inner.email;
    let password = auth_data_inner.password;

    if email.is_empty() || password.is_empty() {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Email or password is empty",
        }));
    }

    let find_user_result =
        web::block(move || find_user_match(AuthData { email, password }, pool)).await?;

    match find_user_result {
        Ok(user) => {
            let user_string = serde_json::to_string(&user).unwrap();
            Identity::login(&req.extensions(), user_string).unwrap();

            Ok(HttpResponse::NoContent().finish())
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[utoipa::path(
    get,
    path = "/auth",
    context_path = "/api",
    tag = "auth",
    responses(
        (status = 200, description = "The user corresponding to your current auth credentials", body = [SlimUser]),
        (status = 400, description = "Error message indicitating you are not currently signed in", body = [DefaultError]),
    )
)]
pub async fn get_me(
    logged_user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_query_id: uuid::Uuid = logged_user.id;

    let user_result = web::block(move || get_user_by_id_query(&user_query_id, pool)).await?;

    match user_result {
        Ok(user) => Ok(HttpResponse::Ok().json(SlimUser::from(user))),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

fn find_user_match(auth_data: AuthData, pool: web::Data<Pool>) -> Result<SlimUser, DefaultError> {
    use crate::data::schema::users::dsl::{email, users};

    let mut conn = pool.get().unwrap();

    let mut items = users
        .filter(email.eq(&auth_data.email))
        .load::<User>(&mut conn)
        .unwrap();

    if let Some(user) = items.pop() {
        if let Ok(matching) = verify(&user.hash, &auth_data.password) {
            if matching {
                return Ok(user.into());
            }
        }
    }
    Err(DefaultError {
        message: "Incorrect email or password",
    })
}

#[utoipa::path(
    get,
    path = "/health",
    context_path = "/api",
    tag = "health",
    responses(
        (status = 200, description = "Confirmation that the service is healthy and can make embedding vectors"),
        (status = 400, description = "Service error relating to making an embedding or overall service health", body = [DefaultError]),
    ),
)]
pub async fn health_check() -> Result<HttpResponse, actix_web::Error> {
    let result = operators::card_operator::create_embedding("health check").await;

    result?;
    Ok(HttpResponse::Ok().finish())
}
