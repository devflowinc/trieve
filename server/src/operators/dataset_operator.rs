use std::ops::Add;

use crate::data::models::{
    DatasetAndOrgWithSubAndPlan, DatasetAndUsage, DatasetConfiguration, DatasetUsageCount,
    DatasetWordsLastProcessed, Organization, OrganizationWithSubAndPlan, RedisPool, StripePlan,
    StripeSubscription, UnifiedId, WordDataset, WordInDataset,
};
use crate::handlers::dataset_handler::{GetDatasetsPagination, TagsWithCount};
use crate::operators::clickhouse_operator::ClickHouseEvent;
use crate::operators::qdrant_operator::{
    delete_points_from_qdrant, get_qdrant_collection_from_dataset_config,
};
use crate::{
    data::models::{Dataset, EventType, Pool, WorkerEvent},
    errors::ServiceError,
};
use actix_web::web;
use diesel::dsl::count;
use diesel::prelude::*;
use diesel::result::{DatabaseErrorKind, Error as DBError};
use diesel::upsert::excluded;
use diesel_async::{AsyncConnection, RunQueryDsl};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::clickhouse_operator::EventQueue;

#[tracing::instrument(skip(pool))]
pub async fn create_dataset_query(
    new_dataset: Dataset,
    pool: web::Data<Pool>,
) -> Result<Dataset, ServiceError> {
    use crate::data::schema::datasets::dsl::*;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    diesel::insert_into(datasets)
        .values(&new_dataset)
        .execute(&mut conn)
        .await
        .map_err(|err| match err {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            ) => ServiceError::BadRequest(
                "Could not create dataset because a dataset with the same tracking_id already exists in the organization".to_string(),
            ),
            _ => ServiceError::BadRequest("Could not create dataset".to_string()),
        })?;

    Ok(new_dataset)
}

#[tracing::instrument(skip(pool))]
pub async fn get_dataset_by_id_query(
    id: UnifiedId,
    pool: web::Data<Pool>,
) -> Result<Dataset, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;
    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let dataset = match id {
        UnifiedId::TrieveUuid(id) => datasets_columns::datasets
            .filter(datasets_columns::id.eq(id))
            .filter(datasets_columns::deleted.eq(0))
            .select(Dataset::as_select())
            .first(&mut conn)
            .await
            .map_err(|_| ServiceError::NotFound("Could not find dataset".to_string()))?,
        UnifiedId::TrackingId(id) => datasets_columns::datasets
            .filter(datasets_columns::tracking_id.eq(id))
            .filter(datasets_columns::deleted.eq(0))
            .select(Dataset::as_select())
            .first(&mut conn)
            .await
            .map_err(|_| ServiceError::NotFound("Could not find dataset".to_string()))?,
    };

    Ok(dataset)
}

#[tracing::instrument(skip(pool))]
pub async fn get_deleted_dataset_by_unifiedid_query(
    id: UnifiedId,
    pool: web::Data<Pool>,
) -> Result<Dataset, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;
    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let dataset = match id {
        UnifiedId::TrieveUuid(id) => datasets_columns::datasets
            .filter(datasets_columns::id.eq(id))
            .select(Dataset::as_select())
            .first(&mut conn)
            .await
            .map_err(|_| ServiceError::NotFound("Could not find dataset".to_string()))?,
        UnifiedId::TrackingId(id) => datasets_columns::datasets
            .filter(datasets_columns::tracking_id.eq(id))
            .select(Dataset::as_select())
            .first(&mut conn)
            .await
            .map_err(|_| ServiceError::NotFound("Could not find dataset".to_string()))?,
    };

    Ok(dataset)
}

#[tracing::instrument(skip(pool))]
pub async fn get_dataset_and_organization_from_dataset_id_query(
    id: UnifiedId,
    org_id: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<DatasetAndOrgWithSubAndPlan, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;
    use crate::data::schema::organizations::dsl as organizations_columns;
    use crate::data::schema::stripe_plans::dsl as stripe_plans_columns;
    use crate::data::schema::stripe_subscriptions::dsl as stripe_subscriptions_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let query = datasets_columns::datasets
        .inner_join(organizations_columns::organizations)
        .left_outer_join(
            stripe_subscriptions_columns::stripe_subscriptions
                .on(stripe_subscriptions_columns::organization_id.eq(organizations_columns::id)),
        )
        .left_outer_join(
            stripe_plans_columns::stripe_plans
                .on(stripe_plans_columns::id.eq(stripe_subscriptions_columns::plan_id)),
        )
        .filter(datasets_columns::deleted.eq(0))
        .into_boxed();

    let (dataset, organization, stripe_plan, stripe_subscription) = match id {
        UnifiedId::TrieveUuid(id) => query
            .filter(datasets_columns::id.eq(id))
            .filter(datasets_columns::deleted.eq(0))
            .select((
                Dataset::as_select(),
                organizations_columns::organizations::all_columns(),
                stripe_plans_columns::stripe_plans::all_columns().nullable(),
                stripe_subscriptions_columns::stripe_subscriptions::all_columns().nullable(),
            ))
            .first::<(
                Dataset,
                Organization,
                Option<StripePlan>,
                Option<StripeSubscription>,
            )>(&mut conn)
            .await
            .map_err(|_| ServiceError::NotFound("Could not find dataset".to_string()))?,
        UnifiedId::TrackingId(id) => query
            .filter(datasets_columns::tracking_id.eq(id))
            .filter(datasets_columns::organization_id.eq(org_id.unwrap_or_default()))
            .filter(datasets_columns::deleted.eq(0))
            .select((
                Dataset::as_select(),
                organizations_columns::organizations::all_columns(),
                stripe_plans_columns::stripe_plans::all_columns().nullable(),
                stripe_subscriptions_columns::stripe_subscriptions::all_columns().nullable(),
            ))
            .first::<(
                Dataset,
                Organization,
                Option<StripePlan>,
                Option<StripeSubscription>,
            )>(&mut conn)
            .await
            .map_err(|_| ServiceError::NotFound("Could not find dataset".to_string()))?,
    };

    let org_with_plan_sub: OrganizationWithSubAndPlan =
        OrganizationWithSubAndPlan::from_components(organization, stripe_plan, stripe_subscription);

    Ok(DatasetAndOrgWithSubAndPlan::from_components(
        dataset,
        org_with_plan_sub,
    ))
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeleteMessage {
    pub dataset_id: uuid::Uuid,
    pub attempt_number: usize,
    pub deleted_at: chrono::NaiveDateTime,
    pub empty_dataset: bool,
}

#[tracing::instrument(skip(pool))]
pub async fn soft_delete_dataset_by_id_query(
    id: uuid::Uuid,
    dataset_config: DatasetConfiguration,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
) -> Result<(), ServiceError> {
    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    if dataset_config.LOCKED {
        return Err(ServiceError::BadRequest(
            "Cannot delete a locked dataset".to_string(),
        ));
    }

    diesel::sql_query(format!(
        "UPDATE datasets SET deleted = 1, tracking_id = NULL WHERE id = '{}'::uuid",
        id
    ))
    .execute(&mut conn)
    .await
    .map_err(|err| {
        log::error!("Could not delete dataset: {}", err);
        ServiceError::BadRequest("Could not delete dataset".to_string())
    })?;

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let message = DeleteMessage {
        dataset_id: id,
        attempt_number: 0,
        deleted_at: chrono::Utc::now().naive_utc(),
        empty_dataset: false,
    };

    let serialized_message =
        serde_json::to_string(&message).map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    redis::cmd("lpush")
        .arg("delete_dataset_queue")
        .arg(&serialized_message)
        .query_async(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}

pub async fn clear_dataset_by_dataset_id_query(
    id: uuid::Uuid,
    dataset_config: DatasetConfiguration,
    redis_pool: web::Data<RedisPool>,
) -> Result<(), ServiceError> {
    if dataset_config.LOCKED {
        return Err(ServiceError::BadRequest(
            "Cannot delete a locked dataset".to_string(),
        ));
    }
    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let message = DeleteMessage {
        dataset_id: id,
        attempt_number: 0,
        deleted_at: chrono::Utc::now().naive_utc(),
        empty_dataset: true,
    };

    let serialized_message =
        serde_json::to_string(&message).map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    redis::cmd("lpush")
        .arg("delete_dataset_queue")
        .arg(&serialized_message)
        .query_async(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}

pub async fn clear_dataset_query(
    id: uuid::Uuid,
    deleted_at: chrono::NaiveDateTime,
    pool: web::Data<Pool>,
    event_queue: web::Data<EventQueue>,
    dataset_config: DatasetConfiguration,
) -> Result<(), ServiceError> {
    use crate::data::schema::chunk_group::dsl as chunk_group;
    use crate::data::schema::chunk_group_bookmarks::dsl as chunk_group_bookmarks_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::files::dsl as files_column;

    let mut conn = pool.get().await.unwrap();

    let qdrant_collection = get_qdrant_collection_from_dataset_config(&dataset_config);

    let chunk_groups = chunk_group::chunk_group
        .filter(chunk_group::dataset_id.eq(id))
        .select(chunk_group::id)
        .load::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|err| {
            log::error!("Could not fetch groups: {}", err);
            ServiceError::BadRequest("Could not fetch groups".to_string())
        })?;

    diesel::delete(
        chunk_group_bookmarks_columns::chunk_group_bookmarks
            .filter(chunk_group_bookmarks_columns::group_id.eq_any(chunk_groups))
            .filter(chunk_group_bookmarks_columns::created_at.le(deleted_at)),
    )
    .execute(&mut conn)
    .await
    .map_err(|err| {
        log::error!("Could not delete chunk_group_bookmarks: {}", err);
        ServiceError::BadRequest("Could not delete chunk_group_bookmarks".to_string())
    })?;

    diesel::delete(
        chunk_group::chunk_group
            .filter(chunk_group::dataset_id.eq(id))
            .filter(chunk_group::created_at.le(deleted_at)),
    )
    .execute(&mut conn)
    .await
    .map_err(|err| {
        log::error!("Could not delete groups: {}", err);
        ServiceError::BadRequest("Could not delete groups".to_string())
    })?;

    diesel::delete(
        files_column::files
            .filter(files_column::dataset_id.eq(id))
            .filter(files_column::created_at.le(deleted_at)),
    )
    .execute(&mut conn)
    .await
    .map_err(|err| {
        log::error!("Could not delete files: {}", err);
        ServiceError::BadRequest("Could not delete files".to_string())
    })?;

    let mut last_offset_id = uuid::Uuid::nil();

    loop {
        let chunk_and_qdrant_ids: Vec<(uuid::Uuid, uuid::Uuid)> =
            chunk_metadata_columns::chunk_metadata
                .filter(chunk_metadata_columns::dataset_id.eq(id))
                .filter(chunk_metadata_columns::id.gt(last_offset_id))
                .filter(chunk_metadata_columns::created_at.le(deleted_at))
                .select((
                    chunk_metadata_columns::id,
                    chunk_metadata_columns::qdrant_point_id,
                ))
                .order(chunk_metadata_columns::id)
                .limit(
                    option_env!("DELETE_CHUNK_BATCH_SIZE")
                        .unwrap_or("5000")
                        .parse::<i64>()
                        .unwrap_or(5000),
                )
                .load::<(uuid::Uuid, uuid::Uuid)>(&mut conn)
                .await
                .map_err(|err| {
                    log::error!("Could not fetch chunk IDs: {}", err);
                    ServiceError::BadRequest("Could not fetch chunk IDs to delete".to_string())
                })?;

        let chunk_ids = chunk_and_qdrant_ids
            .iter()
            .map(|(id, _)| *id)
            .collect::<Vec<uuid::Uuid>>();
        let qdrant_point_ids = chunk_and_qdrant_ids
            .iter()
            .map(|(_, qdrant_id)| *qdrant_id)
            .collect::<Vec<uuid::Uuid>>();

        if chunk_ids.is_empty() {
            break;
        }

        diesel::delete(
            chunk_metadata_columns::chunk_metadata
                .filter(chunk_metadata_columns::id.eq_any(&chunk_ids)),
        )
        .execute(&mut conn)
        .await
        .map_err(|err| {
            log::error!("Could not delete chunks in current batch: {}", err);
            ServiceError::BadRequest("Could not delete chunks in current batch".to_string())
        })?;

        delete_points_from_qdrant(qdrant_point_ids, qdrant_collection.clone())
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not delete points in current batch from qdrant: {}",
                    err
                ))
            })?;

        event_queue
            .send(ClickHouseEvent::WorkerEvent(
                WorkerEvent::from_details(
                    id,
                    EventType::BulkChunksDeleted {
                        message: format!("Deleted {} chunks", chunk_ids.len()),
                    },
                )
                .into(),
            ))
            .await;

        log::info!("Deleted {} chunks from {}", chunk_ids.len(), id);

        last_offset_id = *chunk_ids.last().unwrap();
    }

    Ok(())
}

#[tracing::instrument(skip_all)]
pub async fn delete_dataset_by_id_query(
    id: uuid::Uuid,
    deleted_at: chrono::NaiveDateTime,
    pool: web::Data<Pool>,
    clickhouse_client: actix_web::web::Data<clickhouse::Client>,
    event_queue: web::Data<EventQueue>,
    dataset_config: DatasetConfiguration,
) -> Result<Dataset, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool.get().await.unwrap();

    clear_dataset_query(
        id,
        deleted_at,
        pool.clone(),
        event_queue.clone(),
        dataset_config.clone(),
    )
    .await?;

    let dataset: Dataset =
        diesel::delete(datasets_columns::datasets.filter(datasets_columns::id.eq(id)))
            .get_result(&mut conn)
            .await
            .map_err(|err| {
                log::error!("Could not delete dataset: {}", err);
                ServiceError::BadRequest("Could not delete dataset".to_string())
            })?;

    if std::env::var("USE_ANALYTICS").unwrap_or("false".to_string()) != "true" {
        return Ok(dataset);
    }

    clickhouse_client
        .query("DELETE FROM default.dataset_events WHERE dataset_id = ?")
        .bind(id)
        .execute()
        .await
        .map_err(|err| {
            log::error!("Could not delete dataset events: {}", err);
            ServiceError::BadRequest("Could not delete dataset events".to_string())
        })?;

    clickhouse_client
        .query(
            "
        ALTER TABLE default.dataset_events
        DELETE WHERE dataset_id = ?;
        ",
        )
        .bind(id)
        .execute()
        .await
        .unwrap();

    clickhouse_client
        .query(
            "
        ALTER TABLE default.search_queries
        DELETE WHERE dataset_id = ?;
        ",
        )
        .bind(id)
        .execute()
        .await
        .unwrap();

    clickhouse_client
        .query(
            "
        ALTER TABLE default.search_cluster_memberships
        DELETE WHERE cluster_id IN (
            SELECT id FROM cluster_topics WHERE dataset_id = ?
        );
        ",
        )
        .bind(id)
        .execute()
        .await
        .unwrap();

    clickhouse_client
        .query(
            "
        ALTER TABLE default.cluster_topics
        DELETE WHERE dataset_id = ?;
        ",
        )
        .bind(id)
        .execute()
        .await
        .unwrap();

    Ok(dataset)
}

#[tracing::instrument(skip(pool))]
pub async fn update_dataset_query(
    id: uuid::Uuid,
    name: String,
    server_configuration: DatasetConfiguration,
    new_tracking_id: Option<String>,
    pool: web::Data<Pool>,
) -> Result<Dataset, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let configuration = server_configuration.clone().to_json();

    let new_dataset: Dataset = diesel::update(
        datasets_columns::datasets
            .filter(datasets_columns::id.eq(id))
            .filter(datasets_columns::deleted.eq(0)),
    )
    .set((
        new_tracking_id.map(|id| datasets_columns::tracking_id.eq(id)),
        datasets_columns::name.eq(name),
        datasets_columns::updated_at.eq(diesel::dsl::now),
        datasets_columns::server_configuration.eq(configuration),
    ))
    .get_result(&mut conn)
    .await
    .map_err(|e: DBError| {
        match e {
        DBError::DatabaseError(db_error, _) => match db_error {
            DatabaseErrorKind::UniqueViolation => {
                ServiceError::BadRequest("Could not update tracking_id because a dataset with the same tracking_id already exists in the organization".to_string())
            }
            _ => ServiceError::BadRequest("Failed to update dataset".to_string())
        }
        _ => {
            ServiceError::BadRequest("Failed to update dataset".to_string())
        }
    }
    })?;

    Ok(new_dataset)
}

#[tracing::instrument(skip(pool))]
pub async fn get_datasets_by_organization_id(
    org_id: web::Path<uuid::Uuid>,
    pagination: GetDatasetsPagination,
    pool: web::Data<Pool>,
) -> Result<Vec<DatasetAndUsage>, ServiceError> {
    use crate::data::schema::dataset_usage_counts::dsl as dataset_usage_counts_columns;
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let get_datasets_query = datasets_columns::datasets
        .inner_join(dataset_usage_counts_columns::dataset_usage_counts)
        .filter(datasets_columns::deleted.eq(0))
        .filter(datasets_columns::organization_id.eq(org_id.into_inner()))
        .order(datasets_columns::created_at.desc())
        .select((Dataset::as_select(), DatasetUsageCount::as_select()))
        .into_boxed();

    let dataset_and_usages = match pagination.limit {
        Some(limit) => get_datasets_query
            .offset(pagination.offset.unwrap_or(0))
            .limit(limit)
            .load::<(Dataset, DatasetUsageCount)>(&mut conn)
            .await
            .map_err(|_| ServiceError::NotFound("Could not find organization".to_string()))?,
        None => get_datasets_query
            .load::<(Dataset, DatasetUsageCount)>(&mut conn)
            .await
            .map_err(|_| ServiceError::NotFound("Could not find organization".to_string()))?,
    };

    let dataset_and_usages = dataset_and_usages
        .into_iter()
        .map(|(dataset, usage_count)| DatasetAndUsage::from_components(dataset.into(), usage_count))
        .collect::<Vec<DatasetAndUsage>>();

    Ok(dataset_and_usages)
}

pub async fn get_dataset_usage_query(
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<DatasetUsageCount, ServiceError> {
    use crate::data::schema::dataset_usage_counts::dsl as dataset_usage_counts_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let dataset_usage = dataset_usage_counts_columns::dataset_usage_counts
        .filter(dataset_usage_counts_columns::dataset_id.eq(dataset_id))
        .first::<DatasetUsageCount>(&mut conn)
        .await
        .map_err(|_| ServiceError::NotFound("Could not find dataset".to_string()))?;

    Ok(dataset_usage)
}

pub async fn get_tags_in_dataset_query(
    dataset_id: uuid::Uuid,
    page: i64,
    page_size: i64,
    pool: web::Data<Pool>,
) -> Result<(Vec<TagsWithCount>, i64), ServiceError> {
    use crate::data::schema::chunk_metadata_tags::dsl as chunk_metadata_tags_columns;
    use crate::data::schema::dataset_tags::dsl as dataset_tags_columns;

    let mut conn = pool.get().await.unwrap();

    let items = dataset_tags_columns::dataset_tags
        .inner_join(chunk_metadata_tags_columns::chunk_metadata_tags)
        .group_by(dataset_tags_columns::tag)
        .select((
            dataset_tags_columns::tag,
            count(chunk_metadata_tags_columns::chunk_metadata_id),
        ))
        .order_by(count(chunk_metadata_tags_columns::chunk_metadata_id).desc())
        .filter(dataset_tags_columns::dataset_id.eq(dataset_id))
        .limit(page_size)
        .offset((page - 1) * page_size)
        .load(&mut conn)
        .await
        .map_err(|err| {
            ServiceError::BadRequest(format!("Failed to get items with tags {}", err))
        })?;

    let total_count = dataset_tags_columns::dataset_tags
        .select(count(dataset_tags_columns::tag))
        .filter(dataset_tags_columns::dataset_id.eq(dataset_id))
        .first::<i64>(&mut conn)
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Failed to get count of tags {}", err)))?;

    Ok((items, total_count))
}

pub async fn scroll_dataset_ids_query(
    offset: uuid::Uuid,
    limit: i64,
    pool: web::Data<Pool>,
) -> Result<Option<Vec<uuid::Uuid>>, ServiceError> {
    use crate::data::schema::datasets::dsl as datasets_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let datasets = datasets_columns::datasets
        .select(datasets_columns::id)
        .filter(datasets_columns::id.gt(offset))
        .order_by(datasets_columns::id)
        .limit(limit)
        .load::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|_| ServiceError::NotFound("Failed to get datasets".to_string()))?;

    if datasets.is_empty() {
        return Ok(None);
    }
    Ok(Some(datasets))
}

pub async fn scroll_dataset_ids_for_dictionary_query(
    offset: uuid::Uuid,
    limit: i64,
    pool: web::Data<Pool>,
) -> Result<Option<Vec<uuid::Uuid>>, ServiceError> {
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::dataset_words_last_processed::dsl as last_processed_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let dataset_ids = chunk_metadata_columns::chunk_metadata
        .left_join(
            last_processed_columns::dataset_words_last_processed
                .on(last_processed_columns::dataset_id.eq(chunk_metadata_columns::dataset_id)),
        )
        .filter(
            (chunk_metadata_columns::updated_at
                .nullable()
                .gt(last_processed_columns::last_processed))
            .or(last_processed_columns::last_processed.is_null()),
        )
        .filter(chunk_metadata_columns::dataset_id.gt(offset))
        .order_by(chunk_metadata_columns::dataset_id)
        .limit(limit)
        .group_by(chunk_metadata_columns::dataset_id)
        .select(chunk_metadata_columns::dataset_id)
        .load::<uuid::Uuid>(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::InternalServerError(
                "Failed to scroll dataset ids for dictionary".to_string(),
            )
        })?;

    if dataset_ids.is_empty() {
        return Ok(None);
    }
    Ok(Some(dataset_ids))
}

pub async fn add_words_to_dataset(
    words: Vec<WordInDataset>,
    counts: Vec<i32>,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::words_datasets::dsl as words_datasets;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    conn.transaction(|mut conn| {
        Box::pin(async move {
            diesel::insert_into(words_datasets::words_datasets)
                .values(
                    &words
                        .into_iter()
                        .zip(counts)
                        .map(|(w, count)| WordDataset::from_details(w.id, dataset_id, count))
                        .collect_vec(),
                )
                .on_conflict((words_datasets::dataset_id, words_datasets::word_id))
                .do_update()
                .set(
                    words_datasets::count
                        .eq(words_datasets::count.add(excluded(words_datasets::count))),
                )
                .execute(&mut conn)
                .await
                .map_err(|_| {
                    ServiceError::InternalServerError(
                        "Failed to insert words in dataset".to_string(),
                    )
                })
        })
    })
    .await?;

    Ok(())
}

pub async fn get_words_from_dataset(
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<Vec<(String, i32)>, ServiceError> {
    use crate::data::schema::words_datasets::dsl as words_datasets;
    use crate::data::schema::words_in_datasets::dsl as words_in_datasets;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let words = words_datasets::words_datasets
        .inner_join(
            words_in_datasets::words_in_datasets
                .on(words_in_datasets::id.eq(words_datasets::word_id)),
        )
        .select((words_in_datasets::word, words_datasets::count))
        .filter(words_datasets::dataset_id.eq(dataset_id))
        .load::<(String, i32)>(&mut conn)
        .await
        .map_err(|_| {
            ServiceError::InternalServerError("Failed to get words from dataset".to_string())
        })?;

    Ok(words)
}

pub async fn update_dataset_last_processed_query(
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    use crate::data::schema::dataset_words_last_processed::dsl as last_processed_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    conn.transaction(|mut conn| {
        Box::pin(async move {
            diesel::insert_into(last_processed_columns::dataset_words_last_processed)
                .values(&DatasetWordsLastProcessed::from_details(dataset_id))
                .on_conflict(last_processed_columns::dataset_id)
                .do_update()
                .set(last_processed_columns::last_processed.eq(&chrono::Utc::now().naive_local()))
                .execute(&mut conn)
                .await
                .map_err(|_| {
                    ServiceError::InternalServerError(
                        "Failed to update last processed time of dataset".to_string(),
                    )
                })
        })
    })
    .await?;

    Ok(())
}
