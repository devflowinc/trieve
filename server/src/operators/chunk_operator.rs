use crate::data::models::{
    self, ChunkCollision, ChunkFile, ChunkMetadataWithFileData, Dataset, Event,
    FullTextSearchResult, ServerDatasetConfiguration, UnifiedId,
};
use crate::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use crate::operators::model_operator::create_embedding;
use crate::operators::qdrant_operator::get_qdrant_connection;
use crate::operators::search_operator::get_metadata_query;
use crate::{
    data::models::{ChunkMetadata, Pool},
    errors::DefaultError,
};
use actix_web::web;
use diesel::{Connection, JoinOnDsl, NullableExpressionMethods, SelectableHelper};
use itertools::Itertools;
use qdrant_client::qdrant::{PointId, PointVectors};
use simsearch::SimSearch;

#[tracing::instrument(skip(pool))]
pub async fn get_metadata_from_point_ids(
    point_ids: Vec<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkMetadataWithFileData>, DefaultError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().expect("Failed to get connection from pool");

    let chunk_metadata: Vec<ChunkMetadata> = chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::qdrant_point_id.eq_any(&point_ids))
        .select(ChunkMetadata::as_select())
        .load::<ChunkMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    let converted_chunks: Vec<FullTextSearchResult> = chunk_metadata
        .iter()
        .map(|chunk| <ChunkMetadata as Into<FullTextSearchResult>>::into(chunk.clone()))
        .collect::<Vec<FullTextSearchResult>>();

    let chunk_metadata_with_file_id =
        get_metadata_query(converted_chunks, conn).map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    Ok(chunk_metadata_with_file_id)
}

pub async fn get_point_ids_from_unified_chunk_ids(
    chunk_ids: Vec<UnifiedId>,
    pool: web::Data<Pool>,
) -> Result<Vec<uuid::Uuid>, DefaultError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().unwrap();

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
            .select(chunk_metadata_columns::qdrant_point_id)
            .load::<Option<uuid::Uuid>>(&mut conn)
            .map_err(|_| DefaultError {
                message: "Failed to load metadata",
            })?
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
            .select(chunk_metadata_columns::qdrant_point_id)
            .load::<Option<uuid::Uuid>>(&mut conn)
            .map_err(|_| DefaultError {
                message: "Failed to load metadata",
            })?
            .into_iter()
            .flatten()
            .collect(),
    };

    Ok(qdrant_point_ids)
}

pub struct ChunkMetadataWithQdrantId {
    pub metadata: ChunkMetadataWithFileData,
    pub qdrant_id: uuid::Uuid,
}

#[tracing::instrument(skip(pool))]
pub fn get_metadata_and_collided_chunks_from_point_ids_query(
    point_ids: Vec<uuid::Uuid>,
    get_collisions: bool,
    pool: web::Data<Pool>,
) -> Result<
    (
        Vec<ChunkMetadataWithFileData>,
        Vec<ChunkMetadataWithQdrantId>,
    ),
    DefaultError,
> {
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

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

    let chunk_search_result = {
        let mut conn = pool.get().unwrap();
        let chunk_metadata: Vec<ChunkMetadata> = chunk_metadata_columns::chunk_metadata
            .filter(chunk_metadata_columns::qdrant_point_id.eq_any(&point_ids))
            .select(ChunkMetadata::as_select())
            .limit(500)
            .load::<ChunkMetadata>(&mut conn)
            .map_err(|_| DefaultError {
                message: "Failed to load metadata",
            })?;

        chunk_metadata
            .iter()
            .map(|chunk| <ChunkMetadata as Into<FullTextSearchResult>>::into(chunk.clone()))
            .collect::<Vec<FullTextSearchResult>>()
    };

    chunk_search_span.finish();

    let collision_search_span = transaction.start_child(
        "Fetching matching points from qdrant",
        "Fetching matching points from qdrant",
    );

    let (collided_search_result, collided_qdrant_ids) = {
        let mut conn = pool.get().unwrap();
        if get_collisions {
            let chunk_metadata: Vec<(ChunkMetadata, uuid::Uuid)> =
                chunk_collisions_columns::chunk_collisions
                    .inner_join(
                        chunk_metadata_columns::chunk_metadata
                            .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
                    )
                    .select((
                        ChunkMetadata::as_select(),
                        (chunk_collisions_columns::collision_qdrant_id.assume_not_null()),
                    ))
                    .filter(chunk_collisions_columns::collision_qdrant_id.eq_any(point_ids))
                    // TODO: Properly handle this and remove the arbitrary limit
                    .limit(500)
                    .load::<(ChunkMetadata, uuid::Uuid)>(&mut conn)
                    .map_err(|_| DefaultError {
                        message: "Failed to load metadata",
                    })?;

            let collided_qdrant_ids = chunk_metadata
                .iter()
                .map(|(_, qdrant_id)| *qdrant_id)
                .collect::<Vec<uuid::Uuid>>();

            let converted_chunks: Vec<FullTextSearchResult> = chunk_metadata
                .iter()
                .map(|chunk| <ChunkMetadata as Into<FullTextSearchResult>>::into(chunk.0.clone()))
                .collect::<Vec<FullTextSearchResult>>();

            (converted_chunks, collided_qdrant_ids)
        } else {
            let chunk_metadata: Vec<(ChunkMetadata, uuid::Uuid)> =
                chunk_collisions_columns::chunk_collisions
                    .inner_join(
                        chunk_metadata_columns::chunk_metadata
                            .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
                    )
                    .select((
                        ChunkMetadata::as_select(),
                        (chunk_collisions_columns::collision_qdrant_id.assume_not_null()),
                    ))
                    .filter(chunk_collisions_columns::collision_qdrant_id.eq_any(point_ids))
                    .load::<(ChunkMetadata, uuid::Uuid)>(&mut conn)
                    .map_err(|_| DefaultError {
                        message: "Failed to load metadata",
                    })?;

            let converted_chunks: Vec<FullTextSearchResult> = chunk_metadata
                .iter()
                .map(|chunk| <ChunkMetadata as Into<FullTextSearchResult>>::into(chunk.0.clone()))
                .collect::<Vec<FullTextSearchResult>>();

            (converted_chunks, vec![])
        }
    };

    collision_search_span.finish();

    let iter_mapping_extra_data_span = transaction.start_child(
        "Iter mapping to add extra data",
        "Iter mapping to add extra data",
    );

    let (chunk_metadata_with_file_id, collided_chunk_metadata_with_file_id) = {
        let conn = pool.get().unwrap();
        // Assuming that get_metadata will maintain the order of the Vec<> returned
        let split_index = chunk_search_result.len();
        let all_chunks = chunk_search_result
            .iter()
            .chain(collided_search_result.iter())
            .cloned()
            .collect::<Vec<FullTextSearchResult>>();

        let all_metadata = get_metadata_query(all_chunks, conn).map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

        let meta_chunks = all_metadata
            .iter()
            .take(split_index)
            .cloned()
            .collect::<Vec<ChunkMetadataWithFileData>>();

        let meta_collided = all_metadata
            .iter()
            .skip(split_index)
            .cloned()
            .collect::<Vec<ChunkMetadataWithFileData>>();

        (meta_chunks, meta_collided)
    };

    let chunk_metadatas_with_collided_qdrant_ids = collided_chunk_metadata_with_file_id
        .iter()
        .zip(collided_qdrant_ids.iter())
        .map(|(chunk, qdrant_id)| ChunkMetadataWithQdrantId {
            metadata: chunk.clone(),
            qdrant_id: *qdrant_id,
        })
        .collect::<Vec<ChunkMetadataWithQdrantId>>();

    iter_mapping_extra_data_span.finish();
    transaction.finish();

    Ok((
        chunk_metadata_with_file_id,
        chunk_metadatas_with_collided_qdrant_ids,
    ))
}

#[tracing::instrument(skip(pool))]
pub fn get_metadata_from_id_query(
    chunk_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, DefaultError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    let mut conn = pool.get().unwrap();

    chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::id.eq(chunk_id))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
        .select(ChunkMetadata::as_select())
        .first::<ChunkMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })
}

#[tracing::instrument(skip(pool))]
pub fn get_metadata_from_tracking_id_query(
    tracking_id: String,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, DefaultError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().unwrap();

    chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::tracking_id.eq(tracking_id))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_uuid))
        .select(ChunkMetadata::as_select())
        .first::<ChunkMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })
}

#[tracing::instrument(skip(pool))]
pub fn get_metadata_from_ids_query(
    chunk_ids: Vec<uuid::Uuid>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkMetadataWithFileData>, DefaultError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().unwrap();

    let metadatas: Vec<ChunkMetadata> = chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::id.eq_any(chunk_ids))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_uuid))
        .select(ChunkMetadata::as_select())
        .load::<ChunkMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;
    let full_text_metadatas = metadatas
        .iter()
        .map_into::<FullTextSearchResult>()
        .collect_vec();

    Ok(get_metadata_query(full_text_metadatas, conn).unwrap_or_default())
}

#[tracing::instrument(skip(pool))]
pub async fn insert_chunk_metadata_query(
    chunk_data: ChunkMetadata,
    file_uuid: Option<uuid::Uuid>,
    dataset_uuid: uuid::Uuid,
    upsert_by_tracking_id: bool,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, DefaultError> {
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_metadata::dsl::*;

    if upsert_by_tracking_id && chunk_data.tracking_id.is_some() {
        if let Ok(existing_chunk) = get_metadata_from_tracking_id_query(
            chunk_data
                .tracking_id
                .clone()
                .expect("tracking_id must be Some at this point"),
            chunk_data.dataset_id,
            pool.clone(),
        ) {
            let mut update_chunk = chunk_data.clone();
            update_chunk.id = existing_chunk.id;
            update_chunk.created_at = existing_chunk.created_at;
            update_chunk.qdrant_point_id = existing_chunk.qdrant_point_id;

            let updated_chunk =
                update_chunk_metadata_query(update_chunk, file_uuid, dataset_uuid, pool.clone())
                    .await?;

            return Ok(updated_chunk);
        }
    }

    let mut conn = pool.get().unwrap();

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::insert_into(chunk_metadata)
            .values(&chunk_data)
            .execute(conn)?;

        if file_uuid.is_some() {
            diesel::insert_into(chunk_files_columns::chunk_files)
                .values(&ChunkFile::from_details(
                    chunk_data.id,
                    file_uuid.expect("file_uuid should be Some"),
                ))
                .execute(conn)?;
        }

        Ok(())
    });

    match transaction_result {
        Ok(_) => (),
        Err(e) => {
            log::info!("Failed to insert chunk metadata: {:?}", e);
            match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => {
                    return Err(DefaultError {
                        message: "Duplicate tracking_id",
                    });
                }
                _ => {
                    return Err(DefaultError {
                        message: "Failed to insert card metadata",
                    });
                }
            }
        }
    };

    Ok(chunk_data)
}

#[tracing::instrument(skip(pool))]
pub fn insert_duplicate_chunk_metadata_query(
    chunk_data: ChunkMetadata,
    duplicate_chunk: uuid::Uuid,
    file_uuid: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, DefaultError> {
    use crate::data::schema::chunk_collisions::dsl::*;
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_metadata::dsl::*;

    let mut conn = pool.get().unwrap();

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::insert_into(chunk_metadata)
            .values(&chunk_data)
            .execute(conn)?;

        //insert duplicate into chunk_collisions
        diesel::insert_into(chunk_collisions)
            .values(&ChunkCollision::from_details(
                chunk_data.id,
                duplicate_chunk,
            ))
            .execute(conn)?;

        if file_uuid.is_some() {
            diesel::insert_into(chunk_files_columns::chunk_files)
                .values(&ChunkFile::from_details(
                    chunk_data.id,
                    file_uuid.expect("file_uuid should be some"),
                ))
                .execute(conn)?;
        }

        Ok(())
    });

    match transaction_result {
        Ok(_) => (),
        Err(_) => {
            return Err(DefaultError {
                message: "Failed to insert duplicate chunk metadata",
            })
        }
    };
    Ok(chunk_data)
}

pub async fn insert_bulk_chunk_metadatas_query(
    chunk_metadatas: Vec<ChunkMetadata>,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkMetadata>, DefaultError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::events::dsl as events_columns;

    let events = chunk_metadatas
        .iter()
        .map(|chunk_metadata| {
            Event::from_details(
                chunk_metadata.dataset_id,
                models::EventType::CardUploaded {
                    chunk_id: chunk_metadata.id,
                },
            )
        })
        .collect::<Vec<Event>>();

    let mut conn = pool.get().unwrap();

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::insert_into(chunk_metadata_columns::chunk_metadata)
            .values(&chunk_metadatas)
            .on_conflict((
                chunk_metadata_columns::tracking_id,
                chunk_metadata_columns::dataset_id,
            ))
            .do_nothing()
            .execute(conn)?;

        diesel::insert_into(events_columns::events)
            .values(&events)
            .execute(conn)?;

        Ok(chunk_metadatas)
    });

    match transaction_result {
        Ok(chunk_metadatas) => Ok(chunk_metadatas),
        Err(e) => {
            log::error!("Failed to bulk insert chunk metadatas: {:?}", e);
            Err(DefaultError {
                message: "Failed to bulk insert chunk metadatas",
            })
        }
    }
}

#[tracing::instrument(skip(pool))]
pub async fn update_chunk_metadata_query(
    chunk_data: ChunkMetadata,
    file_uuid: Option<uuid::Uuid>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, DefaultError> {
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().unwrap();

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        let updated_chunks: Vec<ChunkMetadata> = diesel::update(
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
        .load(conn)?;

        if file_uuid.is_some() {
            diesel::insert_into(chunk_files_columns::chunk_files)
                .values(ChunkFile::from_details(
                    chunk_data.id,
                    file_uuid.expect("file_uuid should be some"),
                ))
                .execute(conn)?;
        }

        Ok(updated_chunks)
    });

    match transaction_result {
        Ok(updated_chunks) => {
            if let Some(updated_chunk) = updated_chunks.first() {
                Ok(updated_chunk.clone())
            } else {
                Err(DefaultError {
                    message: "Failed to update chunk metadata",
                })
            }
        }
        Err(e) => {
            log::error!("Failed to update chunk metadata: {:?}", e);
            Err(DefaultError {
                message: "Failed to update chunk metadata",
            })
        }
    }
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
) -> Result<(), DefaultError> {
    let chunk_metadata = get_metadata_from_id_query(chunk_uuid, dataset.id, pool.clone())?;
    if chunk_metadata.dataset_id != dataset.id {
        return Err(DefaultError {
            message: "chunk does not belong to dataset",
        });
    }

    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    let mut conn = pool.get().unwrap();

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        {
            diesel::delete(
                chunk_files_columns::chunk_files
                    .filter(chunk_files_columns::chunk_id.eq(chunk_uuid)),
            )
            .execute(conn)?;

            diesel::delete(
                chunk_group_bookmarks_columns::chunk_group_bookmarks
                    .filter(chunk_group_bookmarks_columns::chunk_metadata_id.eq(chunk_uuid)),
            )
            .execute(conn)?;

            let deleted_chunk_collision_count = diesel::delete(
                chunk_collisions_columns::chunk_collisions
                    .filter(chunk_collisions_columns::chunk_id.eq(chunk_uuid)),
            )
            .execute(conn)?;

            if deleted_chunk_collision_count > 0 {
                // there cannot be collisions for a collision, just delete the chunk_metadata without issue
                diesel::delete(
                    chunk_metadata_columns::chunk_metadata
                        .filter(chunk_metadata_columns::id.eq(chunk_uuid))
                        .filter(chunk_metadata_columns::dataset_id.eq(dataset.id)),
                )
                .execute(conn)?;

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
                    .load::<(ChunkCollision, ChunkMetadata)>(conn)?;

            if !chunk_collisions.is_empty() {
                // get the first collision as the latest collision
                let latest_collision = match chunk_collisions.first() {
                    Some(x) => x.0.clone(),
                    None => chunk_collisions[0].0.clone(),
                };

                let mut latest_collision_metadata = match chunk_collisions.first() {
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
                .set(chunk_collisions_columns::collision_qdrant_id.eq::<Option<uuid::Uuid>>(None))
                .execute(conn)?;

                // delete latest_collision from chunk_collisions
                diesel::delete(
                    chunk_collisions_columns::chunk_collisions
                        .filter(chunk_collisions_columns::id.eq(latest_collision.id)),
                )
                .execute(conn)?;

                // delete the original chunk_metadata
                diesel::delete(
                    chunk_metadata_columns::chunk_metadata
                        .filter(chunk_metadata_columns::id.eq(chunk_uuid))
                        .filter(chunk_metadata_columns::dataset_id.eq(dataset.id)),
                )
                .execute(conn)?;

                // set the chunk_metadata of latest_collision to have the qdrant_point_id of the original chunk_metadata
                diesel::update(
                    chunk_metadata_columns::chunk_metadata
                        .filter(chunk_metadata_columns::id.eq(latest_collision.chunk_id))
                        .filter(chunk_metadata_columns::dataset_id.eq(dataset.id)),
                )
                .set((chunk_metadata_columns::qdrant_point_id
                    .eq(latest_collision.collision_qdrant_id),))
                .execute(conn)?;

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
                .execute(conn)?;

                latest_collision_metadata.qdrant_point_id = latest_collision.collision_qdrant_id;

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
            .execute(conn)?;

            Ok(TransactionResult::ChunkCollisionNotDetected)
        }
    });

    let qdrant_collection = config.QDRANT_COLLECTION_NAME;

    let qdrant =
        get_qdrant_connection(Some(&config.QDRANT_URL), Some(&config.QDRANT_API_KEY)).await?;
    match transaction_result {
        Ok(result) => match result {
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
                        Err::<(), DefaultError>(DefaultError {
                            message: "Failed to delete chunk from qdrant",
                        })
                    });
            }
            TransactionResult::ChunkCollisionDetected(latest_collision_metadata) => {
                let collision_content = latest_collision_metadata
                    .chunk_html
                    .clone()
                    .unwrap_or(latest_collision_metadata.content.clone());

                let new_embedding_vector = create_embedding(
                    collision_content.as_str(),
                    "doc",
                    ServerDatasetConfiguration::from_json(dataset.server_configuration.clone()),
                )
                .await
                .map_err(|_e| DefaultError {
                    message: "Failed to create embedding for chunk",
                })?;

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
                        Err::<(), DefaultError>(DefaultError {
                            message: "Failed to update chunk in qdrant",
                        })
                    });
            }
        },

        Err(_) => {
            return Err(DefaultError {
                message: "Failed to delete chunk data",
            })
        }
    };

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub fn get_qdrant_id_from_chunk_id_query(
    chunk_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<uuid::Uuid, DefaultError> {
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().unwrap();

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
            .map_err(|_err| DefaultError {
                message: "Failed to get qdrant_point_id and collision_qdrant_id",
            })?;

    match qdrant_point_ids.first() {
        Some(x) => match x.0 {
            Some(y) => Ok(y),
            None => match x.1 {
                Some(y) => Ok(y),
                None => Err(DefaultError {
                    message: "Both qdrant_point_id and collision_qdrant_id are None",
                }),
            },
        },
        None => Err(DefaultError {
            message: "Failed to get qdrant_point_id for chunk_id",
        }),
    }
}

#[tracing::instrument]
pub fn find_relevant_sentence(
    input: ChunkMetadataWithFileData,
    query: String,
    split_chars: Vec<String>,
) -> Result<ChunkMetadataWithFileData, DefaultError> {
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
pub fn get_row_count_for_dataset_id_query(
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i32, DefaultError> {
    use crate::data::schema::dataset_usage_counts::dsl as dataset_usage_counts_columns;

    let mut conn = pool.get().expect("Failed to get connection to db");

    let chunk_metadata_count = dataset_usage_counts_columns::dataset_usage_counts
        .filter(dataset_usage_counts_columns::dataset_id.eq(dataset_id))
        .select(dataset_usage_counts_columns::chunk_count)
        .first::<i32>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to get chunk count for dataset",
        })?;

    Ok(chunk_metadata_count)
}
