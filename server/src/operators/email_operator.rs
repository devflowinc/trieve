use crate::{errors::DefaultError, get_env};
use sendgrid::v3::{Content, Email, Message, Personalization, Sender};

#[allow(dead_code)]
pub fn send_health_check_error(email: String, error: String) -> Result<(), DefaultError> {
    let sg_email_content = format!(
        "WARNING health check is down. <br/>
        Error message: <br/>
         <code>{}</code>",
        error
    );
    let sg_email_personalization = Personalization::new(Email::new(email));
    let email_address = Email::new(get_env!(
        "SENDGRID_EMAIL_ADDRESS",
        "SENDGRID_EMAIL_ADDRESS should be set"
    ));
    let sg_email = Message::new(email_address)
        .set_subject("WARNING WARNING WARNING production is down WARNING WARING WARNING")
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
