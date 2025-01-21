use super::{
    auth_handler::{AdminOnly, LoggedUser},
    chunk_handler::{ChunkFilter, SearchChunksReqPayload},
};
use crate::operators::chunk_operator::get_metadata_from_tracking_ids_query;
use crate::{
    data::models::{
        escape_quotes, ChunkGroup, ChunkGroupAndFileId, ChunkGroupBookmark, ChunkMetadata,
        ChunkMetadataStringTagSet, DatasetAndOrgWithSubAndPlan, DatasetConfiguration,
        HighlightOptions, MultiQuery, Pool, QueryTypes, RecommendType,
        RecommendationEventClickhouse, RecommendationStrategy, RedisPool, ScoreChunk,
        ScoreChunkDTO, SearchMethod, SearchQueryEventClickhouse, SortOptions, TypoOptions,
        UnifiedId,
    },
    errors::ServiceError,
    middleware::api_version::APIVersion,
    operators::{
        chunk_operator::get_metadata_from_tracking_id_query,
        clickhouse_operator::{get_latency_from_header, ClickHouseEvent, EventQueue},
        group_operator::*,
        qdrant_operator::{
            add_bookmark_to_qdrant_query, recommend_qdrant_groups_query,
            remove_bookmark_from_qdrant_query,
        },
        search_operator::{
            get_metadata_from_groups, hybrid_search_over_groups, parse_query, search_groups_query,
            search_hybrid_groups, search_over_groups_query, GroupScoreChunk, ParsedQuery,
            ParsedQueryTypes, SearchOverGroupsQueryResult, SearchOverGroupsResults,
        },
    },
};
use actix_web::{web, HttpResponse};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use simple_server_timing_header::Timer;
use utoipa::{IntoParams, ToSchema};

pub async fn dataset_owns_group(
    unified_group_id: UnifiedId,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkGroupAndFileId, ServiceError> {
    let group = match unified_group_id {
        UnifiedId::TrieveUuid(group_id) => {
            get_group_by_id_query(group_id, dataset_id, pool.clone()).await?
        }
        UnifiedId::TrackingId(tracking_id) => {
            get_group_from_tracking_id_query(tracking_id, dataset_id, pool.clone()).await?
        }
    };

    if group.dataset_id != dataset_id {
        return Err(ServiceError::Forbidden);
    }

    Ok(group)
}

#[derive(Deserialize, Serialize, Debug, ToSchema, Clone)]
#[schema(title = "single", example = json!({
    "name": "Versions of Oversized T-Shirt",
    "description": "All versions and colorways of the oversized t-shirt",
    "tracking_id": "SNOVERSIZEDTSHIRT",
    "tag_set": ["tshirt", "oversized", "clothing"],
    "metadata": {
        "color": "black",
        "size": "large"
    },
    "upsert_by_tracking_id": false
}))]
pub struct CreateSingleChunkGroupReqPayload {
    /// Name to assign to the chunk_group. Does not need to be unique.
    pub name: Option<String>,
    /// Description to assign to the chunk_group. Convenience field for you to avoid having to remember what the group is for.
    pub description: Option<String>,
    /// Optional tracking id to assign to the chunk_group. This is a unique identifier for the chunk_group.
    pub tracking_id: Option<String>,
    /// Optional metadata to assign to the chunk_group. This is a JSON object that can store any additional information you want to associate with the chunks inside of the chunk_group.
    pub metadata: Option<serde_json::Value>,
    /// Optional tags to assign to the chunk_group. This is a list of strings that can be used to categorize the chunks inside the chunk_group.
    pub tag_set: Option<Vec<String>>,
    /// Upsert when a chunk_group with the same tracking_id exists. By default this is false, and the request will fail if a chunk_group with the same tracking_id exists. If this is true, the chunk_group will be updated if a chunk_group with the same tracking_id exists.
    pub upsert_by_tracking_id: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(title = "batch", example = json!([{
    "name": "Versions of Oversized T-Shirt",
    "description": "All versions and colorways of the oversized t-shirt",
    "tracking_id": "SNOVERSIZEDTSHIRT",
    "tag_set": ["tshirt", "oversized", "clothing"],
    "metadata": {
        "foo": "bar"
    },
    "upsert_by_tracking_id": false
},{
    "name": "Versions of Slim-Fit T-Shirt",
    "description": "All versions and colorways of the slim-fit t-shirt",
    "tracking_id": "SNSLIMFITTSHIRT",
    "tag_set": ["tshirt", "slim", "clothing"],
    "metadata": {
        "foo": "bar"
    },
    "upsert_by_tracking_id": false
}]))]
pub struct CreateBatchChunkGroupReqPayload(pub Vec<CreateSingleChunkGroupReqPayload>);

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(untagged)]
pub enum CreateChunkGroupReqPayloadEnum {
    Single(CreateSingleChunkGroupReqPayload),
    Batch(CreateBatchChunkGroupReqPayload),
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!([{
    "name": "Versions of Oversized T-Shirt",
    "description": "All versions and colorways of the oversized t-shirt",
    "tracking_id": "SNOVERSIZEDTSHIRT",
    "tag_set": ["tshirt", "oversized", "clothing"],
    "metadata": {
        "foo": "bar"
    },
    "created_at": "2021-01-01 00:00:00.000",
    "updated_at": "2021-01-01 00:00:00.000",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
},{
    "name": "Versions of Slim-Fit T-Shirt",
    "description": "All versions and colorways of the slim-fit t-shirt",
    "tracking_id": "SNSLIMFITTSHIRT",
    "tag_set": ["tshirt", "slim", "clothing"],
    "metadata": {
        "foo": "bar"
    },
    "created_at": "2021-01-01 00:00:00.000",
    "updated_at": "2021-01-01 00:00:00.000",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
}]))]
pub struct ChunkGroups(pub Vec<ChunkGroup>);

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(untagged)]
pub enum CreateChunkGroupResponseEnum {
    Single(ChunkGroup),
    Batch(ChunkGroups),
}

impl From<ChunkGroup> for CreateChunkGroupResponseEnum {
    fn from(group: ChunkGroup) -> Self {
        Self::Single(group)
    }
}

impl From<Vec<ChunkGroup>> for CreateChunkGroupResponseEnum {
    fn from(groups: Vec<ChunkGroup>) -> Self {
        Self::Batch(ChunkGroups(groups))
    }
}

/// Create or Upsert Group or Groups
///
/// Create new chunk_group(s). This is a way to group chunks together. If you try to create a chunk_group with the same tracking_id as an existing chunk_group, this operation will fail. Only 1000 chunk groups can be created at a time. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/chunk_group",
    context_path = "/api",
    tag = "Chunk Group",
    request_body(content = CreateChunkGroupReqPayloadEnum, description = "JSON request payload to cretea a chunk_group(s)", content_type = "application/json"),
    responses(
        (status = 200, description = "Returns the created chunk_group if a single chunk_group was specified or an array of all chunk_groups which were created", body = CreateChunkGroupResponseEnum),
        (status = 413, description = "Service error indicating more 1000 chunk groups are trying to be created at once", body = ErrorResponseBody),
        (status = 400, description = "Service error relating to creating the chunk_group(s)", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn create_chunk_group(
    create_group_data: web::Json<CreateChunkGroupReqPayloadEnum>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let payloads = match create_group_data.into_inner() {
        CreateChunkGroupReqPayloadEnum::Single(single) => vec![single],
        CreateChunkGroupReqPayloadEnum::Batch(batch) => batch.0,
    };

    if payloads.len() > 1000 {
        return Err(ServiceError::PayloadTooLarge(
            "Cannot create more than 1000 chunk groups at a time".into(),
        )
        .into());
    }

    let (upsert_payloads, non_upsert_payloads) = payloads
        .into_iter()
        .map(|payload| {
            let non_empty_tracking_id = payload.tracking_id.clone().filter(|id| !id.is_empty());
            CreateSingleChunkGroupReqPayload {
                tracking_id: non_empty_tracking_id,
                ..payload
            }
        })
        .partition::<Vec<_>, _>(|payload| payload.upsert_by_tracking_id.unwrap_or(false));

    let tracking_ids = upsert_payloads
        .iter()
        .filter_map(|payload| payload.tracking_id.clone())
        .collect::<Vec<String>>();
    if tracking_ids.len()
        != tracking_ids
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len()
    {
        return Err(ServiceError::BadRequest(
            "Cannot create multiple groups with the same tracking_id".into(),
        )
        .into());
    }

    let upsert_groups = upsert_payloads
        .into_iter()
        .map(|payload| {
            let group_tag_set = payload.tag_set.clone().map(|tag_set| {
                tag_set
                    .into_iter()
                    .map(|tag| Some(tag.clone()))
                    .collect::<Vec<Option<String>>>()
            });

            ChunkGroup::from_details(
                payload.name.clone(),
                payload.description.clone(),
                dataset_org_plan_sub.dataset.id,
                payload.tracking_id.clone(),
                payload.metadata.clone(),
                group_tag_set,
            )
        })
        .collect::<Vec<ChunkGroup>>();

    let non_upsert_groups = non_upsert_payloads
        .into_iter()
        .map(|payload| {
            let group_tag_set = payload.tag_set.clone().map(|tag_set| {
                tag_set
                    .into_iter()
                    .map(|tag| Some(tag.clone()))
                    .collect::<Vec<Option<String>>>()
            });

            ChunkGroup::from_details(
                payload.name.clone(),
                payload.description.clone(),
                dataset_org_plan_sub.dataset.id,
                payload.tracking_id.clone(),
                payload.metadata.clone(),
                group_tag_set,
            )
        })
        .collect::<Vec<ChunkGroup>>();

    let (upsert_results, non_upsert_results) = futures::future::join(
        create_groups_query(upsert_groups, true, pool.clone()),
        create_groups_query(non_upsert_groups, false, pool.clone()),
    )
    .await;
    let created_groups = upsert_results?
        .into_iter()
        .chain(non_upsert_results?.into_iter())
        .collect::<Vec<ChunkGroup>>();

    if created_groups.len() == 1 {
        match created_groups.get(0) {
            Some(group) => Ok(HttpResponse::Ok().json(group.clone())),
            None => Ok(HttpResponse::Ok().json(serde_json::json!({}))),
        }
    } else {
        Ok(HttpResponse::Ok().json(created_groups))
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GroupData {
    pub groups: Vec<ChunkGroupAndFileId>,
    pub total_pages: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatasetGroupQuery {
    pub dataset_id: uuid::Uuid,
    pub page: u64,
}

/// Get Groups for Dataset
///
/// Fetch the groups which belong to a dataset specified by its id.
#[utoipa::path(
    get,
    path = "/dataset/groups/{dataset_id}/{page}",
    context_path = "/api",
    tag = "Chunk Group",
    responses(
        (status = 200, description = "JSON body representing the groups created by the given dataset", body = GroupData),
        (status = 400, description = "Service error relating to getting the groups created by the given dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("dataset_id" = uuid::Uuid, description = "The id of the dataset to fetch groups for."),
        ("page" = i64, description = "The page of groups to fetch. Page is 1-indexed."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_groups_for_dataset(
    dataset_and_page: web::Path<DatasetGroupQuery>,
    _dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    _required_user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let (groups, group_count) =
        get_groups_for_dataset_query(dataset_and_page.page, dataset_and_page.dataset_id, pool)
            .await?;

    let pages = (group_count as u32).div_ceil(10);

    Ok(HttpResponse::Ok().json(GroupData {
        groups,
        total_pages: pages,
    }))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetGroupByTrackingIDData {
    pub tracking_id: String,
}

/// Get Group by Tracking ID
///
/// Fetch the group with the given tracking id.

#[utoipa::path(
    get,
    path = "/chunk_group/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "Chunk Group",
    responses(
        (status = 200, description = "JSON body representing the group with the given tracking id", body = ChunkGroupAndFileId),
        (status = 400, description = "Service error relating to getting the group with the given tracking id", body = ErrorResponseBody),
        (status = 404, description = "Group not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("tracking_id" = String, description = "The tracking id of the group to fetch."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
/// get_group_by_tracking_id

pub async fn get_group_by_tracking_id(
    data: web::Path<GetGroupByTrackingIDData>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let group = get_group_from_tracking_id_query(
        data.tracking_id.clone(),
        dataset_org_plan_sub.dataset.id,
        pool.clone(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(group))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetGroupData {
    pub group_id: Option<uuid::Uuid>,
    pub tracking_id: Option<String>,
}

/// Get Group
///
/// Fetch the group with the given id.
#[utoipa::path(
    get,
    path = "/chunk_group/{group_id}",
    context_path = "/api",
    tag = "Chunk Group",
    responses(
        (status = 200, description = "JSON body representing the group with the given tracking id", body = ChunkGroupAndFileId),
        (status = 400, description = "Service error relating to getting the group with the given tracking id", body = ErrorResponseBody),
        (status = 404, description = "Group not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("group_id" = Option<uuid::Uuid>, Path, description = "Id of the group you want to fetch."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_chunk_group(
    group_id: web::Path<uuid::Uuid>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let group = get_group_by_id_query(
        group_id.into_inner(),
        dataset_org_plan_sub.dataset.id,
        pool.clone(),
    )
    .await?;

    Ok(HttpResponse::Ok().json(group))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GetChunkGroupCountRequest {
    /// The Id of the group to get the count for, is not required if group_tracking_id is provided.
    pub group_id: Option<uuid::Uuid>,
    /// The tracking id of the group to get the count for, is not required if group_id is provided.
    pub group_tracking_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GetChunkGroupCountResponse {
    /// The Id of the group to get the count for.
    pub group_id: uuid::Uuid,
    /// The count of chunks in the given group.
    pub count: u64,
}

/// Count Chunks in a Group
///
/// Route to get the number of chunks that is in a group
#[utoipa::path(
    post,
    path = "/chunk_group/count",
    context_path = "/api",
    tag = "Chunk Group",
    request_body(content = GetChunkGroupCountRequest, description = "JSON request payload to add a chunk to a group (bookmark it)", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the group with the count", body = GetChunkGroupCountResponse),
        (status = 400, description = "Service error relating to getting the group with the given tracking id", body = ErrorResponseBody),
        (status = 404, description = "Group not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn count_group_chunks(
    body: web::Json<GetChunkGroupCountRequest>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, ServiceError> {
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());

    let group_id = if let Some(group_id) = body.group_id {
        group_id
    } else if let Some(group_tracking_id) = body.group_tracking_id.clone() {
        let groups = get_group_ids_from_tracking_ids_query(
            vec![group_tracking_id],
            dataset_org_plan_sub.dataset.id,
            pool.clone(),
        )
        .await?;

        let (group_id, _) = groups
            .get(0)
            .ok_or(ServiceError::NotFound("Group not found".into()))?;
        *group_id
    } else {
        return Err(ServiceError::BadRequest(
            "No group_id or tracking_id provided, please provide at least one".into(),
        ));
    };

    // check if group exists
    get_group_by_id_query(group_id, dataset_org_plan_sub.dataset.id, pool.clone()).await?;

    let group_size =
        get_group_size_query(group_id, dataset_org_plan_sub.dataset.id, dataset_config).await?;

    Ok(HttpResponse::Ok().json(GetChunkGroupCountResponse {
        group_id,
        count: group_size,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct UpdateGroupByTrackingIDReqPayload {
    /// Tracking Id of the chunk_group to update.
    pub tracking_id: String,
    /// Name to assign to the chunk_group. Does not need to be unique. If not provided, the name will not be updated.
    pub name: Option<String>,
    /// Description to assign to the chunk_group. Convenience field for you to avoid having to remember what the group is for. If not provided, the description will not be updated.
    pub description: Option<String>,
    /// Optional metadata to assign to the chunk_group. This is a JSON object that can store any additional information you want to associate with the chunks inside of the chunk_group.
    pub metadata: Option<serde_json::Value>,
    /// Optional tags to assign to the chunk_group. This is a list of strings that can be used to categorize the chunks inside the chunk_group.
    pub tag_set: Option<Vec<String>>,
}

/// Update Group by Tracking ID
///
/// Update a chunk_group with the given tracking id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    put,
    path = "/chunk_group/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "Chunk Group",
    request_body(content = UpdateGroupByTrackingIDReqPayload, description = "JSON request payload to update a chunkGroup", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunkGroup was updated"),
        (status = 400, description = "Service error relating to updating the chunkGroup", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("tracking_id" = uuid::Uuid, description = "Tracking id of the chunk_group to update"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[deprecated]

pub async fn update_group_by_tracking_id(
    data: web::Json<UpdateGroupByTrackingIDReqPayload>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let group = dataset_owns_group(
        UnifiedId::TrackingId(data.tracking_id.clone()),
        dataset_org_plan_sub.dataset.id,
        pool.clone(),
    )
    .await?;

    let group_tag_set = data.tag_set.clone().map(|tag_set| {
        tag_set
            .into_iter()
            .map(|tag| Some(tag.clone()))
            .collect::<Vec<Option<String>>>()
    });

    let new_group = ChunkGroup::from_details(
        data.name.clone(),
        data.description.clone(),
        dataset_org_plan_sub.dataset.id,
        Some(data.tracking_id.clone()),
        data.metadata.clone().or(group.metadata.clone()),
        group_tag_set,
    );

    update_chunk_group_query(new_group, pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeleteGroupByTrackingIDData {
    pub delete_chunks: Option<bool>,
}

/// Delete Group by Tracking ID
///
/// Delete a chunk_group with the given tracking id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    delete,
    path = "/chunk_group/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "Chunk Group",
    responses(
        (status = 204, description = "Confirmation that the chunkGroup was deleted"),
        (status = 400, description = "Service error relating to deleting the chunkGroup", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("tracking_id" = uuid::Uuid, description = "Tracking id of the chunk_group to delete"),
        ("delete_chunks" = bool, Query, description = "Delete the chunks within the group"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn delete_group_by_tracking_id(
    tracking_id: web::Path<String>,
    data: web::Query<DeleteGroupByTrackingIDData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let delete_group_pool = pool.clone();
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());
    let tracking_id = tracking_id.into_inner();

    let group = dataset_owns_group(
        UnifiedId::TrackingId(tracking_id),
        dataset_org_plan_sub.dataset.id,
        pool,
    )
    .await?;

    let deleted_at = chrono::Utc::now().naive_utc();

    delete_group_by_id_query(
        group.id,
        dataset_org_plan_sub.dataset,
        deleted_at,
        data.delete_chunks,
        delete_group_pool,
        dataset_config,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteGroupData {
    pub delete_chunks: Option<bool>,
}

/// Delete Group
///
/// This will delete a chunk_group. If you set delete_chunks to true, it will also delete the chunks within the group. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    delete,
    path = "/chunk_group/{group_id}",
    context_path = "/api",
    tag = "Chunk Group",
    responses(
        (status = 204, description = "Confirmation that the chunkGroup was deleted"),
        (status = 400, description = "Service error relating to deleting the chunkGroup", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("group_id" = Option<uuid::Uuid>, Path, description = "Id of the group you want to fetch."),
        ("delete_chunks" = bool, Query, description = "Delete the chunks within the group"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn delete_chunk_group(
    group_id: web::Path<uuid::Uuid>,
    data: web::Query<DeleteGroupData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let delete_group_pool = pool.clone();
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());

    let group_id = group_id.into_inner();

    dataset_owns_group(
        UnifiedId::TrieveUuid(group_id),
        dataset_org_plan_sub.dataset.id,
        pool.clone(),
    )
    .await?;

    delete_group_by_id_query(
        group_id,
        dataset_org_plan_sub.dataset,
        chrono::Utc::now().naive_utc(),
        data.delete_chunks,
        delete_group_pool,
        dataset_config,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct UpdateChunkGroupReqPayload {
    /// Id of the chunk_group to update.
    pub group_id: Option<uuid::Uuid>,
    /// Tracking Id of the chunk_group to update.
    pub tracking_id: Option<String>,
    /// Name to assign to the chunk_group. Does not need to be unique. If not provided, the name will not be updated.
    pub name: Option<String>,
    /// Description to assign to the chunk_group. Convenience field for you to avoid having to remember what the group is for. If not provided, the description will not be updated.
    pub description: Option<String>,
    /// Optional metadata to assign to the chunk_group. This is a JSON object that can store any additional information you want to associate with the chunks inside of the chunk_group.
    pub metadata: Option<serde_json::Value>,
    /// Optional tags to assign to the chunk_group. This is a list of strings that can be used to categorize the chunks inside the chunk_group.
    pub tag_set: Option<Vec<String>>,
    /// Flag to update the chunks in the group. If true, each chunk in the group will be updated
    /// by appending the group's tags to the chunk's tags. Default is false.
    pub update_chunks: Option<bool>,
}

/// Update Group
///
/// Update a chunk_group. One of group_id or tracking_id must be provided. If you try to change the tracking_id to one that already exists, this operation will fail. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    put,
    path = "/chunk_group",
    context_path = "/api",
    tag = "Chunk Group",
    request_body(content = UpdateChunkGroupReqPayload, description = "JSON request payload to update a chunkGroup", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunkGroup was updated"),
        (status = 400, description = "Service error relating to updating the chunkGroup", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn update_chunk_group(
    data: web::Json<UpdateChunkGroupReqPayload>,
    pool: web::Data<Pool>,
    redis_pool: web::Data<RedisPool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let name = data.name.clone();
    let description = data.description.clone();
    let group_id = data.group_id;
    let group_tag_set = data.tag_set.clone().map(|tag_set| {
        tag_set
            .into_iter()
            .map(|tag| Some(tag.clone()))
            .collect::<Vec<Option<String>>>()
    });

    let group = if let Some(group_id) = group_id {
        dataset_owns_group(
            UnifiedId::TrieveUuid(group_id),
            dataset_org_plan_sub.dataset.id,
            pool.clone(),
        )
        .await?
        .to_group()
    } else if let Some(tracking_id) = data.tracking_id.clone() {
        dataset_owns_group(
            UnifiedId::TrackingId(tracking_id),
            dataset_org_plan_sub.dataset.id,
            pool.clone(),
        )
        .await?
        .to_group()
    } else {
        return Err(ServiceError::BadRequest("No group id or tracking id provided".into()).into());
    };

    let new_chunk_group = ChunkGroup::from_details_with_id(
        group.id,
        name.unwrap_or(group.name.clone()),
        description.or(Some(group.description.clone())),
        dataset_org_plan_sub.dataset.id,
        data.tracking_id.clone(),
        data.metadata.clone(),
        group_tag_set.or(group.tag_set.clone()),
    );

    update_chunk_group_query(new_chunk_group.clone(), pool).await?;

    if data.update_chunks.unwrap_or(false) {
        soft_update_grouped_chunks_query(
            new_chunk_group,
            group,
            redis_pool,
            dataset_org_plan_sub.dataset.id,
        )
        .await?;
    }

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct AddChunkToGroupReqPayload {
    /// Id of the chunk to make a member of the group.
    pub chunk_id: Option<uuid::Uuid>,
    /// Tracking Id of the chunk to make a member of the group.
    pub chunk_tracking_id: Option<String>,
}

/// Add Chunk to Group
///
/// Route to add a chunk to a group. One of chunk_id or chunk_tracking_id must be provided. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/chunk_group/chunk/{group_id}",
    context_path = "/api",
    tag = "Chunk Group",
    request_body(content = AddChunkToGroupReqPayload, description = "JSON request payload to add a chunk to a group (bookmark it)", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunk was added to the group (bookmark'ed)."),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in.", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("group_id" = uuid, description = "Id of the group to add the chunk to as a bookmark"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn add_chunk_to_group(
    body: web::Json<AddChunkToGroupReqPayload>,
    group_id: web::Path<uuid::Uuid>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let group_id = group_id.into_inner();
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());

    dataset_owns_group(UnifiedId::TrieveUuid(group_id), dataset_id, pool.clone()).await?;

    let chunk_id = if body.chunk_id.is_some() {
        body.chunk_id.unwrap()
    } else if let Some(tracking_id) = body.chunk_tracking_id.clone() {
        let chunk =
            get_metadata_from_tracking_id_query(tracking_id, dataset_id, pool.clone()).await?;
        chunk.id
    } else {
        return Err(ServiceError::BadRequest("No chunk id or tracking id provided".into()).into());
    };

    let qdrant_point_id =
        create_chunk_bookmark_query(pool, ChunkGroupBookmark::from_details(group_id, chunk_id))
            .await?;

    add_bookmark_to_qdrant_query(qdrant_point_id, group_id, dataset_config).await?;

    Ok(HttpResponse::NoContent().finish())
}

/// Add Chunk to Group by Tracking ID
///
/// Route to add a chunk to a group by tracking id. One of chunk_id or chunk_tracking_id must be provided. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/chunk_group/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "Chunk Group",
    request_body(content = AddChunkToGroupReqPayload, description = "JSON request payload to add a chunk to a group via tracking_id", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunk was added to the group"),
        (status = 400, description = "Service error related to adding the chunk group by tracking_id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("tracking_id" = uuid, description = "Tracking id of the group to add the chunk to as a bookmark"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn add_chunk_to_group_by_tracking_id(
    data: web::Json<AddChunkToGroupReqPayload>,
    tracking_id: web::Path<String>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());

    let group = dataset_owns_group(
        UnifiedId::TrackingId(tracking_id.into_inner()),
        dataset_id,
        pool.clone(),
    )
    .await?;
    let group_id = group.id;

    let chunk_id = if data.chunk_id.is_some() {
        data.chunk_id.unwrap()
    } else if let Some(tracking_id) = data.chunk_tracking_id.clone() {
        let chunk =
            get_metadata_from_tracking_id_query(tracking_id, dataset_id, pool.clone()).await?;
        chunk.id
    } else {
        return Err(ServiceError::BadRequest("No chunk id or tracking id provided".into()).into());
    };

    let qdrant_point_id =
        create_chunk_bookmark_query(pool, ChunkGroupBookmark::from_details(group_id, chunk_id))
            .await?;

    add_bookmark_to_qdrant_query(qdrant_point_id, group_id, dataset_config).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[schema(title = "V1")]
pub struct GroupsBookmarkQueryResult {
    pub chunks: Vec<ChunkMetadataStringTagSet>,
    pub group: ChunkGroupAndFileId,
    pub total_pages: u64,
}
#[derive(Debug, Deserialize, Serialize, ToSchema)]
#[schema(title = "V2")]
pub struct GetChunksInGroupsResponseBody {
    pub chunks: Vec<ChunkMetadata>,
    pub group: ChunkGroupAndFileId,
    pub total_pages: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetChunksInGroupPathParams {
    pub group_id: uuid::Uuid,
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

impl From<GroupsBookmarkQueryResult> for GetChunksInGroupsResponseBody {
    fn from(data: GroupsBookmarkQueryResult) -> Self {
        Self {
            chunks: data.chunks.into_iter().map(|c| c.into()).collect(),
            group: data.group,
            total_pages: data.total_pages,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[serde(untagged)]
pub enum GetChunksInGroupResponse {
    V2(GetChunksInGroupsResponseBody),
    V1(GroupsBookmarkQueryResult),
}

/// Get Chunks in Group
///
/// Route to get all chunks for a group. The response is paginated, with each page containing 10 chunks. Page is 1-indexed.
#[utoipa::path(
    get,
    path = "/chunk_group/{group_id}/{page}",
    context_path = "/api",
    tag = "Chunk Group",
    responses(
        (status = 200, description = "Chunks present within the specified group", body = GetChunksInGroupResponse),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = ErrorResponseBody),
        (status = 404, description = "Group not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("group_id" = uuid::Uuid, Path, description = "Id of the group you want to fetch."),
        ("X-API-Version" = Option<APIVersion>, Header, description = "The version of the API to use for the request"),
        ("page" = Option<u64>, description = "The page of chunks to get from the group"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_chunks_in_group(
    group_data: web::Path<GetChunksInGroupPathParams>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    api_version: APIVersion,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let page = group_data.page.unwrap_or(1);
    let limit = group_data.limit.unwrap_or(10);
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let bookmarks = get_bookmarks_for_group_query(
        UnifiedId::TrieveUuid(group_data.group_id),
        page,
        Some(limit),
        dataset_id,
        pool.clone(),
    )
    .await?;

    let response = match api_version {
        APIVersion::V1 => GetChunksInGroupResponse::V1(bookmarks),
        APIVersion::V2 => GetChunksInGroupResponse::V2(bookmarks.into()),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetChunksInGroupByTrackingIdReqPayload {
    pub tracking_id: String,
    pub page: Option<u64>,
}

/// Get Chunks in Group by Tracking ID
///
/// Route to get all chunks for a group. The response is paginated, with each page containing 10 chunks. Support for custom page size is coming soon. Page is 1-indexed.
#[utoipa::path(
    get,
    path = "/chunk_group/tracking_id/{group_tracking_id}/{page}",
    context_path = "/api",
    tag = "Chunk Group",
    responses(
        (status = 200, description = "Chunks present within the specified group", body = GetChunksInGroupResponse),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = ErrorResponseBody),
        (status = 404, description = "Group not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("group_tracking_id" = String, description = "The id of the group to get the chunks from"),
        ("X-API-Version" = Option<APIVersion>, Header, description = "The version of the API to use for the request"),
        ("page" = u64, description = "The page of chunks to get from the group"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_chunks_in_group_by_tracking_id(
    path_data: web::Path<GetChunksInGroupByTrackingIdReqPayload>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    api_version: APIVersion,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let page = path_data.page.unwrap_or(1);
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let bookmarks = {
        get_bookmarks_for_group_query(
            UnifiedId::TrackingId(path_data.tracking_id.clone()),
            page,
            None,
            dataset_id,
            pool.clone(),
        )
        .await?
    };

    let response = match api_version {
        APIVersion::V1 => GetChunksInGroupResponse::V1(bookmarks),
        APIVersion::V2 => GetChunksInGroupResponse::V2(bookmarks.into()),
    };

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GetGroupsForChunksReqPayload {
    pub chunk_ids: Option<Vec<uuid::Uuid>>,
    pub chunk_tracking_ids: Option<Vec<String>>,
}

/// Get Groups for Chunks
///
/// Route to get the groups that a chunk is in.

#[utoipa::path(
    post,
    path = "/chunk_group/chunks",
    context_path = "/api",
    tag = "Chunk Group",
    request_body(content = GetGroupsForChunksReqPayload, description = "JSON request payload to get the groups that a chunk is in", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the groups that the chunk is in", body = Vec<GroupsForChunk>),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_groups_for_chunks(
    data: web::Json<GetGroupsForChunksReqPayload>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _required_user: LoggedUser,
) -> Result<HttpResponse, ServiceError> {
    let mut chunk_ids = vec![];

    if let Some(chunks) = data.chunk_ids.clone() {
        chunk_ids.extend(chunks);
    }

    if let Some(tracking_ids) = data.chunk_tracking_ids.clone() {
        chunk_ids.extend(
            get_metadata_from_tracking_ids_query(
                tracking_ids,
                dataset_org_plan_sub.dataset.id,
                pool.clone(),
            )
            .await?
            .into_iter()
            .map(|chunk| chunk.id)
            .collect_vec(),
        );
    }

    if chunk_ids.is_empty() {
        return Err(ServiceError::NotFound("No valid chunks found. Ensure you pass at least 1 valid tracking_id or chunk_id and that it is present in the dataset".to_string()));
    }

    let dataset_id = dataset_org_plan_sub.dataset.id;

    let groups = get_groups_for_bookmark_query(chunk_ids, dataset_id, pool).await?;

    Ok(HttpResponse::Ok().json(groups))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct RemoveChunkFromGroupReqPayload {
    /// Id of the chunk to remove from the group.
    pub chunk_id: uuid::Uuid,
}

/// Remove Chunk from Group
///
/// Route to remove a chunk from a group. Auth'ed user or api key must be an admin or owner of the dataset's organization to remove a chunk from a group.
#[utoipa::path(
    delete,
    path = "/chunk_group/chunk/{group_id}",
    context_path = "/api",
    tag = "Chunk Group",
    request_body(content = Option<RemoveChunkFromGroupReqPayload>, description = "JSON request payload to remove a chunk from a group", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunk was removed to the group"),
        (status = 400, description = "Service error relating to removing the chunk from the group", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("group_id" = uuid::Uuid, Path, description = "Id of the group you want to remove the chunk from."),
        ("chunk_id" = Option<uuid::Uuid>, Query, description = "Id of the chunk you want to remove from the group"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn remove_chunk_from_group(
    group_id: web::Path<uuid::Uuid>,
    body: Option<web::Json<RemoveChunkFromGroupReqPayload>>,
    query: Option<web::Query<RemoveChunkFromGroupReqPayload>>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let group_id = group_id.into_inner();

    let chunk_id = match (body, query) {
        (Some(body), None) => Ok(body.chunk_id),
        (None, Some(query)) => Ok(query.chunk_id),
        (None, None) => Err(ServiceError::BadRequest(
            "chunk_id not specified".to_string(),
        )),
        (Some(body), Some(_query)) => Ok(body.chunk_id),
    }?;

    let dataset_id = dataset_org_plan_sub.dataset.id;
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());

    dataset_owns_group(UnifiedId::TrieveUuid(group_id), dataset_id, pool.clone()).await?;

    let qdrant_point_id = delete_chunk_from_group_query(chunk_id, group_id, pool).await?;

    remove_bookmark_from_qdrant_query(qdrant_point_id, group_id, dataset_config).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct RecommendGroupsReqPayload {
    /// The ids of the groups to be used as positive examples for the recommendation. The groups in this array will be used to find similar groups.
    pub positive_group_ids: Option<Vec<uuid::Uuid>>,
    /// The ids of the groups to be used as negative examples for the recommendation. The groups in this array will be used to filter out similar groups.
    pub negative_group_ids: Option<Vec<uuid::Uuid>>,
    /// The ids of the groups to be used as positive examples for the recommendation. The groups in this array will be used to find similar groups.
    pub positive_group_tracking_ids: Option<Vec<String>>,
    /// The ids of the groups to be used as negative examples for the recommendation. The groups in this array will be used to filter out similar groups.
    pub negative_group_tracking_ids: Option<Vec<String>>,
    /// Strategy to use for recommendations, either "average_vector" or "best_score". The default is "average_vector". The "average_vector" strategy will construct a single average vector from the positive and negative samples then use it to perform a pseudo-search. The "best_score" strategy is more advanced and navigates the HNSW with a heuristic of picking edges where the point is closer to the positive samples than it is the negatives.
    pub strategy: Option<RecommendationStrategy>,
    /// The type of recommendation to make. This lets you choose whether to recommend based off of `semantic`, `fulltext` or `bm25` similarity. The default is `semantic`.
    pub recommend_type: Option<RecommendType>,
    /// Filters to apply to the chunks to be recommended. This is a JSON object which contains the filters to apply to the chunks to be recommended. The default is None.
    pub filters: Option<ChunkFilter>,
    /// The number of groups to return. This is the number of groups which will be returned in the response. The default is 10.
    pub limit: Option<u64>,
    /// The number of chunks to fetch for each group. This is the number of chunks which will be returned in the response for each group. The default is 3. If this is set to a large number, we recommend setting slim_chunks to true to avoid returning the content and chunk_html of the chunks so as to reduce latency due to content download and serialization.
    pub group_size: Option<u32>,
    /// Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typicall 10-50ms). Default is false.
    pub slim_chunks: Option<bool>,
    /// The user_id is the id of the user who is making the request. This is used to track user interactions with the rrecommendation results.
    pub user_id: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
#[schema(title = "V2")]
pub struct RecommendGroupsResponseBody {
    pub id: uuid::Uuid,
    pub results: Vec<SearchOverGroupsResults>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(untagged)]
pub enum RecommendGroupsResponse {
    #[schema(title = "V2")]
    V2(RecommendGroupsResponseBody),
    #[schema(title = "V1")]
    V1(GroupScoreChunk),
}

/// Get Recommended Groups
///
/// Route to get recommended groups. This route will return groups which are similar to the groups in the request body. You must provide at least one positive group id or group tracking id.
#[utoipa::path(
    post,
    path = "/chunk_group/recommend",
    context_path = "/api",
    tag = "Chunk Group",
    request_body(content = RecommendGroupsReqPayload, description = "JSON request payload to get recommendations of chunks similar to the chunks in the request", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the groups which are similar to the positive groups and dissimilar to the negative ones", body = RecommendGroupsResponse),
        (status = 400, description = "Service error relating to to getting similar chunks", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("X-API-Version" = Option<APIVersion>, Header, description = "The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise.")
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_recommended_groups(
    data: web::Json<RecommendGroupsReqPayload>,
    pool: web::Data<Pool>,
    event_queue: web::Data<EventQueue>,
    api_version: APIVersion,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let positive_group_ids = data.positive_group_ids.clone();
    let negative_group_ids = data.negative_group_ids.clone();
    let positive_tracking_ids = data.positive_group_tracking_ids.clone();
    let negative_tracking_ids = data.negative_group_tracking_ids.clone();

    if positive_group_ids.is_none() && positive_tracking_ids.is_none() {
        return Err(ServiceError::BadRequest(
            "You must provide at least one positive group id or group tracking id".into(),
        )
        .into());
    }

    let limit = data.limit.unwrap_or(10);
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let mut timer = Timer::new();

    let mut positive_qdrant_ids = vec![];

    if let Some(positive_group_ids) = positive_group_ids.clone() {
        positive_qdrant_ids.extend(
            get_point_ids_from_unified_group_ids(
                positive_group_ids
                    .into_iter()
                    .map(UnifiedId::TrieveUuid)
                    .collect(),
                dataset_id,
                pool.clone(),
            )
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not get positive qdrant_point_ids: {}",
                    err
                ))
            })?,
        );
    }

    if let Some(positive_tracking_ids) = positive_tracking_ids.clone() {
        positive_qdrant_ids.extend(
            get_point_ids_from_unified_group_ids(
                positive_tracking_ids
                    .into_iter()
                    .map(UnifiedId::TrackingId)
                    .collect(),
                dataset_id,
                pool.clone(),
            )
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not get positive qdrant_point_ids from tracking_ids: {}",
                    err
                ))
            })?,
        );
    }

    let mut negative_qdrant_ids = vec![];

    if let Some(negative_group_ids) = negative_group_ids.clone() {
        negative_qdrant_ids.extend(
            get_point_ids_from_unified_group_ids(
                negative_group_ids
                    .into_iter()
                    .map(UnifiedId::TrieveUuid)
                    .collect(),
                dataset_id,
                pool.clone(),
            )
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not get negative qdrant_point_ids: {}",
                    err
                ))
            })?,
        );
    }

    if let Some(negative_tracking_ids) = negative_tracking_ids.clone() {
        negative_qdrant_ids.extend(
            get_point_ids_from_unified_group_ids(
                negative_tracking_ids
                    .into_iter()
                    .map(UnifiedId::TrackingId)
                    .collect(),
                dataset_id,
                pool.clone(),
            )
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not get negative qdrant_point_ids from tracking_ids: {}",
                    err
                ))
            })?,
        );
    }

    if positive_qdrant_ids.is_empty() && negative_qdrant_ids.is_empty() {
        return Err(
            ServiceError::BadRequest("Positive chunk ids could not be found".to_string()).into(),
        );
    }

    timer.add("fetched ids from postgres");

    let recommended_groups_from_qdrant = recommend_qdrant_groups_query(
        positive_qdrant_ids,
        negative_qdrant_ids,
        data.strategy.clone(),
        data.recommend_type.clone(),
        data.filters.clone(),
        limit,
        data.group_size.unwrap_or(10),
        dataset_org_plan_sub.dataset.id,
        dataset_config.clone(),
        pool.clone(),
    )
    .await
    .map_err(|err| {
        ServiceError::BadRequest(format!("Could not get recommended groups: {}", err))
    })?;

    let group_qdrant_query_result = SearchOverGroupsQueryResult {
        search_results: recommended_groups_from_qdrant.clone(),
        corrected_query: None,
        total_chunk_pages: (recommended_groups_from_qdrant.len() as f64 / 10.0).ceil() as i64,
    };

    timer.add("recommend_qdrant_groups_query");

    let recommended_chunk_metadatas =
        get_metadata_from_groups(group_qdrant_query_result.clone(), data.slim_chunks, pool).await?;

    let recommended_chunk_metadatas = recommended_groups_from_qdrant
        .into_iter()
        .filter_map(|group| {
            recommended_chunk_metadatas
                .iter()
                .find(|metadata| metadata.group_id == group.group_id)
                .cloned()
        })
        .collect::<Vec<GroupScoreChunk>>();

    timer.add("fetched metadata from ids");

    let recommendation_id = uuid::Uuid::new_v4();
    if !dataset_config.DISABLE_ANALYTICS {
        let clickhouse_event = RecommendationEventClickhouse {
            id: recommendation_id,
            recommendation_type: String::from("group"),
            positive_ids: positive_group_ids
                .unwrap_or_default()
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
            negative_ids: negative_group_ids
                .unwrap_or_default()
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
            positive_tracking_ids: positive_tracking_ids.unwrap_or_default(),
            negative_tracking_ids: negative_tracking_ids.unwrap_or_default(),
            request_params: serde_json::to_string(&data.clone()).unwrap_or_default(),
            top_score: recommended_chunk_metadatas
                .first()
                .map(|x| x.metadata.first().map(|x| x.score).unwrap_or(0.0))
                .unwrap_or(0.0) as f32,
            results: recommended_chunk_metadatas
                .iter()
                .map(|x| serde_json::to_string(x).unwrap_or_default())
                .collect(),
            dataset_id: dataset_org_plan_sub.dataset.id,
            created_at: time::OffsetDateTime::now_utc(),
            user_id: data.user_id.clone().unwrap_or_default(),
        };

        event_queue
            .send(ClickHouseEvent::RecommendationEvent(clickhouse_event))
            .await;
    }

    timer.add("sent to clickhouse");

    if api_version == APIVersion::V1 {
        Ok(HttpResponse::Ok()
            .insert_header((Timer::header_key(), timer.header_value()))
            .json(recommended_chunk_metadatas))
    } else {
        let new_chunk_metadatas = RecommendGroupsResponseBody {
            id: recommendation_id,
            results: recommended_chunk_metadatas
                .iter()
                .map(|group| group.clone().into())
                .collect::<Vec<SearchOverGroupsResults>>(),
        };

        Ok(HttpResponse::Ok()
            .insert_header((Timer::header_key(), timer.header_value()))
            .json(new_chunk_metadatas))
    }
}

#[derive(Serialize, Clone, Debug, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct SearchWithinGroupReqPayload {
    /// The query is the search query. This can be any string. The query will be used to create an embedding vector and/or SPLADE vector which will be used to find the result set.
    pub query: QueryTypes,
    /// The page of chunks to fetch. Page is 1-indexed.
    pub page: Option<u64>,
    /// The page size is the number of chunks to fetch. This can be used to fetch more than 10 chunks at a time.
    pub page_size: Option<u64>,
    /// Get total page count for the query accounting for the applied filters. Defaults to false, but can be set to true when the latency penalty is acceptable (typically 50-200ms).
    pub get_total_pages: Option<bool>,
    /// Filters is a JSON object which can be used to filter chunks. The values on each key in the object will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Group specifies the group to search within. Results will only consist of chunks which are bookmarks within the specified group.
    pub group_id: Option<uuid::Uuid>,
    /// Group_tracking_id specifies the group to search within by tracking id. Results will only consist of chunks which are bookmarks within the specified group. If both group_id and group_tracking_id are provided, group_id will be used.
    pub group_tracking_id: Option<String>,
    /// Search_type can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using scores from a cross encoder model. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE.
    pub search_type: SearchMethod,
    /// Sort Options lets you specify different methods to rerank the chunks in the result set. If not specified, this defaults to the score of the chunks.
    pub sort_options: Option<SortOptions>,
    /// Highlight Options lets you specify different methods to highlight the chunks in the result set. If not specified, this defaults to the score of the chunks.
    pub highlight_options: Option<HighlightOptions>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold. This threshold applies before weight and bias modifications. If not specified, this defaults to 0.0.
    pub score_threshold: Option<f32>,
    /// Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typicall 10-50ms). Default is false.
    pub slim_chunks: Option<bool>,
    /// Set content_only to true to only returning the chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typically 10-50ms). Default is false.
    pub content_only: Option<bool>,
    /// If true, quoted and - prefixed words will be parsed from the queries and used as required and negated words respectively. Default is false.
    pub use_quote_negated_terms: Option<bool>,
    /// If true, stop words (specified in server/src/stop-words.txt in the git repo) will be removed. Queries that are entirely stop words will be preserved.
    pub remove_stop_words: Option<bool>,
    /// The user_id is the id of the user who is making the request. This is used to track user interactions with the search results.
    pub user_id: Option<String>,
    pub typo_options: Option<TypoOptions>,
}

impl From<SearchWithinGroupReqPayload> for SearchChunksReqPayload {
    fn from(search_within_group_data: SearchWithinGroupReqPayload) -> Self {
        Self {
            query: search_within_group_data.query,
            page: search_within_group_data.page,
            page_size: search_within_group_data.page_size,
            get_total_pages: search_within_group_data.get_total_pages,
            filters: search_within_group_data.filters,
            search_type: search_within_group_data.search_type,
            sort_options: search_within_group_data.sort_options,
            scoring_options: None,
            highlight_options: search_within_group_data.highlight_options,
            score_threshold: search_within_group_data.score_threshold,
            slim_chunks: search_within_group_data.slim_chunks,
            content_only: search_within_group_data.content_only,
            use_quote_negated_terms: search_within_group_data.use_quote_negated_terms,
            remove_stop_words: search_within_group_data.remove_stop_words,
            user_id: search_within_group_data.user_id,
            typo_options: search_within_group_data.typo_options,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(title = "V1")]
pub struct SearchWithinGroupResults {
    pub bookmarks: Vec<ScoreChunkDTO>,
    pub group: ChunkGroupAndFileId,
    pub corrected_query: Option<String>,
    pub total_pages: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(title = "V2")]
pub struct SearchWithinGroupResponseBody {
    pub id: uuid::Uuid,
    pub chunks: Vec<ScoreChunk>,
    pub corrected_query: Option<String>,
    pub total_pages: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum SearchGroupResponseTypes {
    #[schema(title = "V2")]
    V2(SearchWithinGroupResponseBody),
    #[schema(title = "V1")]
    V1(SearchWithinGroupResults),
}

impl SearchWithinGroupResults {
    fn into_v2(self, search_id: uuid::Uuid) -> SearchWithinGroupResponseBody {
        SearchWithinGroupResponseBody {
            id: search_id,
            chunks: self
                .bookmarks
                .into_iter()
                .map(|chunk| chunk.into())
                .collect(),
            corrected_query: self.corrected_query,
            total_pages: self.total_pages,
        }
    }
}

/// Search Within Group
///
/// This route allows you to search only within a group. This is useful for when you only want search results to contain chunks which are members of a specific group. If choosing hybrid search, the results will be re-ranked using scores from a cross encoder model.
#[utoipa::path(
    post,
    path = "/chunk_group/search",
    context_path = "/api",
    tag = "Chunk Group",
    request_body(content = SearchWithinGroupReqPayload, description = "JSON request payload to semantically search a group", content_type = "application/json"),
    responses(
        (status = 200, description = "Group chunks which are similar to the embedding vector of the search query", body = SearchGroupResponseTypes),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("X-API-Version" = Option<APIVersion>, Header, description = "The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise.")
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn search_within_group(
    data: web::Json<SearchWithinGroupReqPayload>,
    pool: web::Data<Pool>,
    event_queue: web::Data<EventQueue>,
    redis_pool: web::Data<RedisPool>,
    api_version: APIVersion,
    _required_user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());

    let data = data.into_inner();

    //search over the links as well
    let group_id = data.group_id;
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let search_pool = pool.clone();
    let mut timer = Timer::new();

    let group = {
        if let Some(group_id) = group_id {
            get_group_by_id_query(group_id, dataset_id, pool).await?
        } else if let Some(group_tracking_id) = data.group_tracking_id.clone() {
            get_group_from_tracking_id_query(group_tracking_id, dataset_id, pool).await?
        } else {
            return Err(ServiceError::BadRequest(
                "You must provide either group_id or group_tracking_id".into(),
            )
            .into());
        }
    };

    let parsed_query = match data.query.clone() {
        QueryTypes::Single(query) => ParsedQueryTypes::Single(
            parse_query(
                query.clone(),
                &dataset_org_plan_sub.dataset,
                data.use_quote_negated_terms,
                data.remove_stop_words,
            )
            .await?,
        ),
        QueryTypes::Multi(query) => {
            let parsed_queries = futures::future::join_all(query.into_iter().map(|multi_query| {
                let value = dataset_org_plan_sub.dataset.clone();
                async move {
                    let parsed_query = parse_query(
                        multi_query.query.clone(),
                        &value,
                        data.use_quote_negated_terms,
                        data.remove_stop_words,
                    )
                    .await?;
                    Ok((parsed_query, multi_query.weight))
                        as Result<(ParsedQuery, f32), ServiceError>
                }
            }))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
            ParsedQueryTypes::Multi(parsed_queries)
        }
    };

    let query = match &parsed_query {
        ParsedQueryTypes::Single(query) => query.query.clone(),
        ParsedQueryTypes::Multi(ref query) => serde_json::to_string(
            &query
                .clone()
                .into_iter()
                .map(Into::into)
                .collect::<Vec<MultiQuery>>(),
        )
        .unwrap_or_default(),
    };

    let result_chunks = match data.search_type {
        SearchMethod::Hybrid => {
            search_hybrid_groups(
                data.clone(),
                parsed_query.to_parsed_query()?,
                group,
                search_pool,
                redis_pool,
                dataset_org_plan_sub.dataset.clone(),
                &dataset_config,
                &mut timer,
            )
            .await?
        }
        _ => {
            search_groups_query(
                data.clone(),
                parsed_query,
                group,
                search_pool,
                redis_pool,
                dataset_org_plan_sub.dataset.clone(),
                &dataset_config,
                &mut timer,
            )
            .await?
        }
    };
    timer.add("search_chunks");

    let search_id = uuid::Uuid::new_v4();
    if !dataset_config.DISABLE_ANALYTICS {
        let clickhouse_event = SearchQueryEventClickhouse {
            id: search_id,
            search_type: String::from("search_within_groups"),
            query: query.clone(),
            request_params: serde_json::to_string(&data.clone()).unwrap_or_default(),
            latency: get_latency_from_header(timer.header_value()),
            top_score: result_chunks
                .bookmarks
                .first()
                .map(|x| x.score as f32)
                .unwrap_or(0.0),
            results: result_chunks
                .bookmarks
                .clone()
                .into_iter()
                .map(|x| {
                    let mut json = serde_json::to_value(&x).unwrap_or_default();
                    escape_quotes(&mut json);
                    json.to_string()
                })
                .collect(),
            dataset_id: dataset_org_plan_sub.dataset.id,
            created_at: time::OffsetDateTime::now_utc(),
            query_rating: String::from(""),
            user_id: data.user_id.clone().unwrap_or_default(),
        };

        event_queue
            .send(ClickHouseEvent::SearchQueryEvent(clickhouse_event))
            .await;
    }
    timer.add("send_to_clickhouse");

    if api_version == APIVersion::V1 {
        Ok(HttpResponse::Ok().json(result_chunks))
    } else {
        Ok(HttpResponse::Ok()
            .insert_header((Timer::header_key(), timer.header_value()))
            .insert_header(("X-TR-Query", query.replace("\n", "")))
            .json(result_chunks.into_v2(search_id)))
    }
}

#[derive(Serialize, Debug, Clone, ToSchema, Default)]
pub struct SearchOverGroupsReqPayload {
    /// Can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using scores from a cross encoder model. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE.
    pub search_type: SearchMethod,
    /// Query is the search query. This can be any string. The query will be used to create an embedding vector and/or SPLADE vector which will be used to find the result set.
    pub query: QueryTypes,
    /// Page of group results to fetch. Page is 1-indexed.
    pub page: Option<u64>,
    /// Page size is the number of group results to fetch. The default is 10.
    pub page_size: Option<u64>,
    /// Get total page count for the query accounting for the applied filters. Defaults to false, but can be set to true when the latency penalty is acceptable (typically 50-200ms).
    pub get_total_pages: Option<bool>,
    /// Filters is a JSON object which can be used to filter chunks. The values on each key in the object will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Highlight Options lets you specify different methods to highlight the chunks in the result set. If not specified, this defaults to the score of the chunks.
    pub highlight_options: Option<HighlightOptions>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold. This threshold applies before weight and bias modifications. If not specified, this defaults to 0.0.
    pub score_threshold: Option<f32>,
    /// Group_size is the number of chunks to fetch for each group. The default is 3. If a group has less than group_size chunks, all chunks will be returned. If this is set to a large number, we recommend setting slim_chunks to true to avoid returning the content and chunk_html of the chunks so as to lower the amount of time required for content download and serialization.
    pub group_size: Option<u64>,
    /// Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typicall 10-50ms). Default is false.
    pub slim_chunks: Option<bool>,
    /// If true, quoted and - prefixed words will be parsed from the queries and used as required and negated words respectively. Default is false.
    pub use_quote_negated_terms: Option<bool>,
    /// Sort Options lets you specify different methods to rerank the chunks in the result set. If not specified, this defaults to the score of the chunks.
    pub sort_options: Option<SortOptions>,
    /// If true, stop words (specified in server/src/stop-words.txt in the git repo) will be removed. Queries that are entirely stop words will be
    /// preserved.
    pub remove_stop_words: Option<bool>,
    /// The user_id is the id of the user who is making the request. This is used to track user interactions with the search results.
    pub user_id: Option<String>,
    pub typo_options: Option<TypoOptions>,
}

/// Search Over Groups
///
/// This route allows you to get groups as results instead of chunks. Each group returned will have the matching chunks sorted by similarity within the group. This is useful for when you want to get groups of chunks which are similar to the search query. If choosing hybrid search, the top chunk of each group will be re-ranked using scores from a cross encoder model. Compatible with semantic, fulltext, or hybrid search modes.
#[utoipa::path(
    post,
    path = "/chunk_group/group_oriented_search",
    context_path = "/api",
    tag = "Chunk Group",
    request_body(content = SearchOverGroupsReqPayload, description = "JSON request payload to semantically search over groups", content_type = "application/json"),
    responses(
        (status = 200, description = "Group chunks which are similar to the embedding vector of the search query", body = SearchOverGroupsResponseTypes),
        (status = 400, description = "Service error relating to searching over groups", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("X-API-Version" = Option<APIVersion>, Header, description = "The API version to use for this request. Defaults to V2 for orgs created after July 12, 2024 and V1 otherwise.")
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn search_over_groups(
    data: web::Json<SearchOverGroupsReqPayload>,
    pool: web::Data<Pool>,
    event_queue: web::Data<EventQueue>,
    redis_pool: web::Data<RedisPool>,
    api_version: APIVersion,
    _required_user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());

    let parsed_query = match data.query.clone() {
        QueryTypes::Single(query) => ParsedQueryTypes::Single(
            parse_query(
                query.clone(),
                &dataset_org_plan_sub.dataset,
                data.use_quote_negated_terms,
                data.remove_stop_words,
            )
            .await?,
        ),
        QueryTypes::Multi(query) => {
            let parsed_queries = futures::future::join_all(query.into_iter().map(|multi_query| {
                let value = dataset_org_plan_sub.dataset.clone();
                let data = data.clone();
                async move {
                    let parsed_query = parse_query(
                        multi_query.query.clone(),
                        &value,
                        data.use_quote_negated_terms,
                        data.remove_stop_words,
                    )
                    .await?;
                    Ok((parsed_query, multi_query.weight))
                        as Result<(ParsedQuery, f32), ServiceError>
                }
            }))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;
            ParsedQueryTypes::Multi(parsed_queries)
        }
    };

    let query = match &parsed_query {
        ParsedQueryTypes::Single(query) => query.query.clone(),
        ParsedQueryTypes::Multi(ref query) => serde_json::to_string(
            &query
                .clone()
                .into_iter()
                .map(Into::into)
                .collect::<Vec<MultiQuery>>(),
        )
        .unwrap_or_default(),
    };

    let mut timer = Timer::new();

    let result_chunks = match data.search_type {
        SearchMethod::Hybrid => {
            hybrid_search_over_groups(
                data.clone(),
                parsed_query.to_parsed_query()?,
                pool,
                redis_pool,
                dataset_org_plan_sub.dataset.clone(),
                &dataset_config,
                &mut timer,
            )
            .await?
        }
        _ => {
            search_over_groups_query(
                data.clone(),
                parsed_query,
                pool,
                redis_pool,
                dataset_org_plan_sub.dataset.clone(),
                &dataset_config,
                &mut timer,
            )
            .await?
        }
    };
    timer.add("search_chunks");

    let search_id = uuid::Uuid::new_v4();

    if !dataset_config.DISABLE_ANALYTICS {
        let clickhouse_event = SearchQueryEventClickhouse {
            id: search_id,
            search_type: String::from("search_over_groups"),
            query: query.clone(),
            request_params: serde_json::to_string(&data.clone()).unwrap_or_default(),
            latency: get_latency_from_header(timer.header_value()),
            top_score: result_chunks
                .group_chunks
                .first()
                .map(|x| x.metadata.first().map(|y| y.score as f32).unwrap_or(0.0))
                .unwrap_or(0.0),
            results: result_chunks
                .group_chunks
                .clone()
                .into_iter()
                .map(|x| {
                    let mut json = serde_json::to_value(&x).unwrap_or_default();
                    escape_quotes(&mut json);
                    json.to_string()
                })
                .collect(),
            dataset_id: dataset_org_plan_sub.dataset.id,
            created_at: time::OffsetDateTime::now_utc(),
            query_rating: String::from(""),
            user_id: data.user_id.clone().unwrap_or_default(),
        };

        event_queue
            .send(ClickHouseEvent::SearchQueryEvent(clickhouse_event))
            .await;
    }
    timer.add("send_to_clickhouse");

    if api_version == APIVersion::V1 {
        Ok(HttpResponse::Ok().json(result_chunks))
    } else {
        Ok(HttpResponse::Ok()
            .insert_header((Timer::header_key(), timer.header_value()))
            .insert_header(("X-TR-Query", query.replace("\n", "")))
            .json(result_chunks.into_v2(search_id)))
    }
}
