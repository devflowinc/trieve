use crate::data::models::{
    ChunkCollision, ChunkGroupBookmark, ChunkMetadataTable, ChunkMetadataTags, ChunkMetadataTypes,
    ContentChunkMetadata, Dataset, DatasetTags, IngestSpecificChunkMetadata,
    ServerDatasetConfiguration, SlimChunkMetadata, SlimChunkMetadataTable, UnifiedId,
};
use crate::get_env;
use crate::handlers::chunk_handler::UploadIngestionMessage;
use crate::handlers::chunk_handler::{BulkUploadIngestionMessage, ChunkReqPayload};
use crate::operators::group_operator::{
    check_group_ids_exist_query, get_group_ids_from_tracking_ids_query,
};
use crate::operators::model_operator::create_embedding;
use crate::operators::parse_operator::convert_html_to_text;
use crate::operators::qdrant_operator::get_qdrant_connection;
use crate::{
    data::models::{ChunkMetadata, Pool},
    errors::ServiceError,
};
use actix_web::web;
use chrono::NaiveDateTime;
use dateparser::DateTimeUtc;
use diesel::dsl::{not, sql};
use diesel::prelude::*;
use diesel::sql_types;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use itertools::Itertools;
use qdrant_client::qdrant::{PointId, PointVectors};
use simsearch::{SearchOptions, SimSearch};

#[tracing::instrument(skip(pool))]
pub async fn get_chunk_metadatas_from_point_ids(
    point_ids: Vec<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkMetadata>, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    // Get tag_set
    let chunk_metadata_pairs: Vec<(ChunkMetadataTable, Vec<String>)> =
        chunk_metadata_columns::chunk_metadata
            .left_join(
                chunk_metadata_tags_columns::chunk_metadata_tags
                    .on(chunk_metadata_tags_columns::chunk_metadata_id
                        .eq(chunk_metadata_columns::id)),
            )
            .left_join(
                dataset_tags_columns::dataset_tags
                    .on(dataset_tags_columns::id.eq(chunk_metadata_tags_columns::tag_id)),
            )
            .filter(chunk_metadata_columns::qdrant_point_id.eq_any(&point_ids))
            .select((
                ChunkMetadataTable::as_select(),
                sql::<sql_types::Array<sql_types::Text>>("array_agg(dataset_tags.tag)"),
            ))
            .group_by(chunk_metadata_columns::id)
            .load::<(ChunkMetadataTable, Vec<String>)>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

    let chunk_metadatas = chunk_metadata_pairs
        .into_iter()
        .map(|(table, tag_set)| ChunkMetadata::from_table_and_tag_set(table, tag_set))
        .collect();

    Ok(chunk_metadatas)
}

#[tracing::instrument(skip(pool))]
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
pub async fn get_chunk_metadatas_and_collided_chunks_from_point_ids_query(
    point_ids: Vec<uuid::Uuid>,
    get_collisions: bool,
    pool: web::Data<Pool>,
) -> Result<(Vec<ChunkMetadataTypes>, Vec<ChunkMetadataTypes>), ServiceError> {
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;

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

    // Fetch the chunk metadatas for root chunks
    let chunk_metadatas = {
        let mut conn = pool.get().await.unwrap();
        // If collisions get removed this can be one query
        let chunk_qdrant_pointid_pair: Vec<(uuid::Uuid, Option<uuid::Uuid>)> =
            chunk_metadata_columns::chunk_metadata
                .left_outer_join(
                    chunk_collisions_columns::chunk_collisions
                        .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
                )
                .select((
                    (chunk_metadata_columns::id),
                    (chunk_collisions_columns::collision_qdrant_id).nullable(),
                ))
                .filter(chunk_metadata_columns::qdrant_point_id.eq_any(&point_ids))
                .load::<(uuid::Uuid, Option<uuid::Uuid>)>(&mut conn)
                .await
                .map_err(|_| {
                    ServiceError::BadRequest("Failed to load collision metadata".to_string())
                })?;

        // Get tagset and chunk metadatatable

        let chunk_metadata_pair: Vec<(ChunkMetadataTable, Vec<String>)> =
            chunk_metadata_columns::chunk_metadata
                .left_join(chunk_metadata_tags_columns::chunk_metadata_tags.on(
                    chunk_metadata_tags_columns::chunk_metadata_id.eq(chunk_metadata_columns::id),
                ))
                .left_join(
                    dataset_tags_columns::dataset_tags
                        .on(dataset_tags_columns::id.eq(chunk_metadata_tags_columns::tag_id)),
                )
                .select((
                    ChunkMetadataTable::as_select(),
                    sql::<sql_types::Array<sql_types::Text>>("array_agg(dataset_tags.tag)"),
                ))
                .filter(chunk_metadata_columns::qdrant_point_id.eq_any(&point_ids))
                .group_by(chunk_metadata_columns::id)
                .load::<(ChunkMetadataTable, Vec<String>)>(&mut conn)
                .await
                .map_err(|err| {
                    ServiceError::BadRequest(format!(
                        "Failed to load tagset and metadata: {:?}",
                        err
                    ))
                })?;

        chunk_metadata_pair
            .into_iter()
            .map(|(chunk_table, tag_set)| {
                let qdrant_point_id = chunk_qdrant_pointid_pair
                    .iter()
                    .find(|(chunk_id, _)| *chunk_id == chunk_table.id)
                    .map(|(_, qdrant_point_id)| *qdrant_point_id)
                    .flatten();

                ChunkMetadata::from_table_and_tag_set(
                    ChunkMetadataTable {
                        qdrant_point_id: chunk_table.qdrant_point_id.or(qdrant_point_id),
                        ..chunk_table
                    },
                    tag_set,
                )
                .into()
            })
            .collect::<Vec<ChunkMetadataTypes>>()
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

            let chunk_qdrant_point_id_pair: Vec<(uuid::Uuid, uuid::Uuid)> =
                chunk_collisions_columns::chunk_collisions
                    .inner_join(
                        chunk_metadata_columns::chunk_metadata
                            .on(chunk_metadata_columns::id.eq(chunk_collisions_columns::chunk_id)),
                    )
                    .filter(chunk_collisions_columns::collision_qdrant_id.eq_any(&point_ids))
                    .select((
                        chunk_metadata_columns::id,
                        chunk_collisions_columns::collision_qdrant_id.assume_not_null(),
                    ))
                    .load::<(uuid::Uuid, uuid::Uuid)>(&mut conn)
                    .await
                    .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

            let chunk_metadata_pair: Vec<(ChunkMetadataTable, Vec<String>)> =
                chunk_metadata_columns::chunk_metadata
                    .left_join(
                        chunk_metadata_tags_columns::chunk_metadata_tags
                            .on(chunk_metadata_tags_columns::chunk_metadata_id
                                .eq(chunk_metadata_columns::id)),
                    )
                    .left_join(
                        dataset_tags_columns::dataset_tags
                            .on(dataset_tags_columns::id.eq(chunk_metadata_tags_columns::tag_id)),
                    )
                    .select((
                        ChunkMetadataTable::as_select(),
                        sql::<sql_types::Array<sql_types::Text>>("array_agg(dataset_tags.tag)"),
                    ))
                    .filter(chunk_metadata_columns::qdrant_point_id.eq_any(&point_ids))
                    .group_by(chunk_metadata_columns::id)
                    .load::<(ChunkMetadataTable, Vec<String>)>(&mut conn)
                    .await
                    .map_err(|_| {
                        ServiceError::BadRequest(
                            "Failed to load tagset and metadata with collisions".to_string(),
                        )
                    })?;

            chunk_metadata_pair
                .into_iter()
                .filter_map(|(chunk_table, tag_set)| {
                    let qdrant_point_id = chunk_qdrant_point_id_pair
                        .iter()
                        .find(|(chunk_id, _)| *chunk_id == chunk_table.id)?
                        .1;

                    Some(
                        ChunkMetadata::from_table_and_tag_set(
                            ChunkMetadataTable {
                                qdrant_point_id: chunk_table.qdrant_point_id.or(Some(qdrant_point_id)),
                                ..chunk_table
                            },
                            tag_set,
                        )
                        .into(),
                    )
                })
                .collect::<Vec<ChunkMetadataTypes>>()
        };

        collision_search_span.finish();
        transaction.finish();
        // Return the chunk metadata and the collided chunks
        Ok((chunk_metadatas, collided_chunks))
    } else {
        collision_search_span.finish();
        transaction.finish();
        // Return only the chunk metadata
        Ok((chunk_metadatas, vec![]))
    }
}

#[tracing::instrument(skip(pool))]
pub async fn get_slim_chunks_from_point_ids_query(
    point_ids: Vec<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkMetadataTypes>, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    let slim_chunk_pairs: Vec<(SlimChunkMetadataTable, Vec<String>)> =
        chunk_metadata_columns::chunk_metadata
            .left_join(
                chunk_metadata_tags_columns::chunk_metadata_tags
                    .on(chunk_metadata_tags_columns::chunk_metadata_id
                        .eq(chunk_metadata_columns::id)),
            )
            .left_join(
                dataset_tags_columns::dataset_tags
                    .on(dataset_tags_columns::id.eq(chunk_metadata_tags_columns::tag_id)),
            )
            .filter(chunk_metadata_columns::qdrant_point_id.eq_any(&point_ids))
            .select((
                (
                    chunk_metadata_columns::id,
                    chunk_metadata_columns::link,
                    chunk_metadata_columns::qdrant_point_id,
                    chunk_metadata_columns::created_at,
                    chunk_metadata_columns::updated_at,
                    chunk_metadata_columns::metadata,
                    chunk_metadata_columns::tracking_id,
                    chunk_metadata_columns::time_stamp,
                    chunk_metadata_columns::location,
                    chunk_metadata_columns::dataset_id,
                    chunk_metadata_columns::weight,
                    chunk_metadata_columns::image_urls,
                    chunk_metadata_columns::num_value,
                ),
                sql::<sql_types::Array<sql_types::Text>>("array_agg(dataset_tags.tag)"),
            ))
            .group_by(chunk_metadata_columns::id)
            .load::<(SlimChunkMetadataTable, Vec<String>)>(&mut conn)
            .await
            .map_err(|_| {
                ServiceError::BadRequest("Failed to load slim_chunk_metadatas".to_string())
            })?;

    let slim_chunks = slim_chunk_pairs
        .into_iter()
        .map(|(table, tag_set)| SlimChunkMetadata::from_table_and_tag_set(table, tag_set).into())
        .collect();

    Ok(slim_chunks)
}

#[tracing::instrument(skip(pool))]
pub async fn get_content_chunk_from_point_ids_query(
    point_ids: Vec<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkMetadataTypes>, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let parent_span = sentry::configure_scope(|scope| scope.get_span());
    let transaction: sentry::TransactionOrSpan = match &parent_span {
        Some(parent) => parent
            .start_child(
                "Get content chunk metadata of points",
                "Hitting Postgres to fetch content chunk metadata",
            )
            .into(),
        None => {
            let ctx = sentry::TransactionContext::new(
                "Get content chunk metadata of points",
                "Hitting Postgres to fetch content chunk metadata",
            );
            sentry::start_transaction(ctx).into()
        }
    };
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone())));

    let get_content_chunks_span = transaction.start_child(
        "Fetching matching points to content chunks from qdrant",
        "Fetching matching points to content chunks from qdrant",
    );

    let content_chunks = {
        let mut conn = pool.get().await.unwrap();
        let content_chunk_metadatas: Vec<ContentChunkMetadata> =
            chunk_metadata_columns::chunk_metadata
                .select((
                    chunk_metadata_columns::id,
                    chunk_metadata_columns::qdrant_point_id,
                    chunk_metadata_columns::chunk_html,
                    chunk_metadata_columns::tracking_id,
                    chunk_metadata_columns::time_stamp,
                    chunk_metadata_columns::weight,
                    chunk_metadata_columns::image_urls,
                    chunk_metadata_columns::num_value,
                ))
                .filter(chunk_metadata_columns::qdrant_point_id.eq_any(&point_ids))
                .load(&mut conn)
                .await
                .map_err(|_| {
                    ServiceError::BadRequest("Failed to load content chunk metadatas".to_string())
                })?;

        content_chunk_metadatas
            .iter()
            .map(|content_chunk| content_chunk.clone().into())
            .collect::<Vec<ChunkMetadataTypes>>()
    };

    get_content_chunks_span.finish();

    Ok(content_chunks)
}

#[tracing::instrument(skip(pool))]
pub async fn get_metadata_from_id_query(
    chunk_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;
    let mut conn = pool.get().await.unwrap();

    let (chunk_table, tag_set) = chunk_metadata_columns::chunk_metadata
        .left_join(
            chunk_metadata_tags_columns::chunk_metadata_tags
                .on(chunk_metadata_tags_columns::chunk_metadata_id.eq(chunk_metadata_columns::id)),
        )
        .left_join(
            dataset_tags_columns::dataset_tags
                .on(dataset_tags_columns::id.eq(chunk_metadata_tags_columns::tag_id)),
        )
        .filter(chunk_metadata_columns::id.eq(chunk_id))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
        .select((
            ChunkMetadataTable::as_select(),
            sql::<sql_types::Array<sql_types::Text>>("array_agg(dataset_tags.tag)"),
        ))
        .group_by(chunk_metadata_columns::id)
        .first::<(ChunkMetadataTable, Vec<String>)>(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::NotFound("Chunk with id not found in the specified dataset".to_string())
        })?;

    Ok(ChunkMetadata::from_table_and_tag_set(chunk_table, tag_set))
}

#[tracing::instrument(skip(pool))]
pub async fn get_metadata_from_tracking_id_query(
    tracking_id: String,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;

    let mut conn = pool.get().await.unwrap();

    let (chunk_table, tag_set) = chunk_metadata_columns::chunk_metadata
        .left_join(
            chunk_metadata_tags_columns::chunk_metadata_tags
                .on(chunk_metadata_tags_columns::chunk_metadata_id.eq(chunk_metadata_columns::id)),
        )
        .left_join(
            dataset_tags_columns::dataset_tags
                .on(dataset_tags_columns::id.eq(chunk_metadata_tags_columns::tag_id)),
        )
        .filter(chunk_metadata_columns::tracking_id.eq(tracking_id))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_uuid))
        .select((
            ChunkMetadataTable::as_select(),
            sql::<sql_types::Array<sql_types::Text>>("array_agg(dataset_tags.tag)"),
        ))
        .group_by(chunk_metadata_columns::id)
        .first::<(ChunkMetadataTable, Vec<String>)>(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::NotFound(
                "Chunk with tracking_id not found in the specified dataset".to_string(),
            )
        })?;

    Ok(ChunkMetadata::from_table_and_tag_set(chunk_table, tag_set))
}

#[tracing::instrument(skip(pool))]
pub async fn get_metadata_from_ids_query(
    chunk_ids: Vec<uuid::Uuid>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkMetadata>, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;

    let mut conn = pool.get().await.unwrap();

    let chunk_metadata_pairs: Vec<(ChunkMetadataTable, Vec<String>)> =
        chunk_metadata_columns::chunk_metadata
            .left_join(
                chunk_metadata_tags_columns::chunk_metadata_tags
                    .on(chunk_metadata_tags_columns::chunk_metadata_id
                        .eq(chunk_metadata_columns::id)),
            )
            .left_join(
                dataset_tags_columns::dataset_tags
                    .on(dataset_tags_columns::id.eq(chunk_metadata_tags_columns::tag_id)),
            )
            .filter(chunk_metadata_columns::id.eq_any(chunk_ids))
            .filter(chunk_metadata_columns::dataset_id.eq(dataset_uuid))
            .select((
                ChunkMetadataTable::as_select(),
                sql::<sql_types::Array<sql_types::Text>>("array_agg(dataset_tags.tag)"),
            ))
            .group_by(chunk_metadata_columns::id)
            .load::<(ChunkMetadataTable, Vec<String>)>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

    let chunk_metadatas = chunk_metadata_pairs
        .into_iter()
        .map(|(table, tag_set)| ChunkMetadata::from_table_and_tag_set(table, tag_set).into())
        .collect();

    Ok(chunk_metadatas)
}

#[tracing::instrument(skip(pool))]
pub async fn get_metadata_from_tracking_ids_query(
    tracking_ids: Vec<String>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkMetadata>, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;

    let mut conn = pool.get().await.unwrap();

    let chunk_metadata_pairs: Vec<(ChunkMetadataTable, Vec<String>)> =
        chunk_metadata_columns::chunk_metadata
            .left_join(
                chunk_metadata_tags_columns::chunk_metadata_tags
                    .on(chunk_metadata_tags_columns::chunk_metadata_id
                        .eq(chunk_metadata_columns::id)),
            )
            .left_join(
                dataset_tags_columns::dataset_tags
                    .on(dataset_tags_columns::id.eq(chunk_metadata_tags_columns::tag_id)),
            )
            .filter(chunk_metadata_columns::tracking_id.eq_any(tracking_ids))
            .filter(chunk_metadata_columns::dataset_id.eq(dataset_uuid))
            .select((
                ChunkMetadataTable::as_select(),
                sql::<sql_types::Array<sql_types::Text>>("array_agg(dataset_tags.tag)"),
            ))
            .group_by(chunk_metadata_columns::id)
            .load::<(ChunkMetadataTable, Vec<String>)>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

    let chunk_metadatas = chunk_metadata_pairs
        .into_iter()
        .map(|(table, tag_set)| ChunkMetadata::from_table_and_tag_set(table, tag_set).into())
        .collect();

    Ok(chunk_metadatas)
}

/// Only inserts, does not try to upsert data
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip(pool))]
pub async fn bulk_insert_chunk_metadata_query(
    // ChunkMetadata, group_ids, upsert_by_tracking_id
    mut insertion_data: Vec<(ChunkMetadata, String, Option<Vec<uuid::Uuid>>, bool)>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<(ChunkMetadata, String, Option<Vec<uuid::Uuid>>, bool)>, ServiceError> {
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool
        .clone()
        .get()
        .await
        .expect("Failed to get connection to db");

    let chunkmetadata_to_insert: Vec<ChunkMetadataTable> = insertion_data
        .clone()
        .iter()
        .map(|(chunk_metadata, _, _, _)| chunk_metadata.clone().into())
        .collect();

    let inserted_chunks = diesel::insert_into(chunk_metadata_columns::chunk_metadata)
        .values(&chunkmetadata_to_insert)
        .on_conflict_do_nothing()
        .get_results::<ChunkMetadataTable>(&mut conn)
        .await
        .map_err(|e| {
            sentry::capture_message(
                &format!("Failed to insert chunk_metadata: {:?}", e),
                sentry::Level::Error,
            );
            log::error!("Failed to insert chunk_metadata: {:?}", e);

            ServiceError::BadRequest("Failed to insert chunk_metadata".to_string())
        })?;

    // mutates in place
    insertion_data.retain(|(chunk_metadata, _, _, _)| {
        inserted_chunks
            .iter()
            .any(|inserted_chunk| inserted_chunk.id == chunk_metadata.id)
    });

    let chunk_group_bookmarks_to_insert: Vec<ChunkGroupBookmark> = insertion_data
        .clone()
        .iter()
        .filter_map(|(chunk_metadata, _, group_ids, _)| {
            group_ids.as_ref().map(|group_ids| {
                group_ids
                    .clone()
                    .iter()
                    .map(|group_id| ChunkGroupBookmark::from_details(*group_id, chunk_metadata.id))
                    .collect::<Vec<ChunkGroupBookmark>>()
            })
        })
        .flatten()
        .collect();

    diesel::insert_into(chunk_group_bookmarks_columns::chunk_group_bookmarks)
        .values(chunk_group_bookmarks_to_insert)
        .execute(&mut conn)
        .await
        .map_err(|e| {
            sentry::capture_message(
                &format!("Failed to insert chunk into gropus: {:?}", e),
                sentry::Level::Error,
            );
            ServiceError::BadRequest("Failed to insert chunk into groups".to_string())
        })?;

    let chunk_tags_to_chunk_id: Vec<(Vec<DatasetTags>, uuid::Uuid)> = insertion_data
        .clone()
        .iter()
        .filter_map(|(chunk_metadata, _, _, _)| {
            chunk_metadata.clone().tag_set.map(|tags| {
                let tags = tags
                    .into_iter()
                    .filter_map(|maybe_tag| {
                        maybe_tag.map(|tag| DatasetTags {
                            id: uuid::Uuid::new_v4(),
                            dataset_id: dataset_uuid,
                            tag,
                        })
                    })
                    .collect_vec();

                (tags, chunk_metadata.id)
            })
        })
        .collect_vec();

    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;
    // TODO, dedupe and bulk insert this.
    for (dataset_tags, chunk_uuid) in chunk_tags_to_chunk_id {
        log::info!("Inserting dataset tags {:?}", dataset_tags.clone());

        diesel::insert_into(dataset_tags_columns::dataset_tags)
            .values(&dataset_tags)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to create dataset tag {:}", e);
                ServiceError::BadRequest("Failed to create dataset tag".to_string())
            })?;

        let tag_names = dataset_tags
            .clone()
            .into_iter()
            .map(|tag| tag.tag)
            .collect_vec();

        let dataset_tags_existing =
            get_dataset_tags_id_from_names(pool.clone(), dataset_uuid, tag_names).await?;

        let mut needed_dataset_tags = dataset_tags.clone();
        // Remove all conflicts
        needed_dataset_tags.retain(|dataset_tag| {
            // If it isn't found aka None, then it is not a conflicting one
            dataset_tags_existing
                .iter()
                .find(|predicate_tag| predicate_tag.tag == dataset_tag.tag)
                .is_none()
        });
        // Add Back preexisting ones
        needed_dataset_tags.extend(dataset_tags_existing);

        let mut chunk_metadata_tags: Vec<ChunkMetadataTags> = vec![];
        for dataset_tag in needed_dataset_tags {
            chunk_metadata_tags.push(ChunkMetadataTags {
                id: uuid::Uuid::new_v4(),
                chunk_metadata_id: chunk_uuid,
                tag_id: dataset_tag.id,
            });
        }

        diesel::insert_into(chunk_metadata_tags_columns::chunk_metadata_tags)
            .values(&chunk_metadata_tags)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to create chunk metadata tags {:}", e);
                ServiceError::BadRequest("Failed to create chunk metadata tags".to_string())
            })?;
    }

    Ok(insertion_data)
}

#[tracing::instrument(skip(pool))]
pub async fn get_optional_metadata_from_tracking_id_query(
    tracking_id: String,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Option<ChunkMetadata>, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;

    let mut conn = pool.get().await.unwrap();

    let optional_chunk: Option<(ChunkMetadataTable, Vec<String>)> =
        chunk_metadata_columns::chunk_metadata
            .left_join(
                chunk_metadata_tags_columns::chunk_metadata_tags
                    .on(chunk_metadata_tags_columns::chunk_metadata_id
                        .eq(chunk_metadata_columns::id)),
            )
            .left_join(
                dataset_tags_columns::dataset_tags
                    .on(dataset_tags_columns::id.eq(chunk_metadata_tags_columns::tag_id)),
            )
            .filter(chunk_metadata_columns::tracking_id.eq(tracking_id))
            .filter(chunk_metadata_columns::dataset_id.eq(dataset_uuid))
            .select((
                ChunkMetadataTable::as_select(),
                sql::<sql_types::Array<sql_types::Text>>("array_agg(dataset_tags.tag)"),
            ))
            .group_by(chunk_metadata_columns::id)
            .load::<(ChunkMetadataTable, Vec<String>)>(&mut conn)
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

    Ok(optional_chunk
        .map(|(chunk_table, tag_set)| ChunkMetadata::from_table_and_tag_set(chunk_table, tag_set)))
}

#[tracing::instrument(skip(pool))]
pub async fn insert_chunk_metadata_query(
    chunk_data: ChunkMetadata,
    group_ids: Option<Vec<uuid::Uuid>>,
    dataset_uuid: uuid::Uuid,
    upsert_by_tracking_id: bool,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, ServiceError> {
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;

    if let Some(other_tracking_id) = chunk_data.tracking_id.clone() {
        if upsert_by_tracking_id {
            if let Some(existing_chunk) = get_optional_metadata_from_tracking_id_query(
                other_tracking_id.clone(),
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
                    group_ids,
                    dataset_uuid,
                    pool.clone(),
                )
                .await?;

                return Ok(updated_chunk);
            }
        }
    }

    let chunk_table: ChunkMetadataTable = chunk_data.clone().into();

    let mut conn = pool.get().await.expect("Failed to get connection to db");

    let data_updated = diesel::insert_into(chunk_metadata_columns::chunk_metadata)
        .values(&chunk_table)
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await
        .map_err(|e| {
            sentry::capture_message(
                &format!("Failed to insert chunk_metadata: {:?}", e),
                sentry::Level::Error,
            );
            match e {
                diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                ) => ServiceError::DuplicateTrackingId(
                    chunk_data.tracking_id.clone().unwrap_or("".to_string()),
                ),
                diesel::result::Error::NotFound => ServiceError::DuplicateTrackingId(
                    chunk_data.tracking_id.clone().unwrap_or("".to_string()),
                ),
                _ => {
                    log::error!("Failed to insert chunk_metadata: {:?}", e);
                    ServiceError::BadRequest(format!("Failed to insert chunk_metadata {:}", e))
                }
            }
        })?;

    if data_updated == 0 {
        return Err(ServiceError::DuplicateTrackingId(
            chunk_data.tracking_id.clone().unwrap_or("".to_string()),
        ));
    }

    if let Some(group_ids) = group_ids {
        diesel::insert_into(chunk_group_bookmarks_columns::chunk_group_bookmarks)
            .values(
                &group_ids
                    .into_iter()
                    .map(|group_id| ChunkGroupBookmark::from_details(group_id, chunk_data.id))
                    .collect::<Vec<ChunkGroupBookmark>>(),
            )
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to insert chunk into groups {:?}", e);
                ServiceError::BadRequest("Failed to insert chunk into groups".to_string())
            })?;
    }

    if let Some(tag_set) = chunk_data.tag_set.clone() {
        let dataset_tags_to_add = tag_set
            .into_iter()
            .filter_map(|maybe_tag| {
                maybe_tag.map(|tag| DatasetTags {
                    id: uuid::Uuid::new_v4(),
                    dataset_id: dataset_uuid,
                    tag,
                })
            })
            .collect_vec();

        diesel::insert_into(dataset_tags_columns::dataset_tags)
            .values(&dataset_tags_to_add)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to create dataset tag {:}", e);
                ServiceError::BadRequest("Failed to create dataset tag".to_string())
            })?;

        // Get the proper dataset_tags to add chunk_metadata_tags

        let tag_names = dataset_tags_to_add
            .clone()
            .into_iter()
            .map(|tag| tag.tag)
            .collect_vec();

        let dataset_tags_existing =
            get_dataset_tags_id_from_names(pool, dataset_uuid, tag_names).await?;

        let mut needed_dataset_tags = dataset_tags_to_add.clone();
        // Remove all conflicts
        needed_dataset_tags.retain(|dataset_tag| {
            // If it isn't found aka None, then it is not a conflicting one
            dataset_tags_existing
                .iter()
                .find(|predicate_tag| predicate_tag.tag == dataset_tag.tag)
                .is_none()
        });
        // Add Back preexisting ones
        needed_dataset_tags.extend(dataset_tags_existing);

        let mut chunk_metadata_tags: Vec<ChunkMetadataTags> = vec![];
        for tag_dataset in needed_dataset_tags {
            chunk_metadata_tags.push(ChunkMetadataTags {
                id: uuid::Uuid::new_v4(),
                chunk_metadata_id: chunk_data.id,
                tag_id: tag_dataset.id,
            });
        }

        diesel::insert_into(chunk_metadata_tags_columns::chunk_metadata_tags)
            .values(&chunk_metadata_tags)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to create chunk metadata tags {:}", e);
                ServiceError::BadRequest("Failed to create chunk metadata tags".to_string())
            })?;
    }

    Ok(chunk_data)
}

#[tracing::instrument(skip(pool))]
pub async fn get_dataset_tags_id_from_names(
    pool: web::Data<Pool>,
    dataset_id: uuid::Uuid,
    tag_names: Vec<String>,
) -> Result<Vec<DatasetTags>, ServiceError> {
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;

    let mut conn = pool
        .get()
        .await
        .expect("Failed to get connection from pool");

    dataset_tags_columns::dataset_tags
        .filter(dataset_tags_columns::dataset_id.eq(dataset_id))
        .filter(dataset_tags_columns::tag.eq_any(&tag_names))
        .load::<DatasetTags>(&mut conn)
        .await
        .map_err(|_| ServiceError::NotFound("Dataset Tag Not found".to_string()))
}

/// Bulk revert, assumes upsert chunk_ids were not upserted, only enterted
#[tracing::instrument(skip(pool))]
pub async fn bulk_revert_insert_chunk_metadata_query(
    chunk_ids: Vec<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl::*;

    let mut conn = pool.get().await.expect("Failed to get connection to db");

    diesel::delete(chunk_metadata.filter(id.eq_any(&chunk_ids)))
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

    diesel::delete(
        chunk_group_bookmarks_columns::chunk_group_bookmarks
            .filter(chunk_group_bookmarks_columns::chunk_metadata_id.eq_any(&chunk_ids)),
    )
    .execute(&mut conn)
    .await
    .map_err(|e| {
        log::error!("Failed to revert chunk into groups action {:?}", e);
        ServiceError::BadRequest("Failed to revert chunk into groups action".to_string())
    })?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn insert_duplicate_chunk_metadata_query(
    chunk_data: ChunkMetadata,
    duplicate_chunk: uuid::Uuid,
    group_ids: Option<Vec<uuid::Uuid>>,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::chunk_collisions::dsl::*;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl::*;
    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;

    let mut conn = pool.get().await.unwrap();

    let inserted_chunk = conn
        .transaction::<_, diesel::result::Error, _>(|conn| {
            async {
                let chunk_table: ChunkMetadataTable = chunk_data.clone().into();

                let inserted_chunk = diesel::insert_into(chunk_metadata)
                    .values(&(chunk_table))
                    .get_result::<ChunkMetadataTable>(conn)
                    .await?;

                //insert duplicate into chunk_collisions
                diesel::insert_into(chunk_collisions)
                    .values(&ChunkCollision::from_details(
                        chunk_data.id,
                        duplicate_chunk,
                    ))
                    .execute(conn)
                    .await?;

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

    if let Some(tag_set) = chunk_data.tag_set.clone() {
        let dataset_uuid = chunk_data.dataset_id;
        let dataset_tags_to_add = tag_set
            .into_iter()
            .filter_map(|maybe_tag| {
                maybe_tag.map(|tag| DatasetTags {
                    id: uuid::Uuid::new_v4(),
                    dataset_id: dataset_uuid,
                    tag,
                })
            })
            .collect_vec();

        diesel::insert_into(dataset_tags_columns::dataset_tags)
            .values(&dataset_tags_to_add)
            .on_conflict_do_nothing()
            .get_results::<DatasetTags>(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to create dataset tag {:}", e);
                ServiceError::BadRequest("Failed to create dataset tag".to_string())
            })?;

        // Get the proper dataset_tags to add chunk_metadata_tags

        let tag_names = dataset_tags_to_add
            .clone()
            .into_iter()
            .map(|tag| tag.tag)
            .collect_vec();

        let dataset_tags_existing =
            get_dataset_tags_id_from_names(pool, dataset_uuid, tag_names).await?;

        let mut needed_dataset_tags = dataset_tags_to_add.clone();
        // Remove all conflicts
        needed_dataset_tags.retain(|dataset_tag| {
            // If it isn't found aka None, then it is not a conflicting one
            dataset_tags_existing
                .iter()
                .find(|predicate_tag| predicate_tag.tag == dataset_tag.tag)
                .is_none()
        });

        // Add Back preexisting ones
        needed_dataset_tags.extend(dataset_tags_existing);

        let mut chunk_metadata_tags: Vec<ChunkMetadataTags> = vec![];
        for tag_dataset in needed_dataset_tags {
            chunk_metadata_tags.push(ChunkMetadataTags {
                id: uuid::Uuid::new_v4(),
                chunk_metadata_id: chunk_data.id,
                tag_id: tag_dataset.id,
            });
        }

        diesel::insert_into(chunk_metadata_tags_columns::chunk_metadata_tags)
            .values(&chunk_metadata_tags)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to create chunk metadata tags {:}", e);
                ServiceError::BadRequest("Failed to create chunk metadata tags (duplicate chunk)".to_string())
            })?;
    }

    Ok(())
}

#[tracing::instrument(skip(pool))]
pub async fn update_chunk_metadata_query(
    chunk_data: ChunkMetadata,
    group_ids: Option<Vec<uuid::Uuid>>,
    dataset_uuid: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkMetadata, ServiceError> {
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;

    let mut conn = pool.get().await.unwrap();

    let updated_chunk: ChunkMetadataTable = diesel::update(
        chunk_metadata_columns::chunk_metadata
            .filter(chunk_metadata_columns::id.eq(chunk_data.id))
            .filter(chunk_metadata_columns::dataset_id.eq(dataset_uuid)),
    )
    .set((
        chunk_metadata_columns::link.eq(chunk_data.link),
        chunk_metadata_columns::chunk_html.eq(chunk_data.chunk_html),
        chunk_metadata_columns::metadata.eq(chunk_data.metadata),
        chunk_metadata_columns::tracking_id.eq(chunk_data.tracking_id),
        chunk_metadata_columns::time_stamp.eq(chunk_data.time_stamp),
        chunk_metadata_columns::location.eq(chunk_data.location),
        chunk_metadata_columns::weight.eq(chunk_data.weight),
        chunk_metadata_columns::image_urls.eq(chunk_data.image_urls),
        chunk_metadata_columns::num_value.eq(chunk_data.num_value),
    ))
    .get_result::<ChunkMetadataTable>(&mut conn)
    .await
    .map_err(|e| {
        log::error!("Failed to update chunk_metadata: {:?}", e);
        ServiceError::BadRequest("Failed to update chunk metadata".to_string())
    })?;

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
                .filter(chunk_group_bookmarks_columns::chunk_metadata_id.eq(updated_chunk.id))
                .filter(not(
                    chunk_group_bookmarks_columns::group_id.eq_any(group_id1)
                )),
        )
        .execute(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to delete chunk bookmarks".to_string()))?;
    }

    match chunk_data.tag_set {
        Some(tag_set) => {
            use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;

            let _ = diesel::delete(
                chunk_metadata_tags_columns::chunk_metadata_tags
                    .filter(chunk_metadata_tags_columns::chunk_metadata_id.eq(updated_chunk.id)),
            )
            .execute(&mut conn)
            .await;

            let dataset_tags_to_add = tag_set
                .clone()
                .into_iter()
                .filter_map(|maybe_tag| {
                    maybe_tag.map(|tag| DatasetTags {
                        id: uuid::Uuid::new_v4(),
                        dataset_id: dataset_uuid,
                        tag,
                    })
                })
                .collect_vec();

            diesel::insert_into(dataset_tags_columns::dataset_tags)
                .values(&dataset_tags_to_add)
                .on_conflict_do_nothing()
                .get_results::<DatasetTags>(&mut conn)
                .await
                .map_err(|e| {
                    log::error!("Failed to create dataset tag {:}", e);
                    ServiceError::BadRequest("Failed to create dataset tag".to_string())
                })?;

            // Get the proper dataset_tags to add chunk_metadata_tags
            let tag_names = dataset_tags_to_add
                .clone()
                .into_iter()
                .map(|tag| tag.tag)
                .collect_vec();

            let dataset_tags_existing =
                get_dataset_tags_id_from_names(pool, dataset_uuid, tag_names).await?;

            let mut needed_dataset_tags = dataset_tags_to_add.clone();
            // Remove all conflicts
            needed_dataset_tags.retain(|dataset_tag| {
                // If it isn't found aka None, then it is not a conflicting one
                dataset_tags_existing
                    .iter()
                    .find(|predicate_tag| predicate_tag.tag == dataset_tag.tag)
                    .is_none()
            });
            // Add Back preexisting ones
            needed_dataset_tags.extend(dataset_tags_existing);

            let mut chunk_metadata_tags: Vec<ChunkMetadataTags> = vec![];
            for tag_dataset in needed_dataset_tags {
                chunk_metadata_tags.push(ChunkMetadataTags {
                    id: uuid::Uuid::new_v4(),
                    chunk_metadata_id: chunk_data.id,
                    tag_id: tag_dataset.id,
                });
            }

            diesel::insert_into(chunk_metadata_tags_columns::chunk_metadata_tags)
                .values(&chunk_metadata_tags)
                .on_conflict_do_nothing()
                .execute(&mut conn)
                .await
                .map_err(|e| {
                    log::error!("Failed to update chunk metadata tags {:}", e);
                    ServiceError::BadRequest("Failed to update chunk metadata tags".to_string())
                })?;

            Ok(ChunkMetadata::from_table_and_tag_set_option_string(
                updated_chunk,
                Some(tag_set),
            ))
        }
        None => {
            // Fetch tagset
            let chunk_tagset: Vec<String> = chunk_metadata_columns::chunk_metadata
                .inner_join(chunk_metadata_tags_columns::chunk_metadata_tags.on(
                    chunk_metadata_tags_columns::chunk_metadata_id.eq(chunk_metadata_columns::id),
                ))
                .inner_join(
                    dataset_tags_columns::dataset_tags
                        .on(dataset_tags_columns::id.eq(chunk_metadata_tags_columns::tag_id)),
                )
                .filter(chunk_metadata_columns::id.eq(&chunk_data.id))
                .select(dataset_tags_columns::tag)
                .load::<String>(&mut conn)
                .await
                .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

            Ok(ChunkMetadata::from_table_and_tag_set(
                updated_chunk,
                chunk_tagset,
            ))
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
) -> Result<(), ServiceError> {
    let chunk_metadata = get_metadata_from_id_query(chunk_uuid, dataset.id, pool.clone()).await?;
    if chunk_metadata.dataset_id != dataset.id {
        return Err(ServiceError::BadRequest(
            "chunk does not belong to dataset".to_string(),
        ));
    }

    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    let mut conn = pool.get().await.unwrap();

    let transaction_result = conn
        .transaction::<_, diesel::result::Error, _>(|conn| {
            async move {
                {
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

                    let chunk_collisions: Vec<(ChunkCollision, ChunkMetadataTable)> =
                        chunk_collisions_columns::chunk_collisions
                            .inner_join(
                                chunk_metadata_columns::chunk_metadata
                                    .on(chunk_metadata_columns::qdrant_point_id
                                        .eq(chunk_collisions_columns::collision_qdrant_id)),
                            )
                            .filter(chunk_metadata_columns::id.eq(chunk_uuid))
                            .filter(chunk_metadata_columns::dataset_id.eq(dataset.id))
                            .select((ChunkCollision::as_select(), ChunkMetadataTable::as_select()))
                            .order_by(chunk_collisions_columns::created_at.asc())
                            .load::<(ChunkCollision, ChunkMetadataTable)>(conn)
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
                            ChunkMetadata::from_table_and_tag_set_option_string(
                                latest_collision_metadata,
                                None,
                            ),
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

    let qdrant_collection = format!("{}_vectors", config.EMBEDDING_SIZE);

    let qdrant_client = get_qdrant_connection(
        Some(get_env!("QDRANT_URL", "QDRANT_URL should be set")),
        Some(get_env!("QDRANT_API_KEY", "QDRANT_API_KEY should be set")),
    )
    .await?;

    match transaction_result {
        Ok(result) => match result {
            TransactionResult::ChunkCollisionNotDetected => {
                let _ = qdrant_client
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
                let collision_content = convert_html_to_text(
                    &(latest_collision_metadata
                        .chunk_html
                        .clone()
                        .unwrap_or_default()),
                );

                let new_embedding_vector = create_embedding(
                    collision_content,
                    "doc",
                    ServerDatasetConfiguration::from_json(dataset.server_configuration.clone()),
                )
                .await
                .map_err(|_e| {
                    ServiceError::BadRequest(
                        "Failed to create embedding for collision content".to_string(),
                    )
                })?
                .clone();

                let _ = qdrant_client
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
        },

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

#[tracing::instrument(skip(pool))]
pub async fn get_qdrant_ids_from_chunk_ids_query(
    chunk_ids: Vec<UnifiedId>,
    pool: web::Data<Pool>,
) -> Result<Vec<uuid::Uuid>, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    let qdrant_point_ids: Vec<Option<uuid::Uuid>> = match chunk_ids.get(0) {
        Some(UnifiedId::TrieveUuid(_)) => chunk_metadata_columns::chunk_metadata
            .select(chunk_metadata_columns::qdrant_point_id)
            .filter(
                chunk_metadata_columns::id.eq_any(
                    chunk_ids
                        .iter()
                        .map(|x| x.as_uuid().unwrap())
                        .collect::<Vec<uuid::Uuid>>(),
                ),
            )
            .load(&mut conn)
            .await
            .map_err(|_err| {
                ServiceError::BadRequest(
                    "Failed to get qdrant_point_id and collision_qdrant_id".to_string(),
                )
            })?,
        Some(UnifiedId::TrackingId(_)) => chunk_metadata_columns::chunk_metadata
            .select(chunk_metadata_columns::qdrant_point_id)
            .filter(
                chunk_metadata_columns::tracking_id.eq_any(
                    chunk_ids
                        .iter()
                        .map(|x| x.as_tracking_id().unwrap())
                        .collect::<Vec<String>>(),
                ),
            )
            .load(&mut conn)
            .await
            .map_err(|_err| {
                ServiceError::BadRequest(
                    "Failed to get qdrant_point_id and collision_qdrant_id".to_string(),
                )
            })?,
        None => {
            return Err(ServiceError::BadRequest(
                "Must pass in an ID to the condition".to_string(),
            ))
        }
    };

    Ok(qdrant_point_ids.into_iter().flatten().collect())
}

#[tracing::instrument]
pub fn get_highlights(
    input: ChunkMetadata,
    query: String,
    threshold: Option<f64>,
    delimiters: Vec<String>,
    max_length: Option<u32>,
    max_num: Option<u32>,
) -> Result<(ChunkMetadata, Vec<String>), ServiceError> {
    let content = convert_html_to_text(&(input.chunk_html.clone().unwrap_or_default()));

    let search_options = SearchOptions::new().threshold(threshold.unwrap_or(0.8));
    let mut engine: SimSearch<String> = SimSearch::new_with(search_options);

    let split_content = content
        .split_inclusive(|c: char| delimiters.contains(&c.to_string()))
        .flat_map(|x| {
            x.to_string()
                .split_whitespace()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .chunks(max_length.unwrap_or(5) as usize)
                .map(|x| x.join(" "))
                .collect::<Vec<String>>()
        })
        .collect::<Vec<String>>();

    split_content.iter().for_each(|sentence| {
        engine.insert(sentence.clone(), &sentence.clone());
    });

    let mut new_output = input;

    let results = engine.search(&query);
    let mut matched_phrases = vec![];
    for x in results.iter().take(max_num.unwrap_or(3) as usize) {
        matched_phrases.push(x.clone());
    }
    let result_matches = matched_phrases.clone();

    for phrase in matched_phrases {
        new_output.chunk_html = new_output
            .chunk_html
            .clone()
            .map(|x| x.replace(&phrase, &format!("<mark><b>{}</b></mark>", phrase)));
    }

    new_output.chunk_html = new_output
        .chunk_html
        .map(|x| x.replace("</b></mark><mark><b>", ""));

    Ok((new_output, result_matches))
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

#[tracing::instrument(skip(pool))]
pub async fn create_chunk_metadata(
    chunks: Vec<ChunkReqPayload>,
    dataset_uuid: uuid::Uuid,
    dataset_configuration: ServerDatasetConfiguration,
    pool: web::Data<Pool>,
) -> Result<(BulkUploadIngestionMessage, Vec<ChunkMetadata>), ServiceError> {
    let mut ingestion_messages = vec![];

    let mut chunk_metadatas = vec![];

    for chunk in chunks {
        let chunk_tag_set = chunk
            .tag_set
            .clone()
            .map(|tags| tags.into_iter().map(Some).collect::<Vec<Option<String>>>());

        let chunk_tracking_id = chunk
            .tracking_id
            .clone()
            .filter(|chunk_tracking| !chunk_tracking.is_empty());

        let timestamp = {
            chunk
                .time_stamp
                .clone()
                .map(|ts| -> Result<NaiveDateTime, ServiceError> {
                    Ok(ts
                        .parse::<DateTimeUtc>()
                        .map_err(|_| {
                            ServiceError::BadRequest("Invalid timestamp format".to_string())
                        })?
                        .0
                        .with_timezone(&chrono::Local)
                        .naive_local())
                })
                .transpose()?
        };

        let chunk_metadata = ChunkMetadata::from_details(
            &chunk.chunk_html.clone(),
            &chunk.link,
            &chunk_tag_set,
            None,
            chunk.metadata.clone(),
            chunk_tracking_id,
            timestamp,
            chunk.location,
            chunk.image_urls.clone(),
            dataset_uuid,
            chunk.weight.unwrap_or(0.0),
            chunk.num_value,
        );
        chunk_metadatas.push(chunk_metadata.clone());

        // check if a group_id is not in existent_group_ids and return an error if it is not
        if let Some(group_ids) = chunk.group_ids.clone() {
            let existent_group_ids = check_group_ids_exist_query(
                chunk.group_ids.clone().unwrap_or_default(),
                dataset_uuid,
                pool.clone(),
            )
            .await?;

            for group_id in group_ids {
                if !existent_group_ids.contains(&group_id) {
                    return Err(ServiceError::BadRequest(format!(
                        "Group with id {} does not exist",
                        group_id
                    )));
                }
            }
        }

        let group_ids_from_group_tracking_ids = if let Some(group_tracking_ids) =
            chunk.group_tracking_ids.clone()
        {
            get_group_ids_from_tracking_ids_query(group_tracking_ids, dataset_uuid, pool.clone())
                .await
                .ok()
                .unwrap_or(vec![])
        } else {
            vec![]
        };

        let initial_group_ids = chunk.group_ids.clone().unwrap_or_default();
        let mut chunk_only_group_ids = chunk.clone();
        let deduped_group_ids = group_ids_from_group_tracking_ids
            .into_iter()
            .chain(initial_group_ids.into_iter())
            .unique()
            .collect::<Vec<uuid::Uuid>>();

        chunk_only_group_ids.group_ids = Some(deduped_group_ids);
        chunk_only_group_ids.group_tracking_ids = None;

        let upload_message = UploadIngestionMessage {
            ingest_specific_chunk_metadata: IngestSpecificChunkMetadata {
                id: chunk_metadata.id,
                qdrant_point_id: chunk_metadata.qdrant_point_id,
                dataset_id: dataset_uuid,
                dataset_config: dataset_configuration.clone(),
            },
            dataset_id: dataset_uuid,
            dataset_config: dataset_configuration.clone(),
            chunk: chunk_only_group_ids.clone(),
            upsert_by_tracking_id: chunk.upsert_by_tracking_id.unwrap_or(false),
        };

        ingestion_messages.push(upload_message);
    }

    Ok((
        BulkUploadIngestionMessage {
            attempt_number: 0,
            dataset_id: dataset_uuid,
            dataset_configuration,
            ingestion_messages,
        },
        chunk_metadatas,
    ))
}
