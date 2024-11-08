use std::future::{self, Ready};

use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};

use futures::future::LocalBoxFuture;

use crate::{errors::ServiceError, get_env};

pub struct RequireApiKey;

impl<S, B> Transform<S, ServiceRequest> for RequireApiKey
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
        match req.headers().get("Authorization") {
            Some(key) if key != api_key => {
                return Box::pin(async { Err(ServiceError::Unauthorized.into()) })
            }
            None => {
                return Box::pin(async { Err(ServiceError::Unauthorized.into()) });
            }
            _ => (), // just passthrough
        }

        let future = self.service.call(req);

        Box::pin(async move {
            let response = future.await?;
            Ok(response)
        })
    }
}
