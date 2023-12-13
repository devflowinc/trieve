use crate::get_env;
use crate::{
    data::models::{Invitation, Pool},
    errors::DefaultError,
};
use actix_web::web;
use diesel::prelude::*;
use sendgrid::v3::{Content, Email, Message, Personalization, Sender};

/// Diesel query
pub async fn create_invitation_query(
    email: String,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Invitation, DefaultError> {
    use crate::data::schema::invitations::dsl::invitations;

    let mut conn = pool.get().unwrap();

    let new_invitation = Invitation::from_details(email, dataset_id);

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

pub fn send_invitation(inv_url: String, invitation: Invitation) -> Result<(), DefaultError> {
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
        .set_subject("Reset your Arguflow AI password")
        .add_content(
            Content::new()
                .set_content_type("text/html")
                .set_value(sg_email_content),
        )
        .add_personalization(sg_email_personalization);

    send_email(sg_email)
}

fn send_email(sg_email: Message) -> Result<(), DefaultError> {
    let sg_api_key = get_env!("SENDGRID_API_KEY", "SENDGRID_API_KEY should be set").into();
    let sg_sender = Sender::new(sg_api_key);
    let sg_response = sg_sender.send(&sg_email);
    match sg_response {
        Ok(_) => Ok(()),
        Err(_e) => Err(DefaultError {
            message: "Error sending email.",
        }),
    }
}
