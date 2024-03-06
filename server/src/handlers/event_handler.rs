use super::auth_handler::LoggedUser;
use crate::{
    data::models::{DatasetAndOrgWithSubAndPlan, Pool},
    errors::ServiceError,
    operators::event_operator::get_events_query,
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

/// get_events
///
/// Get events for the auth'ed user. Currently, this is only for events belonging to the auth'ed user. Soon, we plan to associate events to datasets instead of users. Each page contains 10 events.
#[utoipa::path(
    get,
    path = "/events/{page}",
    context_path = "/api",
    tag = "events",
    responses(
        (status = 200, description = "Events for the dataset", body = EventReturn),
        (status = 400, description = "Service error relating to getting events for the dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("page" = i64, description = "Page number of events to get"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
pub async fn get_events(
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    page: web::Path<i64>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let events = web::block(move || {
        get_events_query(dataset_org_plan_sub.dataset.id, page.into_inner(), pool)
    })
    .await?
    .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    Ok(HttpResponse::Ok().json(events))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EventId {
    /// Id of the notification to target.
    pub notification_id: uuid::Uuid,
}
