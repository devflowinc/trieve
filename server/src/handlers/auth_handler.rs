use crate::data::models::{ApiKeyRole, Organization, RedisPool, UserRole};
use crate::get_env;
use crate::handlers::shopify_handler::ShopifyCustomerEvent;
use crate::operators::dittofeed_operator::{
    get_user_ditto_identity, send_user_ditto_identity, DittoBatchRequest, DittoBatchRequestTypes,
    DittoTrackProperties, DittoTrackRequest, DittofeedIdentifyUser, DittofeedUserTraits,
};
use crate::operators::invitation_operator::check_inv_valid;
use crate::operators::organization_operator::{
    create_organization_api_key_query, get_org_from_id_query, get_user_org_count,
};
use crate::operators::payment_operator::create_flat_subscription_query;
use crate::operators::user_operator::{
    add_user_to_organization, associate_user_to_oidc_subject_query, create_user_query,
    get_option_user_by_email_query, get_user_by_oidc_subject_query,
};
use crate::{
    data::models::{Pool, SlimUser, User, UserOrganization},
    errors::ServiceError,
    operators::{
        organization_operator::create_organization_query, user_operator::get_user_by_id_query,
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
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs::read_to_string;
use std::future::{ready, Ready};
use utoipa::{IntoParams, ToSchema};

use super::organization_handler::CreateApiKeyReqPayload;

#[derive(Deserialize, Debug)]
pub struct OpCallback {
    pub state: String,
    pub code: String,
}

pub type LoggedUser = SlimUser;

impl FromRequest for LoggedUser {
    type Error = Error;
    type Future = Ready<Result<LoggedUser, Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            req.extensions()
                .get::<LoggedUser>()
                .cloned()
                .ok_or(ServiceError::Unauthorized.into()),
        )
    }
}

#[derive(Debug, Clone)]
pub struct OrganizationRole {
    pub user: SlimUser,
    pub role: UserRole,
}

#[derive(Debug, Clone)]
pub struct AdminOnly(pub SlimUser);

impl FromRequest for AdminOnly {
    type Error = ServiceError;
    type Future = Ready<Result<Self, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let ext = req.extensions();

        match ext.get::<OrganizationRole>() {
            Some(OrganizationRole {
                user,
                role: UserRole::Owner,
            }) => ready(Ok(Self(user.clone()))),
            Some(OrganizationRole {
                user,
                role: UserRole::Admin,
            }) => ready(Ok(Self(user.clone()))),
            None => ready(Err(ServiceError::Unauthorized)),
            _ => ready(Err(ServiceError::Forbidden)),
        }
    }
}

#[derive(Debug)]
pub struct OwnerOnly(pub SlimUser);

impl FromRequest for OwnerOnly {
    type Error = ServiceError;
    type Future = Ready<Result<Self, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let ext = req.extensions();

        match ext.get::<OrganizationRole>() {
            Some(OrganizationRole {
                user,
                role: UserRole::Owner,
            }) => ready(Ok(Self(user.clone()))),
            None => ready(Err(ServiceError::Unauthorized)),
            _ => ready(Err(ServiceError::Forbidden)),
        }
    }
}

//test

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
    let base_server_url = get_env!(
        "BASE_SERVER_URL",
        "Server hostname for OpenID provider must be set"
    );

    //build OpenId Connect client
    let meta_data = CoreProviderMetadata::discover_async(
        IssuerUrl::new(issuer_url.clone()).expect("IssuerUrl for OpenID provider must be set"),
        async_http_client,
    )
    .await
    .map_err(|err| format!("Failed to discover OIDC provider {:?}", err))
    .unwrap();

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
        RedirectUrl::new(format!("{}/api/auth/callback", base_server_url))
            .expect("Redirect URL for OpenID provider must be set"),
    )
}

/// Create an account for a user given an email and name
///
/// If an organization ID is provided, the user will be added to that organization,
/// otherwise a new organization will be created with the user as the owner based on their name.
///
/// if user_oidc_subject is the subject is not provided, this user will be api only (not able to
/// access the dashboard)
pub async fn create_account(
    email: String,
    name: String,
    user_oidc_subject: Option<String>,
    organization_id: Option<uuid::Uuid>,
    inv_code: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<(User, Vec<UserOrganization>, Vec<Organization>), ServiceError> {
    let (mut role, mut scopes, org) = match organization_id {
        Some(organization_id) => (
            UserRole::User,
            None,
            get_org_from_id_query(organization_id, pool.clone())
                .await?
                .organization,
        ),
        None => {
            let org_name = email
                .split('@')
                .collect::<Vec<&str>>()
                .get(0)
                .unwrap_or(&"")
                .to_string()
                .replace(' ', "-");
            (
                UserRole::Owner,
                None,
                create_organization_query(org_name.as_str(), pool.clone()).await?,
            )
        }
    };
    let org_id = org.id;

    let org_plan_sub = get_org_from_id_query(org_id, pool.clone()).await?;
    let user_org_count_pool = pool.clone();
    let user_org_count = get_user_org_count(org_id, user_org_count_pool).await?;
    if user_org_count >= org_plan_sub.plan.unwrap_or_default().user_count() {
        return Err(ServiceError::BadRequest(
            "User limit reached for organization, must upgrade plan to add more users".to_string(),
        ));
    }

    if let Some(inv_code) = inv_code {
        let invitation =
            check_inv_valid(inv_code, email.clone(), organization_id, pool.clone()).await?;
        role = invitation.role.into();
        scopes = invitation.scopes;
    }

    let user_org = create_user_query(
        user_oidc_subject,
        email,
        Some(name),
        role,
        scopes.map(|scopes| scopes.into_iter().map(|s| s.unwrap_or_default()).collect()),
        org_id,
        pool,
    )
    .await?;

    Ok(user_org)
}

#[derive(Deserialize, Debug)]
pub struct LogoutRequest {
    pub redirect_uri: Option<String>,
}

/// Logout
///
/// Invalidate your current auth credential stored typically stored in a cookie. This does not invalidate your API key.
#[utoipa::path(
    delete,
    path = "/auth",
    context_path = "/api",
    tag = "Auth",
    responses(
        (status = 204, description = "Confirmation that your current auth token has been invalidated. This does not invalidate your API key."),
    ),
)]
pub async fn logout(
    id: Identity,
    data: web::Query<LogoutRequest>,
    req: HttpRequest,
) -> HttpResponse {
    id.logout();
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
    let logout_url = format!(
        "{}/protocol/openid-connect/logout?post_logout_redirect_uri={}&client_id={}",
        issuer_url,
        data.redirect_uri.clone().unwrap_or(
            req.headers()
                .get("Referer")
                .map_or("/", |h| h.to_str().unwrap_or("/"))
                .to_string()
        ),
        client_id
    );

    HttpResponse::Ok().json(json!({
        "logout_url": logout_url,
    }))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OpenIdConnectState {
    pub pkce_verifier: PkceCodeVerifier,
    pub csrf_token: CsrfToken,
    pub nonce: Nonce,
}

const OIDC_SESSION_KEY: &str = "oidc_state";

#[derive(Deserialize, Debug, ToSchema, IntoParams)]
#[schema(
    example = json!({"organization_id": "00000000-0000-0000-0000-000000000000", "redirect_uri": "https://api.trieve.ai", "inv_code": "00000000-0000-0000-0000-000000000000"}),
)]
pub struct AuthQuery {
    /// ID of organization to authenticate into
    pub organization_id: Option<uuid::Uuid>,
    /// URL to redirect to after successful login
    pub redirect_uri: Option<String>,
    /// Code sent via email as a result of successful call to send_invitation
    pub inv_code: Option<uuid::Uuid>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginState {
    /// URL to redirect to after successful login
    pub redirect_uri: String,
    /// ID of organization to authenticate into
    pub organization_id: Option<uuid::Uuid>,
    /// Code sent via email as a result of successful call to send_invitation
    pub inv_code: Option<uuid::Uuid>,
}

/// Login
///
/// This will redirect you to the OAuth provider for authentication with email/pass, SSO, Google, Github, etc.
#[utoipa::path(
    get,
    path = "/auth",
    context_path = "/api",
    tag = "Auth",
    params(AuthQuery),
    responses(
        (status = 303, description = "Response that redirects to OAuth provider through a Location header to be handled by browser."),
        (status = 400, description = "OAuth error likely with OIDC provider.", body = ErrorResponseBody),
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
        .add_scopes([
            Scope::new("profile".to_owned()),
            Scope::new("email".to_owned()),
        ])
        .set_pkce_challenge(pkce_challenge)
        .url();

    let oidc_state = OpenIdConnectState {
        pkce_verifier,
        csrf_token,
        nonce,
    };

    session
        .insert(OIDC_SESSION_KEY, oidc_state)
        .map_err(|err| {
            ServiceError::InternalServerError(format!("Could not set OIDC Session {:?}", err))
        })?;

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
        organization_id: data.organization_id,
        inv_code: data.inv_code,
    };

    session.insert("login_state", login_state).map_err(|err| {
        ServiceError::InternalServerError(format!("Could not set redirect url {:?}", err))
    })?;

    //redirect to OpenIdProvider for authentication
    Ok(HttpResponse::SeeOther()
        .insert_header(("Location", auth_url.as_str()))
        .finish())
}

/// OpenID Connect callback
///
/// This is the callback route for the OAuth provider, it should not be called directly. Redirects to browser with set-cookie header.
#[utoipa::path(
    get,
    path = "/auth/callback",
    context_path = "/api",
    tag = "Auth",
    responses(
        (status = 200, description = "Response that returns with set-cookie header", body = SlimUser),
        (status = 400, description = "Email or password empty or incorrect", body = ErrorResponseBody),
    )
)]
pub async fn oidc_callback(
    req: HttpRequest,
    session: Session,
    oidc_client: web::Data<CoreClient>,
    redis_pool: web::Data<RedisPool>,
    clickhouse_client: web::Data<clickhouse::Client>,
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
            oauth2::RequestTokenError::Request(e) => {
                ServiceError::InternalServerError(e.to_string())
            }
            oauth2::RequestTokenError::Parse(e, _) => {
                ServiceError::InternalServerError(e.to_string())
            }
            oauth2::RequestTokenError::Other(e) => ServiceError::InternalServerError(e.to_string()),
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

    let user_oidc_subject = claims.subject().to_string();

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

    let mut user_is_new = false;

    // Check if a user with this email exists
    let optional_user_from_email = get_option_user_by_email_query(email, pool.clone()).await?;

    let user_from_oidc_subject =
        get_user_by_oidc_subject_query(&user_oidc_subject, pool.clone()).await;

    let (user, user_orgs, orgs) = match (user_from_oidc_subject, &optional_user_from_email) {
        (Ok(user), _) => user,
        (
            Err(_),
            Some(User {
                oidc_subject: None, ..
            }),
        ) => {
            // User exists, but has no current oidc subject in postgres need to associate them
            let user_id = optional_user_from_email.expect("user exists from above").id;
            associate_user_to_oidc_subject_query(user_id, user_oidc_subject.clone(), pool.clone())
                .await?;

            get_user_by_id_query(&user_id, pool.clone()).await?
        }
        (Err(_), None) => {
            // User does not exist from oidc_subject or have a matching email
            user_is_new = true;
            let (user, user_orgs, orgs) = create_account(
                email.to_string(),
                name.iter().next().unwrap().1.to_string(),
                Some(user_oidc_subject),
                login_state.organization_id,
                login_state.inv_code,
                pool.clone(),
            )
            .await?;

            let slim_user = SlimUser::from_details(user.clone(), user_orgs.clone(), orgs.clone());

            match get_user_ditto_identity(slim_user, pool.clone(), &clickhouse_client).await {
                Ok(identify_request) => {
                    match send_user_ditto_identity(identify_request).await {
                        Ok(_) => {
                            log::info!("Sent ditto identity for user {}", user.email);
                        }
                        Err(e) => {
                            log::info!(
                                "Failed to send ditto identity for user {}. Error: {}",
                                user.email,
                                e
                            );
                        }
                    };
                }
                Err(e) => {
                    log::info!("No ditto identity for user {}. Error: {}", user.email, e);
                }
            }

            (user, user_orgs, orgs)
        }
        (
            Err(_),
            Some(User {
                oidc_subject: Some(_),
                ..
            }),
        ) => {
            // This should not be reachable, we should error out.
            // Something fishy is going on with our auth provider.
            unreachable!();
        }
    };

    if login_state.organization_id.is_some()
        && !user_orgs.iter().any(|org| {
            org.organization_id == login_state.organization_id.unwrap_or(uuid::Uuid::default())
        })
    {
        if let Some(inv_code) = login_state.inv_code {
            let invitation = check_inv_valid(
                inv_code,
                email.to_string(),
                login_state.organization_id,
                pool.clone(),
            )
            .await?;
            let user_org = UserOrganization::from_details(
                user.id,
                invitation.organization_id,
                invitation.role.into(),
                invitation
                    .scopes
                    .map(|scopes| scopes.into_iter().map(|s| s.unwrap_or_default()).collect()),
            );
            add_user_to_organization(None, None, user_org, pool, redis_pool.clone()).await?;
        }
    }

    let user_string = serde_json::to_string(&user).map_err(|_| {
        ServiceError::InternalServerError("Failed to serialize user to JSON".into())
    })?;

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let slim_user = SlimUser::from_details(user, user_orgs.clone(), orgs);

    let slim_user_string = serde_json::to_string(&slim_user).map_err(|_| {
        ServiceError::InternalServerError("Failed to serialize slim user to JSON".into())
    })?;

    redis_conn
        .set::<_, _, ()>(slim_user.id.to_string(), slim_user_string)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Identity::login(&req.extensions(), user_string).expect("Failed to set login state for user");
    session.remove(OIDC_SESSION_KEY);
    session.remove("login_state");

    // Add a query param if the user has just been created and is the owner of
    // one organization
    let mut final_redirect = login_state.redirect_uri.clone();
    if user_is_new
        && user_orgs
            .clone()
            .into_iter()
            .any(|org_user| org_user.role == 2)
        && user_orgs.len() == 1
    {
        // Add query param indicating new user
        if final_redirect.contains('?') {
            final_redirect = format!("{}&new_user=true", final_redirect);
        } else {
            final_redirect = format!("{}?new_user=true", final_redirect);
        }
    }

    Ok(HttpResponse::SeeOther()
        .insert_header(("Location", final_redirect))
        .finish())
}

/// Get Me
///
/// Get the user corresponding to your current auth credentials.
#[utoipa::path(
    get,
    path = "/auth/me",
    context_path = "/api",
    tag = "Auth",
    responses(
        (status = 200, description = "The user corresponding to your current auth credentials", body = SlimUser),
        (status = 400, description = "Error message indicitating you are not currently signed in", body = ErrorResponseBody),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_me(
    logged_user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_query_id: uuid::Uuid = logged_user.id;

    let user = get_user_by_id_query(&user_query_id, pool).await?;

    Ok(HttpResponse::Ok().json(SlimUser::from_details(user.0, user.1, user.2)))
}

/// Health Check
///
/// Confirmation that the service is healthy and can make embedding vectors
#[utoipa::path(
    get,
    path = "/health",
    context_path = "/api",
    tag = "Health",
    responses(
        (status = 200, description = "Confirmation that the service is healthy and can make embedding vectors"),
        (status = 400, description = "Service error relating to making an embedding or overall service health", body = ErrorResponseBody),
    ),
)]
pub async fn health_check() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().finish())
}

/// Local login page for cli
pub async fn login_cli() -> Result<HttpResponse, ServiceError> {
    let html_page = read_to_string("src/public/login.html").map_err(|e| {
        ServiceError::InternalServerError(format!("Could not read login page {}", e))
    })?;
    Ok(HttpResponse::Ok().content_type("text/html").body(html_page))
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct CreateApiUserBody {
    pub user_email: String,
    pub user_name: String,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct CreateApiUserResponse {
    pub user: SlimUser,
    pub organization_id: uuid::Uuid,
    pub api_key: String,
}

#[utoipa::path(
    post,
    path = "/auth/create_api_only_user",
    context_path = "/api",
    tag = "Auth",
    responses(
        (status = 200, description = "The user id", body = CreateApiUserResponse),
        (status = 400, description = "Error message indicitating you are not currently signed in", body = ErrorResponseBody),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn create_api_only_user(
    req: HttpRequest,
    body: web::Json<CreateApiUserBody>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let access_key = req
        .headers()
        .get("X-Shopify-Authorization")
        .ok_or(ServiceError::BadRequest(
            "X-Shopify-Authorization header is required".to_string(),
        ))?
        .to_str()
        .map_err(|_| {
            ServiceError::BadRequest("Failed to parse X-Shopify-Authorization header".to_string())
        })?;

    if access_key != get_env!("SHOPIFY_SECRET_KEY", "SHOPIFY_SECRET_KEY must be set") {
        return Err(ServiceError::BadRequest(
            "Invalid session token".to_string(),
        ));
    }

    let user_email = body.user_email.clone();
    let user_name = body.user_name.clone();
    // See if user already exists
    let option_user = get_option_user_by_email_query(&user_email, pool.clone()).await?;

    let (user, user_orgs, orgs) = match option_user {
        Some(user) => get_user_by_id_query(&user.id, pool.clone()).await?,
        None => {
            let (user, user_orgs, orgs) = create_account(
                user_email.clone(),
                user_name.clone(),
                None,
                None,
                None,
                pool.clone(),
            )
            .await?;

            // Associate orgs.0 with api_only_price
            let organization = orgs
                .first()
                .ok_or(ServiceError::BadRequest(
                    "Failed to create organization for api only user; please try again".to_string(),
                ))?
                .clone();

            create_flat_subscription_query(
                uuid::Uuid::new_v4().to_string(),
                uuid::Uuid::parse_str("dead0000-f0ee-4000-a000-000000000000")
                    .expect("hardcoded should be valid uuid"),
                organization.id,
                None,
                pool.clone(),
            )
            .await?;

            // Send user ditto identity with shopify tag
            let ditto_batch_request = vec![
                DittoBatchRequestTypes::Identify(DittofeedIdentifyUser {
                    r#type: Some("identify".to_string()),
                    message_id: uuid::Uuid::new_v4(),
                    user_id: user.id,
                    traits: DittofeedUserTraits {
                        email: user.email.clone(),
                        name: user.name.clone(),
                        created_at: user.created_at,
                        organization_count: 1,
                        dataset_count: 1,
                    },
                }),
                DittoBatchRequestTypes::Track(DittoTrackRequest {
                    r#type: Some("track".to_string()),
                    message_id: format!(
                        "{}-{}-{}",
                        user_email.clone(),
                        user.id,
                        "shopify_installed"
                    ),
                    event: "shopify_installed".to_string(),
                    properties: DittoTrackProperties::DittoShopifyEvent(ShopifyCustomerEvent {
                        organization_id: organization.id,
                        store_name: user_name.clone(),
                        event_type: "shopify_installed".to_string(),
                    }),
                    user_id: user.id,
                }),
            ];

            let _ = send_user_ditto_identity(DittoBatchRequest {
                batch: ditto_batch_request,
            })
            .await;

            (user, user_orgs, orgs)
        }
    };

    let organization = orgs
        .first()
        .ok_or(ServiceError::BadRequest(
            "Failed to create organization for api only user; please try again".to_string(),
        ))?
        .clone();

    let api_key = create_organization_api_key_query(
        organization.id,
        CreateApiKeyReqPayload {
            name: "User API Key".to_string(),
            role: ApiKeyRole::Owner.into(),
            ..Default::default()
        },
        pool.clone(),
    )
    .await
    .map_err(|err| {
        ServiceError::BadRequest(format!(
            "Failed to set new API key for organization {}",
            err
        ))
    })?;

    Ok(HttpResponse::Ok().json(CreateApiUserResponse {
        user: SlimUser::from_details(user, user_orgs, orgs),
        organization_id: organization.id,
        api_key,
    }))
}
