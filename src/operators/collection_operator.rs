use crate::{
    data::models::{
        CardCollectionBookmark, CardMetadata, CardMetadataWithVotesAndFiles, FullTextSearchResult,
    },
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    operators::card_operator::get_metadata,
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

pub fn get_collections_for_specifc_user_query(
    user_id: uuid::Uuid,
    accessing_user_id: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<CardCollection>, DefaultError> {
    use crate::data::schema::card_collection::dsl::*;

    let mut conn = pool.get().unwrap();
    let mut collections = card_collection.filter(author_id.eq(user_id)).into_boxed();

    match accessing_user_id {
        Some(accessing_user_uuid) => {
            if user_id != accessing_user_uuid {
                collections = collections.filter(is_public.eq(true));
            }
        }
        None => collections = collections.filter(is_public.eq(true)),
    }

    let collections = collections
        .load::<CardCollection>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error getting collections",
        })?;

    Ok(collections)
}

pub fn get_collections_for_logged_in_user_query(
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
    current_user_id: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<CardMetadataWithVotesAndFiles>, DefaultError> {
    use crate::data::schema::card_collection_bookmarks::dsl as card_collection_bookmarks_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;

    let mut conn = pool.get().unwrap();

    let bookmarks = card_collection_bookmarks_columns::card_collection_bookmarks
        .filter(card_collection_bookmarks_columns::collection_id.eq(collection))
        .load::<CardCollectionBookmark>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error getting bookmarks",
        })?;

    let bookmark_metadata = card_metadata_columns::card_metadata
        .filter(
            card_metadata_columns::id.eq_any(
                bookmarks
                    .iter()
                    .map(|bookmark| bookmark.card_metadata_id)
                    .collect::<Vec<uuid::Uuid>>(),
            ),
        )
        .select((
            card_metadata_columns::id,
            card_metadata_columns::content,
            card_metadata_columns::link,
            card_metadata_columns::author_id,
            card_metadata_columns::qdrant_point_id,
            card_metadata_columns::created_at,
            card_metadata_columns::updated_at,
            card_metadata_columns::oc_file_path,
            card_metadata_columns::card_html,
            card_metadata_columns::private,
        ))
        .load::<CardMetadata>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error getting bookmarks",
        })?;

    let converted_cards: Vec<FullTextSearchResult> = bookmark_metadata
        .iter()
        .map(|card| <CardMetadata as Into<FullTextSearchResult>>::into(card.clone()))
        .collect::<Vec<FullTextSearchResult>>();

    let card_metadata_with_upvotes_and_file_id =
        get_metadata(converted_cards, current_user_id, conn).map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;
    Ok(card_metadata_with_upvotes_and_file_id)
}

pub fn get_collections_for_bookmark_query(
    bookmark: uuid::Uuid,
    current_user_id: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<CardCollectionBookmark>, DefaultError> {
    use crate::data::schema::card_collection::dsl as card_collection_columns;
    use crate::data::schema::card_collection_bookmarks::dsl as card_collection_bookmarks_columns;

    let mut conn = pool.get().unwrap();

    let user_collections: Vec<uuid::Uuid> = card_collection_columns::card_collection
        .filter(card_collection_columns::author_id.eq(current_user_id.unwrap()))
        .select(card_collection_columns::id)
        .load::<uuid::Uuid>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error getting bookmarks",
        })?;

    let collections = card_collection_bookmarks_columns::card_collection_bookmarks
        .filter(card_collection_bookmarks_columns::card_metadata_id.eq(bookmark))
        .filter(card_collection_bookmarks_columns::collection_id.eq_any(user_collections))
        .load::<CardCollectionBookmark>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error getting bookmarks",
        })?;

    Ok(collections)
}
pub fn delete_bookmark_query(
    bookmark: uuid::Uuid,
    collection: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::card_collection_bookmarks::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::delete(
        card_collection_bookmarks
            .filter(card_metadata_id.eq(bookmark))
            .filter(collection_id.eq(collection)),
    )
    .execute(&mut conn)
    .map_err(|_err| {
        log::error!("Error deleting bookmark {:}", _err);
        DefaultError {
            message: "Error deleting bookmark",
        }
    })?;

    Ok(())
}
