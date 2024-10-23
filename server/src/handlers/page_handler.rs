use actix_web::{web, HttpResponse};
use minijinja::context;

use crate::{data::models::Templates, operators::page_operator::get_page_by_dataset_id};

#[utoipa::path(
    get,
    path = "/public_page",
    context_path = "/api",
    tag = "Organization",
    responses(
        (status = 200, description = "Organization with the id that was requested", body = OrganizationWithSubAndPlan),
        (status = 400, description = "Service error relating to finding the organization by id", body = ErrorResponseBody),
        (status = 404, description = "Organization not found", body = ErrorResponseBody)
    ),
    params(
        ("datasetId" = Option<uuid::Uuid>, Path, description = "The id of the organization you want to fetch."),
    ),
)]
pub async fn public_page(
    dataset_id: web::Path<uuid::Uuid>,
    templates: Templates<'_>,
) -> Result<HttpResponse, actix_web::Error> {

    // get_page_by_dataset_id(dataset_id).await?;

    let templ = templates.get_template("page.html").unwrap();
    let response_body = templ
        .render(context! {
            datasetId => *dataset_id,
            apiKey => "aaadd"
        })
        .unwrap();

    Ok(HttpResponse::Ok().body(response_body))
}
