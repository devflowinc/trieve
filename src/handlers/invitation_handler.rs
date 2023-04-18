use actix_web::{web, HttpResponse};
use diesel::prelude::*;
use serde::Deserialize;

use crate::{
    operators::email_operator::send_invitation,
    data::models::{Invitation, Pool},
};

#[derive(Deserialize)]
pub struct InvitationData {
    pub email: String,
}

pub async fn post_invitation(
    invitation_data: web::Json<InvitationData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    // run diesel blocking code
    web::block(move || create_invitation(invitation_data.into_inner().email, pool)).await??;

    Ok(HttpResponse::Ok().finish())
}

fn create_invitation(
    email: String,
    pool: web::Data<Pool>,
) -> Result<(), crate::errors::ServiceError> {
    let invitation = dbg!(create_invitation_query(email, pool)?);
    send_invitation(&invitation)
}

/// Diesel query
fn create_invitation_query(email: String, pool: web::Data<Pool>) -> Result<Invitation, crate::errors::ServiceError> {
    use crate::data::schema::invitations::dsl::invitations;

    let mut conn = pool.get().unwrap();

    let new_invitation = Invitation::from(email);

    let inserted_invitation = diesel::insert_into(invitations)
        .values(&new_invitation)
        .get_result(&mut conn)?;

    Ok(inserted_invitation)
}
