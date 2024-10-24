use crate::{
    data::models::{DatasetAndOrgWithSubAndPlan, Pool},
    errors::ServiceError, operators::user_operator::set_user_api_key_query,
};
use actix_web::{web, HttpResponse};
use minijinja::context;
use serde::Deserialize;

use crate::{
    data::models::Templates,
    operators::page_operator::{get_page_by_dataset_id, upsert_page_visiblity},
};

use super::{auth_handler::LoggedUser, user_handler::SetUserApiKeyRequest};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicPageParams {
    dataset_id: Option<uuid::Uuid>,
}

#[utoipa::path(
    get,
    path = "/public_page",
    context_path = "/api",
    tag = "Public",
    responses(
        (status = 200, description = "Public Page associated to the dataset", body = OrganizationWithSubAndPlan),
        (status = 400, description = "Service error relating to finding the organization by id", body = ErrorResponseBody),
        (status = 404, description = "Organization not found", body = ErrorResponseBody)
    ),
    params(
        ("datasetId" = Option<uuid::Uuid>, Path, description = "The id of the organization you want to fetch."),
    ),
)]
pub async fn public_page(
    page_params: web::Query<PublicPageParams>,
    pool: web::Data<Pool>,
    templates: Templates<'_>,
) -> Result<HttpResponse, ServiceError> {
    let Some(dataset_id) = page_params.dataset_id else {
        return Ok(HttpResponse::NotFound().finish());
    };

    let page = get_page_by_dataset_id(dataset_id, pool).await?;

    if let Some(page) = page {
        if page.is_public {
            let templ = templates.get_template("page.html").unwrap();
            let response_body = templ
                .render(context! {
                    datasetId => dataset_id,
                    apiKey => page.api_key
                })
                .unwrap();

            Ok(HttpResponse::Ok().body(response_body))
        } else {
            Ok(HttpResponse::Forbidden().finish())
        }
    } else {
        Ok(HttpResponse::Forbidden().finish())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetDatasetVisibilityPayload {
    pub is_public: bool,
    pub api_key_params: SetUserApiKeyRequest,
}

#[utoipa::path(
    put,
    path = "/dataset/visibility",
    context_path = "/api",
    tag = "Public",
    responses(
        (status = 200, description = "Public Page associated to the dataset", body = OrganizationWithSubAndPlan),
        (status = 400, description = "Service error relating to finding the organization by id", body = ErrorResponseBody),
        (status = 404, description = "Organization not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("datasetId" = Option<uuid::Uuid>, Path, description = "The id of the organization you want to fetch."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn set_dataset_visiblity(
    page_params: web::Json<SetDatasetVisibilityPayload>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let role = page_params.api_key_params.role;

    let new_api_key = set_user_api_key_query(
        user.id,
        page_params.api_key_params.name.clone(),
        role.into(),
        page_params.api_key_params.dataset_ids.clone(),
        page_params.api_key_params.organization_ids.clone(),
        page_params.api_key_params.scopes.clone(),
        pool.clone()
    )
    .await
    .map_err(|_err| ServiceError::BadRequest("Failed to set new API key for user".into()))?;

    let dataset_id = dataset_org_plan_sub.dataset.id;

    let page = upsert_page_visiblity(dataset_id, page_params.is_public, new_api_key, pool).await?;

    Ok(HttpResponse::Ok().json(page))
}
