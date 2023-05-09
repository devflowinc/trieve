use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use serde::Deserialize;
use serde_json::to_string;

use crate::{
    data::{
        models::{Invitation, Pool},
        validators::email_regex,
    },
    errors::DefaultError,
    operators::email_operator::send_invitation,
};

#[derive(Deserialize)]
pub struct InvitationData {
    pub email: String,
    pub referral_tokens: Vec<String>,
}

pub async fn post_invitation(
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

    let stringified_referral_tokens = to_string(&invitation_referral_tokens).unwrap();
    let create_invitation_result =
        web::block(move || create_invitation(email, stringified_referral_tokens, pool)).await?;

    match create_invitation_result {
        Ok(()) => Ok(HttpResponse::Ok().finish()),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

fn create_invitation(
    email: String,
    invitation_referral_tokens: String,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    let invitation = create_invitation_query(email, invitation_referral_tokens, pool)?;
    send_invitation(&invitation)
}

/// Diesel query
fn create_invitation_query(
    email: String,
    invitation_referral_tokens: String,
    pool: web::Data<Pool>,
) -> Result<Invitation, DefaultError> {
    use crate::data::schema::invitations::dsl::invitations;

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
