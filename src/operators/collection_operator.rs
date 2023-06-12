use crate::{
    data::models::CardCollectionBookmark,
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
};
use actix_web::web;

use crate::{
    data::models::{CardCollection, Pool},
    errors::DefaultError,
};

pub fn create_collection_query(
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

pub fn get_collections_for_user_query(
    current_user_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<CardCollection>, DefaultError> {
    use crate::data::schema::card_collection::dsl::*;

    let mut conn = pool.get().unwrap();

    let collections = card_collection
        .filter(author_id.eq(current_user_id))
        .load::<CardCollection>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error getting collections",
        })?;

    Ok(collections)
}

pub fn get_collection_by_id_query(
    collection_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<CardCollection, DefaultError> {
    use crate::data::schema::card_collection::dsl::*;

    let mut conn = pool.get().unwrap();

    let collection = card_collection
        .filter(id.eq(collection_id))
        .first::<CardCollection>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Collection not found",
        })?;

    Ok(collection)
}

pub fn delete_collection_by_id_query(
    collection_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::card_collection::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::delete(card_collection.filter(id.eq(collection_id)))
        .execute(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error deleting collection",
        })?;

    Ok(())
}

pub fn update_card_collection_query(
    collection: CardCollection,
    new_name: Option<String>,
    new_description: Option<String>,
    new_is_public: Option<bool>,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::card_collection::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::update(card_collection.filter(id.eq(collection.id)))
        .set((
            name.eq(new_name.unwrap_or(collection.name)),
            description.eq(new_description.unwrap_or(collection.description)),
            is_public.eq(new_is_public.unwrap_or(collection.is_public)),
        ))
        .execute(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error updating collection",
        })?;

    Ok(())
}

pub fn create_card_bookmark_query(
    pool: web::Data<Pool>,
    bookmark: CardCollectionBookmark,
) -> Result<(), DefaultError> {
    use crate::data::schema::card_collection_bookmarks::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::insert_into(card_collection_bookmarks)
        .values(&bookmark)
        .execute(&mut conn)
        .map_err(|_err| {
            log::error!("Error creating bookmark {:}", _err);
            DefaultError {
                message: "Error creating bookmark",
            }
        })?;

    Ok(())
}

pub fn get_bookmarks_for_collection_query(
    collection: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<CardCollectionBookmark>, DefaultError> {
    use crate::data::schema::card_collection_bookmarks::dsl::*;

    let mut conn = pool.get().unwrap();

    let bookmarks = card_collection_bookmarks
        .filter(collection_id.eq(collection))
        .load::<CardCollectionBookmark>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error getting bookmarks",
        })?;

    Ok(bookmarks)
}

pub fn delete_bookmark_query(
    collection: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::card_collection_bookmarks::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::delete(card_collection_bookmarks.filter(collection_id.eq(collection)))
        .execute(&mut conn)
        .map_err(|_err| {
            log::error!("Error deleting bookmark {:}", _err);
            DefaultError {
                message: "Error deleting bookmark",
            }
        })?;

    Ok(())
}
