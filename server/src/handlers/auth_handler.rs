use crate::data::models::{DatasetAndOrgWithSubAndPlan, Organization, StripePlan, UserRole};
use crate::get_env;
use crate::operators::organization_operator::{get_organization_by_id_query, get_user_org_count};
use crate::operators::user_operator::create_user_query;
use crate::{
    data::models::{Pool, ServerDatasetConfiguration, SlimUser, User, UserOrganization},
    errors::ServiceError,
    operators::{
        self,
        invitation_operator::get_invitation_by_id_query,
        organization_operator::{create_organization_query, get_org_from_dataset_id_query},
        user_operator::{get_user_by_id_query, get_user_from_api_key_query},
    },
};
use actix_identity::Identity;
use actix_session::Session;
use actix_web::{
    dev::Payload, web, Error, FromRequest, HttpMessage as _, HttpRequest, HttpResponse,
};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, Scope, TokenResponse,
};
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata};
use openidconnect::{AccessTokenHash, ClientId, IssuerUrl, Nonce};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::future::{ready, Ready};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct AuthData {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct OpCallback {
    pub state: String,
    pub session_state: String,
    pub code: String,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
struct AFClaims {}

pub type LoggedUser = SlimUser;

impl FromRequest for LoggedUser {
    type Error = Error;
    type Future = Ready<Result<LoggedUser, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        if let Ok(identity) = Identity::from_request(req, pl).into_inner() {
            if let Ok(user_json) = identity.id() {
                if let Ok(user) = serde_json::from_str::<LoggedUser>(&user_json) {
                    req.extensions_mut().insert(user.clone());
                    return ready(Ok(user));
                }
            }
        }

        if let Some(authen_header) = req.headers().get("Authorization") {
            if let Ok(authen_header) = authen_header.to_str() {
                if let Some(pool) = req.app_data::<web::Data<Pool>>() {
                    if let Ok(user) = get_user_from_api_key_query(authen_header, pool) {
                        req.extensions_mut().insert(user.clone());
                        return ready(Ok(user));
                    }
                }
            }
        }

        ready(Err(ServiceError::Unauthorized.into()))
    }
}

pub struct AdminOnly(pub SlimUser);

impl FromRequest for AdminOnly {
    type Error = Error;
    type Future = Ready<Result<AdminOnly, Error>>;

    // TODO: rewrite this logic to pull current org and then the role from there

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        let ext = req.extensions();

        let org_id = if req.headers().get("AF-Organization").is_some() {
            let org_id = req
                .headers()
                .get("AF-Organization")
                .unwrap()
                .to_str()
                .map_err(|_| {
                    ready(Err::<&str, Error>(
                        ServiceError::InternalServerError("Could not get org id".into()).into(),
                    ))
                })
                .unwrap()
                .parse::<uuid::Uuid>()
                .map_err(|_| {
                    ready(Err::<&str, Error>(
                        ServiceError::InternalServerError("Could not get org id".into()).into(),
                    ))
                })
                .unwrap();
            org_id
        } else {
            let dataset_org_plan_sub = match ext.get::<DatasetAndOrgWithSubAndPlan>() {
                Some(dataset_org_plan_sub) => dataset_org_plan_sub.clone(),
                None => {
                    return ready(Err(ServiceError::InternalServerError(
                        "Could not get dataset and org from request".to_string(),
                    )
                    .into()))
                }
            };
            dataset_org_plan_sub.organization.id
        };

        if let Ok(identity) = Identity::from_request(req, pl).into_inner() {
            if let Ok(user_json) = identity.id() {
                if let Ok(user) = serde_json::from_str::<LoggedUser>(&user_json) {
                    let user_org = match user
                        .user_orgs
                        .iter()
                        .find(|org| org.organization_id == org_id)
                    {
                        Some(user_org) => user_org,
                        None => return ready(Err(ServiceError::Forbidden.into())),
                    };

                    if user_org.role >= UserRole::Admin.into() {
                        return ready(Ok(AdminOnly(user)));
                    };
                }
            }
        }

        if let Some(authen_header) = req.headers().get("Authorization") {
            if let Ok(authen_header) = authen_header.to_str() {
                if let Some(pool) = req.app_data::<web::Data<Pool>>() {
                    if let Ok(user) = get_user_from_api_key_query(authen_header, pool) {
                        let user_org = match user
                            .user_orgs
                            .iter()
                            .find(|org| org.organization_id == org_id)
                        {
                            Some(user_org) => user_org,
                            None => return ready(Err(ServiceError::Forbidden.into())),
                        };

                        if user_org.role >= UserRole::Admin.into() {
                            return ready(Ok(AdminOnly(user)));
                        };
                    }
                }
            }
        }

        ready(Err(ServiceError::Unauthorized.into()))
    }
}

pub struct OwnerOnly(pub SlimUser);

impl FromRequest for OwnerOnly {
    type Error = Error;
    type Future = Ready<Result<OwnerOnly, Error>>;

    fn from_request(req: &HttpRequest, pl: &mut Payload) -> Self::Future {
        let ext = req.extensions();
        let org_id = if req.headers().get("AF-Organization").is_some() {
            let org_id = req
                .headers()
                .get("AF-Organization")
                .unwrap()
                .to_str()
                .map_err(|_| {
                    Err::<&str, Error>(
                        ServiceError::InternalServerError("Could not get org id".into()).into(),
                    )
                })
                .unwrap()
                .parse::<uuid::Uuid>()
                .map_err(|_| {
                    Err::<&str, Error>(
                        ServiceError::InternalServerError("Could not get org id".into()).into(),
                    )
                })
                .unwrap();
            org_id
        } else {
            let dataset_org_plan_sub = match ext.get::<DatasetAndOrgWithSubAndPlan>() {
                Some(dataset_org_plan_sub) => dataset_org_plan_sub.clone(),
                None => {
                    return ready(Err(ServiceError::InternalServerError(
                        "Could not get dataset and org from request".to_string(),
                    )
                    .into()))
                }
            };
            dataset_org_plan_sub.organization.id
        };

        if let Ok(identity) = Identity::from_request(req, pl).into_inner() {
            if let Ok(user_json) = identity.id() {
                if let Ok(user) = serde_json::from_str::<LoggedUser>(&user_json) {
                    let user_org = match user
                        .user_orgs
                        .iter()
                        .find(|org| org.organization_id == org_id)
                    {
                        Some(user_org) => user_org,
                        None => return ready(Err(ServiceError::Forbidden.into())),
                    };

                    if user_org.role >= UserRole::Owner.into() {
                        return ready(Ok(OwnerOnly(user)));
                    };
                }
            }
        }

        if let Some(authen_header) = req.headers().get("Authorization") {
            if let Ok(authen_header) = authen_header.to_str() {
                if let Some(pool) = req.app_data::<web::Data<Pool>>() {
                    if let Ok(user) = get_user_from_api_key_query(authen_header, pool) {
                        let user_org = match user
                            .user_orgs
                            .iter()
                            .find(|org| org.organization_id == org_id)
                        {
                            Some(user_org) => user_org,
                            None => return ready(Err(ServiceError::Forbidden.into())),
                        };

                        if user_org.role >= UserRole::Owner.into() {
                            return ready(Ok(OwnerOnly(user)));
                        };
                    }
                }
            }
        }

        ready(Err(ServiceError::Unauthorized.into()))
    }
}

pub async fn build_oidc_client() -> CoreClient {
    let issuer_url = get_env!(
        "OIDC_ISSUER_URL",
        "Issuer URL for OpenID provider must be set"
    )
    .to_string();
    let client_id = get_env!(
        "OIDC_CLIENT_ID",
        "Client ID for OpenID provider must be set"
    )
    .to_string();
    let auth_redirect_url = get_env!(
        "OIDC_AUTH_REDIRECT_URL",
        "Auth redirect URL for OpenID provider must be set"
    )
    .to_string();
    let client_secret = get_env!(
        "OIDC_CLIENT_SECRET",
        "Client secret for OpenID provider must be set"
    )
    .to_string();

    //build OpenId Connect client
    let meta_data = CoreProviderMetadata::discover_async(
        IssuerUrl::new(issuer_url.clone()).expect("IssuerUrl for OpenID provider must be set"),
        async_http_client,
    )
    .await
    .expect("Failed to discover OpenID provider");

    CoreClient::new(
        ClientId::new(client_id.clone()),
        Some(ClientSecret::new(client_secret.clone())),
        IssuerUrl::new(issuer_url.clone()).expect("IssuerUrl for OpenID provider must be set"),
        AuthUrl::new(auth_redirect_url.clone()).expect("Auth configuration is not a valid URL"),
        meta_data.token_endpoint().cloned(),
        meta_data.userinfo_endpoint().cloned(),
        meta_data.jwks().to_owned(),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8090/api/auth/callback".to_string())
            .expect("Redirect URL for OpenID provider must be set"),
    )
}

pub async fn create_account(
    email: String,
    name: String,
    user_id: uuid::Uuid,
    dataset_id: Option<uuid::Uuid>,
    inv_code: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<(User, Vec<UserOrganization>, Vec<Organization>), ServiceError> {
    let (owner, org) = match dataset_id {
        Some(dataset_id) => (false, get_org_from_dataset_id_query(dataset_id, pool.clone())
            .await
            .map_err(|error| {
                ServiceError::InternalServerError(
                    error.message.to_string()
                )
            })?),
        None => {
            let org_name = email.split('@').collect::<Vec<&str>>()[0].to_string().replace(" ", "-");
            (true, create_organization_query(org_name.as_str(), json!({}), pool.clone())
                .await
                .map_err(|error| {
                    ServiceError::InternalServerError(
                        error.message.to_string()
                    )
                })?)
        }
    };
    let org_id = org.id;

    let org_plan_sub = get_organization_by_id_query(org_id, pool.clone())
        .await
        .map_err(|error| {
            ServiceError::InternalServerError(error.message.to_string())
        })?;
    let user_org_count_pool = pool.clone();
    let user_org_count = web::block(move || get_user_org_count(org_id, user_org_count_pool))
        .await
        .map_err(|_| {
            ServiceError::InternalServerError("Blocking error getting org user count".to_string())
        })?
        .map_err(|err| ServiceError::InternalServerError(err.message.to_string()))?;
    if user_org_count
        >= org_plan_sub
            .plan
            .unwrap_or(StripePlan::default())
            .user_count
    {
        return Err(ServiceError::BadRequest(
            "User limit reached for organization, must upgrade plan to add more users".to_string(),
        ));
    }

    if org.registerable == Some(false) {
        if let Some(inv_code) = inv_code {
            let invitation = get_invitation_by_id_query(inv_code, pool.clone())
                .await
                .map_err(|_| {
                    ServiceError::InternalServerError(
                        "Could not find invitation for user".to_string(),
                    )
                })?;

            if invitation.email != email {
                return Err(ServiceError::BadRequest(
                    "Email does not match invitation".to_string(),
                ));
            }

            if invitation.dataset_id != dataset_id.unwrap() {
                return Err(ServiceError::BadRequest(
                    "Dataset ID does not match invitation".to_string(),
                ));
            }

            if invitation.expired() {
                return Err(ServiceError::BadRequest(
                    "Invitation has expired".to_string(),
                ));
            }

            if invitation.used {
                return Err(ServiceError::BadRequest(
                    "Invitation has already been used".to_string(),
                ));
            }
        } else {
            return Err(ServiceError::BadRequest(
                "This organization is not registerable".to_string(),
            ));
        }
    }

    let user_org =
        web::block(move || create_user_query(user_id, email, Some(name), owner, org_id, pool))
            .await
            .map_err(|_| {
                ServiceError::InternalServerError("Blocking error creating user".to_string())
            })?
            .map_err(|err| ServiceError::InternalServerError(err.message.to_string()))?;

    Ok(user_org)
}

#[utoipa::path(
    delete,
    path = "/auth",
    context_path = "/api",
    tag = "auth",
    responses(
        (status = 204, description = "Confirmation that your current auth credentials have been cleared"),
    )
)]
pub async fn logout(id: Identity) -> HttpResponse {
    id.logout();
    HttpResponse::NoContent().finish()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenIdConnectState {
    pub pkce_verifier: PkceCodeVerifier,
    pub csrf_token: CsrfToken,
    pub nonce: Nonce,
}

const OIDC_SESSION_KEY: &str = "oidc_state";

#[derive(Deserialize, Debug)]
pub struct AuthQuery {
    pub dataset_id: Option<uuid::Uuid>,
    pub redirect_uri: Option<String>,
    pub inv_code: Option<uuid::Uuid>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginState {
    pub redirect_uri: String,
    pub dataset_id: Option<uuid::Uuid>,
    pub inv_code: Option<uuid::Uuid>,
}

#[utoipa::path(
    post,
    path = "/auth",
    context_path = "/api",
    tag = "auth",
    responses(
        (status = 200, description = "Response that redirects to OAuth provider"),
        (status = 400, description = "OAuth Error", body = [DefaultError]),
    )
)]
pub async fn login(
    req: HttpRequest,
    session: Session,
    data: web::Query<AuthQuery>,
    oidc_client: web::Data<CoreClient>,
) -> Result<HttpResponse, Error> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (auth_url, csrf_token, nonce) = oidc_client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_string()))
        .set_pkce_challenge(pkce_challenge)
        .url();

    let oidc_state = OpenIdConnectState {
        pkce_verifier,
        csrf_token,
        nonce,
    };

    session
        .insert(OIDC_SESSION_KEY, oidc_state)
        .map_err(|_| ServiceError::InternalServerError("Could not set OIDC Session".into()))?;

    let redirect_uri = match data.redirect_uri.clone() {
        Some(redirect_uri) => redirect_uri,
        None => req
            .headers()
            .get("Referer")
            .map(|h| h.to_str().unwrap_or("/"))
            .unwrap_or("/")
            .to_string(),
    };

    let login_state = LoginState {
        redirect_uri,
        dataset_id: data.dataset_id,
        inv_code: data.inv_code,
    };

    session
        .insert("login_state", login_state)
        .map_err(|_| ServiceError::InternalServerError("Could not set redirect url".into()))?;

    //redirect to OpenIdProvider for authentication
    Ok(HttpResponse::SeeOther()
        .insert_header(("Location", auth_url.as_str()))
        .finish())
}

#[utoipa::path(
    get,
    path = "/auth/callback",
    context_path = "/api",
    tag = "auth",
    responses(
        (status = 200, description = "Response that returns with set-cookie header", body = [SlimUser]),
        (status = 400, description = "Email or password empty or incorrect", body = [DefaultError]),
    )
)]
pub async fn callback(
    req: HttpRequest,
    session: Session,
    oidc_client: web::Data<CoreClient>,
    pool: web::Data<Pool>,
    query: web::Query<OpCallback>,
) -> Result<HttpResponse, Error> {
    let state: OpenIdConnectState = session
        .get(OIDC_SESSION_KEY)
        .map_err(|_| ServiceError::InternalServerError("Could not get OIDC Session".into()))?
        .ok_or(ServiceError::Unauthorized)?;

    let code_verifier = state.pkce_verifier;
    let code = query.code.clone();
    let nonce = state.nonce;

    let token_response = oidc_client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(code_verifier)
        .request_async(async_http_client)
        .await
        .map_err(|e| match e {
            oauth2::RequestTokenError::ServerResponse(e) => {
                ServiceError::InternalServerError(e.to_string())
            }
            _ => ServiceError::InternalServerError("Unknown error".into()),
        })?;

    let id_token = token_response
        .extra_fields()
        .id_token()
        .ok_or_else(|| ServiceError::InternalServerError("Empty ID Token".into()))?;

    let id_token_verifier = oidc_client.id_token_verifier();
    let claims = id_token
        .claims(&id_token_verifier, &nonce)
        .map_err(|_| ServiceError::InternalServerError("Claims Verification Error".into()))?;

    match claims.access_token_hash() {
        None => Err(ServiceError::BadRequest(
            "Missing access token hash".to_string(),
        ))?,
        Some(given_token_hash) => {
            let calculated_token_hash = AccessTokenHash::from_token(
                token_response.access_token(),
                &id_token.signing_alg().map_err(|_| {
                    ServiceError::BadRequest("ID token hash unavailable".to_string())
                })?,
            )
            .map_err(|_| ServiceError::BadRequest("ID token hash unavailable".to_string()))?;

            if calculated_token_hash != *given_token_hash {
                Err(ServiceError::BadRequest(
                    "ID token hash invalid".to_string(),
                ))
            } else {
                Ok(())
            }
        }
    }?;

    let user_id = claims
        .subject()
        .to_string()
        .parse::<uuid::Uuid>()
        .map_err(|_| {
            ServiceError::InternalServerError("Failed to parse user ID from claims".into())
        })?;

    let email = claims.email().ok_or_else(|| {
        ServiceError::InternalServerError("Failed to parse email from claims".into())
    })?;

    let name = claims.name().ok_or_else(|| {
        ServiceError::InternalServerError("Failed to parse name from claims".into())
    })?;

    let login_state = session
        .get::<LoginState>("login_state")
        .map_err(|_| ServiceError::InternalServerError("Could not get redirect url".into()))?
        .ok_or(ServiceError::Unauthorized)?;

    let user = match get_user_by_id_query(&user_id, pool.clone()) {
        Ok(user) => user,
        Err(_) => {
            create_account(
                email.to_string(),
                name.iter().next().unwrap().1.to_string(),
                user_id,
                login_state.dataset_id,
                login_state.inv_code,
                pool,
            )
            .await?
        }
    };

    let slim_user: SlimUser = SlimUser::from_details(user.0, user.1, user.2);

    let user_string = serde_json::to_string(&slim_user).map_err(|_| {
        ServiceError::InternalServerError("Failed to serialize user to JSON".into())
    })?;

    Identity::login(&req.extensions(), user_string).unwrap();

    session.remove(OIDC_SESSION_KEY);
    session.remove("login_state");

    Ok(HttpResponse::SeeOther()
        .insert_header(("Location", login_state.redirect_uri))
        .finish())
}

#[utoipa::path(
    get,
    path = "/auth",
    context_path = "/api",
    tag = "auth",
    responses(
        (status = 200, description = "The user corresponding to your current auth credentials", body = [SlimUser]),
        (status = 400, description = "Error message indicitating you are not currently signed in", body = [DefaultError]),
    )
)]
pub async fn get_me(
    logged_user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_query_id: uuid::Uuid = logged_user.id;

    let user_result = web::block(move || get_user_by_id_query(&user_query_id, pool)).await?;

    match user_result {
        Ok(user) => Ok(HttpResponse::Ok().json(SlimUser::from_details(user.0, user.1, user.2))),
        Err(e) => Ok(HttpResponse::BadRequest().json(e)),
    }
}

#[utoipa::path(
    get,
    path = "/health",
    context_path = "/api",
    tag = "health",
    responses(
        (status = 200, description = "Confirmation that the service is healthy and can make embedding vectors"),
        (status = 400, description = "Service error relating to making an embedding or overall service health", body = [DefaultError]),
    ),
)]
pub async fn health_check(
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let result = operators::model_operator::create_embedding(
        "health check",
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration),
    )
    .await;

    result?;
    Ok(HttpResponse::Ok().finish())
}
