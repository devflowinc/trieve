use actix_web::HttpResponse;

use crate::errors::ServiceError;

pub async fn ingest() -> Result<HttpResponse, ServiceError>{
    Ok(HttpResponse::Ok().json("Hello, world!"))
}
