use crate::{
    data::models::{Event, Pool},
    errors::ServiceError,
};
use actix_web::web;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
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
        .unwrap_or(false) { 
        let query = format!(
            "INSERT INTO dataset_events (id, dataset_id, event_type, event_data, created_at) VALUES ('{}', '{}', '{}', '{}', now())",
            event.id,
            event.dataset_id,
            event.event_type,
            event.event_data    
        );

        client
            .query(&query)
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
#[tracing::instrument(skip(pool))]
pub async fn get_events_query(
    dataset_id: uuid::Uuid,
    page: i64,
    page_size: i64,
    event_types: Vec<String>,
    pool: web::Data<Pool>,
) -> Result<EventReturn, ServiceError> {
    use crate::data::schema::dataset_event_counts::dsl as dataset_event_counts_columns;
    use crate::data::schema::events::dsl as events_columns;
    let mut conn = pool.get().await.unwrap();

    let events_and_count = events_columns::events
        .left_join(
            dataset_event_counts_columns::dataset_event_counts
                .on(events_columns::dataset_id.eq(dataset_event_counts_columns::dataset_uuid)),
        )
        .filter(events_columns::dataset_id.eq(dataset_id))
        .filter(
            events_columns::event_type.eq_any(
                event_types
                    .iter()
                    .map(|event_type| event_type.as_str())
                    .collect::<Vec<&str>>(),
            ),
        )
        .select((
            Event::as_select(),
            crate::data::schema::dataset_event_counts::dsl::notification_count.nullable(),
        ))
        .order(events_columns::created_at.desc())
        .limit(page_size)
        .offset((page - 1) * page_size)
        .load::<(Event, Option<i32>)>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to get events".to_string()))?;

    let events: Vec<Event> = events_and_count
        .iter()
        .map(|(event, _)| event.clone())
        .collect();
    let event_count: i32 = events_and_count
        .get(0)
        .map(|(_, count)| count.unwrap_or(0))
        .unwrap_or(0);
    Ok(EventReturn {
        events,
        page_count: (event_count as f64 / 10.0).ceil() as i32,
    })
}
