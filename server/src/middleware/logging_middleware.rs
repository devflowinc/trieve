use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::{self, Bytes},
    Error, HttpMessage,
};

use actix_http::h1::Payload;
use clickhouse::Row;
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use std::{
    future::{ready, Ready},
    rc::Rc,
};

use crate::{data::models::DatasetAndOrgWithSubAndPlan, errors::ServiceError};

pub struct LoggingMiddleware<S> {
    service: Rc<S>,
}

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct LoggedEvent {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: uuid::Uuid,
    pub path: String,
    pub method: String,
    #[serde(with = "clickhouse::serde::uuid::option")]
    pub dataset_id: Option<uuid::Uuid>,
    pub request_payload: String,
    pub response_payload: String,
    pub latency: f32,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
}

impl<S> Service<ServiceRequest> for LoggingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        // Clone the Rc pointers so we can move them into the async block.
        let srv = self.service.clone();
        Box::pin(async move {
            let tx_ctx = sentry::TransactionContext::new("logging middleware", "get request body");
            let transaction = sentry::start_transaction(tx_ctx);
            sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone().into())));

            let get_body_span = transaction.start_child("get_req_body", "Getting request body");
            let request_body = req.extract::<Bytes>().await.unwrap();

            let (_, mut orig_payload) = Payload::create(true);
            orig_payload.unread_data(request_body.clone());
            req.set_payload(actix_http::Payload::from(orig_payload));

            let req_body_data = std::str::from_utf8(request_body.as_ref()).unwrap_or("Unknown");
            get_body_span.finish();

            let dataset_id = req
                .extensions()
                .get::<DatasetAndOrgWithSubAndPlan>()
                .or(None)
                .map(|d| d.dataset.id);

            transaction.finish();

            let start_time = std::time::Instant::now();

            let res = srv.call(req).await?;

            let end_time = std::time::Instant::now();

            let latency = end_time - start_time;

            let tx_ctx = sentry::TransactionContext::new("logging middleware", "get response body");
            let transaction = sentry::start_transaction(tx_ctx);
            sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone().into())));

            let get_body_span = transaction.start_child("get_res_body", "Getting response body");
            // We need to clone the response body so we can log it and still return it to the client.
            let mut response_bytes = Bytes::new();

            let new_resp = res.map_body(|_, body| {
                // Get the response body as bytes and store it new buffer as `try_into_bytes` consumes the body.
                if let Ok(b) = body.try_into_bytes() {
                    response_bytes = b.clone();
                }

                // Return the body as a BoxBody so we can return it to the client.
                BoxBody::new(response_bytes.clone())
            });

            let res_body_data = std::str::from_utf8(response_bytes.as_ref()).unwrap_or("Unknown");

            get_body_span.finish();

            let http_req = new_resp.request();

            let event = LoggedEvent {
                id: uuid::Uuid::new_v4(),
                path: http_req.path().to_string(),
                method: http_req.method().to_string(),
                dataset_id,
                request_payload: req_body_data.to_string(),
                response_payload: res_body_data.to_string(),
                latency: latency.as_secs_f32(),
                created_at: OffsetDateTime::now_utc(),
            };

            let clickhouse_client = http_req
                .app_data::<web::Data<clickhouse::Client>>()
                .unwrap();

            let mut inserter = clickhouse_client.insert("trieve.events").map_err(|err| {
                log::error!("Error creating ClickHouse inserter: {:?}", err);
                Into::<Error>::into(ServiceError::InternalServerError(
                    "Error creating ClickHouse inserter".to_string(),
                ))
            })?;

            inserter.write(&event).await.map_err(|err| {
                log::error!("Error writing to ClickHouse: {:?}", err);
                Into::<Error>::into(ServiceError::InternalServerError(
                    "Error writing to ClickHouse".to_string(),
                ))
            })?;

            inserter.end().await.map_err(|err| {
                log::error!("Error ending ClickHouse inserter: {:?}", err);
                Into::<Error>::into(ServiceError::InternalServerError(
                    "Error ending ClickHouse inserter".to_string(),
                ))
            })?;

            Ok(new_resp)
        })
    }
}

pub struct LoggingMiddlewareFactory;

impl<S> Transform<S, ServiceRequest> for LoggingMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggingMiddleware {
            service: Rc::new(service),
        }))
    }
}
