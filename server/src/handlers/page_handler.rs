use crate::{
    data::models::{DatasetConfiguration, Pool, UnifiedId},
    errors::ServiceError,
    get_env,
    operators::dataset_operator::get_dataset_by_id_query,
};
use actix_web::{web, HttpResponse};
use minijinja::context;
use serde::Deserialize;

use crate::data::models::Templates;

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
        println!("no dataset");
        return Ok(HttpResponse::NotFound().finish());
    };

    println!("id is present");
    let dataset = get_dataset_by_id_query(UnifiedId::TrieveUuid(dataset_id), pool).await?;
    println!("dataset found");

    let config = DatasetConfiguration::from_json(dataset.server_configuration);

    let base_server_url = get_env!(
        "BASE_SERVER_URL",
        "Server hostname for OpenID provider must be set"
    );

    if config.PUBLIC_DATASET.enabled {
        let templ = templates.get_template("page.html").unwrap();
        let response_body = templ
            .render(context! {
                datasetId => dataset_id,
                baseUrl => base_server_url.clone(),
                apiKey => config.PUBLIC_DATASET.api_key
            })
            .unwrap();

        Ok(HttpResponse::Ok().body(response_body))
    } else {
        Ok(HttpResponse::Forbidden().finish())
    }
}
