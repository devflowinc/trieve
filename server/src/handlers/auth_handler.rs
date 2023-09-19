use std::future::{ready, Ready};

use actix_identity::Identity;
use actix_web::{
    dev::Payload, web, Error, FromRequest, HttpMessage as _, HttpRequest, HttpResponse,
};
use diesel::prelude::*;
use serde::Deserialize;

use crate::{
    data::models::{Pool, SlimUser, User},
    errors::{DefaultError, ServiceError},
    operators::{
        self,
        user_operator::{get_user_by_id_query, get_user_from_api_key_query},
    },
    AppMutexStore,
};

use crate::handlers::register_handler;

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub email: String,
    pub password: String,
}

// we need the same data
// simple aliasing makes the intentions clear and its more readable
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
                    if let Ok(user) = get_user_from_api_key_query(&authen_header.to_string(), pool)
                    {
                        return ready(Ok(user));
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

pub async fn logout(id: Identity) -> HttpResponse {
    id.logout();
    HttpResponse::NoContent().finish()
}

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

pub async fn health_check(
    mutex_store: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    let result = operators::card_operator::create_embedding("health check", mutex_store).await;

    result?;
    Ok(HttpResponse::Ok().finish())
}
