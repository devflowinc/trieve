use super::auth_handler::LoggedUser;
use crate::{
    data::models::{DatasetAndOrgWithSubAndPlan, EventType, Pool},
    errors::ServiceError,
    operators::event_operator::get_events_query,
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::{schema, ToSchema};

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "page": 1,
    "page_size": 10,
    "type": "failed"
}))]
pub struct GetEventsData {
    /// The page number to get. Default is 1.
    pub page: Option<i64>,
    /// The number of items per page. Default is 10.
    pub page_size: Option<i64>,
    /// The type of events to get. Any combination of file_uploaded, card_uploaded, card_action_failed, or card_updated. Leave undefined to get all events.
    pub event_types: Option<Vec<String>>,
}

/// Get events for the dataset
///
/// Get events for the auth'ed user. Currently, this is only for events belonging to the auth'ed user. Soon, we plan to associate events to datasets instead of users.
#[utoipa::path(
    post,
    path = "/events",
    context_path = "/api",
    tag = "events",
    request_body(content = GetEventsData, description = "JSON request payload to get events for a dataset", content_type = "application/json"),
    responses(
        (status = 200, description = "Events for the dataset", body = EventReturn),
        (status = 400, description = "Service error relating to getting events for the dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_events(
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    get_events_data: web::Json<GetEventsData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let event_types = match &get_events_data.event_types {
        Some(event_types) => {
            // if empty array, return all event types
            if event_types.is_empty() {
                EventType::get_all_event_types()
            } else {
                event_types.clone()
            }
        }
        None => EventType::get_all_event_types(),
    };

    let events = get_events_query(
        dataset_org_plan_sub.dataset.id,
        get_events_data.page.unwrap_or(1),
        get_events_data.page_size.unwrap_or(10),
        event_types,
        pool,
    )
    .await
    .map_err(|e| ServiceError::BadRequest(e.to_string()))?;

    Ok(HttpResponse::Ok().json(events))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EventId {
    /// Id of the notification to target.
    pub notification_id: uuid::Uuid,
}
