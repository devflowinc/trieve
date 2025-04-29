use super::email_operator::send_email;
use crate::data::models::{Invitation, Pool, Templates};
use crate::errors::ServiceError;
use actix_web::web;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use minijinja::context;

/// Diesel query

pub async fn create_invitation_query(
    email: String,
    organization_id: uuid::Uuid,
    user_role: i32,
    scopes: Option<Vec<String>>,
    pool: web::Data<Pool>,
) -> Result<Invitation, ServiceError> {
    use crate::data::schema::invitations::dsl::invitations;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let new_invitation = Invitation::from_details(email, organization_id, user_role, scopes);

    let inserted_invitation = diesel::insert_into(invitations)
        .values(&new_invitation)
        .get_result(&mut conn)
        .await
        .map_err(|_db_error| ServiceError::BadRequest("Error inserting invitation.".to_string()))?;

    Ok(inserted_invitation)
}

pub async fn get_invitation_by_id_query(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Invitation, ServiceError> {
    use crate::data::schema::invitations::dsl as invitations_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let invitation = invitations_columns::invitations
        .filter(invitations_columns::id.eq(id))
        .first::<Invitation>(&mut conn)
        .await
        .map_err(|_db_error| ServiceError::BadRequest("Error getting invitation.".to_string()))?;

    Ok(invitation)
}

pub async fn send_invitation(
    inv_url: String,
    invitation: Invitation,
    templates: Templates<'_>,
    org_name: String,
) -> Result<(), ServiceError> {
    let templ = templates.get_template("user_email_invite.html").unwrap();
    let sg_email_content = templ
        .render(context! {
            org_name,
            org_redirect_url => inv_url,
            email => invitation.email,
            new_user => true
        })
        .map_err(|e| {
            ServiceError::InternalServerError(format!("Error rendering template {}", e))
        })?;

    send_email(sg_email_content, invitation.email, None)
}

pub async fn send_invitation_for_existing_user(
    email: String,
    org_name: String,
    templates: Templates<'_>,
    organization_url: String,
) -> Result<(), ServiceError> {
    let split_org_url = organization_url.split('?').collect::<Vec<&str>>();

    let org_redirect_url = format!(
        "{}org?{}",
        split_org_url.get(0).unwrap_or(&""),
        split_org_url.get(1).unwrap_or(&"")
    );

    let templ = templates.get_template("user_email_invite.html").unwrap();
    let sg_email_content = templ
        .render(context! {
            org_name,
            org_redirect_url,
            email,
            new_user => false,
        })
        .map_err(|e| {
            ServiceError::InternalServerError(format!("Error rendering template {}", e))
        })?;

    send_email(sg_email_content, email, None)
}

pub async fn set_invitation_used(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::invitations::dsl as invitations_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    diesel::update(invitations_columns::invitations)
        .filter(invitations_columns::id.eq(id))
        .set(invitations_columns::used.eq(true))
        .execute(&mut conn)
        .await
        .map_err(|_db_error| {
            ServiceError::BadRequest("Error setting invitation as used.".to_string())
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

    if invitation.email.to_lowercase() != email.to_lowercase() {
        return Err(ServiceError::BadRequest(
            "Email does not match invitation".to_string(),
        ));
    }

    if invitation.organization_id != organization_id.unwrap() {
        return Err(ServiceError::BadRequest(
            "Dataset ID does not match invitation".to_string(),
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

pub async fn get_invitations_for_organization_query(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<Invitation>, ServiceError> {
    use crate::data::schema::invitations::dsl as invitations_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    let invitations = invitations_columns::invitations
        .filter(invitations_columns::organization_id.eq(organization_id))
        .load::<Invitation>(&mut conn)
        .await
        .map_err(|_db_error| ServiceError::BadRequest("Error getting invitations.".to_string()))?;

    Ok(invitations)
}

pub async fn delete_invitation_by_id_query(
    id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::invitations::dsl as invitations_columns;

    let mut conn = pool.get().await.map_err(|_e| {
        ServiceError::InternalServerError("Failed to get postgres connection".to_string())
    })?;

    diesel::delete(invitations_columns::invitations.filter(invitations_columns::id.eq(id)))
        .execute(&mut conn)
        .await
        .map_err(|_db_error| ServiceError::BadRequest("Error deleting invitation.".to_string()))?;

    Ok(())
}
