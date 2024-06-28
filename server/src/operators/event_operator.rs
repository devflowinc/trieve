use crate::{
    data::models::{ClickhouseEvent, Event},
    errors::ServiceError,
};
use actix_web::web;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[tracing::instrument(skip(client))]
pub async fn create_event_query(
    event: Event,
    client: web::Data<clickhouse::Client>,
) -> Result<(), ServiceError> {
    if std::env::var("USE_ANALYTICS")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false)
    {
        client
            .query("INSERT INTO trieve.dataset_events (id, dataset_id, event_type, event_data, created_at) VALUES (?, ?, ?, ?, now())")
            .bind(event.id)
            .bind(event.dataset_id)
            .bind(event.event_type)
            .bind(event.event_data)
            .execute()
            .await
            .map_err(|err| {
                log::error!("Failed to create event {:?}", err);
                ServiceError::BadRequest("Failed to create event".to_string())
            })?;
    }

    Ok(())
}
#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct EventReturn {
    pub events: Vec<Event>,
    pub page_count: i32,
}
#[tracing::instrument(skip(clickhouse_client))]
pub async fn get_events_query(
    dataset_id: uuid::Uuid,
    page: i64,
    page_size: i64,
    event_types: Vec<String>,
    clickhouse_client: web::Data<clickhouse::Client>,
) -> Result<EventReturn, ServiceError> {
    let query = format!(
        "
        SELECT
            id,
            dataset_id,
            event_type,
            event_data,
            created_at
        FROM default.dataset_events
        WHERE dataset_id = '{dataset_id}' AND event_type IN ({event_types})
        ORDER BY created_at DESC
        LIMIT {limit} OFFSET {offset}
        ",
        dataset_id = dataset_id.to_string(),
        event_types = event_types
            .iter()
            .map(|x| format!("'{}'", x))
            .collect::<Vec<String>>()
            .join(","),
        limit = page_size,
        offset = (page - 1) * page_size,
    );

    let events_and_count: Vec<ClickhouseEvent> = clickhouse_client
        .query(&query)
        .fetch_all()
        .await
        .map_err(|err| {
            log::error!("Failed to get events {:?}", err);
            ServiceError::BadRequest("Failed to get events".to_string())
        })?;

    let events = events_and_count
        .into_iter()
        .map(|event| event.into())
        .collect();

    let count_query = format!(
        "
        SELECT count(*) / {page_size}
        FROM default.dataset_events
        WHERE dataset_id = '{dataset_id}' AND event_type IN ({event_types})
        ",
        dataset_id = dataset_id.to_string(),
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

    Ok(EventReturn {
        events,
        page_count: count,
    })
}
