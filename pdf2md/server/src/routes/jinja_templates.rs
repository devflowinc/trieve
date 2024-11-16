use crate::{
    errors::{ErrorResponseBody, ServiceError},
    get_env, Templates,
};
use actix_web::{get, HttpResponse};
use minijinja::context;

#[utoipa::path(
  get,
  path = "/public_page/{dataset_id}",
  context_path = "/api",
  tag = "Public",
  responses(
      (status = 200, description = "Public Page associated to the dataset"),
      (status = 400, description = "Service error relating to loading the public page", body = ErrorResponseBody),
  ),
  params(
      ("dataset_id" = uuid::Uuid, Path, description = "The id of the organization you want to fetch."),
  ),
)]
#[get("")]
pub async fn public_page(templates: Templates<'_>) -> Result<HttpResponse, ServiceError> {
    let base_server_url = get_env!(
        "BASE_SERVER_URL",
        "Server hostname for OpenID provider must be set"
    );

    let templ = templates.get_template("page.html").unwrap();
    let response_body = templ
        .render(context! {
            base_server_url,
        })
        .unwrap();

    Ok(HttpResponse::Ok().body(response_body))
}
