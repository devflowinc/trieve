use crate::errors::ServiceError;
use crate::operators::qdrant_operator::{
    remove_bookmark_from_qdrant_query, update_group_tag_sets_in_qdrant_query,
};
use crate::{
    data::models::{
        ChunkGroup, ChunkGroupAndFileId, ChunkGroupBookmark, ChunkMetadataTable, Dataset,
        DatasetConfiguration, FileGroup, Pool, RedisPool, UnifiedId,
    },
    handlers::group_handler::GroupsBookmarkQueryResult,
    operators::chunk_operator::{delete_chunk_metadata_query, get_chunk_metadatas_from_point_ids},
};
use actix_web::web;
use diesel::prelude::*;
use diesel::{dsl::sql, sql_types::Text, upsert::excluded};
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[tracing::instrument(skip(pool))]
pub async fn get_group_from_tracking_id_query(
    tracking_id: String,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkGroupAndFileId, ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group_columns;
    use crate::data::schema::groups_from_files::dsl as groups_from_files_columns;

    let mut conn = pool.get().await.unwrap();

    let (group, file_id): (ChunkGroup, Option<uuid::Uuid>) = chunk_group_columns::chunk_group
        .left_join(
            groups_from_files_columns::groups_from_files
                .on(chunk_group_columns::id.eq(groups_from_files_columns::group_id)),
        )
        .filter(chunk_group_columns::dataset_id.eq(dataset_uuid))
        .filter(chunk_group_columns::tracking_id.eq(tracking_id))
        .select((
            ChunkGroup::as_select(),
            groups_from_files_columns::file_id.nullable(),
        ))
        .first::<(ChunkGroup, Option<uuid::Uuid>)>(&mut conn)
        .await
        .map_err(|_| ServiceError::NotFound("Group with tracking_id not found".to_string()))?;

    Ok(ChunkGroupAndFileId::from_group(group, file_id))
}

#[tracing::instrument(skip(pool))]
pub async fn get_group_ids_from_tracking_ids_query(
    tracking_ids: Vec<String>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<(uuid::Uuid, Option<String>)>, ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group_columns;

    let mut conn = pool.get().await.unwrap();

    let group_id_tracking_ids = chunk_group_columns::chunk_group
        .filter(chunk_group_columns::dataset_id.eq(dataset_uuid))
        .filter(chunk_group_columns::tracking_id.eq_any(tracking_ids))
        .select((chunk_group_columns::id, chunk_group_columns::tracking_id))
        .load::<(uuid::Uuid, Option<String>)>(&mut conn)
        .await
        .map_err(|_err| ServiceError::BadRequest("Groups not found".to_string()))?;

    Ok(group_id_tracking_ids)
}

#[tracing::instrument(skip(pool))]
pub async fn update_group_by_tracking_id_query(
    tracking_id: String,
    dataset_uuid: uuid::Uuid,
    new_name: Option<String>,
    new_description: Option<String>,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group_columns;

    let mut conn = pool.get().await.unwrap();

    diesel::update(
        chunk_group_columns::chunk_group
            .filter(chunk_group_columns::dataset_id.eq(dataset_uuid))
            .filter(chunk_group_columns::tracking_id.eq(tracking_id)),
    )
    .set((
        chunk_group_columns::name.eq(new_name.unwrap_or("".to_string())),
        chunk_group_columns::description.eq(new_description.unwrap_or("".to_string())),
    ))
    .execute(&mut conn)
    .await
    .map_err(|err| ServiceError::BadRequest(format!("Error updating group {:?}", err)))?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn create_groups_query(
    new_groups: Vec<ChunkGroup>,
    upsert_by_tracking_id: bool,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkGroup>, ServiceError> {
    if new_groups.is_empty() {
        return Ok(vec![]);
    }

    use crate::data::schema::chunk_group::dsl as chunk_group_columns;

    let mut conn = pool.get().await.unwrap();

    let inserted_groups = if upsert_by_tracking_id {
        diesel::insert_into(chunk_group_columns::chunk_group)
            .values(&new_groups)
            .on_conflict((
                chunk_group_columns::dataset_id,
                chunk_group_columns::tracking_id,
            ))
            .do_update()
            .set((
                chunk_group_columns::name.eq(excluded(chunk_group_columns::name)),
                chunk_group_columns::description.eq(excluded(chunk_group_columns::description)),
                chunk_group_columns::metadata.eq(excluded(chunk_group_columns::metadata)),
                chunk_group_columns::tag_set.eq(excluded(chunk_group_columns::tag_set)),
            ))
            .returning(ChunkGroup::as_select())
            .get_results::<ChunkGroup>(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Error upserting groups {:?}", e);
                ServiceError::BadRequest("Error upserting groups".to_string())
            })?
    } else {
        diesel::insert_into(chunk_group_columns::chunk_group)
            .values(&new_groups)
            .on_conflict_do_nothing()
            .get_results::<ChunkGroup>(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Error inserting groups {:?}", e);
                ServiceError::BadRequest("Error inserting groups".to_string())
            })?
    };

    Ok(inserted_groups)
}

#[tracing::instrument(skip(pool))]
pub async fn get_groups_for_dataset_query(
    page: u64,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(Vec<ChunkGroupAndFileId>, Option<i32>), ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group_columns;
    use crate::data::schema::dataset_group_counts::dsl as dataset_group_count_columns;
    use crate::data::schema::groups_from_files::dsl as groups_from_files_columns;

    let page = if page == 0 { 1 } else { page };
    let mut conn = pool.get().await.unwrap();

    let group_count_result = dataset_group_count_columns::dataset_group_counts
        .filter(dataset_group_count_columns::dataset_id.eq(dataset_uuid))
        .select(dataset_group_count_columns::group_count)
        .first::<i32>(&mut conn)
        .await;

    let group_count: Option<i32> = match group_count_result {
        Ok(count) => Some(count),
        Err(_) => return Ok((vec![], None)),
    };

    let groups = chunk_group_columns::chunk_group
        .filter(chunk_group_columns::dataset_id.eq(dataset_uuid))
        .order_by(chunk_group_columns::updated_at.desc())
        .offset(((page - 1) * 10).try_into().unwrap_or(0))
        .limit(10)
        .load::<ChunkGroup>(&mut conn)
        .await
        .map_err(|_err| ServiceError::BadRequest("Error getting groups for dataset".to_string()))?;

    let file_ids = groups_from_files_columns::groups_from_files
        .filter(
            groups_from_files_columns::group_id
                .eq_any(groups.iter().map(|x| x.id).collect::<Vec<uuid::Uuid>>()),
        )
        .select((
            groups_from_files_columns::group_id,
            groups_from_files_columns::file_id,
        ))
        .load::<(uuid::Uuid, uuid::Uuid)>(&mut conn)
        .await
        .map_err(|_err| ServiceError::BadRequest("Error getting file ids".to_string()))?;

    let group_and_files = groups
        .into_iter()
        .map(|group| {
            let file_id = file_ids
                .iter()
                .find(|(group_id, _)| group.id == *group_id)
                .map(|(_, file_id)| *file_id);

            ChunkGroupAndFileId {
                id: group.id,
                dataset_id: group.dataset_id,
                name: group.name,
                description: group.description,
                tracking_id: group.tracking_id,
                tag_set: group.tag_set,
                metadata: group.metadata,
                file_id,
                created_at: group.created_at,
                updated_at: group.updated_at,
            }
        })
        .collect();

    Ok((group_and_files, group_count))
}

#[tracing::instrument(skip(pool))]
pub async fn get_group_by_id_query(
    group_id: uuid::Uuid,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkGroupAndFileId, ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group_columns;
    use crate::data::schema::groups_from_files::dsl as groups_from_files_columns;

    let mut conn = pool.get().await.unwrap();

    let (group, file_id): (ChunkGroup, Option<uuid::Uuid>) = chunk_group_columns::chunk_group
        .left_join(
            groups_from_files_columns::groups_from_files
                .on(chunk_group_columns::id.eq(groups_from_files_columns::group_id)),
        )
        .filter(chunk_group_columns::dataset_id.eq(dataset_uuid))
        .filter(chunk_group_columns::id.eq(group_id))
        .select((
            ChunkGroup::as_select(),
            groups_from_files_columns::file_id.nullable(),
        ))
        .first::<(ChunkGroup, Option<uuid::Uuid>)>(&mut conn)
        .await
        .map_err(|_| ServiceError::NotFound(format!("Group with id not found {:?}", group_id)))?;

    Ok(ChunkGroupAndFileId::from_group(group, file_id))
}

#[tracing::instrument(skip(pool))]
pub async fn delete_group_by_id_query(
    group_id: uuid::Uuid,
    dataset: Dataset,
    delete_chunks: Option<bool>,
    pool: web::Data<Pool>,
    dataset_config: DatasetConfiguration,
) -> Result<(), ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::events::dsl as events_columns;
    use crate::data::schema::groups_from_files::dsl as groups_from_files_columns;

    let mut conn = pool.get().await.unwrap();

    let delete_chunks = delete_chunks.unwrap_or(false);
    let chunks = chunk_group_bookmarks_columns::chunk_group_bookmarks
        .inner_join(chunk_metadata_columns::chunk_metadata)
        .filter(chunk_group_bookmarks_columns::group_id.eq(group_id))
        .select(ChunkMetadataTable::as_select())
        .load::<ChunkMetadataTable>(&mut conn)
        .await
        .map_err(|_err| ServiceError::BadRequest("Error getting chunks".to_string()))?;

    let transaction_result = conn
        .transaction::<_, diesel::result::Error, _>(|conn| {
            async move {
                diesel::delete(
                    events_columns::events
                        .filter(events_columns::event_type.eq("file_uploaded"))
                        .filter(
                            sql::<Text>(&format!("events.event_data->>'{}'", "group_id"))
                                .eq(group_id.to_string()),
                        ),
                )
                .execute(conn)
                .await?;

                diesel::delete(
                    groups_from_files_columns::groups_from_files
                        .filter(groups_from_files_columns::group_id.eq(group_id)),
                )
                .execute(conn)
                .await?;

                diesel::delete(
                    chunk_group_bookmarks_columns::chunk_group_bookmarks
                        .filter(chunk_group_bookmarks_columns::group_id.eq(group_id)),
                )
                .execute(conn)
                .await?;

                diesel::delete(
                    chunk_group_columns::chunk_group
                        .filter(chunk_group_columns::id.eq(group_id))
                        .filter(chunk_group_columns::dataset_id.eq(dataset.id)),
                )
                .execute(conn)
                .await?;

                Ok(())
            }
            .scope_boxed()
        })
        .await;

    if delete_chunks {
        let chunk_ids = chunks.iter().map(|chunk| chunk.id).collect();
        delete_chunk_metadata_query(
            chunk_ids,
            dataset.clone(),
            pool.clone(),
            dataset_config.clone(),
        )
        .await?;
    } else {
        let remove_chunks_from_groups_futures = chunks.iter().map(|chunk| {
            remove_bookmark_from_qdrant_query(
                chunk.qdrant_point_id,
                group_id,
                dataset_config.clone(),
            )
        });

        futures::future::join_all(remove_chunks_from_groups_futures).await;
    }

    match transaction_result {
        Ok(_) => Ok(()),
        Err(_) => Err(ServiceError::BadRequest("Error deleting group".to_string())),
    }
}

#[tracing::instrument(skip(pool))]
pub async fn update_chunk_group_query(
    group: ChunkGroup,
    pool: web::Data<Pool>,
) -> Result<ChunkGroup, ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group_columns;

    let mut conn = pool.get().await.unwrap();

    let updated_group: ChunkGroup = diesel::update(
        chunk_group_columns::chunk_group
            .filter(chunk_group_columns::id.eq(group.id))
            .filter(chunk_group_columns::dataset_id.eq(group.dataset_id)),
    )
    .set((
        chunk_group_columns::name.eq(group.name),
        chunk_group_columns::description.eq(group.description),
        chunk_group_columns::tracking_id.eq(group.tracking_id),
        chunk_group_columns::metadata.eq(group.metadata),
        chunk_group_columns::tag_set.eq(group.tag_set),
    ))
    .get_result(&mut conn)
    .await
    .map_err(|err| ServiceError::BadRequest(format!("Error updating group {:?}", err)))?;

    Ok(updated_group)
}

#[tracing::instrument(skip(pool))]
pub async fn create_chunk_bookmark_query(
    pool: web::Data<Pool>,
    bookmark: ChunkGroupBookmark,
) -> Result<uuid::Uuid, ServiceError> {
    use crate::data::schema::chunk_group_bookmarks::dsl::*;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    diesel::insert_into(chunk_group_bookmarks)
        .values(&bookmark)
        .on_conflict((group_id, chunk_metadata_id))
        .do_nothing()
        .execute(&mut conn)
        .await
        .map_err(|_err| {
            log::error!("Error creating bookmark {:}", _err);
            ServiceError::BadRequest("Error creating bookmark".to_string())
        })?;

    let qdrant_point_id = chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::id.eq(bookmark.chunk_metadata_id))
        .select(chunk_metadata_columns::qdrant_point_id)
        .first::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|_err| {
            log::error!("Error getting qdrant_point_id {:}", _err);
            ServiceError::BadRequest("Error getting qdrant_point_id".to_string())
        })?;

    Ok(qdrant_point_id)
}

#[tracing::instrument(skip(pool))]
pub async fn get_bookmarks_for_group_query(
    group_id: UnifiedId,
    page: u64,
    limit: Option<u64>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<GroupsBookmarkQueryResult, ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group_columns;
    let mut conn = pool.clone().get().await.unwrap();

    let group_uuid = match group_id {
        UnifiedId::TrackingId(id) => chunk_group_columns::chunk_group
            .filter(chunk_group_columns::tracking_id.eq(id))
            .select(chunk_group_columns::id)
            .first::<uuid::Uuid>(&mut conn)
            .await
            .map_err(|_| ServiceError::NotFound("Group with id not found".to_string()))?,
        UnifiedId::TrieveUuid(id) => id,
    };

    let chunks_future = get_chunk_point_ids_in_chunk_group_query(
        group_uuid,
        page,
        limit.unwrap_or(10),
        dataset_uuid,
        pool.clone(),
    );

    let chunk_group_future = get_group_by_id_query(group_uuid, dataset_uuid, pool.clone());

    let chunk_count_future =
        get_chunk_metadata_count_in_chunk_group_query(group_uuid, pool.clone());

    let (chunk_metadata_point_ids, chunk_count_result, chunk_group_result) =
        futures::future::join3(chunks_future, chunk_count_future, chunk_group_future).await;

    let chunk_metadata_point_ids = chunk_metadata_point_ids?;
    let chunk_metadatas =
        get_chunk_metadatas_from_point_ids(chunk_metadata_point_ids, pool.clone()).await?;
    let chunk_metadata_string_tag_sets = chunk_metadatas
        .iter()
        .map(|chunk_metadata| chunk_metadata.metadata().into())
        .collect();

    let chunk_count = chunk_count_result?;
    let chunk_group = chunk_group_result?;

    Ok(GroupsBookmarkQueryResult {
        chunks: chunk_metadata_string_tag_sets,
        group: chunk_group,
        total_pages: (chunk_count as f64 / 10.0).ceil() as u64,
    })
}
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GroupsForChunk {
    pub chunk_uuid: uuid::Uuid,
    pub slim_groups: Vec<ChunkGroupAndFileId>,
}

#[tracing::instrument(skip(pool))]
pub async fn get_groups_for_bookmark_query(
    chunk_ids: Vec<uuid::Uuid>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<GroupsForChunk>, ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::groups_from_files::dsl as groups_from_files_columns;

    let mut conn = pool.get().await.unwrap();

    let groups: Vec<(ChunkGroup, uuid::Uuid, Option<uuid::Uuid>)> =
        chunk_group_columns::chunk_group
            .left_join(
                chunk_group_bookmarks_columns::chunk_group_bookmarks
                    .on(chunk_group_columns::id.eq(chunk_group_bookmarks_columns::group_id)),
            )
            .left_join(
                groups_from_files_columns::groups_from_files
                    .on(chunk_group_columns::id.eq(groups_from_files_columns::group_id)),
            )
            .filter(chunk_group_columns::dataset_id.eq(dataset_uuid))
            .filter(chunk_group_bookmarks_columns::chunk_metadata_id.eq_any(chunk_ids))
            .select((
                (ChunkGroup::as_select()),
                chunk_group_bookmarks_columns::chunk_metadata_id.nullable(),
                groups_from_files_columns::file_id.nullable(),
            ))
            .load::<(ChunkGroup, Option<uuid::Uuid>, Option<uuid::Uuid>)>(&mut conn)
            .await
            .map_err(|_err| {
                ServiceError::BadRequest("Error getting groups for chunks".to_string())
            })?
            .into_iter()
            .map(|(chunk_group, chunk_id, file_id)| match chunk_id {
                Some(chunk_id) => (chunk_group, chunk_id, file_id),
                None => (chunk_group, uuid::Uuid::default(), file_id),
            })
            .collect();

    let bookmark_groups: Vec<GroupsForChunk> =
        groups.into_iter().fold(Vec::new(), |mut acc, item| {
            if item.1 == uuid::Uuid::default() {
                return acc;
            }

            //check if chunk in output already
            if let Some(output_item) = acc.iter_mut().find(|x| x.chunk_uuid == item.1) {
                //if it is, add group to it
                output_item
                    .slim_groups
                    .push(ChunkGroupAndFileId::from_group(item.0, item.2));
            } else {
                //if not make new output item
                acc.push(GroupsForChunk {
                    chunk_uuid: item.1,
                    slim_groups: vec![ChunkGroupAndFileId::from_group(item.0, item.2)],
                });
            }
            acc
        });

    Ok(bookmark_groups)
}

#[tracing::instrument(skip(pool))]
pub async fn delete_chunk_from_group_query(
    chunk_id: uuid::Uuid,
    group_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<uuid::Uuid, ServiceError> {
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    diesel::delete(
        chunk_group_bookmarks_columns::chunk_group_bookmarks
            .filter(chunk_group_bookmarks_columns::chunk_metadata_id.eq(chunk_id))
            .filter(chunk_group_bookmarks_columns::group_id.eq(group_id)),
    )
    .execute(&mut conn)
    .await
    .map_err(|_err| {
        log::error!("Error deleting bookmark {:}", _err);
        ServiceError::BadRequest("Error deleting bookmark".to_string())
    })?;

    let qdrant_point_id = chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::id.eq(chunk_id))
        .select(chunk_metadata_columns::qdrant_point_id)
        .first::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|_err| {
            log::error!("Error getting qdrant_point_id {:}", _err);
            ServiceError::BadRequest("Error getting qdrant_point_id".to_string())
        })?;

    Ok(qdrant_point_id)
}

#[tracing::instrument(skip(pool))]
pub async fn create_group_from_file_query(
    group_id: uuid::Uuid,
    file_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::groups_from_files::dsl as groups_from_files_columns;

    let file_group = FileGroup::from_details(file_id, group_id);

    let mut conn = pool.get().await.unwrap();

    diesel::insert_into(groups_from_files_columns::groups_from_files)
        .values(&file_group)
        .execute(&mut conn)
        .await
        .map_err(|_err| {
            log::error!("Error creating group from file {:}", _err);
            ServiceError::BadRequest("Error creating group from file".to_string())
        })?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn get_point_ids_from_unified_group_ids(
    group_ids: Vec<UnifiedId>,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<uuid::Uuid>, ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    if group_ids.is_empty() {
        return Ok(vec![]);
    }

    let qdrant_point_ids: Vec<uuid::Uuid> = match group_ids.get(0) {
        Some(UnifiedId::TrieveUuid(_)) => chunk_group_columns::chunk_group
            .inner_join(chunk_group_bookmarks_columns::chunk_group_bookmarks)
            .inner_join(chunk_metadata_columns::chunk_metadata.on(
                chunk_group_bookmarks_columns::chunk_metadata_id.eq(chunk_metadata_columns::id),
            ))
            .filter(
                chunk_group_columns::id.eq_any(
                    &group_ids
                        .iter()
                        .map(|x| x.as_uuid().expect("Failed to convert to Uuid"))
                        .collect::<Vec<uuid::Uuid>>(),
                ),
            )
            .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
            .select(chunk_metadata_columns::qdrant_point_id)
            .load::<uuid::Uuid>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?,
        Some(UnifiedId::TrackingId(_)) => chunk_group_columns::chunk_group
            .inner_join(chunk_group_bookmarks_columns::chunk_group_bookmarks)
            .inner_join(chunk_metadata_columns::chunk_metadata.on(
                chunk_group_bookmarks_columns::chunk_metadata_id.eq(chunk_metadata_columns::id),
            ))
            .filter(
                chunk_group_columns::tracking_id.eq_any(
                    &group_ids
                        .iter()
                        .map(|x| x.as_tracking_id().expect("Failed to convert to String"))
                        .collect::<Vec<String>>(),
                ),
            )
            .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
            .select(chunk_metadata_columns::qdrant_point_id)
            .load::<uuid::Uuid>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?,
        _ => vec![],
    };

    Ok(qdrant_point_ids)
}

#[tracing::instrument(skip(pool))]
pub async fn get_groups_from_group_ids_query(
    group_ids: Vec<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkGroupAndFileId>, ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group_columns;
    use crate::data::schema::groups_from_files::dsl as groups_from_files_columns;

    let mut conn = pool.get().await.unwrap();

    let chunk_groups_and_files: Vec<(ChunkGroup, Option<uuid::Uuid>)> =
        chunk_group_columns::chunk_group
            .left_join(
                groups_from_files_columns::groups_from_files
                    .on(chunk_group_columns::id.eq(groups_from_files_columns::group_id)),
            )
            .filter(chunk_group_columns::id.eq_any(&group_ids))
            .select((
                ChunkGroup::as_select(),
                groups_from_files_columns::file_id.nullable(),
            ))
            .load::<(ChunkGroup, Option<uuid::Uuid>)>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to fetch group".to_string()))?;

    Ok(chunk_groups_and_files
        .iter()
        .map(|(group, file_id)| ChunkGroupAndFileId::from_group(group.clone(), *file_id))
        .collect())
}

#[tracing::instrument(skip(pool))]
pub async fn check_group_ids_exist_query(
    group_ids: Vec<uuid::Uuid>,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<uuid::Uuid>, ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group_columns;

    let mut conn = pool.get().await.unwrap();

    let existing_group_ids: Vec<uuid::Uuid> = chunk_group_columns::chunk_group
        .filter(chunk_group_columns::dataset_id.eq(dataset_id))
        .filter(chunk_group_columns::id.eq_any(&group_ids))
        .select(chunk_group_columns::id)
        .load::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|e| {
            log::error!("Error getting group ids for exist check {:?}", e);
            sentry::capture_message(
                &format!("Error getting group ids for exist check {:?}", e),
                sentry::Level::Error,
            );

            ServiceError::BadRequest("Failed to load group ids for exist check".to_string())
        })?;

    Ok(existing_group_ids)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GroupUpdateMessage {
    pub dataset_id: uuid::Uuid,
    pub group: ChunkGroup,
    pub prev_group: ChunkGroup,
    pub attempt_number: usize,
}

pub async fn soft_update_grouped_chunks_query(
    new_group: ChunkGroup,
    prev_group: ChunkGroup,
    redis_pool: web::Data<RedisPool>,
    dataset_id: uuid::Uuid,
) -> Result<(), ServiceError> {
    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let message = GroupUpdateMessage {
        dataset_id,
        group: new_group,
        prev_group,
        attempt_number: 0,
    };

    let serialized_message =
        serde_json::to_string(&message).map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    redis::cmd("lpush")
        .arg("group_update_queue")
        .arg(&serialized_message)
        .query_async(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}

pub async fn update_grouped_chunks_query(
    prev_group: ChunkGroup,
    new_group: ChunkGroup,
    pool: web::Data<Pool>,
    dataset_config: DatasetConfiguration,
) -> Result<(), ServiceError> {
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut offset = uuid::Uuid::nil();

    let mut conn = pool.get().await.unwrap();

    let qdrant_collection = format!("{}_vectors", dataset_config.EMBEDDING_SIZE);

    let group_id = new_group.id;
    let prev_group_tag_set = prev_group
        .tag_set
        .unwrap_or_default()
        .iter()
        .filter_map(|x| x.clone())
        .collect::<Vec<String>>();
    let new_group_tag_set = new_group
        .tag_set
        .unwrap_or_default()
        .iter()
        .filter_map(|x| x.clone())
        .collect::<Vec<String>>();

    loop {
        let qdrant_ids: Vec<(uuid::Uuid, uuid::Uuid)> =
            chunk_group_bookmarks_columns::chunk_group_bookmarks
                .inner_join(chunk_metadata_columns::chunk_metadata.on(
                    chunk_metadata_columns::id.eq(chunk_group_bookmarks_columns::chunk_metadata_id),
                ))
                .filter(chunk_group_bookmarks_columns::group_id.eq(group_id))
                .filter(chunk_metadata_columns::id.gt(offset))
                .limit(1000)
                .order_by(chunk_metadata_columns::id)
                .select((
                    chunk_metadata_columns::qdrant_point_id,
                    chunk_metadata_columns::id,
                ))
                .load::<(uuid::Uuid, uuid::Uuid)>(&mut conn)
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to load chunks".to_string()))?;
        if qdrant_ids.is_empty() {
            break;
        }

        offset = qdrant_ids.last().unwrap().1;

        let points: Vec<uuid::Uuid> = qdrant_ids.iter().map(|(point_id, _)| *point_id).collect();

        update_group_tag_sets_in_qdrant_query(
            qdrant_collection.clone(),
            prev_group_tag_set.clone(),
            new_group_tag_set.clone(),
            points,
        )
        .await?;
    }

    Ok(())
}

pub async fn get_chunk_point_ids_in_chunk_group_query(
    group_id: uuid::Uuid,
    page: u64,
    limit: u64,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<uuid::Uuid>, ServiceError> {
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    let chunk_ids = chunk_group_bookmarks_columns::chunk_group_bookmarks
        .inner_join(chunk_metadata_columns::chunk_metadata)
        .filter(chunk_group_bookmarks_columns::group_id.eq(group_id))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
        .select(chunk_metadata_columns::qdrant_point_id)
        .offset(((page - 1) * limit).try_into().unwrap_or(0))
        .limit(limit.try_into().unwrap_or(10))
        .load::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to load chunks for group".to_string()))?;

    Ok(chunk_ids)
}

pub async fn get_chunk_metadata_count_in_chunk_group_query(
    group_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<i64, ServiceError> {
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;

    let mut conn = pool.get().await.unwrap();

    let count = chunk_group_bookmarks_columns::chunk_group_bookmarks
        .filter(chunk_group_bookmarks_columns::group_id.eq(group_id))
        .count()
        .get_result::<i64>(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::BadRequest("Failed to get count of chunks for group".to_string())
        })?;

    Ok(count)
}
