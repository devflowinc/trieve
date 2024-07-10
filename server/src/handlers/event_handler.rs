use super::auth_handler::LoggedUser;
use crate::{
    data::models::{DatasetAndOrgWithSubAndPlan, EventType, EventTypeRequest},
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
    "type": ["chunk_action_failed"]
}))]
pub struct GetEventsData {
    /// The page number to get. Default is 1.
    pub page: Option<i64>,
    /// The number of items per page. Default is 10.
    pub page_size: Option<i64>,
    /// The types of events to get. Any combination of file_uploaded, chunk_uploaded, chunk_action_failed, chunk_updated, or qdrant_index_failed. Leave undefined to get all events.
    pub event_types: Option<Vec<EventTypeRequest>>,
}

/// Get events for the dataset
///
/// Get events for the dataset specified by the TR-Dataset header.
#[utoipa::path(
    post,
    path = "/events",
    context_path = "/api",
    tag = "Events",
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
#[tracing::instrument(skip(clickhouse_client))]
pub async fn get_events(
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    get_events_data: web::Json<GetEventsData>,
    clickhouse_client: web::Data<clickhouse::Client>,
) -> Result<HttpResponse, actix_web::Error> {
    let event_types = get_events_data
        .event_types
        .as_ref()
        .map(|types| {
            if types.is_empty() {
                EventType::get_all_event_types()
            } else {
                types.to_vec()
            }
        })
        .unwrap_or(EventType::get_all_event_types());

    let events = get_events_query(
        dataset_org_plan_sub.dataset.id,
        get_events_data.page.unwrap_or(1),
        get_events_data.page_size.unwrap_or(10),
        event_types,
        clickhouse_client.clone(),
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
