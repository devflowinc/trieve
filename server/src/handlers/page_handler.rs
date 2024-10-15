use actix_web::HttpResponse;
use minijinja::context;

use crate::data::models::Templates;

pub async fn public_page(templates: Templates<'_>) -> Result<HttpResponse, actix_web::Error> {
    let templ = templates.get_template("page.html").unwrap();
    let response_body = templ.render(context! {}).unwrap();

    Ok(HttpResponse::Ok().body(response_body))
}
