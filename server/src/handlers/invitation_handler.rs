use super::auth_handler::AdminOnly;
use crate::{
    data::models::{Invitation, Pool},
    errors::{DefaultError, ServiceError},
    operators::invitation_operator::{create_invitation_query, send_invitation},
};
use actix_web::{web, HttpResponse};
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
    /// The id of the organization to invite the user to.
    pub organization_id: uuid::Uuid,
    /// The role the user will have in the organization. 0 = User, 1 = Admin, 2 = Owner.
    pub user_role: i32,
    /// The email of the user to invite. Must be a valid email as they will be sent an email to register.
    pub email: String,
    /// The url of the app that the user will be directed to in order to set their password. Usually admin.trieve.ai, but may differ for local dev or self-hosted setups.
    pub app_url: String,
    /// The url that the user will be redirected to after setting their password.
    pub redirect_uri: String,
}

/// send_invitation
///
/// Invitations act as a way to invite users to join an organization. After a user is invited, they will automatically be added to the organization with the role specified in the invitation once they set their.
#[utoipa::path(
    post,
    path = "/invitation",
    context_path = "/api",
    tag = "invitation",
    request_body(content = InvitationData, description = "JSON request payload to send an invitation", content_type = "application/json"),
    responses(
        (status = 204, description = "Ok response. Indicates that invitation email was sent correctly."),
        (status = 400, description = "Invalid email or some other error", body = DefaultError),
    ),
    params(
        ("TR-Organization" = String, Header, description = "The organization id to use for the request"),
    ),
    security(
        ("Api Key" = ["admin"]),
        ("Cookie Auth" = ["admin"])
    )
)]
pub async fn post_invitation(
    invitation_data: web::Json<InvitationData>,
    pool: web::Data<Pool>,
    user: AdminOnly,
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

    let org_role = user
        .0
        .user_orgs
        .iter()
        .find(|org| org.organization_id == invitation_data.organization_id);

    if org_role.is_none() || org_role.expect("cannot be none").role < invitation_data.user_role {
        return Ok(
            HttpResponse::BadRequest().json(crate::errors::DefaultError {
                message: "Can not invite user with higher role than yours",
            }),
        );
    }

    let invitation = create_invitation(
        invitation_data.app_url,
        email,
        invitation_data.organization_id,
        invitation_data.redirect_uri,
        invitation_data.user_role,
        pool,
    )
    .await
    .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    send_invitation(invitation.registration_url, invitation.invitation)
        .await
        .map_err(|e| {
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
    organization_id: uuid::Uuid,
    redirect_uri: String,
    user_role: i32,
    pool: web::Data<Pool>,
) -> Result<InvitationWithUrl, DefaultError> {
    let invitation = create_invitation_query(email, organization_id, user_role, pool).await?;
    // send_invitation(app_url, &invitation)

    //TODO:figure out how to get redirect_uri
    let registration_url = format!(
        "{}/auth?inv_code={}&organization_id={}&redirect_uri={}",
        app_url, invitation.id, organization_id, redirect_uri
    );
    Ok(InvitationWithUrl {
        invitation,
        registration_url,
    })
}
