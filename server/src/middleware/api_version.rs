use crate::{data::models::DatasetAndOrgWithSubAndPlan, errors::ServiceError};
use actix_http::header::{HeaderName, HeaderValue};
use actix_web::{
    dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    Error, FromRequest, HttpMessage, HttpRequest,
};
use chrono::NaiveDate;
use derive_more::Display;
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Eq, ToSchema, Serialize, Deserialize, Display)]
pub enum APIVersion {
    #[display("1.0")]
    V1,
    #[display("2.0")]
    V2,
}

impl FromRequest for APIVersion {
    type Error = Error;
    type Future = Ready<Result<APIVersion, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            req.extensions().get::<APIVersion>().cloned().ok_or(
                ServiceError::InternalServerError(
                    "Dataset not found; likely TR-Dataset header was not provided".to_string(),
                )
                .into(),
            ),
        )
    }
}

impl APIVersion {
    fn from_dataset(dataset: &DatasetAndOrgWithSubAndPlan) -> Self {
        let versioning_date = std::env::var("V2_VERSIONING_DATE")
            .unwrap_or_else(|_| "2024-07-14".to_string())
            .parse::<NaiveDate>()
            .unwrap_or_else(|e| {
                log::error!("Error parsing V2_VERSIONING_DATE: {}", e);
                NaiveDate::from_ymd(2024, 7, 14)
            })
            .and_hms(0, 0, 0);

        if dataset
            .organization
            .organization
            .created_at
            .gt(&versioning_date)
        {
            return Self::V2;
        }
        Self::V1
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

    fn call(&self, req: ServiceRequest) -> Self::Future {
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
                            "1.0" => APIVersion::V1,
                            "2.0" => APIVersion::V2,
                            "V1" => APIVersion::V1,
                            "V2" => APIVersion::V2,
                            "v1" => APIVersion::V1,
                            "v2" => APIVersion::V2,
                            _ => APIVersion::from_dataset(dataset),
                        },
                        None => APIVersion::from_dataset(dataset),
                    })
                } else {
                    None
                }
            };

            if let Some(version) = version.clone() {
                req.extensions_mut().insert(version);
            }

            let mut res = srv.call(req).await?;

            if let Some(version) = version.clone() {
                res.headers_mut().insert(
                    HeaderName::from_static("x-api-version"),
                    HeaderValue::from_str(&version.to_string()).expect("A valid header string"),
                );
            }

            Ok(res)
        })
    }
}
