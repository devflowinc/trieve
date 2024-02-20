use crate::{
    data::models::{Event, Pool},
    errors::DefaultError,
};
use actix_web::web;
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn create_event_query(event: Event, pool: web::Data<Pool>) -> Result<(), DefaultError> {
    use crate::data::schema::events::dsl as events_columns;

    let mut conn = pool.get().unwrap();

    diesel::insert_into(events_columns::events)
        .values(&event)
        .execute(&mut conn)
        .map_err(|err| {
            log::error!("Failed to create event: {:?}", err);
            DefaultError {
                message: "Failed to create event",
            }
        })?;

    Ok(())
}
#[derive(Debug, Deserialize, Serialize, Clone, ToSchema)]
pub struct EventReturn {
    pub events: Vec<Event>,
    pub page_count: i32,
}
pub fn get_events_query(
    dataset_id: uuid::Uuid,
    page: i64,
    pool: web::Data<Pool>,
) -> Result<EventReturn, DefaultError> {
    use crate::data::schema::dataset_event_counts::dsl as dataset_event_counts_columns;
    use crate::data::schema::events::dsl as events_columns;
    let mut conn = pool.get().unwrap();

    let events_and_count = events_columns::events
        .left_join(
            dataset_event_counts_columns::dataset_event_counts
                .on(events_columns::dataset_id.eq(dataset_event_counts_columns::dataset_uuid)),
        )
        .filter(events_columns::dataset_id.eq(dataset_id))
        .select((
            Event::as_select(),
            crate::data::schema::dataset_event_counts::dsl::notification_count.nullable(),
        ))
        .order(events_columns::created_at.desc())
        .limit(10)
        .offset((page - 1) * 10)
        .load::<(Event, Option<i32>)>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to get events",
        })?;

    let events: Vec<Event> = events_and_count
        .iter()
        .map(|(event, _)| event.clone())
        .collect();
    let event_count: i32 = events_and_count
        .first()
        .map(|(_, count)| count.unwrap_or(0))
        .unwrap_or(0);
    Ok(EventReturn {
        events,
        page_count: (event_count as f64 / 10.0).ceil() as i32,
    })
}
