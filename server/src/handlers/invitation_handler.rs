use super::auth_handler::RequireAuth;
use crate::{
    data::models::{Dataset, Invitation, Pool},
    errors::{DefaultError, ServiceError},
    operators::invitation_operator::{create_invitation_query, send_invitation},
};
use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn email_regex() -> regex::Regex {
    regex::Regex::new(r"^[a-zA-Z0-9.!#$%&’*+/=?^_`{|}~-]+@[a-zA-Z0-9-]+(?:\.[a-zA-Z0-9-]+)*")
        .unwrap()
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct InvitationResponse {
    pub registration_url: String,
}

#[derive(Deserialize, ToSchema)]
pub struct InvitationData {
    pub email: String,
    pub redirect_uri: String,
}

#[utoipa::path(
    post,
    path = "/invitation",
    context_path = "/api",
    tag = "invitation",
    request_body(content = InvitationData, description = "JSON request payload to send an invitation", content_type = "application/json"),
    responses(
        (status = 204, description = "Ok"),
        (status = 400, description = "Invalid email", body = [DefaultError]),
    )
)]
pub async fn post_invitation(
    request: HttpRequest,
    invitation_data: web::Json<InvitationData>,
    pool: web::Data<Pool>,
    dataset: Dataset,
    _required_user: RequireAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let invitation_data = invitation_data.into_inner();
    let email = invitation_data.email;
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

    let invitation = create_invitation(
        host_name,
        email,
        dataset,
        invitation_data.redirect_uri,
        pool,
    )
    .await
    .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    send_invitation(invitation.registration_url, invitation.invitation).map_err(|e| {
        ServiceError::BadRequest(format!("Could not send invitation: {}", e.message))
    })?;

    Ok(HttpResponse::NoContent().finish())
}

pub struct InvitationWithUrl {
    invitation: Invitation,
    registration_url: String,
}

pub async fn create_invitation(
    app_url: String,
    email: String,
    dataset: Dataset,
    redirect_uri: String,
    pool: web::Data<Pool>,
) -> Result<InvitationWithUrl, DefaultError> {
    let invitation = create_invitation_query(email, dataset.id, pool).await?;
    // send_invitation(app_url, &invitation)

    //TODO:figure out how to get redirect_uri
    let registration_url = format!(
        "{}/auth?inv_code={}&dataset_id={}&redirect_uri={}",
        app_url, invitation.id, dataset.id, redirect_uri
    );
    Ok(InvitationWithUrl {
        invitation,
        registration_url,
    })
}
