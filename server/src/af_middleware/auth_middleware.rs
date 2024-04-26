use crate::{
    data::models::{Pool, UnifiedId, UserRole},
    errors::ServiceError,
    handlers::auth_handler::{LoggedUser, OrganizationRole},
    operators::{
        dataset_operator::get_dataset_and_organization_from_dataset_id_query,
        organization_operator::{
            get_arbitrary_org_owner_from_dataset_id, get_arbitrary_org_owner_from_org_id,
        },
        user_operator::get_user_from_api_key_query,
    },
};
use actix_identity::Identity;
use actix_web::{
    dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, FromRequest, HttpMessage, HttpRequest,
};
use futures_util::future::LocalBoxFuture;
use sentry::Transaction;
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
            sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone().into())));

            let pool = req.app_data::<web::Data<Pool>>().unwrap().to_owned();

            let get_user_span = transaction.start_child("get_user", "Getting user");

            let (http_req, pl) = req.parts_mut();
            let user = get_user(http_req, pl, transaction.clone()).await;
            if let Some(ref user) = user {
                req.extensions_mut().insert(user.clone());
            };

            get_user_span.finish();

            let org_id = match req.headers().get("TR-Dataset") {
                Some(dataset_header) => {
                    let dataset_id = dataset_header
                        .to_str()
                        .map_err(|_| {
                            ServiceError::BadRequest("Dataset must be valid string".to_string())
                        })?
                        .to_string();

                    let get_dataset_and_org_span = transaction
                        .start_child("get_dataset_and_org", "Getting dataset and organization");

                    let dataset_org_plan_sub = match dataset_id.parse::<uuid::Uuid>() {
                        Ok(dataset_id) => {
                            get_dataset_and_organization_from_dataset_id_query(
                                UnifiedId::TrieveUuid(dataset_id),
                                pool.clone(),
                            )
                            .await?
                        }
                        Err(_) => {
                            get_dataset_and_organization_from_dataset_id_query(
                                UnifiedId::TrackingId(dataset_id),
                                pool.clone(),
                            )
                            .await?
                        }
                    };

                    get_dataset_and_org_span.finish();

                    req.extensions_mut().insert(dataset_org_plan_sub.clone());

                    dataset_org_plan_sub.organization.organization.id
                }
                None => {
                    if let Some(org_header) = req.headers().get("TR-Organization") {
                        org_header
                            .to_str()
                            .map_err(|_| {
                                Into::<Error>::into(ServiceError::BadRequest(
                                    "Could not convert Organization to str".to_string(),
                                ))
                            })?
                            .parse::<uuid::Uuid>()
                            .map_err(|_| {
                                Into::<Error>::into(ServiceError::BadRequest(
                                    "Could not convert Organization to UUID".to_string(),
                                ))
                            })?
                    } else {
                        let res = srv.call(req).await?;
                        return Ok(res);
                    }
                }
            };

            if let Some(user) = user {
                let find_user_org_span =
                    transaction.start_child("find_user_org_role", "Finding user org role");

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
                    log::error!("User does not have permission to access this organization");
                    Err(ServiceError::Forbidden)
                }?;

                req.extensions_mut().insert(role);

                find_user_org_span.finish();
            }

            transaction.finish();

            let res = srv.call(req).await?;

            Ok(res)
        })
    }
}

async fn get_user(req: &HttpRequest, pl: &mut Payload, tx: Transaction) -> Option<LoggedUser> {
    let get_user_from_identity_span =
        tx.start_child("get_user_from_identity", "Getting user from identity");
    if let Ok(identity) = Identity::from_request(req, pl).into_inner() {
        if let Ok(user_json) = identity.id() {
            if let Ok(user) = serde_json::from_str::<LoggedUser>(&user_json) {
                return Some(user);
            }
        }
    }
    get_user_from_identity_span.finish();

    if let Some(authen_header) = req.headers().get("Authorization") {
        if let Ok(authen_header) = authen_header.to_str() {
            if authen_header == std::env::var("ADMIN_API_KEY").unwrap_or("".to_string()) {
                if let Some(org_id_header) = req.headers().get("TR-Organization") {
                    if let Ok(org_id) = org_id_header.to_str() {
                        if let Ok(org_id) = org_id.parse::<uuid::Uuid>() {
                            if let Some(pool) = req.app_data::<web::Data<Pool>>() {
                                if let Ok(user) =
                                    get_arbitrary_org_owner_from_org_id(org_id, pool.clone()).await
                                {
                                    return Some(user);
                                }
                            }
                        }
                    }
                }

                if let Some(dataset_id_header) = req.headers().get("TR-Dataset") {
                    if let Ok(dataset_id) = dataset_id_header.to_str() {
                        if let Ok(dataset_id) = dataset_id.parse::<uuid::Uuid>() {
                            if let Some(pool) = req.app_data::<web::Data<Pool>>() {
                                if let Ok(user) = get_arbitrary_org_owner_from_dataset_id(
                                    dataset_id,
                                    pool.clone(),
                                )
                                .await
                                {
                                    return Some(user);
                                }
                            }
                        }
                    }
                }
            }

            let get_user_from_api_key_span =
                tx.start_child("get_user_from_api_key", "Getting user from api key");
            //TODO: Cache the api key in redis
            if let Some(pool) = req.app_data::<web::Data<Pool>>() {
                if let Ok(user) = get_user_from_api_key_query(authen_header, pool).await {
                    return Some(user);
                }
            }
            get_user_from_api_key_span.finish();
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
