use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web::{self, Bytes},
    Error,
};

use actix_http::h1::Payload;
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};

use std::{
    future::{ready, Ready},
    rc::Rc,
};

use crate::{
    data::models::{Pool, RedisPool},
    errors::ServiceError,
};

pub struct LoggingMiddleware<S> {
    service: Rc<S>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggedEvent {
    pub path: String,
    pub method: String,
    pub headers: String,
    pub body: String,
}

impl<S, B> Service<ServiceRequest> for LoggingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        // Clone the Rc pointers so we can move them into the async block.
        let srv = self.service.clone();
        Box::pin(async move {
            let tx_ctx =
                sentry::TransactionContext::new("middleware", "get dataset, org, and/or user");
            let transaction = sentry::start_transaction(tx_ctx);
            sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone().into())));

            let get_body_span = transaction.start_child("get_req_body", "Getting request body");
            let request_body = req.extract::<Bytes>().await.unwrap();

            let (_, mut orig_payload) = Payload::create(true);
            orig_payload.unread_data(request_body.clone());
            req.set_payload(actix_http::Payload::from(orig_payload));

            let body_data = std::str::from_utf8(request_body.as_ref()).unwrap_or("Unknown");
            get_body_span.finish();

            let event = LoggedEvent {
                path: req.path().to_string(),
                method: req.method().to_string(),
                headers: format!("{:?}", req.headers()),
                body: body_data.to_string(),
            };

            let send_to_redis_span =
                transaction.start_child("send_to_redis", "Sending event to redis");

            let redis_pool = req.app_data::<web::Data<RedisPool>>().unwrap();
            let mut redis_conn = redis_pool
                .get()
                .await
                .map_err(|err| Into::<Error>::into(ServiceError::BadRequest(err.to_string())))?;

            let stringified_event = serde_json::to_string(&event).unwrap();

            redis::cmd("lpush")
                .arg("events")
                .arg(&stringified_event)
                .query_async(&mut *redis_conn)
                .await
                .map_err(|err| Into::<Error>::into(ServiceError::BadRequest(err.to_string())))?;

            drop(redis_conn);

            send_to_redis_span.finish();

            transaction.finish();

            let res = srv.call(req).await?;

            Ok(res)
        })
    }
}

pub struct LoggingMiddlewareFactory;

impl<S, B> Transform<S, ServiceRequest> for LoggingMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
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
