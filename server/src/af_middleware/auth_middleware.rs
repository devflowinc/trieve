use crate::{
    data::models::{DatasetAndOrgWithSubAndPlan, Pool, UserRole},
    errors::ServiceError,
    handlers::auth_handler::{LoggedUser, OrganizationRole},
    operators::{
        dataset_operator::get_dataset_by_id_query,
        organization_operator::get_organization_by_key_query,
        user_operator::get_user_from_api_key_query,
    },
};
use actix_identity::Identity;
use actix_web::{
    dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, FromRequest, HttpMessage, HttpRequest,
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

pub struct AuthenticationMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthenticationMiddleware<S>
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
            let org_id_span = transaction.start_child("orgid", "Getting organization id");

            let org_id = match req.headers().get("TR-Organization") {
                Some(org_header) => {
                    let orgid_result = org_header
                        .to_str()
                        .map_err(|_| {
                            Into::<Error>::into(ServiceError::BadRequest(
                                "Could not convert Organization to str".to_string(),
                            ))
                        })?
                        .parse::<uuid::Uuid>();

                    if let Some(dataset_header) = req.headers().get("TR-Dataset") {
                        let pool = req.app_data::<web::Data<Pool>>().unwrap().to_owned();

                        let dataset_id = dataset_header
                            .to_str()
                            .map_err(|_| {
                                ServiceError::BadRequest("Dataset must be valid string".to_string())
                            })?
                            .parse::<uuid::Uuid>()
                            .map_err(|_| {
                                ServiceError::BadRequest("Dataset must be valid UUID".to_string())
                            })?;

                        let dataset = get_dataset_by_id_query(dataset_id, pool.clone()).await?;
                        let org_plan_sub = get_organization_by_key_query(
                            dataset.organization_id.into(),
                            pool.clone(),
                        )
                        .await
                        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

                        let dataset_org_plan_sub =
                            DatasetAndOrgWithSubAndPlan::from_components(dataset, org_plan_sub);

                        req.extensions_mut().insert(dataset_org_plan_sub.clone());
                    }

                    match orgid_result {
                        Ok(org_id) => org_id,
                        Err(_) => {
                            let pool = req.app_data::<web::Data<Pool>>().unwrap().to_owned();
                            let organization = get_organization_by_key_query(
                                org_header
                                    .to_str()
                                    .map_err(|_| {
                                        Into::<Error>::into(ServiceError::InternalServerError(
                                            "Could not convert Organization to str".to_string(),
                                        ))
                                    })?
                                    .to_string()
                                    .into(),
                                pool,
                            )
                            .await
                            .map_err(|_| {
                                Into::<Error>::into(ServiceError::InternalServerError(
                                    "Could not get org id".into(),
                                ))
                            })?;
                            organization.id
                        }
                    }
                }

                None => match req.headers().get("TR-Dataset") {
                    Some(dataset_header) => {
                        let pool = req.app_data::<web::Data<Pool>>().unwrap().to_owned();

                        let dataset_id = dataset_header
                            .to_str()
                            .map_err(|_| {
                                ServiceError::BadRequest("Dataset must be valid string".to_string())
                            })?
                            .parse::<uuid::Uuid>()
                            .map_err(|_| {
                                ServiceError::BadRequest("Dataset must be valid UUID".to_string())
                            })?;

                        let dataset = get_dataset_by_id_query(dataset_id, pool.clone()).await?;
                        let org_plan_sub = get_organization_by_key_query(
                            dataset.organization_id.into(),
                            pool.clone(),
                        )
                        .await
                        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
                        let dataset_org_plan_sub =
                            DatasetAndOrgWithSubAndPlan::from_components(dataset, org_plan_sub);

                        req.extensions_mut().insert(dataset_org_plan_sub.clone());

                        dataset_org_plan_sub.organization.id
                    }
                    None => {
                        let (http_req, pl) = req.parts_mut();
                        let user = get_user(http_req, pl);
                        if let Some(user) = user {
                            req.extensions_mut().insert(user.clone());
                        }
                        let res = srv.call(req).await?;

                        org_id_span.finish();
                        transaction.finish();

                        return Ok(res);
                    }
                },
            };

            let (http_req, pl) = req.parts_mut();
            let user = get_user(http_req, pl);

            if let Some(user) = user {
                req.extensions_mut().insert(user.clone());
                let user_org = user
                    .user_orgs
                    .iter()
                    .find(|org| org.organization_id == org_id)
                    .ok_or(ServiceError::Forbidden)?;

                let role = if user_org.role >= UserRole::User.into() {
                    Ok(OrganizationRole {
                        user: user.clone(),
                        role: UserRole::from(user_org.role),
                    })
                } else {
                    Err(ServiceError::Forbidden)
                }?;

                req.extensions_mut().insert(role);
            }

            let res = srv.call(req).await?;

            org_id_span.finish();
            transaction.finish();

            Ok(res)
        })
    }
}

fn get_user(req: &HttpRequest, pl: &mut Payload) -> Option<LoggedUser> {
    if let Ok(identity) = Identity::from_request(req, pl).into_inner() {
        if let Ok(user_json) = identity.id() {
            if let Ok(user) = serde_json::from_str::<LoggedUser>(&user_json) {
                return Some(user);
            }
        }
    }

    if let Some(authen_header) = req.headers().get("Authorization") {
        if let Ok(authen_header) = authen_header.to_str() {
            if let Some(pool) = req.app_data::<web::Data<Pool>>() {
                if let Ok(user) = get_user_from_api_key_query(authen_header, pool) {
                    return Some(user);
                }
            }
        }
    }

    None
}

pub struct AuthMiddlewareFactory;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddlewareFactory
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationMiddleware {
            service: Rc::new(service),
        }))
    }
}
