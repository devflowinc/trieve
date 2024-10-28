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
use utoipa::ToSchema;

use crate::data::models::Templates;

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
pub enum PublicPageTheme {
    #[default]
    #[serde(rename = "light")]
    Light,
    #[serde(rename = "dark")]
    Dark,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct PublicPageParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dataset_id: Option<uuid::Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_queries: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub responsive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<PublicPageTheme>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_options: Option<AutocompleteReqPayload>,
    //pub openKeyCombination: { key?: string; label?: string; ctrl?: boolean }[],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand_logo_img_src_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub problem_link: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_search_queries: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_ai_questions: Option<Vec<String>>,
}

#[utoipa::path(
    get,
    path = "/public_page/{dataset_id}",
    context_path = "/api",
    tag = "Public",
    responses(
        (status = 200, description = "Public Page associated to the dataset"),
        (status = 400, description = "Service error relating to loading the public page", body = ErrorResponseBody),
        (status = 404, description = "Dataset not found", body = ErrorResponseBody)
    ),
    params(
        ("dataset_id" = uuid::Uuid, Path, description = "The id of the organization you want to fetch."),
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
                    dataset_id: Some(dataset_id),
                    base_url: Some(base_server_url.to_string()),
                    api_key: Some(config.PUBLIC_DATASET.api_key.unwrap_or_default()),
                    ..config.PUBLIC_DATASET.extra_params.unwrap_or_default()
                }
            })
            .unwrap();

        Ok(HttpResponse::Ok().body(response_body))
    } else {
        Ok(HttpResponse::Forbidden().finish())
    }
}
