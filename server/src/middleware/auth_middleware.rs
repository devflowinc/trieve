use crate::{
    data::models::{
        ApiKeyRequestParams, Pool, RedisPool, SlimUser, UnifiedId, User, UserApiKey, UserRole,
    },
    errors::ServiceError,
    handlers::{
        auth_handler::{AdminOnly, LoggedUser, OrganizationRole, OwnerOnly},
        chunk_handler::{AutocompleteReqPayload, ScrollChunksReqPayload, SearchChunksReqPayload},
        group_handler::{
            AutocompleteSearchOverGroupsReqPayload, SearchOverGroupsReqPayload,
            SearchWithinGroupReqPayload,
        },
        message_handler::CreateMessageReqPayload,
    },
    operators::{
        dataset_operator::get_dataset_and_organization_from_dataset_id_query,
        organization_operator::{
            get_arbitrary_org_owner_from_dataset_id, get_arbitrary_org_owner_from_org_id,
            get_assumed_user_by_organization_api_key, get_org_from_id_query,
        },
        user_operator::{get_user_by_id_query, get_user_from_api_key_query},
    },
};
use actix_identity::Identity;
use actix_web::{
    dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    http::header::HeaderMap,
    web::{self, Json},
    Error, FromRequest, HttpMessage,
};
use core::error;
use futures_util::future::LocalBoxFuture;
use redis::AsyncCommands;
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
            let pool = req.app_data::<web::Data<Pool>>().unwrap().to_owned();

            let mut user = get_user(&mut req, pool.clone()).await;
            let mut api_key = None;
            if user.is_none() {
                (user, api_key) = auth_with_api_key(&mut req, pool.clone()).await?;
            }

            if let Some(user) = user.clone() {
                req.extensions_mut().insert(user);
            }

            let org_id = match get_dataset_id_from_headers(req.headers()) {
                Some(dataset_id) => {
                    let dataset_org_plan_sub = match dataset_id.parse::<uuid::Uuid>() {
                        Ok(dataset_id) => {
                            get_dataset_and_organization_from_dataset_id_query(
                                UnifiedId::TrieveUuid(dataset_id),
                                None,
                                pool.clone(),
                            )
                            .await?
                        }
                        Err(_) => {
                            if let Some(org_header) = get_org_id_from_headers(req.headers()) {
                                let org_id = org_header.parse::<uuid::Uuid>().map_err(|_| {
                                    Into::<Error>::into(ServiceError::BadRequest(
                                        "Could not convert Organization to UUID".to_string(),
                                    ))
                                })?;

                                get_dataset_and_organization_from_dataset_id_query(
                                    UnifiedId::TrackingId(dataset_id.clone()),
                                    Some(org_id),
                                    pool.clone(),
                                )
                                .await?
                            } else {
                                return Err(ServiceError::BadRequest(
                                "Using Dataset Tracking IDs requires providing the TR-Organization header".to_string(),
                            ).into());
                            }
                        }
                    };

                    if let Some(user_api_key) = api_key {
                        if let Some(api_key_org_ids) = user_api_key.organization_ids {
                            if !api_key_org_ids.is_empty()
                                && !api_key_org_ids.contains(&Some(
                                    dataset_org_plan_sub
                                        .organization
                                        .organization
                                        .id
                                        .to_string(),
                                ))
                            {
                                return Err(ServiceError::Unauthorized.into());
                            }
                        }

                        if let Some(api_key_dataset_ids) = user_api_key.dataset_ids {
                            if !api_key_dataset_ids.is_empty()
                                && !api_key_dataset_ids
                                    .contains(&Some(dataset_org_plan_sub.dataset.id.to_string()))
                            {
                                return Err(ServiceError::Unauthorized.into());
                            }
                        }

                        let route = format!("{} {}", req.method(), req.match_info().as_str());
                        if let Some(api_key_scopes) = user_api_key.scopes.as_ref() {
                            if check_scopes(api_key_scopes, &route) {
                                if let Some(ref mut user) = user {
                                    user.user_orgs.iter_mut().for_each(|org| {
                                        if org.organization_id
                                            == dataset_org_plan_sub.organization.organization.id
                                        {
                                            org.role = UserRole::Admin.into();
                                        }
                                    });
                                }
                            }
                        }
                    }

                    req.extensions_mut().insert(dataset_org_plan_sub.clone());
                    req.extensions_mut()
                        .insert(dataset_org_plan_sub.organization.clone());

                    dataset_org_plan_sub.organization.organization.id
                }
                None => {
                    if let Some(org_header) = get_org_id_from_headers(req.headers()) {
                        let org_id = org_header.parse::<uuid::Uuid>().map_err(|_| {
                            Into::<Error>::into(ServiceError::BadRequest(
                                "Could not convert Organization to UUID".to_string(),
                            ))
                        })?;
                        let org_plan_and_sub = get_org_from_id_query(org_id, pool.clone()).await?;
                        req.extensions_mut().insert(org_plan_and_sub.clone());
                        org_id
                    } else {
                        let res = srv.call(req).await?;
                        return Ok(res);
                    }
                }
            };

            if let Some(ref mut user) = user {
                let return_user = user.clone();
                let user_org = user
                    .user_orgs
                    .iter_mut()
                    .find(|org| org.organization_id == org_id)
                    .ok_or(ServiceError::Forbidden)?;

                let route = format!("{} {}", req.method(), req.match_info().as_str());
                if let Some(scopes) = &user_org.scopes {
                    if check_scopes(scopes, &route) {
                        user_org.role = UserRole::Owner.into();
                    } else {
                        return Err(ServiceError::Forbidden.into());
                    }
                }

                let org_role = if user_org.role >= UserRole::User.into() {
                    Ok(OrganizationRole {
                        user: return_user,
                        role: UserRole::from(user_org.role),
                    })
                } else {
                    Err(ServiceError::Forbidden)
                }?;

                req.extensions_mut().insert(org_role);
            }

            let res = srv.call(req).await?;

            Ok(res)
        })
    }
}

async fn get_user(req: &mut ServiceRequest, pool: web::Data<Pool>) -> Option<LoggedUser> {
    let (http_req, pl) = req.parts_mut();
    if let Ok(identity) = Identity::from_request(http_req, pl).into_inner() {
        if let Ok(user_json) = identity.id() {
            if let Ok(user) = serde_json::from_str::<User>(&user_json) {
                let redis_pool = req.app_data::<web::Data<RedisPool>>().unwrap().to_owned();

                let mut redis_conn = redis_pool.get().await.ok()?;

                let slim_user_string: Result<String, _> = redis_conn.get(user.id.to_string()).await;

                match slim_user_string {
                    Ok(slim_user_string) => {
                        let slim_user = serde_json::from_str::<SlimUser>(&slim_user_string).ok()?;

                        return Some(slim_user);
                    }
                    Err(_) => {
                        let (user, user_orgs, orgs) =
                            get_user_by_id_query(&user.id, pool).await.ok()?;
                        let slim_user = SlimUser::from_details(user, user_orgs, orgs);

                        let slim_user_string = serde_json::to_string(&slim_user).ok()?;
                        redis_conn
                            .set::<_, _, ()>(slim_user.id.to_string(), slim_user_string)
                            .await
                            .ok()?;

                        return Some(slim_user);
                    }
                }
            }
        }
    }

    None
}

// Can either be Bearer {}, or x-api-key, or Authorization
fn get_api_key_from_headers(headers: &HeaderMap) -> Option<String> {
    if let Some(auth_header_value) = headers.get("Authorization") {
        // Check if the Authorization header is a Bearer token
        if let Ok(auth_header_value) = auth_header_value.to_str() {
            if let Some(stripeped_auth_header) = auth_header_value.strip_prefix("Bearer ") {
                return Some(stripeped_auth_header.to_string());
            } else {
                return Some(auth_header_value.to_string());
            }
        }
    }
    // Check for x-api-key,
    if let Some(api_key_header_value) = headers.get("x-api-key") {
        if let Ok(api_key_header_value) = api_key_header_value.to_str() {
            if let Some(stripeped_api_key_header_value) =
                api_key_header_value.strip_prefix("Bearer ")
            {
                return Some(stripeped_api_key_header_value.to_string());
            } else {
                return Some(api_key_header_value.to_string());
            }
        }
    }

    None
}

fn get_dataset_id_from_headers(headers: &HeaderMap) -> Option<String> {
    if let Some(dataset_id_header) = headers.get("TR-Dataset") {
        if let Ok(dataset_id) = dataset_id_header.to_str() {
            return Some(dataset_id.to_string());
        }
    }

    if let Some(dataset_id_header) = headers.get("x-dataset") {
        if let Ok(dataset_id) = dataset_id_header.to_str() {
            return Some(dataset_id.to_string());
        }
    }

    None
}

fn get_org_id_from_headers(headers: &HeaderMap) -> Option<String> {
    if let Some(org_id_header) = headers.get("TR-Organization") {
        if let Ok(org_id) = org_id_header.to_str() {
            return Some(org_id.to_string());
        }
    }

    if let Some(org_id_header) = headers.get("x-organization") {
        if let Ok(org_id) = org_id_header.to_str() {
            return Some(org_id.to_string());
        }
    }

    None
}

// Helper function to check if API key scopes match the route
fn check_scopes(scopes: &[Option<String>], route: &str) -> bool {
    if scopes.is_empty() {
        return false;
    }

    let curly_matcher = regex::Regex::new(r"\{[a-zA-Z0-9_-]+\}").expect("Valid regex");

    scopes.contains(&Some(route.to_string()))
        || scopes
            .iter()
            .filter_map(|scope| scope.as_ref())
            .any(|scope| {
                let wildcard_scope = curly_matcher
                    .replace_all(scope, "[a-zA-Z0-9_-]+")
                    .to_string();
                if let Ok(wildcard_scope_regex) = regex::Regex::new(&wildcard_scope) {
                    wildcard_scope_regex.is_match(route)
                } else {
                    false
                }
            })
}

async fn auth_with_api_key(
    req: &mut ServiceRequest,
    pool: web::Data<Pool>,
) -> Result<(Option<LoggedUser>, Option<UserApiKey>), ServiceError> {
    if let Some(authen_header) = get_api_key_from_headers(req.headers()) {
        if authen_header == std::env::var("ADMIN_API_KEY").unwrap_or("".to_string()) {
            if let Some(org_id) = get_org_id_from_headers(req.headers()) {
                if let Ok(org_id) = org_id.parse::<uuid::Uuid>() {
                    if let Ok(user) =
                        get_arbitrary_org_owner_from_org_id(org_id, pool.clone()).await
                    {
                        return Ok((Some(user), None));
                    }
                }
            }

            if let Some(dataset_id) = get_dataset_id_from_headers(req.headers()) {
                if let Ok(dataset_id) = dataset_id.parse::<uuid::Uuid>() {
                    if let Ok(user) =
                        get_arbitrary_org_owner_from_dataset_id(dataset_id, pool.clone()).await
                    {
                        return Ok((Some(user), None));
                    }
                }
            }
        }

        if let Ok((user, api_key)) =
            get_assumed_user_by_organization_api_key(authen_header.as_str(), pool.clone()).await
        {
            if let Some(ref api_key_params) = api_key.params {
                let params = serde_json::from_value(api_key_params.clone()).unwrap();
                insert_api_key_payload(req, params).await.map_err(|_| {
                    ServiceError::BadRequest("Could not insert api key payload".to_string())
                })?;
            }
            return Ok((Some(user), Some(api_key)));
        }

        if let Ok((user, api_key)) =
            get_user_from_api_key_query(authen_header.as_str(), pool.clone()).await
        {
            if let Some(ref api_key_params) = api_key.params {
                let params = serde_json::from_value(api_key_params.clone()).unwrap();
                insert_api_key_payload(req, params).await.map_err(|_| {
                    ServiceError::BadRequest("Could not insert api key payload".to_string())
                })?;
            }
            return Ok((Some(user), Some(api_key)));
        }
    }

    Ok((None, None))
}

fn bytes_to_payload(buf: web::Bytes) -> Payload {
    let (_, mut pl) = actix_http::h1::Payload::create(true);
    pl.unread_data(buf);
    Payload::from(pl)
}

pub async fn insert_api_key_payload(
    req: &mut ServiceRequest,
    api_key_params: ApiKeyRequestParams,
) -> Result<(), Box<dyn error::Error>> {
    match req.path() {
        "/api/chunk/autocomplete" => {
            let body = req.extract::<Json<AutocompleteReqPayload>>().await?;
            let new_body = api_key_params.combine_with_autocomplete(body.into_inner());
            let body_bytes = serde_json::to_vec(&web::Json(new_body)).unwrap();
            req.set_payload(bytes_to_payload(body_bytes.into()));
        }
        "/api/chunk/search" => {
            let body = req.extract::<Json<SearchChunksReqPayload>>().await?;
            log::info!("api_key_req_params: {:?}", api_key_params);
            let new_body = api_key_params.combine_with_search_chunks(body.into_inner());
            log::info!("new_body: {:?}", new_body);
            let body_bytes = serde_json::to_vec(&web::Json(new_body)).unwrap();
            req.set_payload(bytes_to_payload(body_bytes.into()));
        }
        "/api/chunk_group/group_oriented_search" => {
            let body = req.extract::<Json<SearchOverGroupsReqPayload>>().await?;
            let new_body = api_key_params.combine_with_search_over_groups(body.into_inner());
            let body_bytes = serde_json::to_vec(&web::Json(new_body)).unwrap();
            req.set_payload(bytes_to_payload(body_bytes.into()));
        }
        "/api/chunk_group/group_oriented_autocomplete" => {
            let body = req
                .extract::<Json<AutocompleteSearchOverGroupsReqPayload>>()
                .await?;
            let new_body =
                api_key_params.combine_with_autocomplete_search_over_groups(body.into_inner());
            let body_bytes = serde_json::to_vec(&web::Json(new_body)).unwrap();
            req.set_payload(bytes_to_payload(body_bytes.into()));
        }
        "/api/chunk_group/search" => {
            let body = req.extract::<Json<SearchWithinGroupReqPayload>>().await?;
            let new_body = api_key_params.combine_with_search_within_group(body.into_inner());
            let body_bytes = serde_json::to_vec(&web::Json(new_body)).unwrap();
            req.set_payload(bytes_to_payload(body_bytes.into()));
        }
        "/api/message" => {
            let body = req.extract::<Json<CreateMessageReqPayload>>().await?;
            let new_body = api_key_params.combine_with_create_message(body.into_inner());
            let body_bytes = serde_json::to_vec(&web::Json(new_body)).unwrap();
            req.set_payload(bytes_to_payload(body_bytes.into()));
        }
        "/api/chunks/scroll" => {
            let body = req.extract::<Json<ScrollChunksReqPayload>>().await?;
            let new_body = api_key_params.combine_with_scroll_chunks(body.into_inner());
            let body_bytes = serde_json::to_vec(&web::Json(new_body)).unwrap();
            req.set_payload(bytes_to_payload(body_bytes.into()));
        }
        _ => {}
    };

    Ok(())
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

pub fn get_role_for_org(user: &SlimUser, org_id: &uuid::Uuid) -> Option<UserRole> {
    user.user_orgs
        .iter()
        .find(|user_org| user_org.organization_id == *org_id)
        .map(|user_org| UserRole::from(user_org.role))
}

pub fn verify_owner(user: &OwnerOnly, org_id: &uuid::Uuid) -> bool {
    if let Some(user_role) = get_role_for_org(&user.0, org_id) {
        return user_role >= UserRole::Owner;
    }

    false
}

pub fn verify_admin(user: &AdminOnly, org_id: &uuid::Uuid) -> bool {
    if let Some(user_role) = get_role_for_org(&user.0, org_id) {
        return user_role >= UserRole::Admin;
    }

    false
}

pub fn verify_member(user: &LoggedUser, org_id: &uuid::Uuid) -> bool {
    if let Some(user_role) = get_role_for_org(user, org_id) {
        return user_role >= UserRole::User;
    }

    false
}
