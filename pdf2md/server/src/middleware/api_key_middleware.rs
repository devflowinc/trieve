use crate::{errors::ServiceError, get_env};
use actix_web::{
    dev::{Payload, Service, ServiceRequest, ServiceResponse, Transform},
    FromRequest, HttpMessage, HttpRequest,
};
use futures::future::LocalBoxFuture;
use std::future::{self, ready, Ready};

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

impl<S, B> Transform<S, ServiceRequest> for ApiKey
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = ApiKeyMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ready(Ok(ApiKeyMiddleware { service }))
    }
}

pub struct ApiKeyMiddleware<S> {
    service: S,
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
            req.extensions_mut().insert(api_key);
        }

        let future = self.service.call(req);

        Box::pin(async move {
            let response = future.await?;
            Ok(response)
        })
    }
}
