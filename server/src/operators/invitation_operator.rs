use crate::errors::ServiceError;
use crate::get_env;
use crate::{
    data::models::{Invitation, Pool},
    errors::DefaultError,
};
use actix_web::web;
use diesel::prelude::*;
use sendgrid::v3::{Content, Email, Message, Personalization};

use super::email_operator::send_email;

/// Diesel query
pub async fn create_invitation_query(
    email: String,
    organization_id: uuid::Uuid,
    user_role: i32,
    pool: web::Data<Pool>,
) -> Result<Invitation, DefaultError> {
    use crate::data::schema::invitations::dsl::invitations;

    let mut conn = pool.get().unwrap();

    let new_invitation = Invitation::from_details(email, organization_id, user_role);

    let inserted_invitation = diesel::insert_into(invitations)
        .values(&new_invitation)
        .get_result(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error inserting invitation.",
        })?;

    Ok(inserted_invitation)
}

pub async fn get_invitation_by_id_query(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Invitation, DefaultError> {
    use crate::data::schema::invitations::dsl as invitations_columns;

    let mut conn = pool.get().unwrap();

    let invitation = invitations_columns::invitations
        .filter(invitations_columns::id.eq(id))
        .first::<Invitation>(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error getting invitation.",
        })?;

    Ok(invitation)
}

pub async fn send_invitation(inv_url: String, invitation: Invitation) -> Result<(), DefaultError> {
    let sg_email_content = format!(
        "You have been invited to join an Arguflow AI dataset. <br/>
         Please click on the link below to register. <br/>
         <a href=\"{}\">
         {}</a>",
        inv_url,
        inv_url.split('?').collect::<Vec<&str>>()[0]
    );

    let sg_email_personalization = Personalization::new(Email::new(invitation.email.as_str()));
    let email_address = Email::new(get_env!(
        "SENDGRID_EMAIL_ADDRESS",
        "SENDGRID_EMAIL_ADDRESS should be set"
    ));

    let sg_email = Message::new(email_address)
        .set_subject("Invitation to join Arguflow AI dataset")
        .add_content(
            Content::new()
                .set_content_type("text/html")
                .set_value(sg_email_content),
        )
        .add_personalization(sg_email_personalization);

    send_email(sg_email).await
}

pub async fn set_invitation_used(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::invitations::dsl as invitations_columns;

    let mut conn = pool.get().unwrap();

    diesel::update(invitations_columns::invitations)
        .filter(invitations_columns::id.eq(id))
        .set(invitations_columns::used.eq(true))
        .execute(&mut conn)
        .map_err(|_db_error| DefaultError {
            message: "Error setting invitation as used.",
        })?;

    Ok(())
}

pub async fn check_inv_valid(
    inv_code: uuid::Uuid,
    email: String,
    organization_id: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Invitation, ServiceError> {
    let invitation = get_invitation_by_id_query(inv_code, pool.clone())
        .await
        .map_err(|_| {
            ServiceError::InternalServerError("Could not find invitation for user".to_string())
        })?;

    if invitation.email != email {
        return Err(ServiceError::BadRequest(
            "Email does not match invitation".to_string(),
        ));
    }

    if invitation.organization_id != organization_id.unwrap() {
        return Err(ServiceError::BadRequest(
            "Dataset ID does not match invitation".to_string(),
        ));
    }

    if invitation.expired() {
        return Err(ServiceError::BadRequest(
            "Invitation has expired".to_string(),
        ));
    }

    if invitation.used {
        return Err(ServiceError::BadRequest(
            "Invitation has already been used".to_string(),
        ));
    }
    set_invitation_used(invitation.id, pool.clone())
        .await
        .map_err(|_| {
            ServiceError::InternalServerError("Could not set invitation as used".to_string())
        })?;

    Ok(invitation)
}
