use crate::handlers::metrics_handler::Metrics;
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    web,
};

#[tracing::instrument(skip_all)]
pub async fn error_logging_middleware(
    metrics: web::Data<Metrics>,
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let path = req.path().to_string();
    let method = req.method().to_string();
    let base_server_url =
        std::env::var("BASE_SERVER_URL").unwrap_or_else(|_| "https://api.trieve.ai".to_string());

    let response = next.call(req).await;

    match response {
        Ok(response) => {
            let status = response.status();
            if !status.is_success() {
                metrics.register_error(status.as_u16(), method, path, base_server_url);
            }

            Ok(response)
        }
        Err(e) => {
            let status = e.error_response().status();
            if !status.is_success() {
                metrics.register_error(status.as_u16(), method, path, base_server_url);
            }
            Err(e)
        }
    }
}
