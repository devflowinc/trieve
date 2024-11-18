use crate::{
    errors::{ErrorResponseBody, ServiceError},
    get_env, Templates,
};
use actix_web::{get, web, HttpResponse};
use minijinja::context;

#[utoipa::path(
  get,
  path = "/",
  context_path = "/",
  tag = "UI",
  responses(
      (status = 200, description = "UI meant for public consumption"),
      (status = 400, description = "Service error relating to loading the public page", body = ErrorResponseBody),
  ),
)]
#[get("/")]
pub async fn public_page(templates: Templates<'_>) -> Result<HttpResponse, ServiceError> {
    let templ = templates.get_template("demo-ui.html").unwrap();
    let trieve_api_key = get_env!("API_KEY", "API_KEY should be set");
    let response_body = templ
        .render(context! {
            trieve_api_key
        })
        .unwrap();

    Ok(HttpResponse::Ok().body(response_body))
}

#[utoipa::path(
    get,
    path = "/static/{file_name}",
    context_path = "/static",
    tag = "UI",
    responses(
        (status = 200, description = "File"),
        (status = 400, description = "Service error relating to getting the file", body = ErrorResponseBody),
    ),
  )]
#[get("/{file_name}")]
pub async fn static_files(file_name: web::Path<String>) -> Result<HttpResponse, ServiceError> {
    let sanitized_file_name = file_name.replace("..", "");
    let file = std::fs::read_to_string(format!("./static/{}", sanitized_file_name))
        .map_err(|_| ServiceError::InternalServerError("Failed to read file".to_string()))?;

    Ok(HttpResponse::Ok().body(file))
}
