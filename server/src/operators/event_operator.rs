use crate::{
    data::models::{Event, Pool},
    errors::DefaultError,
    handlers::event_handler::Events,
};
use actix_web::web;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn add_group_created_event_query(
    event: Event,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
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
    pub event: Vec<Events>,
    pub full_count: i32,
    pub total_pages: i64,
}
pub fn get_events_query(
    dataset_id: uuid::Uuid,
    page: i64,
    pool: web::Data<Pool>,
) -> Result<EventReturn, DefaultError> {
    use crate::data::schema::events::dsl as events_columns;

    let mut conn = pool.get().unwrap();

    let file_upload_completed = events_columns::events
        .filter(events_columns::dataset_id.eq(dataset_id))
        .select(Event::as_select())
        .order(events_columns::created_at.desc())
        .limit(10)
        .offset((page - 1) * 10)
        .load::<Event>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to get events",
        })?;

    let combined_events: Vec<Events> = file_upload_completed
        .iter()
        .map(|c| Events::FileUploadComplete(c.clone()))
        .collect();

    let event_count = file_upload_completed.len();

    Ok(EventReturn {
        event: combined_events,
        full_count: event_count as i32,
        total_pages: (event_count as f64 / 10.0).ceil() as i64,
    })
}
