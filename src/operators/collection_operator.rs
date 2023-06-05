use crate::diesel::RunQueryDsl;
use actix_web::web;

use crate::{
    data::models::{CardCollection, Pool},
    errors::DefaultError,
};

pub fn create_collection_operation(
    new_collection: CardCollection,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::card_collection::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::insert_into(card_collection)
        .values(&new_collection)
        .execute(&mut conn)
        .map_err(|err| {
            log::error!("Error creating collection {:}", err);
            DefaultError {
                    message: "Error creating collection",
                }
        })?;

    Ok(())
}
