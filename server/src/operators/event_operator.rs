use crate::{
    data::models::{EventTypeRequest, WorkerEvent, WorkerEventClickhouse},
    errors::ServiceError,
};
use actix_web::web;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct EventReturn {
    pub events: Vec<WorkerEvent>,
    pub event_types: Vec<String>,
    pub page_count: i32,
}

pub async fn get_events_query(
    dataset_id: uuid::Uuid,
    page: i64,
    page_size: i64,
    event_types: Vec<EventTypeRequest>,
    clickhouse_client: web::Data<clickhouse::Client>,
) -> Result<EventReturn, ServiceError> {
    let query = format!(
        "
        SELECT
            id,
            dataset_id,
            organization_id,
            event_type,
            event_data,
            created_at
        FROM dataset_events
        WHERE dataset_id = '{dataset_id}' AND event_type IN ({event_types})
        ORDER BY created_at DESC
        LIMIT {limit} OFFSET {offset}
        ",
        dataset_id = dataset_id,
        event_types = event_types
            .iter()
            .map(|x| format!("'{}'", x))
            .collect::<Vec<String>>()
            .join(","),
        limit = page_size,
        offset = (page - 1) * page_size,
    );

    let events_and_count: Vec<WorkerEventClickhouse> = clickhouse_client
        .query(&query)
        .fetch_all()
        .await
        .map_err(|err| {
            log::error!("Failed to get all events and counts {:?}", err);
            ServiceError::BadRequest("Failed to get all events and counts".to_string())
        })?;

    let events = events_and_count
        .into_iter()
        .map(|event| event.into())
        .collect();

    let count_query = format!(
        "
        SELECT count(*) / {page_size}
        FROM dataset_events
        WHERE dataset_id = '{dataset_id}' AND event_type IN ({event_types})
        ",
        dataset_id = dataset_id,
        event_types = event_types
            .iter()
            .map(|x| format!("'{}'", x))
            .collect::<Vec<String>>()
            .join(","),
        page_size = page_size,
    );

    let count: i32 = clickhouse_client
        .query(&count_query)
        .fetch()
        .map_err(|err| {
            log::error!("Failed to get events count {:?}", err);
            ServiceError::BadRequest("Failed to get events count".to_string())
        })?
        .next()
        .await
        .map_err(|err| {
            log::error!("Failed to get events count {:?}", err);
            ServiceError::BadRequest("Failed to get events count".to_string())
        })?
        .unwrap_or(0.0_f64)
        .ceil() as i32;

    let event_types_query = format!(
        "
        SELECT DISTINCT event_type
        FROM dataset_events
        WHERE dataset_id = '{dataset_id}'
        ORDER BY event_type
        ",
        dataset_id = dataset_id,
    );

    let event_types: Vec<String> = clickhouse_client
        .query(&event_types_query)
        .fetch_all()
        .await
        .map_err(|err| {
            log::error!("Failed to get event types {:?}", err);
            ServiceError::BadRequest("Failed to get event types".to_string())
        })?;

    Ok(EventReturn {
        events,
        event_types,
        page_count: count,
    })
}
