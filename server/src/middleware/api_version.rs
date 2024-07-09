use crate::{data::models::DatasetAndOrgWithSubAndPlan, errors::ServiceError};
use actix_http::header::{HeaderName, HeaderValue};
use actix_web::{
    dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    Error, FromRequest, HttpMessage, HttpRequest,
};
use chrono::NaiveDate;
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

#[derive(Clone, Debug)]
pub enum APIVersion {
    One,
    Two,
}

impl FromRequest for APIVersion {
    type Error = Error;
    type Future = Ready<Result<APIVersion, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            req.extensions().get::<APIVersion>().cloned().ok_or(
                ServiceError::InternalServerError("API version not found".to_string()).into(),
            ),
        )
    }
}

impl APIVersion {
    fn from_dataset(dataset: &DatasetAndOrgWithSubAndPlan) -> Self {
        let versioning_date = NaiveDate::from_ymd_opt(2024, 7, 4)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();
        if dataset
            .organization
            .organization
            .created_at
            .gt(&versioning_date)
        {
            return Self::Two;
        }
        Self::One
    }
    fn to_header_str(&self) -> &str {
        match self {
            APIVersion::One => "1.0",
            APIVersion::Two => "2.0",
        }
    }
}

pub struct ApiVersionCheckFactory;

impl<S, B> Transform<S, ServiceRequest> for ApiVersionCheckFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiVersionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiVersionMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct ApiVersionMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ApiVersionMiddleware<S>
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
        let srv = self.service.clone();
        Box::pin(async move {
            let version = {
                let extensions = req.extensions();
                if let Some(dataset) = extensions.get::<DatasetAndOrgWithSubAndPlan>() {
                    let version_header = req
                        .headers()
                        .get("X-API-Version")
                        .and_then(|v| v.to_str().ok());

                    Some(match version_header {
                        Some(v) => match v {
                            "1.0" | "1" => APIVersion::One,
                            "2.0" | "2" => APIVersion::Two,
                            _ => APIVersion::from_dataset(dataset),
                        },
                        None => APIVersion::from_dataset(dataset),
                    })
                } else {
                    None
                }
            };

            if let Some(version) = version {
                req.headers_mut().insert(
                    HeaderName::from_static("X-API-Version"),
                    HeaderValue::from_str(version.to_header_str()).expect("A valid header string"),
                );
                req.extensions_mut().insert(version);
            }

            srv.call(req).await
        })
    }
}
