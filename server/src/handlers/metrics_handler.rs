use actix_web::{web, HttpResponse};

use crate::metrics::Metrics;

pub async fn get_metrics(metrics: web::Data<Metrics>) -> Result<HttpResponse, actix_web::Error> {
    let reponse = metrics.get_response();
    // Set the proper content-type for prometheus
    Ok(HttpResponse::Ok().content_type("text/plain").body(reponse))
}
