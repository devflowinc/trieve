use actix_web::{web, HttpRequest, HttpResponse};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use crate::{
    data::{
        models::{Invitation, Pool},
        validators::email_regex,
    },
    errors::{DefaultError, ServiceError},
    operators::user_operator::get_user_by_email_query,
};

#[derive(Deserialize, Serialize)]
pub struct InvitationResponse {
    pub registration_url: String,
}

#[derive(Deserialize)]
pub struct InvitationData {
    pub email: String,
    pub referral_tokens: Vec<String>,
}

pub async fn post_invitation(
    request: HttpRequest,
    invitation_data: web::Json<InvitationData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let invitation_data = invitation_data.into_inner();
    let email = invitation_data.email;
    let invitation_referral_tokens = invitation_data.referral_tokens;
    if !email_regex().is_match(&email) {
        return Ok(
            HttpResponse::BadRequest().json(crate::errors::DefaultError {
                message: "Invalid email",
            }),
        );
    }

    // get the host from the request
    let host_name = request
        .headers()
        .get("Origin")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let stringified_referral_tokens = to_string(&invitation_referral_tokens).unwrap();
    let registration_url =
        web::block(move || create_invitation(host_name, email, stringified_referral_tokens, pool))
            .await?
            .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    Ok(HttpResponse::Ok().json(InvitationResponse { registration_url }))
}

pub fn create_invitation(
    app_url: String,
    email: String,
    invitation_referral_tokens: String,
    pool: web::Data<Pool>,
) -> Result<String, DefaultError> {
    let invitation = create_invitation_query(email, invitation_referral_tokens, pool)?;
    // send_invitation(app_url, &invitation)

    Ok(format!(
        "{}/auth/register/{}?email={}",
        app_url, invitation.id, invitation.email
    )
    .to_string())
}

/// Diesel query
fn create_invitation_query(
    email: String,
    invitation_referral_tokens: String,
    pool: web::Data<Pool>,
) -> Result<Invitation, DefaultError> {
    use crate::data::schema::invitations::dsl::invitations;

    let user_exists = get_user_by_email_query(&email, &pool).is_ok();
    if user_exists {
        return Err(DefaultError {
            message: "An account with this email already exists.",
        });
    }

    let mut conn = pool.get().unwrap();

    let mut new_invitation = Invitation::from(email);
    new_invitation.referral_tokens = Some(invitation_referral_tokens);

    let inserted_invitation = diesel::insert_into(invitations)
        .values(&new_invitation)
        .get_result(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error inserting invitation.",
        })?;

    Ok(inserted_invitation)
}
