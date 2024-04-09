use crate::data::models::{
    ChunkCollision, ChunkFile, ChunkFileWithName, ChunkGroupBookmark,
    Dataset, FullTextSearchResult, ServerDatasetConfiguration, UnifiedId,
};
use crate::operators::model_operator::create_embeddings;
use crate::operators::qdrant_operator::get_qdrant_connection;
use crate::operators::search_operator::get_metadata_query;
use crate::{
    data::models::{ChunkMetadata, Pool},
    errors::ServiceError,
};
use actix_web::web;
use diesel::dsl::not;
use diesel::prelude::*;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use itertools::Itertools;
use qdrant_client::qdrant::{PointId, PointVectors};
use simsearch::SimSearch;

#[tracing::instrument(skip(pool))]
pub async fn get_metadata_from_point_ids(
    point_ids: Vec<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkMetadata>, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    let chunk_metadata: Vec<ChunkMetadata> = chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::qdrant_point_id.eq_any(&point_ids))
        .select(ChunkMetadata::as_select())
        .load::<ChunkMetadata>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

    let converted_chunks: Vec<FullTextSearchResult> = chunk_metadata
        .iter()
        .map(|chunk| <ChunkMetadata as Into<FullTextSearchResult>>::into(chunk.clone()))
        .collect::<Vec<FullTextSearchResult>>();

    let chunk_metadata_with_file_id = get_metadata_query(converted_chunks, pool)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

    Ok(chunk_metadata_with_file_id)
}

pub async fn get_point_ids_from_unified_chunk_ids(
    chunk_ids: Vec<UnifiedId>,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<uuid::Uuid>, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    let qdrant_point_ids: Vec<uuid::Uuid> = match chunk_ids[0] {
        UnifiedId::TrieveUuid(_) => chunk_metadata_columns::chunk_metadata
            .filter(
                chunk_metadata_columns::id.eq_any(
                    &chunk_ids
                        .iter()
                        .map(|x| x.as_uuid().expect("Failed to convert to uuid"))
                        .collect::<Vec<uuid::Uuid>>(),
                ),
            )
            .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
            .select(chunk_metadata_columns::qdrant_point_id)
            .load::<Option<uuid::Uuid>>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?
            .into_iter()
            .flatten()
            .collect(),
        UnifiedId::TrackingId(_) => chunk_metadata_columns::chunk_metadata
            .filter(
                chunk_metadata_columns::tracking_id.eq_any(
                    &chunk_ids
                        .iter()
                        .map(|x| x.as_tracking_id().expect("Failed to convert to String"))
                        .collect::<Vec<String>>(),
                ),
            )
            .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
            .select(chunk_metadata_columns::qdrant_point_id)
            .load::<Option<uuid::Uuid>>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?
            .into_iter()
            .flatten()
            .collect(),
    };

    Ok(qdrant_point_ids)
}

pub struct ChunkMetadataWithQdrantId {
    pub metadata: ChunkMetadata,
    pub qdrant_id: uuid::Uuid,
}

#[tracing::instrument(skip(pool))]
pub async fn get_metadata_and_collided_chunks_from_point_ids_query(
    point_ids: Vec<uuid::Uuid>,
    get_collisions: bool,
    pool: web::Data<Pool>,
) -> Result<
    (
        Vec<ChunkMetadata>,
        Vec<ChunkMetadataWithQdrantId>,
    ),
    ServiceError,
> {
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::files::dsl as files_columns;

    let parent_span = sentry::configure_scope(|scope| scope.get_span());
    let transaction: sentry::TransactionOrSpan = match &parent_span {
        Some(parent) => parent
            .start_child(
                "Get metadata of points",
                "Hitting Postgres to fetch metadata",
            )
            .into(),
        None => {
            let ctx = sentry::TransactionContext::new(
                "Get metadata of points",
                "Hitting Postgres to fetch metadata",
            );
            sentry::start_transaction(ctx).into()
        }
    };
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone())));

    let chunk_search_span = transaction.start_child(
        "Fetching matching points from qdrant",
        "Fetching matching points from qdrant",
    );

    // Fetch the chunk metadata for root cards
    let chunk_search_result = {
        let mut conn = pool.get().await.unwrap();
        let chunk_metadata = chunk_metadata_columns::chunk_metadata
            .left_outer_join(
                chunk_files_columns::chunk_files
                    .on(chunk_metadata_columns::id.eq(chunk_files_columns::chunk_id)),
            )
            .left_outer_join(
                files_columns::files.on(chunk_files_columns::file_id.eq(files_columns::id)),
            )
            .left_outer_join(
                chunk_collisions_columns::chunk_collisions
                    .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
            )
            .select((
                ChunkMetadata::as_select(),
                (chunk_collisions_columns::collision_qdrant_id).nullable(),
            ))
            .filter(chunk_metadata_columns::qdrant_point_id.eq_any(&point_ids))
            .load::<(ChunkMetadata, Option<uuid::Uuid>)>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

        chunk_metadata
            .iter()
            .map(|chunk| ChunkMetadata {
                id: chunk.0.id,
                content: chunk.0.content.clone(),
                link: chunk.0.link.clone(),
                tag_set: chunk.0.tag_set.clone(),
                qdrant_point_id: chunk.0.qdrant_point_id.unwrap_or_else(|| {
                    chunk
                        .1
                        .expect("Must have qdrant_id from collision or metadata")
                }),
                created_at: chunk.0.created_at,
                updated_at: chunk.0.updated_at,
                chunk_html: chunk.0.chunk_html.clone(),
                metadata: chunk.0.metadata.clone(),
                tracking_id: chunk.0.tracking_id.clone(),
                time_stamp: chunk.0.time_stamp,
                weight: chunk.0.weight,
            })
            .collect::<Vec<ChunkMetadata>>()
    };

    chunk_search_span.finish();

    let collision_search_span = transaction.start_child(
        "Fetching matching points from qdrant",
        "Fetching matching points from qdrant",
    );

    if get_collisions {
        // Fetch the collided chunks
        let collided_chunks = {
            let mut conn = pool.get().await.unwrap();
            let chunk_metadata = chunk_collisions_columns::chunk_collisions
                .inner_join(
                    chunk_metadata_columns::chunk_metadata
                        .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
                )
                .left_outer_join(
                    chunk_files_columns::chunk_files
                        .on(chunk_metadata_columns::id.eq(chunk_files_columns::chunk_id)),
                )
                .left_outer_join(
                    files_columns::files.on(chunk_files_columns::file_id.eq(files_columns::id)),
                )
                .filter(chunk_collisions_columns::collision_qdrant_id.eq_any(point_ids))
                .select((
                    ChunkMetadata::as_select(),
                    chunk_collisions_columns::collision_qdrant_id.assume_not_null(),
                    (
                        chunk_files_columns::chunk_id,
                        chunk_files_columns::file_id,
                        files_columns::file_name,
                    )
                        .nullable(),
                ))
                .load::<(ChunkMetadata, uuid::Uuid, Option<ChunkFileWithName>)>(&mut conn)
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

            // Convert the collided chunks into the appropriate format
            chunk_metadata
                .iter()
                .map(|chunk| {
                    let chunk_metadata = ChunkMetadata {
                        id: chunk.0.id,
                        content: chunk.0.content.clone(),
                        link: chunk.0.link.clone(),
                        tag_set: chunk.0.tag_set.clone(),
                        qdrant_point_id: chunk.0.qdrant_point_id.unwrap_or(chunk.1),
                        created_at: chunk.0.created_at,
                        updated_at: chunk.0.updated_at,
                        chunk_html: chunk.0.chunk_html.clone(),
                        metadata: chunk.0.metadata.clone(),
                        tracking_id: chunk.0.tracking_id.clone(),
                        time_stamp: chunk.0.time_stamp,
                        weight: chunk.0.weight,
                    };
                    ChunkMetadataWithQdrantId {
                        metadata: chunk_metadata,
                        qdrant_id: chunk.1,
                    }
                })
                .collect::<Vec<ChunkMetadataWithQdrantId>>()
        };

        collision_search_span.finish();
        transaction.finish();
        // Return the chunk metadata and the collided chunks
        Ok((chunk_search_result, collided_chunks))
    } else {
        collision_search_span.finish();
        transaction.finish();
        // Return only the chunk metadata
        Ok((chunk_search_result, vec![]))
    }
}

#[tracing::instrument(skip(pool))]
pub async fn get_metadata_from_id_query(
    chunk_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    let mut conn = pool.get().await.unwrap();

    chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::id.eq(chunk_id))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
        .select(ChunkMetadata::as_select())
        .first::<ChunkMetadata>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))
}

#[tracing::instrument(skip(pool))]
pub async fn get_metadata_from_tracking_id_query(
    tracking_id: String,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::tracking_id.eq(tracking_id))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_uuid))
        .select(ChunkMetadata::as_select())
        .first::<ChunkMetadata>(&mut conn)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to load execute get_metadata_from_tracking_id_query: {:?}",
                e
            );

            ServiceError::BadRequest(
                "Failed to execute get_metadata_from_tracking_id_query".to_string(),
            )
        })
}

#[tracing::instrument(skip(pool))]
pub async fn get_optional_metadata_from_tracking_id_query(
    tracking_id: String,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Option<ChunkMetadata>, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    let optional_chunk: Option<ChunkMetadata> = chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::tracking_id.eq(tracking_id))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_uuid))
        .select(ChunkMetadata::as_select())
        .load::<ChunkMetadata>(&mut conn)
        .await
        .map_err(|e| {
            log::error!(
                "Failed to execute get_optional_metadata_from_tracking_id_query: {:?}",
                e
            );

            ServiceError::BadRequest(
                "Failed to execute get_optional_metadata_from_tracking_id_query".to_string(),
            )
        })?
        .pop();

    Ok(optional_chunk)
}

#[tracing::instrument(skip(pool))]
pub async fn get_metadata_from_ids_query(
    chunk_ids: Vec<uuid::Uuid>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkMetadata>, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    let metadatas: Vec<ChunkMetadata> = chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::id.eq_any(chunk_ids))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_uuid))
        .select(ChunkMetadata::as_select())
        .load::<ChunkMetadata>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;
    let full_text_metadatas = metadatas
        .iter()
        .map_into::<FullTextSearchResult>()
        .collect_vec();

    Ok(get_metadata_query(full_text_metadatas, pool)
        .await
        .unwrap_or_default())
}

#[tracing::instrument(skip(pool))]
pub async fn insert_chunk_metadata_query(
    chunk_data: ChunkMetadata,
    file_uuid: Option<uuid::Uuid>,
    group_ids: Option<Vec<uuid::Uuid>>,
    dataset_uuid: uuid::Uuid,
    upsert_by_tracking_id: bool,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, ServiceError> {
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl::*;

    if upsert_by_tracking_id && chunk_data.tracking_id.is_some() {
        if let Some(existing_chunk) = get_optional_metadata_from_tracking_id_query(
            chunk_data
                .tracking_id
                .clone()
                .expect("tracking_id must be Some at this point"),
            chunk_data.dataset_id,
            pool.clone(),
        )
        .await?
        {
            let mut update_chunk = chunk_data.clone();
            update_chunk.id = existing_chunk.id;
            update_chunk.created_at = existing_chunk.created_at;
            update_chunk.qdrant_point_id = existing_chunk.qdrant_point_id;

            let updated_chunk = update_chunk_metadata_query(
                update_chunk,
                file_uuid,
                group_ids,
                dataset_uuid,
                pool.clone(),
            )
            .await?;

            return Ok(updated_chunk);
        }
    }

    let mut conn = pool.get().await.expect("Failed to get connection to db");

    if let Some(other_tracking_id) = chunk_data.tracking_id.clone() {
        let existing_chunk = get_optional_metadata_from_tracking_id_query(
            other_tracking_id.clone(),
            chunk_data.dataset_id,
            pool.clone(),
        )
        .await?;

        if existing_chunk.is_some() {
            log::info!("Avoided potential write conflict by pre-checking tracking_id");
            return Err(ServiceError::DuplicateTrackingId(other_tracking_id));
        }
    }

    let inserted_chunk = diesel::insert_into(chunk_metadata)
        .values(&chunk_data)
        .get_result::<ChunkMetadata>(&mut conn)
        .await
        .map_err(|e| {
            sentry::capture_message(
                &format!("Failed to insert chunk_metadata: {:?}", e),
                sentry::Level::Error,
            );
            log::error!("Failed to insert chunk_metadata: {:?}", e);
            match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => ServiceError::DuplicateTrackingId(
                    chunk_data.tracking_id.clone().unwrap_or("".to_string()),
                ),
                _ => ServiceError::BadRequest("Failed to insert chunk_metadata".to_string()),
            }
        })?;

    if let Some(file_uuid) = file_uuid {
        diesel::insert_into(chunk_files_columns::chunk_files)
            .values(&ChunkFile::from_details(inserted_chunk.id, file_uuid))
            .execute(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to insert chunk file: {:?}", e);
                ServiceError::BadRequest("Failed to insert chunk file".to_string())
            })?;
    }

    if let Some(group_ids) = group_ids {
        diesel::insert_into(chunk_group_bookmarks_columns::chunk_group_bookmarks)
            .values(
                &group_ids
                    .into_iter()
                    .map(|group_id| ChunkGroupBookmark::from_details(group_id, inserted_chunk.id))
                    .collect::<Vec<ChunkGroupBookmark>>(),
            )
            .execute(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to insert chunk into groups {:?}", e);
                ServiceError::BadRequest("Failed to insert chunk into groups".to_string())
            })?;
    }

    Ok(chunk_data)
}

#[tracing::instrument(skip(pool))]
pub async fn revert_insert_chunk_metadata_query(
    chunk_id: uuid::Uuid,
    file_uuid: Option<uuid::Uuid>,
    group_ids: Option<Vec<uuid::Uuid>>,
    dataset_uuid: uuid::Uuid,
    upsert_by_tracking_id: bool,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl::*;

    if upsert_by_tracking_id {
        // TODO Properly revert here
        return Ok(());
    }

    let mut conn = pool.get().await.expect("Failed to get connection to db");

    diesel::delete(chunk_metadata.filter(id.eq(chunk_id)))
        .execute(&mut conn)
        .await
        .map_err(|e| {
            sentry::capture_message(
                &format!("Failed to revert insert transaction: {:?}", e),
                sentry::Level::Error,
            );
            log::error!("Failed to revert insert transaction: {:?}", e);

            ServiceError::BadRequest("Failed to revert insert transaction".to_string())
        })?;

    if let Some(file_uuid) = file_uuid {
        diesel::delete(
            chunk_files_columns::chunk_files.filter(chunk_files_columns::chunk_id.eq(file_uuid)),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| {
            log::error!("Failed to revert chunk file action: {:?}", e);
            ServiceError::BadRequest("Failed to revert chunk file action".to_string())
        })?;
    }

    if group_ids.is_some() {
        diesel::delete(
            chunk_group_bookmarks_columns::chunk_group_bookmarks
                .filter(chunk_group_bookmarks_columns::chunk_metadata_id.eq(chunk_id)),
        )
        .execute(&mut conn)
        .await
        .map_err(|e| {
            log::error!("Failed to revert chunk into groups action {:?}", e);
            ServiceError::BadRequest("Failed to revert chunk into groups action".to_string())
        })?;
    }

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn insert_duplicate_chunk_metadata_query(
    chunk_data: ChunkMetadata,
    duplicate_chunk: uuid::Uuid,
    file_uuid: Option<uuid::Uuid>,
    group_ids: Option<Vec<uuid::Uuid>>,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::chunk_collisions::dsl::*;
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl::*;

    let mut conn = pool.get().await.unwrap();

    let inserted_chunk = conn
        .transaction::<_, diesel::result::Error, _>(|conn| {
            async {
                let inserted_chunk = diesel::insert_into(chunk_metadata)
                    .values(&chunk_data)
                    .get_result::<ChunkMetadata>(conn)
                    .await?;

                //insert duplicate into chunk_collisions
                diesel::insert_into(chunk_collisions)
                    .values(&ChunkCollision::from_details(
                        chunk_data.id,
                        duplicate_chunk,
                    ))
                    .execute(conn)
                    .await?;

                if file_uuid.is_some() {
                    diesel::insert_into(chunk_files_columns::chunk_files)
                        .values(&ChunkFile::from_details(
                            chunk_data.id,
                            file_uuid.expect("file_uuid should be some"),
                        ))
                        .execute(conn)
                        .await?;
                }

                Ok(inserted_chunk)
            }
            .scope_boxed()
        })
        .await
        .map_err(|e| {
            log::error!("Failed to insert duplicate chunk metadata: {:?}", e);
            sentry::capture_message(
                &format!("Failed to insert duplicate chunk metadata: {:?}", e),
                sentry::Level::Error,
            );

            ServiceError::BadRequest("Failed to insert duplicate chunk metadata".to_string())
        })?;

    if let Some(group_ids) = group_ids {
        diesel::insert_into(chunk_group_bookmarks_columns::chunk_group_bookmarks)
            .values(
                &group_ids
                    .into_iter()
                    .map(|group_id| ChunkGroupBookmark::from_details(group_id, inserted_chunk.id))
                    .collect::<Vec<ChunkGroupBookmark>>(),
            )
            .execute(&mut conn)
            .await
            .map_err(|e| {
                log::error!(
                    "Failed to insert duplicate chunk_metadata into groups {:?}",
                    e
                );
                sentry::capture_message(
                    &format!(
                        "Failed to insert duplicate chunk_metadata into groups {:?}",
                        e
                    ),
                    sentry::Level::Error,
                );

                ServiceError::BadRequest(
                    "Failed to insert duplicate chunk_metadata into groups".to_string(),
                )
            })?;
    }

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn update_chunk_metadata_query(
    chunk_data: ChunkMetadata,
    file_uuid: Option<uuid::Uuid>,
    group_ids: Option<Vec<uuid::Uuid>>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, ServiceError> {
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    let updated_chunk = conn
        .transaction::<_, diesel::result::Error, _>(|conn| {
            async move {
                let updated_chunk: ChunkMetadata = diesel::update(
                    chunk_metadata_columns::chunk_metadata
                        .filter(chunk_metadata_columns::id.eq(chunk_data.id))
                        .filter(chunk_metadata_columns::dataset_id.eq(dataset_uuid)),
                )
                .set((
                    chunk_metadata_columns::link.eq(chunk_data.link),
                    chunk_metadata_columns::chunk_html.eq(chunk_data.chunk_html),
                    chunk_metadata_columns::content.eq(chunk_data.content),
                    chunk_metadata_columns::metadata.eq(chunk_data.metadata),
                    chunk_metadata_columns::tag_set.eq(chunk_data.tag_set),
                    chunk_metadata_columns::weight.eq(chunk_data.weight),
                ))
                .get_result::<ChunkMetadata>(conn)
                .await?;

                if file_uuid.is_some() {
                    diesel::insert_into(chunk_files_columns::chunk_files)
                        .values(ChunkFile::from_details(
                            chunk_data.id,
                            file_uuid.expect("file_uuid should be some"),
                        ))
                        .execute(conn)
                        .await?;
                }

                Ok(updated_chunk)
            }
            .scope_boxed()
        })
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to update chunk metadata".to_string()))?;

    if let Some(group_ids) = group_ids {
        let group_id1 = group_ids.clone();
        diesel::insert_into(chunk_group_bookmarks_columns::chunk_group_bookmarks)
            .values(
                &group_ids
                    .into_iter()
                    .map(|group_id| ChunkGroupBookmark::from_details(group_id, updated_chunk.id))
                    .collect::<Vec<ChunkGroupBookmark>>(),
            )
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to create bookmark".to_string()))?;

        diesel::delete(
            chunk_group_bookmarks_columns::chunk_group_bookmarks
                .filter(chunk_group_bookmarks_columns::chunk_metadata_id.eq(chunk_data.id))
                .filter(not(
                    chunk_group_bookmarks_columns::group_id.eq_any(group_id1)
                )),
        )
        .execute(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to delete chunk bookmarks".to_string()))?;
    }

    Ok(updated_chunk)
}

pub enum TransactionResult {
    ChunkCollisionDetected(ChunkMetadata),
    ChunkCollisionNotDetected,
}

#[tracing::instrument(skip(pool))]
pub async fn delete_chunk_metadata_query(
    chunk_uuid: uuid::Uuid,
    dataset: Dataset,
    pool: web::Data<Pool>,
    config: ServerDatasetConfiguration,
) -> Result<(), ServiceError> {
    let chunk_metadata = get_metadata_from_id_query(chunk_uuid, dataset.id, pool.clone()).await?;
    if chunk_metadata.dataset_id != dataset.id {
        return Err(ServiceError::BadRequest(
            "chunk does not belong to dataset".to_string(),
        ));
    }

    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    let mut conn = pool.get().await.unwrap();

    let transaction_result = conn
        .transaction::<_, diesel::result::Error, _>(|conn| {
            async move {
                {
                    diesel::delete(
                        chunk_files_columns::chunk_files
                            .filter(chunk_files_columns::chunk_id.eq(chunk_uuid)),
                    )
                    .execute(conn)
                    .await?;

                    diesel::delete(
                        chunk_group_bookmarks_columns::chunk_group_bookmarks.filter(
                            chunk_group_bookmarks_columns::chunk_metadata_id.eq(chunk_uuid),
                        ),
                    )
                    .execute(conn)
                    .await?;

                    let deleted_chunk_collision_count = diesel::delete(
                        chunk_collisions_columns::chunk_collisions
                            .filter(chunk_collisions_columns::chunk_id.eq(chunk_uuid)),
                    )
                    .execute(conn)
                    .await?;

                    if deleted_chunk_collision_count > 0 {
                        // there cannot be collisions for a collision, just delete the chunk_metadata without issue
                        diesel::delete(
                            chunk_metadata_columns::chunk_metadata
                                .filter(chunk_metadata_columns::id.eq(chunk_uuid))
                                .filter(chunk_metadata_columns::dataset_id.eq(dataset.id)),
                        )
                        .execute(conn)
                        .await?;

                        return Ok(TransactionResult::ChunkCollisionNotDetected);
                    }

                    let chunk_collisions: Vec<(ChunkCollision, ChunkMetadata)> =
                        chunk_collisions_columns::chunk_collisions
                            .inner_join(
                                chunk_metadata_columns::chunk_metadata
                                    .on(chunk_metadata_columns::qdrant_point_id
                                        .eq(chunk_collisions_columns::collision_qdrant_id)),
                            )
                            .filter(chunk_metadata_columns::id.eq(chunk_uuid))
                            .filter(chunk_metadata_columns::dataset_id.eq(dataset.id))
                            .select((ChunkCollision::as_select(), ChunkMetadata::as_select()))
                            .order_by(chunk_collisions_columns::created_at.asc())
                            .load::<(ChunkCollision, ChunkMetadata)>(conn)
                            .await?;

                    if !chunk_collisions.is_empty() {
                        // get the first collision as the latest collision
                        let latest_collision = match chunk_collisions.get(0) {
                            Some(x) => x.0.clone(),
                            None => chunk_collisions[0].0.clone(),
                        };

                        let mut latest_collision_metadata = match chunk_collisions.get(0) {
                            Some(x) => x.1.clone(),
                            None => chunk_collisions[0].1.clone(),
                        };

                        // update all collisions except latest_collision to point to a qdrant_id of None
                        diesel::update(
                            chunk_collisions_columns::chunk_collisions.filter(
                                chunk_collisions_columns::id.eq_any(
                                    chunk_collisions
                                        .iter()
                                        .filter(|x| x.0.id != latest_collision.id)
                                        .map(|x| x.0.id)
                                        .collect::<Vec<uuid::Uuid>>(),
                                ),
                            ),
                        )
                        .set(
                            chunk_collisions_columns::collision_qdrant_id
                                .eq::<Option<uuid::Uuid>>(None),
                        )
                        .execute(conn)
                        .await?;

                        // delete latest_collision from chunk_collisions
                        diesel::delete(
                            chunk_collisions_columns::chunk_collisions
                                .filter(chunk_collisions_columns::id.eq(latest_collision.id)),
                        )
                        .execute(conn)
                        .await?;

                        // delete the original chunk_metadata
                        diesel::delete(
                            chunk_metadata_columns::chunk_metadata
                                .filter(chunk_metadata_columns::id.eq(chunk_uuid))
                                .filter(chunk_metadata_columns::dataset_id.eq(dataset.id)),
                        )
                        .execute(conn)
                        .await?;

                        // set the chunk_metadata of latest_collision to have the qdrant_point_id of the original chunk_metadata
                        diesel::update(
                            chunk_metadata_columns::chunk_metadata
                                .filter(chunk_metadata_columns::id.eq(latest_collision.chunk_id))
                                .filter(chunk_metadata_columns::dataset_id.eq(dataset.id)),
                        )
                        .set((chunk_metadata_columns::qdrant_point_id
                            .eq(latest_collision.collision_qdrant_id),))
                        .execute(conn)
                        .await?;

                        // set the collision_qdrant_id of all other collisions to be the same as they were to begin with
                        diesel::update(
                            chunk_collisions_columns::chunk_collisions.filter(
                                chunk_collisions_columns::id.eq_any(
                                    chunk_collisions
                                        .iter()
                                        .skip(1)
                                        .map(|x| x.0.id)
                                        .collect::<Vec<uuid::Uuid>>(),
                                ),
                            ),
                        )
                        .set((chunk_collisions_columns::collision_qdrant_id
                            .eq(latest_collision.collision_qdrant_id),))
                        .execute(conn)
                        .await?;

                        latest_collision_metadata.qdrant_point_id =
                            latest_collision.collision_qdrant_id;

                        return Ok(TransactionResult::ChunkCollisionDetected(
                            latest_collision_metadata,
                        ));
                    }

                    // if there were no collisions, just delete the chunk_metadata without issue
                    diesel::delete(
                        chunk_metadata_columns::chunk_metadata
                            .filter(chunk_metadata_columns::id.eq(chunk_uuid))
                            .filter(chunk_metadata_columns::dataset_id.eq(dataset.id)),
                    )
                    .execute(conn)
                    .await?;

                    Ok(TransactionResult::ChunkCollisionNotDetected)
                }
            }
            .scope_boxed()
        })
        .await;

    let qdrant_collection = config.QDRANT_COLLECTION_NAME;

    let qdrant =
        get_qdrant_connection(Some(&config.QDRANT_URL), Some(&config.QDRANT_API_KEY)).await?;
    match transaction_result {
        Ok(result) => {
            match result {
                TransactionResult::ChunkCollisionNotDetected => {
                    let _ = qdrant
                        .delete_points(
                            qdrant_collection,
                            None,
                            &vec![<String as Into<PointId>>::into(
                                chunk_metadata
                                    .qdrant_point_id
                                    .unwrap_or_default()
                                    .to_string(),
                            )]
                            .into(),
                            None,
                        )
                        .await
                        .map_err(|_e| {
                            Err::<(), ServiceError>(ServiceError::BadRequest(
                                "Failed to delete chunk from qdrant".to_string(),
                            ))
                        });
                }
                TransactionResult::ChunkCollisionDetected(latest_collision_metadata) => {
                    let collision_content = latest_collision_metadata
                        .chunk_html
                        .clone()
                        .unwrap_or(latest_collision_metadata.content.clone());

                    let new_embedding_vectors = create_embeddings(
                        vec![collision_content],
                        "doc",
                        ServerDatasetConfiguration::from_json(dataset.server_configuration.clone()),
                    )
                    .await
                    .map_err(|_e| {
                        ServiceError::BadRequest("Failed to create embedding for chunk".to_string())
                    })?;

                    let new_embedding_vector = new_embedding_vectors.get(0).ok_or(ServiceError::BadRequest(
                    "Failed to get embedding vector due to empty result from create_embedding".to_string(),
                ))?
                .clone();

                    let _ = qdrant
                        .update_vectors_blocking(
                            qdrant_collection,
                            None,
                            &[PointVectors {
                                id: Some(<String as Into<PointId>>::into(
                                    latest_collision_metadata
                                        .qdrant_point_id
                                        .unwrap_or_default()
                                        .to_string(),
                                )),
                                vectors: Some(new_embedding_vector.into()),
                            }],
                            None,
                        )
                        .await
                        .map_err(|_e| {
                            Err::<(), ServiceError>(ServiceError::BadRequest(
                                "Failed to update chunk in qdrant".to_string(),
                            ))
                        });
                }
            }
        }

        Err(_) => {
            return Err(ServiceError::BadRequest(
                "Failed to delete chunk data".to_string(),
            ))
        }
    };

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn get_qdrant_id_from_chunk_id_query(
    chunk_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<uuid::Uuid, ServiceError> {
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    let qdrant_point_ids: Vec<(Option<uuid::Uuid>, Option<uuid::Uuid>)> =
        chunk_metadata_columns::chunk_metadata
            .left_outer_join(
                chunk_collisions_columns::chunk_collisions
                    .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
            )
            .select((
                chunk_metadata_columns::qdrant_point_id,
                chunk_collisions_columns::collision_qdrant_id.nullable(),
            ))
            .filter(chunk_metadata_columns::id.eq(chunk_id))
            .load(&mut conn)
            .await
            .map_err(|_err| {
                ServiceError::BadRequest(
                    "Failed to get qdrant_point_id and collision_qdrant_id".to_string(),
                )
            })?;

    match qdrant_point_ids.get(0) {
        Some(x) => match x.0 {
            Some(y) => Ok(y),
            None => match x.1 {
                Some(y) => Ok(y),
                None => Err(ServiceError::BadRequest(
                    "Both qdrant_point_id and collision_qdrant_id are None".to_string(),
                )),
            },
        },
        None => Err(ServiceError::BadRequest(
            "Failed to get qdrant_point_id for chunk_id".to_string(),
        )),
    }
}

#[tracing::instrument]
pub fn find_relevant_sentence(
    input: ChunkMetadata,
    query: String,
    split_chars: Vec<String>,
) -> Result<ChunkMetadata, ServiceError> {
    let content = &input.chunk_html.clone().unwrap_or(input.content.clone());
    let mut engine: SimSearch<String> = SimSearch::new();
    let mut split_content = content
        .split_inclusive(|c: char| split_chars.contains(&c.to_string()))
        .map(|x| x.to_string())
        .collect::<Vec<String>>();

    //insert all sentences into the engine
    split_content
        .iter()
        .enumerate()
        .for_each(|(idx, sentence)| {
            engine.insert(
                format!("{:?}¬{}", idx, &sentence.clone()),
                &sentence.clone(),
            );
        });

    let mut new_output = input;

    //search for the query
    let results = engine.search(&query);
    let amount = if split_content.len() < 5 { 2 } else { 3 };
    for x in results.iter().take(amount) {
        let split_x: Vec<&str> = x.split('¬').collect();
        if split_x.len() < 2 {
            continue;
        }
        let sentence_index = split_x[0].parse::<usize>().unwrap();
        let highlighted_sentence = format!("{}{}{}", "<b>", split_x[1], "</b>");
        split_content[sentence_index] = highlighted_sentence;
    }

    new_output.chunk_html = Some(split_content.iter().join(""));
    Ok(new_output)
}

#[tracing::instrument(skip(pool))]
pub async fn get_row_count_for_dataset_id_query(
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<usize, ServiceError> {
    use crate::data::schema::dataset_usage_counts::dsl as dataset_usage_counts_columns;

    let mut conn = pool.get().await.expect("Failed to get connection to db");

    let chunk_metadata_count = dataset_usage_counts_columns::dataset_usage_counts
        .filter(dataset_usage_counts_columns::dataset_id.eq(dataset_id))
        .select(dataset_usage_counts_columns::chunk_count)
        .first::<i32>(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::BadRequest("Failed to get chunk count for dataset".to_string())
        })?;

    Ok(chunk_metadata_count as usize)
}
