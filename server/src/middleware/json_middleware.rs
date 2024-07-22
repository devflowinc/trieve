use actix_http::header::HeaderValue;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

pub struct JsonMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JsonMiddleware<S>
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
            if req
                .headers()
                .get(actix_http::header::CONTENT_TYPE)
                .is_none()
            {
                req.headers_mut().insert(
                    actix_http::header::CONTENT_TYPE,
                    HeaderValue::from_static("application/json"),
                );
            }
            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}

pub struct JsonMiddlewareFactory;

impl<S, B> Transform<S, ServiceRequest> for JsonMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = JsonMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JsonMiddleware {
            service: Rc::new(service),
        }))
    }
}
