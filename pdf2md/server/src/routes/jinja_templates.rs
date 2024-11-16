use std::{env, os};

use crate::{
    errors::{ErrorResponseBody, ServiceError},
    Templates,
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
    let is_dev = env::var("DEV_MODE")
        .unwrap_or("false".to_string())
        .eq("true");
    let templ = templates.get_template("demo-ui.html").unwrap();
    let response_body = templ
        .render(context! {
            is_dev => is_dev
        })
        .unwrap();

    Ok(HttpResponse::Ok().body(response_body))
}
