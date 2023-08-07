use crate::{
    data::models::{
        CardCollectionAndFileWithCount, CardCollectionBookmark, CardMetadataWithCount,
        CardMetadataWithVotesAndFiles, FileCollection, FullTextSearchResult, SlimCollection,
    },
    diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl},
    operators::card_operator::get_metadata_query,
};

use actix_web::web;
use diesel::{
    dsl::sql, sql_types::Int8, BoolExpressionMethods, JoinOnDsl, NullableExpressionMethods,
};
use serde::{Deserialize, Serialize};

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

pub fn create_collection_and_add_bookmarks_query(
    new_collection: CardCollection,
    bookmarks: Vec<uuid::Uuid>,
    created_file_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<CardCollection, DefaultError> {
    use crate::data::schema::card_collection::dsl::*;

    let mut conn = pool.get().unwrap();

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::insert_into(card_collection)
            .values(&new_collection)
            .execute(conn)?;

        use crate::data::schema::card_collection_bookmarks::dsl::*;

        diesel::insert_into(card_collection_bookmarks)
            .values(
                bookmarks
                    .iter()
                    .map(|bookmark| {
                        CardCollectionBookmark::from_details(new_collection.id, *bookmark)
                    })
                    .collect::<Vec<CardCollectionBookmark>>(),
            )
            .execute(conn)?;

        use crate::data::schema::collections_from_files::dsl::*;

        diesel::insert_into(collections_from_files)
            .values(&FileCollection::from_details(
                created_file_id,
                new_collection.id,
            ))
            .execute(conn)?;

        Ok(())
    });

    match transaction_result {
        Ok(_) => (),
        Err(err) => {
            log::error!("Error creating collection {:}", err);
            return Err(DefaultError {
                message: "Error creating collection",
            });
        }
    }
    Ok(new_collection)
}

pub fn get_collections_for_specifc_user_query(
    user_id: uuid::Uuid,
    accessing_user_id: Option<uuid::Uuid>,
    page: u64,
    pool: web::Data<Pool>,
) -> Result<Vec<CardCollectionAndFileWithCount>, DefaultError> {
    use crate::data::schema::card_collection::dsl::*;
    use crate::data::schema::collections_from_files::dsl as collections_from_files_columns;
    use crate::data::schema::user_collection_counts::dsl as user_collection_count_columns;

    let page = if page == 0 { 1 } else { page };
    let mut conn = pool.get().unwrap();
    let mut collections = card_collection
        .left_outer_join(
            collections_from_files_columns::collections_from_files
                .on(id.eq(collections_from_files_columns::collection_id)),
        )
        .left_outer_join(
            user_collection_count_columns::user_collection_counts
                .on(author_id.eq(user_collection_count_columns::user_id)),
        )
        .select((
            id,
            author_id,
            name,
            is_public,
            description,
            created_at,
            updated_at,
            collections_from_files_columns::file_id.nullable(),
            user_collection_count_columns::collection_count.nullable(),
        ))
        .order_by(updated_at.desc())
        .filter(author_id.eq(user_id))
        .into_boxed();

    match accessing_user_id {
        Some(accessing_user_uuid) => {
            if user_id != accessing_user_uuid {
                collections = collections.filter(is_public.eq(true));
            }
        }
        None => collections = collections.filter(is_public.eq(true)),
    }

    let collections = collections
        .limit(10)
        .offset(((page - 1) * 10).try_into().unwrap_or(0))
        .load::<CardCollectionAndFileWithCount>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error getting collections",
        })?;

    Ok(collections)
}

pub fn get_collections_for_logged_in_user_query(
    current_user_id: uuid::Uuid,
    page: u64,
    pool: web::Data<Pool>,
) -> Result<Vec<CardCollectionAndFileWithCount>, DefaultError> {
    use crate::data::schema::card_collection::dsl::*;
    use crate::data::schema::collections_from_files::dsl as collections_from_files_columns;
    use crate::data::schema::user_collection_counts::dsl as user_collection_count_columns;

    let page = if page == 0 { 1 } else { page };

    let mut conn = pool.get().unwrap();

    let collections = card_collection
        .left_outer_join(
            collections_from_files_columns::collections_from_files
                .on(id.eq(collections_from_files_columns::collection_id)),
        )
        .left_outer_join(
            user_collection_count_columns::user_collection_counts
                .on(author_id.eq(user_collection_count_columns::user_id)),
        )
        .select((
            id,
            author_id,
            name,
            is_public,
            description,
            created_at,
            updated_at,
            collections_from_files_columns::file_id.nullable(),
            user_collection_count_columns::collection_count.nullable(),
        ))
        .filter(author_id.eq(current_user_id))
        .order(updated_at.desc())
        .limit(5)
        .offset(((page - 1) * 5).try_into().unwrap_or(0))
        .load::<CardCollectionAndFileWithCount>(&mut conn)
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
    use crate::data::schema::card_collection::dsl as card_collection_columns;
    use crate::data::schema::card_collection_bookmarks::dsl as card_collection_bookmarks_columns;
    use crate::data::schema::collections_from_files::dsl as collections_from_files_columns;
    use crate::data::schema::file_upload_completed_notifications::dsl as file_upload_completed_notifications_columns;

    let mut conn = pool.get().unwrap();

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::delete(
            file_upload_completed_notifications_columns::file_upload_completed_notifications
                .filter(
                    file_upload_completed_notifications_columns::collection_uuid.eq(collection_id),
                ),
        )
        .execute(conn)?;

        diesel::delete(
            collections_from_files_columns::collections_from_files
                .filter(collections_from_files_columns::collection_id.eq(collection_id)),
        )
        .execute(conn)?;

        diesel::delete(
            card_collection_bookmarks_columns::card_collection_bookmarks
                .filter(card_collection_bookmarks_columns::collection_id.eq(collection_id)),
        )
        .execute(conn)?;

        diesel::delete(
            card_collection_columns::card_collection
                .filter(card_collection_columns::id.eq(collection_id)),
        )
        .execute(conn)?;

        Ok(())
    });

    match transaction_result {
        Ok(_) => Ok(()),
        Err(_) => Err(DefaultError {
            message: "Error deleting collection",
        }),
    }
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
pub struct CollectionsBookmarkQueryResult {
    pub metadata: Vec<CardMetadataWithVotesAndFiles>,
    pub total_pages: i64,
}
pub fn get_bookmarks_for_collection_query(
    collection: uuid::Uuid,
    page: u64,
    current_user_id: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<CollectionsBookmarkQueryResult, DefaultError> {
    use crate::data::schema::card_collection_bookmarks::dsl as card_collection_bookmarks_columns;
    use crate::data::schema::card_collisions::dsl as card_collisions_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;
    let page = if page == 0 { 1 } else { page };

    let mut conn = pool.get().unwrap();

    let bookmarks = card_collection_bookmarks_columns::card_collection_bookmarks
        .filter(card_collection_bookmarks_columns::collection_id.eq(collection))
        .load::<CardCollectionBookmark>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error getting bookmarks",
        })?;

    let bookmark_metadata: Vec<(CardMetadataWithCount, Option<uuid::Uuid>)> =
        card_metadata_columns::card_metadata
            .filter(
                card_metadata_columns::id.eq_any(
                    bookmarks
                        .iter()
                        .map(|bookmark| bookmark.card_metadata_id)
                        .collect::<Vec<uuid::Uuid>>(),
                ),
            )
            .left_join(
                card_collisions_columns::card_collisions
                    .on(card_metadata_columns::id.eq(card_collisions_columns::card_id)),
            )
            .select((
                (
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
                    sql::<Int8>("count(*) OVER() AS full_count"),
                ),
                card_collisions_columns::collision_qdrant_id.nullable(),
            ))
            .limit(10)
            .offset(((page - 1) * 10).try_into().unwrap_or(0))
            .load::<(CardMetadataWithCount, Option<uuid::Uuid>)>(&mut conn)
            .map_err(|_err| DefaultError {
                message: "Error getting bookmarks",
            })?;

    let converted_cards: Vec<FullTextSearchResult> = bookmark_metadata
        .iter()
        .map(|(card, collided_id)| match collided_id {
            Some(id) => {
                let mut card = card.clone();
                card.qdrant_point_id = Some(*id);
                <CardMetadataWithCount as Into<FullTextSearchResult>>::into(card)
            }
            None => <CardMetadataWithCount as Into<FullTextSearchResult>>::into(card.clone()),
        })
        .collect::<Vec<FullTextSearchResult>>();

    let card_metadata_with_upvotes_and_file_id =
        get_metadata_query(converted_cards, current_user_id, conn).map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    let total_pages = match bookmark_metadata.get(0) {
        Some(metadata) => (metadata.0.count as f64 / 10.0).ceil() as i64,
        None => 0,
    };

    Ok(CollectionsBookmarkQueryResult {
        metadata: card_metadata_with_upvotes_and_file_id,
        total_pages,
    })
}
#[derive(Serialize, Deserialize, Debug)]
pub struct BookmarkCollectionResult {
    pub card_uuid: uuid::Uuid,
    pub slim_collections: Vec<SlimCollection>,
}

pub fn get_collections_for_bookmark_query(
    card_ids: Vec<uuid::Uuid>,
    current_user_id: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<BookmarkCollectionResult>, DefaultError> {
    use crate::data::schema::card_collection::dsl as card_collection_columns;
    use crate::data::schema::card_collection_bookmarks::dsl as card_collection_bookmarks_columns;

    let mut conn = pool.get().unwrap();

    let collections: Vec<(SlimCollection, uuid::Uuid)> = card_collection_columns::card_collection
        .left_join(
            card_collection_bookmarks_columns::card_collection_bookmarks
                .on(card_collection_columns::id
                    .eq(card_collection_bookmarks_columns::collection_id)),
        )
        .filter(
            card_collection_columns::is_public
                .eq(true)
                .or(card_collection_columns::author_id.eq(current_user_id.unwrap_or_default())),
        )
        .filter(card_collection_bookmarks_columns::card_metadata_id.eq_any(card_ids))
        .select((
            card_collection_columns::id,
            card_collection_columns::name,
            card_collection_columns::author_id,
            card_collection_bookmarks_columns::card_metadata_id.nullable(),
        ))
        .limit(1000)
        .load::<(uuid::Uuid, String, uuid::Uuid, Option<uuid::Uuid>)>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error getting bookmarks",
        })?
        .into_iter()
        .map(|(id, name, author_id, card_id)| match card_id {
            Some(card_id) => (
                SlimCollection {
                    id,
                    name,
                    author_id,
                    of_current_user: author_id == current_user_id.unwrap_or_default(),
                },
                card_id,
            ),
            None => (
                SlimCollection {
                    id,
                    name,
                    author_id,
                    of_current_user: author_id == current_user_id.unwrap_or_default(),
                },
                uuid::Uuid::default(),
            ),
        })
        .collect();

    log::info!("collections: {:?}", collections.len());

    let bookmark_collections: Vec<BookmarkCollectionResult> =
        collections.into_iter().fold(Vec::new(), |mut acc, item| {
            if item.1 == uuid::Uuid::default() {
                return acc;
            }

            //check if card in output already
            if let Some(output_item) = acc.iter_mut().find(|x| x.card_uuid == item.1) {
                //if it is, add collection to it
                output_item.slim_collections.push(item.0);
            } else {
                //if not make new output item
                acc.push(BookmarkCollectionResult {
                    card_uuid: item.1,
                    slim_collections: vec![item.0],
                });
            }
            acc
        });

    Ok(bookmark_collections)
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
