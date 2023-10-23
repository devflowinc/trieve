use crate::{
    data::models::{Invitation, Pool, SlimUser, User},
    errors::DefaultError,
};
use actix_web::{web, HttpResponse};
use argon2::{self, Config};
use diesel::prelude::*;
use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct SetPasswordData {
    pub password: String,
    pub password_confirmation: String,
}

pub static SECRET_KEY: Lazy<String> =
    Lazy::new(|| std::env::var("SECRET_KEY").unwrap_or_else(|_| "0123".repeat(16)));

pub static SALT: Lazy<String> =
    Lazy::new(|| std::env::var("SALT").unwrap_or_else(|_| "supersecuresalt".to_string()));

pub fn hash_password(password: &str) -> Result<String, DefaultError> {
    let config = Config {
        secret: SECRET_KEY.as_bytes(),
        ..Default::default()
    };
    argon2::hash_encoded(password.as_bytes(), SALT.as_bytes(), &config).map_err(|_err| {
        DefaultError {
            message: "Error processing password, try again",
        }
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
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Password must be at least 8 characters",
        }));
    }
    if password != password_confirmation {
        return Ok(HttpResponse::BadRequest().json(DefaultError {
            message: "Passwords do not match",
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
        .filter(crate::data::schema::invitations::columns::id.eq(invitation_id))
        .load::<Invitation>(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Invalid Invitation",
        })
        .and_then(|mut result| {
            let invitation = match result.pop() {
                Some(it) => it,
                None => {
                    return Err(DefaultError {
                        message: "Invalid Invitation",
                    })
                }
            };

            if invitation.expires_at <= chrono::Utc::now().naive_local() {
                return Err(DefaultError {
                    message: "Invitation Expired",
                });
            };

            let password: String =
                hash_password(&password).map_err(|_hash_error| DefaultError {
                    message: "Error Processing Password, Try Again",
                })?;

            let user = User::from_details(invitation.email, password);
            let inserted_user: User = diesel::insert_into(users)
                .values(&user)
                .get_result(&mut conn)
                .map_err(|_db_error| DefaultError {
                    message: "Error Inserting User, Try Again",
                })?;

            Ok(inserted_user.into())
        })
}

pub fn generate_api_key() -> String {
    let rng = rand::thread_rng();
    let api_key: String = format!(
        "af-{}",
        rng.sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>()
    );

    api_key
}
