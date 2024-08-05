use crate::{
    data::models::{Pool, WordInDataset},
    errors::ServiceError,
};
use actix_web::web;
use diesel::prelude::*;
use diesel::upsert::excluded;
use diesel_async::RunQueryDsl;

#[tracing::instrument(skip(pool))]
pub async fn create_words_query(
    words: Vec<WordInDataset>,
    pool: web::Data<Pool>,
) -> Result<Vec<WordInDataset>, ServiceError> {
    use crate::data::schema::words_in_datasets::dsl as words_in_datasets_columns;
    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    let words = diesel::insert_into(words_in_datasets_columns::words_in_datasets)
        .values(&words)
        .on_conflict(words_in_datasets_columns::word)
        .do_update()
        .set(words_in_datasets_columns::word.eq(excluded(words_in_datasets_columns::word)))
        .get_results::<WordInDataset>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Error inserting words".to_string()))?;

    Ok(words)
}
