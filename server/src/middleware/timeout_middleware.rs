use crate::errors::ServiceError;
use crate::operators::email_operator::send_email;
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
};

pub async fn timeout_15secs(
    service_req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let path = service_req.path().to_string();
    let method = service_req.method().as_str().to_string();
    let queries = service_req.query_string().to_string();
    let headers = service_req
        .headers()
        .iter()
        .filter_map(|(k, v)| {
            if k.to_string().to_lowercase() == "authorization" {
                None
            } else {
                format!("{}: {}", k, v.to_str().unwrap()).into()
            }
        })
        .collect::<Vec<String>>();

    let base_server_url =
        std::env::var("BASE_SERVER_URL").unwrap_or_else(|_| "https://api.trieve.ai".to_string());

    let mut timeout_secs = 15;
    if method == "POST" && path == "/api/file" {
        timeout_secs = 300;
    }

    match tokio::time::timeout(
        std::time::Duration::from_secs(timeout_secs),
        next.call(service_req),
    )
    .await
    {
        Ok(res) => res,
        Err(_err) => {
            let email_body = format!(
                "Request timeout: {}\n\n<br/><br/>Method: {}\n<br/><br/>Queries: {}\n<br/><br/>Headers: {:?}",
                path, method, queries, headers
            );
            log::info!("Request timeout: {}", path);
            let emails_enabled = std::env::var("ENABLE_408_EMIALS").unwrap_or("false".to_string());
            if emails_enabled == "true" {
                let _ = send_email(
                    email_body,
                    "webmaster@trieve.ai".to_string(),
                    Some(format!(
                        "🤖🤖 {} Request timeout {} 🤖🤖",
                        base_server_url, path
                    )),
                );
            }

            Err(ServiceError::RequestTimeout(
                "Trieve is currently under extended load and we are working to autoscale. If you continue facing this issue, please send an email to humans@trieve.ai with 'request timeout' in the subject line and we will get back to you as soon as possible.".to_string()
            ).into())
        }
    }
}
