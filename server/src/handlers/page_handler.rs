use crate::handlers::chunk_handler::AutocompleteReqPayload;
use crate::{
    data::models::{DatasetConfiguration, Pool, UnifiedId},
    errors::ServiceError,
    get_env,
    operators::dataset_operator::get_dataset_by_id_query,
};
use actix_web::{web, HttpResponse};
use minijinja::context;
use serde::{Deserialize, Serialize};

use crate::data::models::Templates;

#[derive(Serialize, Deserialize, Debug, Default)]
enum PublicPageTheme {
    #[default]
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PublicPageParameters {
    dataset_id: uuid::Uuid,
    base_url: String,
    api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    analytics: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suggested_queries: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    responsive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    theme: Option<PublicPageTheme>,
    #[serde(skip_serializing_if = "Option::is_none")]
    search_options: Option<AutocompleteReqPayload>,
    // openKeyCombination: { key?: string; label?: string; ctrl?: boolean }[],
    #[serde(skip_serializing_if = "Option::is_none")]
    brand_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    brand_logo_img_src_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    problem_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    accent_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_search_queries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_ai_questions: Option<Vec<String>>,
}

#[utoipa::path(
    get,
    path = "/public_page/{dataset_id}",
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
    dataset_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    templates: Templates<'_>,
) -> Result<HttpResponse, ServiceError> {
    let dataset_id = dataset_id.into_inner();

    let dataset = get_dataset_by_id_query(UnifiedId::TrieveUuid(dataset_id), pool).await?;

    let config = DatasetConfiguration::from_json(dataset.server_configuration);

    let base_server_url = get_env!(
        "BASE_SERVER_URL",
        "Server hostname for OpenID provider must be set"
    );

    if config.PUBLIC_DATASET.enabled {
        let templ = templates.get_template("page.html").unwrap();
        let response_body = templ
            .render(context! {
                params => PublicPageParameters {
                    dataset_id,
                    base_url: base_server_url.to_string(),
                    api_key: config.PUBLIC_DATASET.api_key,
                    ..Default::default()
                }
            })
            .unwrap();

        Ok(HttpResponse::Ok().body(response_body))
    } else {
        Ok(HttpResponse::Forbidden().finish())
    }
}
