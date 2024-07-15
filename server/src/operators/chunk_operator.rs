use std::collections::{HashMap, HashSet};

use crate::data::models::{
    ChunkData, ChunkGroupBookmark, ChunkMetadataTable, ChunkMetadataTags, ChunkMetadataTypes,
    ContentChunkMetadata, Dataset, DatasetTags, IngestSpecificChunkMetadata,
    ServerDatasetConfiguration, SlimChunkMetadata, SlimChunkMetadataTable, UnifiedId,
};
use crate::get_env;
use crate::handlers::chunk_handler::UploadIngestionMessage;
use crate::handlers::chunk_handler::{BulkUploadIngestionMessage, ChunkReqPayload};
use crate::operators::group_operator::{
    check_group_ids_exist_query, get_group_ids_from_tracking_ids_query,
};
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
use diesel::upsert::excluded;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use itertools::Itertools;
use qdrant_client::qdrant::PointId;
use simsearch::{SearchOptions, SimSearch};

#[tracing::instrument(skip(pool))]
pub async fn get_chunk_metadatas_from_point_ids(
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

    // Get tag_set
    let chunk_metadata_pairs: Vec<(ChunkMetadataTable, Option<Vec<String>>)> =
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
                sql::<sql_types::Array<sql_types::Text>>(
                    "array_remove(array_agg(dataset_tags.tag), null)",
                )
                .nullable(),
            ))
            .group_by(chunk_metadata_columns::id)
            .load::<(ChunkMetadataTable, Option<Vec<String>>)>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

    let chunk_metadatas = chunk_metadata_pairs
        .into_iter()
        .map(|(table, tag_set)| {
            ChunkMetadataTypes::Metadata(
                ChunkMetadata::from_table_and_tag_set(table, tag_set.unwrap_or_default()).into(),
            )
        })
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
            .load::<uuid::Uuid>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?,
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
            .load::<uuid::Uuid>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?,
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
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkMetadataTypes>, ServiceError> {
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
        // Get tagset and chunk metadatatable

        let chunk_metadata_pair: Vec<(ChunkMetadataTable, Option<Vec<String>>)> =
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
                    sql::<sql_types::Array<sql_types::Text>>(
                        "array_remove(array_agg(dataset_tags.tag), null)",
                    )
                    .nullable(),
                ))
                .filter(chunk_metadata_columns::qdrant_point_id.eq_any(&point_ids))
                .group_by(chunk_metadata_columns::id)
                .load::<(ChunkMetadataTable, Option<Vec<String>>)>(&mut conn)
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
                ChunkMetadata::from_table_and_tag_set(chunk_table, tag_set.unwrap_or(vec![])).into()
            })
            .collect::<Vec<ChunkMetadataTypes>>()
    };

    chunk_search_span.finish();
    transaction.finish();
    // Return only the chunk metadata
    Ok(chunk_metadatas)
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

    let slim_chunk_pairs: Vec<(SlimChunkMetadataTable, Option<Vec<String>>)> =
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
                sql::<sql_types::Array<sql_types::Text>>(
                    "array_remove(array_agg(dataset_tags.tag), null)",
                )
                .nullable(),
            ))
            .group_by(chunk_metadata_columns::id)
            .load::<(SlimChunkMetadataTable, Option<Vec<String>>)>(&mut conn)
            .await
            .map_err(|_| {
                ServiceError::BadRequest("Failed to load slim_chunk_metadatas".to_string())
            })?;

    let slim_chunks = slim_chunk_pairs
        .into_iter()
        .map(|(table, tag_set)| {
            SlimChunkMetadata::from_table_and_tag_set(table, tag_set.unwrap_or(vec![])).into()
        })
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
            sql::<sql_types::Array<sql_types::Text>>(
                "array_remove(array_agg(dataset_tags.tag), null)",
            )
            .nullable(),
        ))
        .group_by(chunk_metadata_columns::id)
        .first::<(ChunkMetadataTable, Option<Vec<String>>)>(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::NotFound("Chunk with id not found in the specified dataset".to_string())
        })?;

    Ok(ChunkMetadata::from_table_and_tag_set(
        chunk_table,
        tag_set.unwrap_or(vec![]),
    ))
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
            sql::<sql_types::Array<sql_types::Text>>(
                "array_remove(array_agg(dataset_tags.tag), null)",
            )
            .nullable(),
        ))
        .group_by(chunk_metadata_columns::id)
        .first::<(ChunkMetadataTable, Option<Vec<String>>)>(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::NotFound(
                "Chunk with tracking_id not found in the specified dataset".to_string(),
            )
        })?;

    Ok(ChunkMetadata::from_table_and_tag_set(
        chunk_table,
        tag_set.unwrap_or(vec![]),
    ))
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

    let chunk_metadata_pairs: Vec<(ChunkMetadataTable, Option<Vec<String>>)> =
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
                sql::<sql_types::Array<sql_types::Text>>(
                    "array_remove(array_agg(dataset_tags.tag), null)",
                )
                .nullable(),
            ))
            .group_by(chunk_metadata_columns::id)
            .load::<(ChunkMetadataTable, Option<Vec<String>>)>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

    let chunk_metadatas = chunk_metadata_pairs
        .into_iter()
        .map(|(table, tag_set)| {
            ChunkMetadata::from_table_and_tag_set(table, tag_set.unwrap_or(vec![]))
        })
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

    let chunk_metadata_pairs: Vec<(ChunkMetadataTable, Option<Vec<String>>)> =
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
                sql::<sql_types::Array<sql_types::Text>>(
                    "array_remove(array_agg(dataset_tags.tag), null)",
                )
                .nullable(),
            ))
            .group_by(chunk_metadata_columns::id)
            .load::<(ChunkMetadataTable, Option<Vec<String>>)>(&mut conn)
            .await
            .map_err(|_| ServiceError::BadRequest("Failed to load metadata".to_string()))?;

    let chunk_metadatas = chunk_metadata_pairs
        .into_iter()
        .map(|(table, tag_set)| {
            ChunkMetadata::from_table_and_tag_set(table, tag_set.unwrap_or(vec![]))
        })
        .collect();

    Ok(chunk_metadatas)
}

/// Only inserts, does not try to upsert data
#[allow(clippy::type_complexity)]
#[tracing::instrument(skip(pool))]
pub async fn bulk_insert_chunk_metadata_query(
    mut insertion_data: Vec<ChunkData>,
    dataset_uuid: uuid::Uuid,
    upsert_by_tracking_id: bool,
    pool: web::Data<Pool>,
) -> Result<Vec<ChunkData>, ServiceError> {
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool
        .clone()
        .get()
        .await
        .expect("Failed to get connection to db");

    let (chunks_with_tracking_id, chunks_without_tracking_id): (Vec<ChunkData>, Vec<ChunkData>) =
        insertion_data
            .clone()
            .into_iter()
            .partition(|data| data.chunk_metadata.tracking_id.is_some());
    let chunks_with_tracking_id: Vec<ChunkMetadataTable> = chunks_with_tracking_id
        .iter()
        .map(|data| data.chunk_metadata.clone().into())
        .unique_by(|chunk: &ChunkMetadataTable| (chunk.tracking_id.clone(), chunk.dataset_id))
        .collect();
    let chunk_metadatas_to_insert: Vec<ChunkMetadataTable> = chunks_without_tracking_id
        .iter()
        .map(|data| data.chunk_metadata.clone().into())
        .chain(chunks_with_tracking_id.into_iter())
        .collect();

    let inserted_chunks = if upsert_by_tracking_id {
        let temp_inserted_chunks = diesel::insert_into(chunk_metadata_columns::chunk_metadata)
            .values(&chunk_metadatas_to_insert)
            .on_conflict((
                chunk_metadata_columns::tracking_id,
                chunk_metadata_columns::dataset_id,
            ))
            .do_update()
            .set((
                chunk_metadata_columns::link.eq(excluded(chunk_metadata_columns::link)),
                chunk_metadata_columns::chunk_html.eq(excluded(chunk_metadata_columns::chunk_html)),
                chunk_metadata_columns::metadata.eq(excluded(chunk_metadata_columns::metadata)),
                chunk_metadata_columns::time_stamp.eq(excluded(chunk_metadata_columns::time_stamp)),
                chunk_metadata_columns::weight.eq(excluded(chunk_metadata_columns::weight)),
                chunk_metadata_columns::location.eq(excluded(chunk_metadata_columns::location)),
                chunk_metadata_columns::image_urls.eq(excluded(chunk_metadata_columns::image_urls)),
                chunk_metadata_columns::num_value.eq(excluded(chunk_metadata_columns::num_value)),
            ))
            .returning(ChunkMetadataTable::as_select())
            .get_results::<ChunkMetadataTable>(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to upsert chunk_metadata: {:?}", e);

                ServiceError::BadRequest("Failed to upsert chunk_metadata".to_string())
            })?;

        insertion_data.retain(|chunk_data| {
            temp_inserted_chunks.iter().any(|inserted_chunk| {
                inserted_chunk.id == chunk_data.chunk_metadata.id
                    || inserted_chunk.tracking_id == chunk_data.chunk_metadata.tracking_id
            })
        });

        temp_inserted_chunks
    } else {
        let temp_inserted_chunks = diesel::insert_into(chunk_metadata_columns::chunk_metadata)
            .values(&chunk_metadatas_to_insert)
            .on_conflict_do_nothing()
            .get_results::<ChunkMetadataTable>(&mut conn)
            .await
            .map_err(|e| {
                log::error!("Failed to insert chunk_metadata: {:?}", e);

                ServiceError::BadRequest("Failed to insert chunk_metadata".to_string())
            })?;

        insertion_data.retain(|chunk_data| {
            temp_inserted_chunks
                .iter()
                .any(|inserted_chunk| inserted_chunk.id == chunk_data.chunk_metadata.id)
        });

        temp_inserted_chunks
    };

    let insertion_data = insertion_data
        .into_iter()
        .map(|chunk_data| {
            let chunk_metadata_table = inserted_chunks
                .iter()
                .find(|inserted_chunk: &&ChunkMetadataTable| {
                    inserted_chunk.id == chunk_data.chunk_metadata.id
                        || (inserted_chunk
                            .tracking_id
                            .as_ref()
                            .is_some_and(|tracking_id| !tracking_id.is_empty())
                            && inserted_chunk.tracking_id == chunk_data.chunk_metadata.tracking_id)
                })
                .expect("Will always be present due to previous retain")
                .clone();

            let tag_set = chunk_data
                .chunk_metadata
                .tag_set
                .clone()
                .unwrap_or_default()
                .iter()
                .filter_map(|maybe_tag| maybe_tag.clone())
                .collect_vec();
            let chunk_metadata =
                ChunkMetadata::from_table_and_tag_set(chunk_metadata_table, tag_set);

            ChunkData {
                chunk_metadata,
                content: chunk_data.content,
                group_ids: chunk_data.group_ids,
                upsert_by_tracking_id: chunk_data.upsert_by_tracking_id,
                boost_phrase: chunk_data.boost_phrase,
                distance_phrase: chunk_data.distance_phrase,
            }
        })
        .collect::<Vec<ChunkData>>();

    let chunk_group_bookmarks_to_insert: Vec<ChunkGroupBookmark> = insertion_data
        .clone()
        .iter()
        .filter_map(|data| {
            data.group_ids.as_ref().map(|group_ids| {
                group_ids
                    .clone()
                    .iter()
                    .map(|group_id| {
                        ChunkGroupBookmark::from_details(*group_id, data.chunk_metadata.id)
                    })
                    .collect::<Vec<ChunkGroupBookmark>>()
            })
        })
        .flatten()
        .collect();

    diesel::insert_into(chunk_group_bookmarks_columns::chunk_group_bookmarks)
        .values(chunk_group_bookmarks_to_insert)
        .on_conflict_do_nothing()
        .execute(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to insert chunk into groups".to_string()))?;

    let chunk_tags_to_chunk_id: Vec<(Vec<DatasetTags>, uuid::Uuid)> = insertion_data
        .clone()
        .iter()
        .filter_map(|data| {
            data.chunk_metadata.clone().tag_set.map(|tags| {
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

                (tags, data.chunk_metadata.id)
            })
        })
        .collect_vec();

    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;
    // TODO, dedupe and bulk insert this.
    for (dataset_tags, chunk_uuid) in chunk_tags_to_chunk_id {
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
            !dataset_tags_existing
                .iter()
                .any(|predicate_tag| predicate_tag.tag == dataset_tag.tag)
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

    let optional_chunk: Option<(ChunkMetadataTable, Option<Vec<String>>)> =
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
                sql::<sql_types::Array<sql_types::Text>>(
                    "array_remove(array_agg(dataset_tags.tag), null)",
                )
                .nullable(),
            ))
            .group_by(chunk_metadata_columns::id)
            .load::<(ChunkMetadataTable, Option<Vec<String>>)>(&mut conn)
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

    Ok(optional_chunk.map(|(chunk_table, tag_set)| {
        ChunkMetadata::from_table_and_tag_set(chunk_table, tag_set.unwrap_or(vec![]))
    }))
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
            !dataset_tags_existing
                .iter()
                .any(|predicate_tag| predicate_tag.tag == dataset_tag.tag)
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
                !dataset_tags_existing
                    .iter()
                    .any(|predicate_tag| predicate_tag.tag == dataset_tag.tag)
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

#[tracing::instrument(skip(pool))]
pub async fn delete_chunk_metadata_query(
    chunk_uuid: Vec<uuid::Uuid>,
    dataset: Dataset,
    pool: web::Data<Pool>,
    config: ServerDatasetConfiguration,
) -> Result<(), ServiceError> {
    let chunk_metadata =
        get_metadata_from_ids_query(chunk_uuid.clone(), dataset.id, pool.clone()).await?;
    if chunk_metadata.get(0).unwrap().dataset_id != dataset.id {
        return Err(ServiceError::BadRequest(
            "chunk does not belong to dataset".to_string(),
        ));
    }

    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    let mut conn = pool.get().await.unwrap();

    let transaction_result = conn
        .transaction::<_, diesel::result::Error, _>(|conn| {
            async move {
                {
                    diesel::delete(chunk_group_bookmarks_columns::chunk_group_bookmarks.filter(
                        chunk_group_bookmarks_columns::chunk_metadata_id.eq_any(chunk_uuid.clone()),
                    ))
                    .execute(conn)
                    .await?;

                    // if there were no collisions, just delete the chunk_metadata without issue
                    diesel::delete(
                        chunk_metadata_columns::chunk_metadata
                            .filter(chunk_metadata_columns::id.eq_any(chunk_uuid.clone()))
                            .filter(chunk_metadata_columns::dataset_id.eq(dataset.id)),
                    )
                    .execute(conn)
                    .await?;

                    Ok(())
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

    let point_ids = chunk_metadata
        .iter()
        .map(|x| x.qdrant_point_id)
        .map(|x| x.to_string().into())
        .collect::<Vec<PointId>>();

    match transaction_result {
        Ok(()) => qdrant_client
            .delete_points_blocking(qdrant_collection, None, &point_ids.into(), None)
            .await
            .map_err(|_e| {
                ServiceError::BadRequest("Failed to delete chunk from qdrant".to_string())
            })
            .map(|_| ()),
        Err(_) => {
            return Err(ServiceError::BadRequest(
                "Failed to delete chunk data".to_string(),
            ))
        }
    }
}

#[tracing::instrument(skip(pool))]
pub async fn get_qdrant_id_from_chunk_id_query(
    chunk_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<uuid::Uuid, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    let qdrant_point_ids: Vec<uuid::Uuid> = chunk_metadata_columns::chunk_metadata
        .select(chunk_metadata_columns::qdrant_point_id)
        .filter(chunk_metadata_columns::id.eq(chunk_id))
        .load(&mut conn)
        .await
        .map_err(|_err| ServiceError::BadRequest("Failed to get qdrant_point_id".to_string()))?;

    match qdrant_point_ids.get(0) {
        Some(x) => Ok(*x),
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

    let qdrant_point_ids: Vec<uuid::Uuid> = match chunk_ids.get(0) {
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

    Ok(qdrant_point_ids)
}

pub fn get_slice_from_vec_string(vec: Vec<String>, index: usize) -> Result<String, ServiceError> {
    match vec.get(index) {
        Some(x) => Ok(x.clone()),
        None => Err(ServiceError::BadRequest(
            "Index out of bounds for vec".to_string(),
        )),
    }
}

#[tracing::instrument]
pub fn get_highlights(
    input: ChunkMetadata,
    query: String,
    threshold: Option<f64>,
    delimiters: Vec<String>,
    max_length: Option<u32>,
    max_num: Option<u32>,
    window_size: Option<u32>,
) -> Result<(ChunkMetadata, Vec<String>), ServiceError> {
    let content = convert_html_to_text(&(input.chunk_html.clone().unwrap_or_default()));
    let search_options = SearchOptions::new().threshold(threshold.unwrap_or(0.8));
    let mut engine: SimSearch<usize> = SimSearch::new_with(search_options);
    let split_content = content
        .split_inclusive(|c: char| delimiters.contains(&c.to_string()))
        .flat_map(|x| {
            x.to_string()
                .split_inclusive(' ')
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .chunks(max_length.unwrap_or(5) as usize)
                .map(|x| x.join(""))
                .collect::<Vec<String>>()
        })
        .collect::<Vec<String>>();

    split_content.iter().enumerate().for_each(|(i, x)| {
        engine.insert(i, x);
    });

    let new_output = input;
    let results = engine.search(&query);
    let mut matched_idxs = vec![];
    let mut matched_idxs_set = HashSet::new();
    for x in results.iter().take(max_num.unwrap_or(3) as usize) {
        matched_idxs_set.insert(*x);
        matched_idxs.push(*x);
    }
    matched_idxs.sort();
    let window = window_size.unwrap_or(0);
    if window == 0 {
        return Ok((
            apply_highlights_to_html(
                new_output,
                matched_idxs
                    .iter()
                    .map(|x| split_content.get(*x).unwrap().clone())
                    .collect(),
            ),
            matched_idxs
                .iter()
                .map(|x| split_content.get(*x).unwrap().clone())
                .collect(),
        ));
    }
    let half_window = std::cmp::max(window / 2, 1);
    // edge case 1: When the half window size is greater than the length of left or right phrase,
    // we need to search further to get the correct windowed phrase
    // edge case 2: When two windowed phrases overlap, we need to trim the first one.
    let mut windowed_phrases = vec![];
    // Used to keep track of the number of words used in the phrase
    let mut used_phrases: HashMap<usize, usize> = HashMap::new();
    for idx in matched_idxs.clone() {
        let phrase = get_slice_from_vec_string(split_content.clone(), idx)?;
        let mut next_phrase = String::new();
        if idx < split_content.len() - 1 {
            let mut start = idx + 1;
            let mut count: usize = 0;
            while (count as u32) < half_window {
                if start >= split_content.len() || matched_idxs_set.contains(&start) {
                    break;
                }
                let slice = get_slice_from_vec_string(split_content.clone(), start)?;
                let candidate_words = slice
                    .split_inclusive(' ')
                    .take(half_window as usize - count)
                    .collect::<Vec<&str>>();
                used_phrases.insert(
                    start,
                    std::cmp::min(candidate_words.len(), half_window as usize - count),
                );
                count += candidate_words.len();
                next_phrase.push_str(&candidate_words.join(""));
                start += 1;
            }
        }
        let mut prev_phrase = String::new();
        if idx > 0 {
            let mut start = idx - 1;
            let mut count: usize = 0;
            while (count as u32) < half_window {
                let slice = get_slice_from_vec_string(split_content.clone(), start)?;
                let split_words = slice.split_inclusive(' ').collect::<Vec<&str>>();
                if matched_idxs_set.contains(&start) {
                    break;
                }
                if used_phrases.contains_key(&start)
                    && split_words.len()
                        > *used_phrases
                            .get(&start)
                            .ok_or(ServiceError::BadRequest("Index out of bounds".to_string()))?
                {
                    let remaining_count = half_window as usize - count;
                    let available_word_len = split_words.len()
                        - *used_phrases
                            .get(&start)
                            .ok_or(ServiceError::BadRequest("Index out of bounds".to_string()))?;
                    if remaining_count > available_word_len {
                        count += remaining_count - available_word_len;
                    } else {
                        break;
                    }
                }
                if used_phrases.contains_key(&start)
                    && split_words.len()
                        <= *used_phrases
                            .get(&start)
                            .ok_or(ServiceError::BadRequest("Index out of bounds".to_string()))?
                {
                    break;
                }
                let candidate_words = split_words
                    .into_iter()
                    .rev()
                    .take(half_window as usize - count)
                    .collect::<Vec<&str>>();
                count += candidate_words.len();
                prev_phrase = format!("{}{}", candidate_words.iter().rev().join(""), prev_phrase);
                if start == 0 {
                    break;
                }
                start -= 1;
            }
        }
        let highlighted_phrase = phrase.replace(
            phrase.trim(),
            &format!("<mark><b>{}</b></mark>", phrase.trim()),
        );
        let windowed_phrase = format!("{}{}{}", prev_phrase, highlighted_phrase, next_phrase);
        windowed_phrases.push(windowed_phrase);
    }
    let matched_phrases = matched_idxs
        .clone()
        .iter()
        .filter_map(|x| split_content.get(*x).cloned())
        .collect::<Vec<String>>();
    let result_matches = if windowed_phrases.is_empty() {
        matched_phrases.clone()
    } else {
        windowed_phrases.clone()
    };
    Ok((
        apply_highlights_to_html(new_output, matched_phrases),
        result_matches,
    ))
}

fn apply_highlights_to_html(input: ChunkMetadata, phrases: Vec<String>) -> ChunkMetadata {
    let mut meta_data = input;
    let mut chunk_html = meta_data.chunk_html.clone().unwrap_or_default();
    let mut replaced_phrases = HashSet::new();
    for phrase in phrases.clone() {
        if replaced_phrases.contains(&phrase) {
            continue;
        }
        let replace_phrase = phrase.clone();
        chunk_html = chunk_html
            .replace(
                &replace_phrase,
                &format!("<mark><b>{}</b></mark>", replace_phrase),
            )
            .replace("</b></mark><mark><b>", "");
        replaced_phrases.insert(phrase);
    }
    meta_data.chunk_html = Some(chunk_html);
    meta_data
}

#[tracing::instrument(skip(pool))]
pub async fn get_row_count_for_organization_id_query(
    organization_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<usize, ServiceError> {
    use crate::data::schema::organization_usage_counts::dsl as organization_usage_counts_columns;

    let mut conn = pool.get().await.expect("Failed to get connection to db");

    let chunk_metadata_count = organization_usage_counts_columns::organization_usage_counts
        .filter(organization_usage_counts_columns::org_id.eq(organization_id))
        .select(organization_usage_counts_columns::chunk_count)
        .first::<i32>(&mut conn)
        .await
        .map_err(|_| {
            log::error!("Failed to get chunk count for organization");
            ServiceError::BadRequest("Failed to get chunk count for organization".to_string())
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
            uuid::Uuid::new_v4(),
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

        let group_ids_from_group_tracking_ids =
            if let Some(group_tracking_ids) = chunk.group_tracking_ids.clone() {
                let group_id_tracking_ids = get_group_ids_from_tracking_ids_query(
                    group_tracking_ids.clone(),
                    dataset_uuid,
                    pool.clone(),
                )
                .await?;

                let group_ids = group_id_tracking_ids
                    .clone()
                    .into_iter()
                    .map(|(group_id, _)| group_id)
                    .collect::<Vec<uuid::Uuid>>();
                let found_group_tracking_ids = group_id_tracking_ids
                    .into_iter()
                    .filter_map(|(_, group_tracking_id)| group_tracking_id)
                    .collect::<Vec<String>>();

                for group_tracking_id in group_tracking_ids {
                    if !found_group_tracking_ids.contains(&group_tracking_id) {
                        return Err(ServiceError::BadRequest(format!(
                            "Group with tracking id {} does not exist",
                            group_tracking_id
                        )));
                    }
                }

                group_ids
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
            chunk: chunk_only_group_ids.clone(),
            upsert_by_tracking_id: chunk.upsert_by_tracking_id.unwrap_or(false),
        };

        ingestion_messages.push(upload_message);
    }

    Ok((
        BulkUploadIngestionMessage {
            attempt_number: 0,
            dataset_id: dataset_uuid,
            ingestion_messages,
        },
        chunk_metadatas,
    ))
}

#[tracing::instrument(skip(pool))]
pub async fn get_pg_point_ids_from_qdrant_point_ids(
    qdrant_point_ids: Vec<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<uuid::Uuid>, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;

    let mut conn = pool.get().await.unwrap();

    let chunk_ids: Vec<uuid::Uuid> = chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::qdrant_point_id.eq_any(qdrant_point_ids))
        .select(chunk_metadata_columns::qdrant_point_id)
        .load(&mut conn)
        .await
        .map_err(|_| ServiceError::BadRequest("Failed to get chunk ids".to_string()))?;

    Ok(chunk_ids)
}
