use actix_web::{web, HttpResponse};
use argon2::{self, Config};
use diesel::prelude::*;
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::{
    data::models::{Invitation, Pool, SlimUser, User},
    errors::ServiceError,
};

// UserData is used to extract data from a post request by the client
#[derive(Debug, Deserialize)]
pub struct UserData {
    pub password: String,
}

pub static SECRET_KEY: Lazy<String> =
    Lazy::new(|| std::env::var("SECRET_KEY").unwrap_or_else(|_| "0123".repeat(16)));

const SALT: &[u8] = b"supersecuresalt";

pub fn hash_password(password: &str) -> Result<String, ServiceError> {
    let config = Config {
        secret: SECRET_KEY.as_bytes(),
        ..Default::default()
    };
    argon2::hash_encoded(password.as_bytes(), SALT, &config).map_err(|err| {
        dbg!(err);
        ServiceError::InternalServerError
    })
}

pub async fn register_user(
    invitation_id: web::Path<String>,
    user_data: web::Json<UserData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    log::info!("Registering user with invitation id: {}", invitation_id);
    let user = web::block(move || {
        query(
            invitation_id.into_inner(),
            user_data.into_inner().password,
            pool,
        )
    })
    .await??;

    Ok(HttpResponse::Ok().json(&user))
}

fn query(
    invitation_id: String,
    password: String,
    pool: web::Data<Pool>,
) -> Result<SlimUser, crate::errors::ServiceError> {
    use crate::data::schema::{invitations::dsl::*, users::dsl::*};

    let mut conn = pool.get().unwrap();

    let invitation_id = uuid::Uuid::parse_str(&invitation_id)?;

    invitations
        .filter(id.eq(invitation_id))
        .load::<Invitation>(&mut conn)
        .map_err(|_db_error| ServiceError::BadRequest("Invalid Invitation".into()))
        .and_then(|mut result| {
            if let Some(invitation) = result.pop() {
                // if invitation is not expired
                if invitation.expires_at > chrono::Local::now().naive_local() {
                    // try hashing the password, else return the error that will be converted to ServiceError
                    let password: String = hash_password(&password)?;
                    dbg!(&password);

                    let user = User::from_details(invitation.email, password);
                    let inserted_user: User = diesel::insert_into(users)
                        .values(&user)
                        .get_result(&mut conn)?;
                    dbg!(&inserted_user);

                    return Ok(inserted_user.into());
                }
            }
            Err(ServiceError::BadRequest("Invalid Invitation".into()))
        })
}
