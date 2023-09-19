use actix_service::Service;
use actix_web::{dev, Error, HttpRequest, Result, FromRequest};
use actix_identity::Identity;
use std::sync::Arc;
use std::env;

pub async fn auth_middleware<S>(
    req: HttpRequest,
    srv: S,
) -> Result<dev::ServiceResponse, Error>
where
    S: dev::Service<Request = HttpRequest, Response = dev::ServiceResponse, Error = Error>,
{
    // Check if the environment variable AUTH_REQUIRED is set to "true"
    let auth_required = env::var("AUTH_REQUIRED").ok().map(|val| val == "true").unwrap_or(false);

    if auth_required {
        // If authentication is required, check if the user is logged in using the LoggedUser guard
        let logged_user = LoggedUser::from_request(&req, &srv.data()).await?;

        // Continue processing the request with the LoggedUser guard
        let req = req.extensions(logged_user);

        srv.call(req).await
    } else {
        // If authentication is not required, proceed without authentication
        srv.call(req).await
    }
}
