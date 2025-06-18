use crate::errors::ServiceError;
use crate::operators::email_operator::send_email;
use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::Next,
    web,
};

pub async fn timeout_15secs(
    service_req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let disable_timeout_middleware =
        std::env::var("DISABLE_TIMEOUT_MIDDLEWARE").unwrap_or_else(|_| "false".to_string());

    if disable_timeout_middleware == *"true" {
        return next.call(service_req).await;
    }

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
                match v.to_str() {
                    Ok(value) => format!("{}: {}", k, value).into(),
                    Err(_) => None,
                }
            }
        })
        .collect::<Vec<String>>();

    let base_server_url =
        std::env::var("BASE_SERVER_URL").unwrap_or_else(|_| "https://api.trieve.ai".to_string());

    let mut timeout_secs = 15;
    if method == "POST" && path == "/api/file" {
        timeout_secs = 300;
    }

    if method == "POST" && path == "/api/message/edit_image" {
        timeout_secs = 300;
    }

    if path == "/api/message"
        || path == "/api/message/get_tool_function_params"
        || path == "/api/chunk/generate"
    {
        timeout_secs = 120;
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
                path, method, queries, headers.join("\n")
            );
            log::info!("Request timeout: {}", path);
            let emails_enabled = std::env::var("ENABLE_408_EMAILS").unwrap_or("false".to_string());

            if emails_enabled == "true" {
                let _ = web::block(move || {
                    let _ = send_email(
                        email_body,
                        "webmaster@trieve.ai".to_string(),
                        Some(format!(
                            "🤖🤖 {} Request timeout {} 🤖🤖",
                            base_server_url, path
                        )),
                    );
                })
                .await;
            }

            Err(ServiceError::RequestTimeout(
                "Trieve is currently under extended load and we are working to autoscale. If you continue facing this issue, please send an email to humans@trieve.ai with 'request timeout' in the subject line and we will get back to you as soon as possible.".to_string()
            ).into())
        }
    }
}
