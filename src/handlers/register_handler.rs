use actix_web::{web, HttpResponse};
use argon2::{self, Config};
use diesel::prelude::*;
use once_cell::sync::Lazy;
use serde::Deserialize;

use crate::{
    data::models::{Invitation, Pool, SlimUser, User},
    errors::{DefaultError, ServiceError},
};

// UserData is used to extract data from a post request by the client
#[derive(Debug, Deserialize)]
pub struct SetPasswordData {
    pub password: String,
    pub password_confirmation: String,
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
    password_data: web::Json<SetPasswordData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let password_data_inner = password_data.into_inner();
    let password = password_data_inner.password;
    let password_confirmation = password_data_inner.password_confirmation;
    let invitation_id = invitation_id.into_inner();

    if password.len() < 8 {
        log::info!("Password too short: {}", password);
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Password must be at least 8 characters".into(),
        }));
    }
    if password != password_confirmation {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Passwords do not match".into(),
        }));
    }
    if !invitation_id.contains('-') {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Invalid Invitation".into(),
        }));
    }

    let user =
        web::block(move || insert_user_from_invitation(invitation_id, password, pool)).await?;

    match user {
        Ok(user) => Ok(HttpResponse::Ok().json(&user)),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

fn insert_user_from_invitation(
    invitation_id: String,
    password: String,
    pool: web::Data<Pool>,
) -> Result<SlimUser, DefaultError> {
    use crate::data::schema::{invitations::dsl::*, users::dsl::*};

    let mut conn = pool.get().unwrap();

    let invitation_id =
        uuid::Uuid::try_parse(&invitation_id).map_err(|_uuid_error| DefaultError {
            message: "Invalid Invitation",
        })?;

    invitations
        .filter(id.eq(invitation_id))
        .load::<Invitation>(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Invalid Invitation",
        })
        .and_then(|mut result| {
            if let Some(invitation) = result.pop() {
                // if invitation is not expired
                if invitation.expires_at > chrono::Local::now().naive_local() {
                    let password: String =
                        hash_password(&password).map_err(|_hash_error| DefaultError {
                            message: "Error Processing Password, Try Again",
                        })?;
                    dbg!(&password);

                    let user = User::from_details(invitation.email, password);
                    let inserted_user: User = diesel::insert_into(users)
                        .values(&user)
                        .get_result(&mut conn)
                        .map_err(|_db_error| DefaultError {
                            message: "Error Inserting User, Try Again",
                        })?;
                    dbg!(&inserted_user);

                    return Ok(inserted_user.into());
                }
            }
            Err(DefaultError {
                message: "Invitation Expired",
            })
        })
}
