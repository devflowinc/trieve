use super::auth_handler::AdminOnly;
use crate::{
    data::models::{Invitation, OrganizationWithSubAndPlan, Pool, RedisPool, Templates},
    errors::ServiceError,
    middleware::auth_middleware::verify_admin,
    operators::{
        invitation_operator::{
            create_invitation_query, delete_invitation_by_id_query, get_invitation_by_id_query,
            get_invitations_for_organization_query, send_invitation,
            send_invitation_for_existing_user,
        },
        organization_operator::get_org_users_by_id_query,
        user_operator::add_existing_user_to_org,
    },
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

#[derive(Deserialize, ToSchema, Serialize, Clone, Debug)]
pub struct InvitationData {
    /// The role the user will have in the organization. 0 = User, 1 = Admin, 2 = Owner.
    pub user_role: i32,
    /// The email of the user to invite. Must be a valid email as they will be sent an email to register.
    pub email: String,
    /// The url of the app that the user will be directed to in order to set their password. Usually admin.trieve.ai, but may differ for local dev or self-hosted setups.
    pub app_url: String,
    /// The url that the user will be redirected to after setting their password.
    pub redirect_uri: String,
}

/// Send Invitation
///
/// Invitations act as a way to invite users to join an organization. After a user is invited, they will automatically be added to the organization with the role specified in the invitation once they set their. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/invitation",
    context_path = "/api",
    tag = "Invitation",
    request_body(content = InvitationData, description = "JSON request payload to send an invitation", content_type = "application/json"),
    responses(
        (status = 204, description = "Ok response. Indicates that invitation email was sent correctly."),
        (status = 400, description = "Invalid email or some other error", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn post_invitation(
    invitation_data: web::Json<InvitationData>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    org_with_plan_and_sub: OrganizationWithSubAndPlan,
    templates: Templates<'_>,
    user: AdminOnly,
) -> Result<HttpResponse, ServiceError> {
    let invitation_data = invitation_data.into_inner();
    let email = invitation_data.email.trim().to_string();
    if !email_regex().is_match(&email) {
        return Err(ServiceError::BadRequest("Invalid email".to_string()));
    }

    if user.0.email == email {
        return Err(ServiceError::BadRequest(
            "Can not invite yourself".to_string(),
        ));
    }

    let existing_user_org_id = org_with_plan_and_sub.organization.id;
    let org_role = user
        .0
        .user_orgs
        .iter()
        .find(|org| org.organization_id == existing_user_org_id);

    if org_role.is_none() || org_role.expect("cannot be none").role < invitation_data.user_role {
        return Err(ServiceError::BadRequest(
            "Can not invite user with higher role than yours".to_string(),
        ));
    }

    let user_in_org =
        get_org_users_by_id_query(org_with_plan_and_sub.organization.id, pool.clone())
            .await?
            .iter()
            .find(|user| user.email == email)
            .cloned();

    if let Some(user) = user_in_org {
        return Err(ServiceError::BadRequest(format!(
            "User with email {} is already in the organization",
            user.email
        )));
    }

    let existing_user_role = invitation_data.user_role;
    let added_user_to_org = add_existing_user_to_org(
        email.clone(),
        existing_user_org_id,
        existing_user_role.into(),
        pool.clone(),
        redis_pool,
    )
    .await?;

    if added_user_to_org {
        send_invitation_for_existing_user(
            email.clone(),
            org_with_plan_and_sub.organization.name,
            templates,
            invitation_data.redirect_uri,
        )
        .await?;
        return Ok(HttpResponse::NoContent().finish());
    }

    let org_invitations =
        get_invitations_for_organization_query(existing_user_org_id, pool.clone())
            .await?
            .iter()
            .find(|inv| inv.email == email)
            .cloned();

    if let Some(inv) = org_invitations {
        return Err(ServiceError::BadRequest(format!(
            "User with email {} has already been invited",
            inv.email
        )));
    }

    let invitation = create_invitation(
        invitation_data.app_url,
        email,
        existing_user_org_id,
        invitation_data.redirect_uri,
        invitation_data.user_role,
        pool,
    )
    .await?;

    send_invitation(
        invitation.registration_url,
        invitation.invitation,
        templates,
        org_with_plan_and_sub.organization.name,
    )
    .await?;

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
) -> Result<InvitationWithUrl, ServiceError> {
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

/// Get Invitations
///
/// Get all invitations for the organization. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    get,
    path = "/invitations/{organization_id}",
    context_path = "/api",
    tag = "Invitation",
    responses(
        (status = 200, description = "Invitations for the dataset", body = Vec<Invitation>),
        (status = 400, description = "Service error relating to getting invitations for the dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
        ("organization_id" = uuid, Path, description = "The organization id to get invitations for"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_invitations(
    user: AdminOnly,
    organization_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    if !verify_admin(&user, &organization_id.clone()) {
        return Err(ServiceError::Forbidden);
    }
    let invitations =
        get_invitations_for_organization_query(organization_id.into_inner(), pool).await?;
    Ok(HttpResponse::Ok().json(invitations))
}

/// Delete Invitation
///
/// Delete an invitation by id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    delete,
    path = "/invitation/{invitation_id}",
    context_path = "/api",
    tag = "Invitation",
    responses(
        (status = 204, description = "Ok response. Indicates that invitation was deleted."),
        (status = 400, description = "Service error relating to deleting invitation", body = ErrorResponseBody),
    ),
    params(
        ("TR-Organization" = uuid::Uuid, Header, description = "The organization id to use for the request"),
        ("invitation_id" = uuid, Path, description = "The id of the invitation to delete"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn delete_invitation(
    user: AdminOnly,
    invitation_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let invite_id = invitation_id.into_inner();
    let invite = get_invitation_by_id_query(invite_id, pool.clone()).await?;

    if !verify_admin(&user, &invite.organization_id) {
        return Err(ServiceError::Forbidden);
    }

    delete_invitation_by_id_query(invite_id, pool).await?;
    Ok(HttpResponse::NoContent().finish())
}
