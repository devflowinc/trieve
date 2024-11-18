use crate::{errors::ServiceError, get_env};
use actix_web::{
    dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform},
    FromRequest, HttpMessage, HttpRequest,
};
use futures::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

#[derive(Clone, Debug)]
pub struct ApiKey;

impl FromRequest for ApiKey {
    type Error = ServiceError;
    type Future = Ready<Result<Self, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let ext = req.extensions();

        match ext.get::<ApiKey>() {
            Some(_) => ready(Ok(Self)),
            None => ready(Err(ServiceError::Unauthorized)),
        }
    }
}

pub struct ApiKeyMiddlewareFactory;

impl<S, B> Transform<S, ServiceRequest> for ApiKeyMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = ApiKeyMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct ApiKeyMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ApiKeyMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, actix_web::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let api_key = get_env!("API_KEY", "API_KEY should be set");
        if req
            .headers()
            .get("Authorization")
            .is_some_and(|v| v == api_key)
        {
            req.extensions_mut().insert(ApiKey);
        }

        let future = self.service.call(req);

        Box::pin(async move {
            let response = future.await?;
            Ok(response)
        })
    }
}
