use sendgrid::v3::{
    Sender, Content, Email, Message, Personalization
};
use crate::{errors::ServiceError, models::Invitation};

pub fn send_invitation(invitation: &Invitation) -> Result<(), ServiceError> {
    let sg_api_key = std::env::var("SENDGRID_API_KEY").expect("SENDGRID_API_KEY must be set");
    let sg_sender = Sender::new(sg_api_key);

    let sg_email_content = format!(
        "Please click on the link below to complete registration. <br/>
         <a href=\"http://localhost:3000/register.html?id={}&email={}\">
         http://localhost:3030/register</a> <br>
         your Invitation expires on <strong>{}</strong>",
        invitation.id,
        invitation.email,
        invitation.expires_at.format("%I:%M %p %A, %-d %B, %C%y")
    );
    let sg_email_personalization = Personalization::new(Email::new(invitation.email.as_str()));
    let sg_email = Message::new(Email::new("no-reply@arguflow.com"))
        .set_subject("You have been invited to join Arguflow Editor")
        .add_content(
            Content::new()
                .set_content_type("text/html")
                .set_value(sg_email_content),
        )
        .add_personalization(sg_email_personalization);

    let sg_response = sg_sender.send(&sg_email);
    match sg_response {
        Ok(_) => {
            log::info!("Email sent successfully");
            Ok(())
        },
        Err(e) => {
            log::error!("Error sending email: {}", e);
            Err(ServiceError::InternalServerError)
        },
    }
}
