use crate::{errors::DefaultError, get_env};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

#[tracing::instrument]
pub fn get_smtp_creds() -> Credentials {
    let smtp_username = get_env!("SMTP_USERNAME", "SMTP_USERNAME should be set");
    let smtp_password = get_env!("SMTP_PASSWORD", "SMTP_PASSWORD should be set");

    Credentials::new(smtp_username.to_owned(), smtp_password.to_owned())
}

#[tracing::instrument]
pub async fn send_email(html_email_body: String, to_address: String) -> Result<(), DefaultError> {
    let smtp_relay = get_env!("SMTP_RELAY", "SMTP_RELAY should be set");
    let smtp_email_address = get_env!("SMTP_EMAIL_ADDRESS", "SMTP_EMAIL_ADDRESS should be set");

    let smtp_creds = get_smtp_creds();
    let mailer = SmtpTransport::relay(smtp_relay)
        .expect("Failed to create mailer")
        .credentials(smtp_creds)
        .build();

    let email = Message::builder()
        .from(smtp_email_address.parse().expect("Invalid email address"))
        .to(to_address.parse().expect("Invalid email address"))
        .subject("Trieve Sign Up Invitation")
        .header(ContentType::TEXT_HTML)
        .body(html_email_body)
        .expect("Failed to create email");

    match mailer.send(&email) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("Error sending email: {:?}", e);
            Err(DefaultError {
                message: "Error sending email.",
            })
        }
    }
}
