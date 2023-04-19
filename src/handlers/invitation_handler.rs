use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use serde::Deserialize;

use crate::{
    data::{
        models::{Invitation, Pool},
        validators::email_regex,
    },
    operators::email_operator::send_invitation, errors::DefaultError,
};

#[derive(Deserialize)]
pub struct InvitationData {
    pub email: String,
}

pub async fn post_invitation(
    invitation_data: web::Json<InvitationData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let email = invitation_data.into_inner().email;
    if !email_regex().is_match(&email) {
        return Ok(
            HttpResponse::BadRequest().json(crate::errors::DefaultError {
                message: "Invalid email".into(),
            }),
        );
    }

    let create_invitation_result = web::block(move || create_invitation(email, pool)).await?;

    match create_invitation_result {
        Ok(()) => {
            Ok(HttpResponse::Ok().finish())
        }
        Err(e) => {
            Ok(HttpResponse::BadRequest().json(e))
        }
    }
}

fn create_invitation(
    email: String,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    let invitation = create_invitation_query(email, pool)?;
    send_invitation(&invitation)
}

/// Diesel query
fn create_invitation_query(
    email: String,
    pool: web::Data<Pool>,
) -> Result<Invitation, DefaultError> {
    use crate::data::schema::invitations::dsl::invitations;

    let mut conn = pool.get().unwrap();

    let new_invitation = Invitation::from(email);

    let inserted_invitation = diesel::insert_into(invitations)
        .values(&new_invitation)
        .get_result(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error inserting invitation.".into(),
        })?;

    Ok(inserted_invitation)
}
