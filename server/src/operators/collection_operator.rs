use crate::{
    data::models::{ChunkCollection, Pool},
    errors::DefaultError,
};
use crate::{
    data::models::{
        ChunkCollectionAndFileWithCount, ChunkCollectionBookmark, ChunkMetadataWithCount,
        ChunkMetadataWithFileData, FileCollection, FullTextSearchResult, SlimCollection,
    },
    diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl},
    errors::ServiceError,
    operators::search_operator::get_metadata_query,
};
use actix_web::web;
use diesel::{
    dsl::sql, sql_types::Int8, BoolExpressionMethods, JoinOnDsl, NullableExpressionMethods,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn create_collection_query(
    new_collection: ChunkCollection,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::chunk_collection::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::insert_into(chunk_collection)
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
    new_collection: ChunkCollection,
    bookmark_ids: Vec<uuid::Uuid>,
    created_file_id: uuid::Uuid,
    given_dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkCollection, DefaultError> {
    use crate::data::schema::chunk_collection::dsl::*;

    let mut conn = pool.get().unwrap();

    chunk_collection
        .filter(dataset_id.eq(given_dataset_id))
        .filter(id.eq(new_collection.id))
        .first::<ChunkCollection>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Collection not found, likely incorrect dataset_id",
        })?;

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::insert_into(chunk_collection)
            .values(&new_collection)
            .execute(conn)?;

        use crate::data::schema::chunk_collection_bookmarks::dsl::*;

        diesel::insert_into(chunk_collection_bookmarks)
            .values(
                bookmark_ids
                    .iter()
                    .map(|bookmark| {
                        ChunkCollectionBookmark::from_details(new_collection.id, *bookmark)
                    })
                    .collect::<Vec<ChunkCollectionBookmark>>(),
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

pub fn get_collections_for_specific_user_query(
    user_id: uuid::Uuid,
    page: u64,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkCollectionAndFileWithCount>, DefaultError> {
    use crate::data::schema::chunk_collection::dsl::*;
    use crate::data::schema::collections_from_files::dsl as collections_from_files_columns;
    use crate::data::schema::user_collection_counts::dsl as user_collection_count_columns;

    let page = if page == 0 { 1 } else { page };
    let mut conn = pool.get().unwrap();
    let collections = chunk_collection
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
            description,
            created_at,
            updated_at,
            collections_from_files_columns::file_id.nullable(),
            user_collection_count_columns::collection_count.nullable(),
        ))
        .order_by(updated_at.desc())
        .filter(author_id.eq(user_id))
        .filter(dataset_id.eq(dataset_uuid))
        .into_boxed();

    let collections = collections
        .limit(10)
        .offset(((page - 1) * 10).try_into().unwrap_or(0))
        .load::<ChunkCollectionAndFileWithCount>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error getting collections",
        })?;

    Ok(collections)
}

pub fn get_collections_for_logged_in_user_query(
    current_user_id: uuid::Uuid,
    page: u64,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkCollectionAndFileWithCount>, DefaultError> {
    use crate::data::schema::chunk_collection::dsl::*;
    use crate::data::schema::collections_from_files::dsl as collections_from_files_columns;
    use crate::data::schema::user_collection_counts::dsl as user_collection_count_columns;

    let page = if page == 0 { 1 } else { page };

    let mut conn = pool.get().unwrap();

    let collections = chunk_collection
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
            description,
            created_at,
            updated_at,
            collections_from_files_columns::file_id.nullable(),
            user_collection_count_columns::collection_count.nullable(),
        ))
        .filter(author_id.eq(current_user_id))
        .filter(dataset_id.eq(dataset_uuid))
        .order(updated_at.desc())
        .limit(5)
        .offset(((page - 1) * 5).try_into().unwrap_or(0))
        .load::<ChunkCollectionAndFileWithCount>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error getting collections",
        })?;

    Ok(collections)
}

pub fn get_collection_by_id_query(
    collection_id: uuid::Uuid,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkCollection, DefaultError> {
    use crate::data::schema::chunk_collection::dsl::*;

    let mut conn = pool.get().unwrap();

    let collection = chunk_collection
        .filter(dataset_id.eq(dataset_uuid))
        .filter(id.eq(collection_id))
        .first::<ChunkCollection>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Collection not found",
        })?;

    Ok(collection)
}

pub fn delete_collection_by_id_query(
    collection_id: uuid::Uuid,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::chunk_collection::dsl as chunk_collection_columns;
    use crate::data::schema::chunk_collection_bookmarks::dsl as chunk_collection_bookmarks_columns;
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
            chunk_collection_bookmarks_columns::chunk_collection_bookmarks
                .filter(chunk_collection_bookmarks_columns::collection_id.eq(collection_id)),
        )
        .execute(conn)?;

        diesel::delete(
            chunk_collection_columns::chunk_collection
                .filter(chunk_collection_columns::id.eq(collection_id))
                .filter(chunk_collection_columns::dataset_id.eq(dataset_uuid)),
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

pub fn update_chunk_collection_query(
    collection: ChunkCollection,
    new_name: Option<String>,
    new_description: Option<String>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::chunk_collection::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::update(
        chunk_collection
            .filter(id.eq(collection.id))
            .filter(dataset_id.eq(dataset_uuid)),
    )
    .set((
        name.eq(new_name.unwrap_or(collection.name)),
        description.eq(new_description.unwrap_or(collection.description)),
    ))
    .execute(&mut conn)
    .map_err(|_err| DefaultError {
        message: "Error updating collection",
    })?;

    Ok(())
}

pub fn create_chunk_bookmark_query(
    pool: web::Data<Pool>,
    bookmark: ChunkCollectionBookmark,
) -> Result<(), DefaultError> {
    use crate::data::schema::chunk_collection_bookmarks::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::insert_into(chunk_collection_bookmarks)
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
    pub metadata: Vec<ChunkMetadataWithFileData>,
    pub collection: ChunkCollection,
    pub total_pages: i64,
}
pub fn get_bookmarks_for_collection_query(
    collection: uuid::Uuid,
    page: u64,
    limit: Option<i64>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<CollectionsBookmarkQueryResult, ServiceError> {
    use crate::data::schema::chunk_collection::dsl as chunk_collection_columns;
    use crate::data::schema::chunk_collection_bookmarks::dsl as chunk_collection_bookmarks_columns;
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    let page = if page == 0 { 1 } else { page };
    let limit = limit.unwrap_or(10);

    let mut conn = pool.get().unwrap();

    let bookmark_metadata: Vec<(ChunkMetadataWithCount, Option<uuid::Uuid>, ChunkCollection)> =
        chunk_metadata_columns::chunk_metadata
            .left_join(
                chunk_collection_bookmarks_columns::chunk_collection_bookmarks
                    .on(chunk_collection_bookmarks_columns::chunk_metadata_id
                        .eq(chunk_metadata_columns::id)),
            )
            .left_join(chunk_collection_columns::chunk_collection.on(
                chunk_collection_columns::id.eq(chunk_collection_bookmarks_columns::collection_id),
            ))
            .left_join(
                chunk_collisions_columns::chunk_collisions
                    .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
            )
            .filter(
                chunk_collection_bookmarks_columns::collection_id
                    .eq(collection)
                    .and(chunk_collection_columns::dataset_id.eq(dataset_uuid))
                    .and(chunk_metadata_columns::dataset_id.eq(dataset_uuid)),
            )
            .select((
                (
                    chunk_metadata_columns::id,
                    chunk_metadata_columns::content,
                    chunk_metadata_columns::link,
                    chunk_metadata_columns::author_id,
                    chunk_metadata_columns::qdrant_point_id,
                    chunk_metadata_columns::created_at,
                    chunk_metadata_columns::updated_at,
                    chunk_metadata_columns::tag_set,
                    chunk_metadata_columns::chunk_html,
                    chunk_metadata_columns::metadata,
                    chunk_metadata_columns::tracking_id,
                    chunk_metadata_columns::time_stamp,
                    chunk_metadata_columns::weight,
                    sql::<Int8>("count(*) OVER() AS full_count"),
                ),
                chunk_collisions_columns::collision_qdrant_id.nullable(),
                (
                    chunk_collection_columns::id.assume_not_null(),
                    chunk_collection_columns::author_id.assume_not_null(),
                    chunk_collection_columns::name.assume_not_null(),
                    chunk_collection_columns::description.assume_not_null(),
                    chunk_collection_columns::created_at.assume_not_null(),
                    chunk_collection_columns::updated_at.assume_not_null(),
                    chunk_collection_columns::dataset_id.assume_not_null(),
                ),
            ))
            .limit(limit)
            .offset(((page - 1) * limit as u64).try_into().unwrap_or(0))
            .load::<(ChunkMetadataWithCount, Option<uuid::Uuid>, ChunkCollection)>(&mut conn)
            .map_err(|_err| ServiceError::BadRequest("Error getting bookmarks".to_string()))?;

    let chunk_collection = if let Some(bookmark) = bookmark_metadata.first() {
        bookmark.2.clone()
    } else {
        chunk_collection_columns::chunk_collection
            .filter(chunk_collection_columns::id.eq(collection))
            .filter(chunk_collection_columns::dataset_id.eq(dataset_uuid))
            .first::<ChunkCollection>(&mut conn)
            .map_err(|_err| ServiceError::BadRequest("Error getting collection".to_string()))?
    };

    let converted_chunks: Vec<FullTextSearchResult> = bookmark_metadata
        .iter()
        .map(
            |(chunk, collided_id, _chunk_collection)| match collided_id {
                Some(id) => {
                    let mut chunk = chunk.clone();
                    chunk.qdrant_point_id = Some(*id);
                    <ChunkMetadataWithCount as Into<FullTextSearchResult>>::into(chunk)
                }
                None => <ChunkMetadataWithCount as Into<FullTextSearchResult>>::into(chunk.clone()),
            },
        )
        .collect::<Vec<FullTextSearchResult>>();

    let chunk_metadata_with_file_id = get_metadata_query(converted_chunks, conn)
        .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

    let total_pages = match bookmark_metadata.first() {
        Some(metadata) => (metadata.0.count as f64 / 10.0).ceil() as i64,
        None => 0,
    };

    Ok(CollectionsBookmarkQueryResult {
        metadata: chunk_metadata_with_file_id,
        collection: chunk_collection,
        total_pages,
    })
}
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct BookmarkCollectionResult {
    pub chunk_uuid: uuid::Uuid,
    pub slim_collections: Vec<SlimCollection>,
}

pub fn get_collections_for_bookmark_query(
    chunk_ids: Vec<uuid::Uuid>,
    current_user_id: Option<uuid::Uuid>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<BookmarkCollectionResult>, DefaultError> {
    use crate::data::schema::chunk_collection::dsl as chunk_collection_columns;
    use crate::data::schema::chunk_collection_bookmarks::dsl as chunk_collection_bookmarks_columns;

    let mut conn = pool.get().unwrap();

    let collections: Vec<(SlimCollection, uuid::Uuid)> = chunk_collection_columns::chunk_collection
        .left_join(
            chunk_collection_bookmarks_columns::chunk_collection_bookmarks
                .on(chunk_collection_columns::id
                    .eq(chunk_collection_bookmarks_columns::collection_id)),
        )
        .filter(chunk_collection_columns::dataset_id.eq(dataset_uuid))
        .filter(chunk_collection_bookmarks_columns::chunk_metadata_id.eq_any(chunk_ids))
        .select((
            chunk_collection_columns::id,
            chunk_collection_columns::name,
            chunk_collection_columns::author_id,
            chunk_collection_bookmarks_columns::chunk_metadata_id.nullable(),
        ))
        .limit(1000)
        .load::<(uuid::Uuid, String, uuid::Uuid, Option<uuid::Uuid>)>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Error getting bookmarks",
        })?
        .into_iter()
        .map(|(id, name, author_id, chunk_id)| match chunk_id {
            Some(chunk_id) => (
                SlimCollection {
                    id,
                    name,
                    author_id,
                    of_current_user: author_id == current_user_id.unwrap_or_default(),
                },
                chunk_id,
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

    let bookmark_collections: Vec<BookmarkCollectionResult> =
        collections.into_iter().fold(Vec::new(), |mut acc, item| {
            if item.1 == uuid::Uuid::default() {
                return acc;
            }

            //check if chunk in output already
            if let Some(output_item) = acc.iter_mut().find(|x| x.chunk_uuid == item.1) {
                //if it is, add collection to it
                output_item.slim_collections.push(item.0);
            } else {
                //if not make new output item
                acc.push(BookmarkCollectionResult {
                    chunk_uuid: item.1,
                    slim_collections: vec![item.0],
                });
            }
            acc
        });

    Ok(bookmark_collections)
}
pub fn delete_bookmark_query(
    bookmark_id: uuid::Uuid,
    collection_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::chunk_collection::dsl as chunk_collection_columns;
    use crate::data::schema::chunk_collection_bookmarks::dsl as chunk_collection_bookmarks_columns;

    let mut conn = pool.get().unwrap();

    chunk_collection_columns::chunk_collection
        .filter(chunk_collection_columns::id.eq(collection_id))
        .filter(chunk_collection_columns::dataset_id.eq(dataset_id))
        .first::<ChunkCollection>(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Collection not found, likely incorrect dataset_id",
        })?;

    diesel::delete(
        chunk_collection_bookmarks_columns::chunk_collection_bookmarks
            .filter(chunk_collection_bookmarks_columns::chunk_metadata_id.eq(bookmark_id))
            .filter(chunk_collection_bookmarks_columns::collection_id.eq(collection_id)),
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
