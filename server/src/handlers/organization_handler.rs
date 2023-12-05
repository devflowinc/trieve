use super::auth_handler::RequireAuth;
use crate::{
    data::models::Pool,
    errors::ServiceError,
    operators::organization_operator::{
        create_organization_query, delete_organization_by_id_query, get_organization_by_id_query,
        update_organization_query,
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub async fn get_organization_by_id(
    organization_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    _required_user: RequireAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_id = organization_id.into_inner();

    let organization = web::block(move || get_organization_by_id_query(organization_id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(organization))
}

pub async fn delete_organization_by_id(
    organization_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    _required_user: RequireAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_id = organization_id.into_inner();

    web::block(move || delete_organization_by_id_query(organization_id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateOrganizationData {
    organization_uuid: uuid::Uuid,
    name: String,
    configuration: serde_json::Value,
}

pub async fn update_organization(
    organization: web::Json<UpdateOrganizationData>,
    pool: web::Data<Pool>,
    _required_user: RequireAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_update_data = organization.into_inner();

    web::block(move || {
        update_organization_query(
            organization_update_data.organization_uuid,
            organization_update_data.name.as_str(),
            organization_update_data.configuration,
            pool,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().finish())
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CreateOrganizationData {
    name: String,
    configuration: serde_json::Value,
}

pub async fn create_organization(
    organization: web::Json<CreateOrganizationData>,
    pool: web::Data<Pool>,
    _required_user: RequireAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let organization_create_data = organization.into_inner();

    let created_organization = web::block(move || {
        create_organization_query(
            organization_create_data.name.as_str(),
            organization_create_data.configuration,
            pool,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(created_organization))
}
