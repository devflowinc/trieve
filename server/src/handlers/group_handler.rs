use super::{
    auth_handler::{AdminOnly, LoggedUser},
    chunk_handler::{parse_query, ChunkFilter, SearchChunksReqPayload},
};
use crate::{
    data::models::{
        ChunkGroup, ChunkGroupAndFile, ChunkGroupBookmark, ChunkMetadata,
        DatasetAndOrgWithSubAndPlan, Pool, ScoreChunkDTO, ServerDatasetConfiguration, UnifiedId,
    },
    errors::ServiceError,
    operators::{
        chunk_operator::get_metadata_from_tracking_id_query,
        group_operator::*,
        qdrant_operator::{
            add_bookmark_to_qdrant_query, recommend_qdrant_groups_query,
            remove_bookmark_from_qdrant_query,
        },
        search_operator::{
            full_text_search_over_groups, get_metadata_from_groups, hybrid_search_over_groups,
            search_full_text_groups, search_hybrid_groups, search_semantic_groups,
            semantic_search_over_groups, GroupScoreChunk, SearchOverGroupsQueryResult,
        },
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use simple_server_timing_header::Timer;
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};

#[tracing::instrument(skip(pool))]
pub async fn dataset_owns_group(
    unified_group_id: UnifiedId,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkGroup, ServiceError> {
    let group = match unified_group_id {
        UnifiedId::TrieveUuid(group_id) => {
            get_group_by_id_query(group_id, dataset_id, pool).await?
        }
        UnifiedId::TrackingId(tracking_id) => {
            get_group_from_tracking_id_query(tracking_id, dataset_id, pool).await?
        }
    };

    if group.dataset_id != dataset_id {
        return Err(ServiceError::Forbidden);
    }

    Ok(group)
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct CreateChunkGroupReqPayload {
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

/// Create Chunk Group
///
/// Create a new chunk_group. This is a way to group chunks together. If you try to create a chunk_group with the same tracking_id as an existing chunk_group, this operation will fail. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/chunk_group",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = CreateChunkGroupReqPayload, description = "JSON request payload to cretea a chunkGroup", content_type = "application/json"),
    responses(
        (status = 200, description = "Returns the created chunkGroup", body = ChunkGroup),
        (status = 400, description = "Service error relating to creating the chunkGroup", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn create_chunk_group(
    body: web::Json<CreateChunkGroupReqPayload>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let name = body.name.clone();
    let description = body.description.clone();

    let group_tag_set = body.tag_set.clone().map(|tag_set| {
        tag_set
            .into_iter()
            .map(|tag| Some(tag.clone()))
            .collect::<Vec<Option<String>>>()
    });

    let group = ChunkGroup::from_details(
        name,
        description,
        dataset_org_plan_sub.dataset.id,
        body.tracking_id.clone(),
        body.metadata.clone(),
        group_tag_set,
    );
    {
        let group = group.clone();
        create_group_query(group, body.upsert_by_tracking_id.unwrap_or(false), pool).await?;
    }

    Ok(HttpResponse::Ok().json(group))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GroupData {
    pub groups: Vec<ChunkGroupAndFile>,
    pub total_pages: i32,
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
    tag = "chunk_group",
    responses(
        (status = 200, description = "JSON body representing the groups created by the given dataset", body = GroupData),
        (status = 400, description = "Service error relating to getting the groups created by the given dataset", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid::Uuid, description = "The id of the dataset to fetch groups for."),
        ("page" = i64, description = "The page of groups to fetch. Page is 1-indexed."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_groups_for_dataset(
    dataset_and_page: web::Path<DatasetGroupQuery>,
    _dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    _required_user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let groups =
        get_groups_for_dataset_query(dataset_and_page.page, dataset_and_page.dataset_id, pool)
            .await?;

    Ok(HttpResponse::Ok().json(GroupData {
        groups: groups.0,
        total_pages: groups.1.unwrap_or(1),
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
    tag = "chunk_group",
    responses(
        (status = 200, description = "JSON body representing the group with the given tracking id", body = ChunkGroup),
        (status = 400, description = "Service error relating to getting the group with the given tracking id", body = ErrorResponseBody),
        (status = 404, description = "Group not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("tracking_id" = String, description = "The tracking id of the group to fetch."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
/// get_group_by_tracking_id
#[tracing::instrument(skip(pool))]
pub async fn get_group_by_tracking_id(
    data: web::Path<GetGroupByTrackingIDData>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let group = get_group_from_tracking_id_query(
        data.tracking_id.clone(),
        dataset_org_plan_sub.dataset.id,
        pool,
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
    tag = "chunk_group",
    responses(
        (status = 200, description = "JSON body representing the group with the given tracking id", body = ChunkGroup),
        (status = 400, description = "Service error relating to getting the group with the given tracking id", body = ErrorResponseBody),
        (status = 404, description = "Group not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("group_id" = Option<uuid::Uuid>, Path, description = "Id of the group you want to fetch."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
/// get_group
#[tracing::instrument(skip(pool))]
pub async fn get_chunk_group(
    group_id: web::Path<uuid::Uuid>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let group =
        get_group_by_id_query(group_id.into_inner(), dataset_org_plan_sub.dataset.id, pool).await?;

    Ok(HttpResponse::Ok().json(group))
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
    tag = "chunk_group",
    request_body(content = UpdateGroupByTrackingIDReqPayload, description = "JSON request payload to update a chunkGroup", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunkGroup was updated"),
        (status = 400, description = "Service error relating to updating the chunkGroup", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("tracking_id" = uuid::Uuid, description = "Tracking id of the chunk_group to update"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[deprecated]
#[tracing::instrument(skip(pool))]
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
    tag = "chunk_group",
    responses(
        (status = 204, description = "Confirmation that the chunkGroup was deleted"),
        (status = 400, description = "Service error relating to deleting the chunkGroup", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("tracking_id" = uuid::Uuid, description = "Tracking id of the chunk_group to delete"),
        ("delete_chunks" = bool, Query, description = "Delete the chunks within the group"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn delete_group_by_tracking_id(
    tracking_id: web::Path<String>,
    data: web::Query<DeleteGroupByTrackingIDData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let delete_group_pool = pool.clone();
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );
    let tracking_id = tracking_id.into_inner();

    let group = dataset_owns_group(
        UnifiedId::TrackingId(tracking_id),
        dataset_org_plan_sub.dataset.id,
        pool,
    )
    .await?;

    delete_group_by_id_query(
        group.id,
        dataset_org_plan_sub.dataset,
        data.delete_chunks,
        delete_group_pool,
        server_dataset_config,
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
    tag = "chunk_group",
    responses(
        (status = 204, description = "Confirmation that the chunkGroup was deleted"),
        (status = 400, description = "Service error relating to deleting the chunkGroup", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("group_id" = Option<uuid::Uuid>, Path, description = "Id of the group you want to fetch."),
        ("delete_chunks" = bool, Query, description = "Delete the chunks within the group"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn delete_chunk_group(
    group_id: web::Path<uuid::Uuid>,
    data: web::Query<DeleteGroupData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let delete_group_pool = pool.clone();
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

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
        data.delete_chunks,
        delete_group_pool,
        server_dataset_config,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct UpdateChunkGroupData {
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
}

/// Update Group
///
/// Update a chunk_group. If you try to change the tracking_id to one that already exists, this operation will fail. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    put,
    path = "/chunk_group",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = UpdateChunkGroupData, description = "JSON request payload to update a chunkGroup", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunkGroup was updated"),
        (status = 400, description = "Service error relating to updating the chunkGroup", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn update_chunk_group(
    data: web::Json<UpdateChunkGroupData>,
    pool: web::Data<Pool>,
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
    } else if let Some(tracking_id) = data.tracking_id.clone() {
        dataset_owns_group(
            UnifiedId::TrackingId(tracking_id),
            dataset_org_plan_sub.dataset.id,
            pool.clone(),
        )
        .await?
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

    update_chunk_group_query(new_chunk_group, pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct AddChunkToGroupData {
    /// Id of the chunk to make a member of the group.
    pub chunk_id: Option<uuid::Uuid>,
    /// Tracking Id of the chunk to make a member of the group.
    pub tracking_id: Option<String>,
}

/// Add Chunk to Group
///
/// Route to add a chunk to a group. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/chunk_group/chunk/{group_id}",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = AddChunkToGroupData, description = "JSON request payload to add a chunk to a group (bookmark it)", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunk was added to the group (bookmark'ed)."),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in.", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("group_id" = uuid, description = "Id of the group to add the chunk to as a bookmark"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn add_chunk_to_group(
    body: web::Json<AddChunkToGroupData>,
    group_id: web::Path<uuid::Uuid>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let group_id = group_id.into_inner();
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    dataset_owns_group(UnifiedId::TrieveUuid(group_id), dataset_id, pool.clone()).await?;

    let id = if body.chunk_id.is_some() {
        body.chunk_id.unwrap()
    } else if let Some(tracking_id) = body.tracking_id.clone() {
        let chunk =
            get_metadata_from_tracking_id_query(tracking_id, dataset_id, pool.clone()).await?;
        chunk.id
    } else {
        return Err(ServiceError::BadRequest("No chunk id or tracking id provided".into()).into());
    };

    let qdrant_point_id =
        create_chunk_bookmark_query(pool, ChunkGroupBookmark::from_details(group_id, id)).await?;

    if let Some(qdrant_point_id) = qdrant_point_id {
        add_bookmark_to_qdrant_query(qdrant_point_id, group_id, server_dataset_config).await?;
    }

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct AddChunkToGroupByTrackingIdData {
    /// Id of the chunk to make a member of the group.
    pub chunk_id: uuid::Uuid,
}

/// Add Chunk to Group by Tracking ID
///
/// Route to add a chunk to a group by tracking id. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    post,
    path = "/chunk_group/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = AddChunkToGroupData, description = "JSON request payload to add a chunk to a group (bookmark it)", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunk was added to the group (bookmark'ed)."),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in.", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("tracking_id" = uuid, description = "Id of the group to add the chunk to as a bookmark"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
#[deprecated]
pub async fn add_chunk_to_group_by_tracking_id(
    body: web::Json<AddChunkToGroupByTrackingIdData>,
    tracking_id: web::Path<String>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let chunk_metadata_id = body.chunk_id;
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let group = dataset_owns_group(
        UnifiedId::TrackingId(tracking_id.into_inner()),
        dataset_id,
        pool.clone(),
    )
    .await?;
    let group_id = group.id;

    let qdrant_point_id = create_chunk_bookmark_query(
        pool,
        ChunkGroupBookmark::from_details(group_id, chunk_metadata_id),
    )
    .await?;

    if let Some(qdrant_point_id) = qdrant_point_id {
        add_bookmark_to_qdrant_query(qdrant_point_id, group_id, server_dataset_config).await?;
    }

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct BookmarkData {
    pub chunks: Vec<ChunkMetadata>,
    pub group: ChunkGroup,
    pub total_pages: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAllBookmarksData {
    pub group_id: uuid::Uuid,
    pub page: Option<u64>,
}

/// Get Chunks in Group
///
/// Route to get all chunks for a group. The response is paginated, with each page containing 10 chunks. Page is 1-indexed.
#[utoipa::path(
    get,
    path = "/chunk_group/{group_id}/{page}",
    context_path = "/api",
    tag = "chunk_group",
    responses(
        (status = 200, description = "Chunks present within the specified group", body = BookmarkData),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = ErrorResponseBody),
        (status = 404, description = "Group not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("group_id" = uuid::Uuid, Path, description = "Id of the group you want to fetch."),
        ("page" = Option<u64>, description = "The page of chunks to get from the group"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_chunks_in_group(
    group_data: web::Path<GetAllBookmarksData>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let page = group_data.page.unwrap_or(1);
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let bookmarks = get_bookmarks_for_group_query(
        UnifiedId::TrieveUuid(group_data.group_id),
        page,
        None,
        dataset_id,
        pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(BookmarkData {
        chunks: bookmarks.metadata,
        group: bookmarks.group,
        total_pages: bookmarks.total_pages,
    }))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAllBookmarksByTrackingIdData {
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
    tag = "chunk_group",
    responses(
        (status = 200, description = "Chunks present within the specified group", body = BookmarkData),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = ErrorResponseBody),
        (status = 404, description = "Group not found", body = ErrorResponseBody)
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("group_tracking_id" = String, description = "The id of the group to get the chunks from"),
        ("page" = u64, description = "The page of chunks to get from the group"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_chunks_in_group_by_tracking_id(
    path_data: web::Path<GetAllBookmarksByTrackingIdData>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
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
            pool,
        )
        .await?
    };

    Ok(HttpResponse::Ok().json(BookmarkData {
        chunks: bookmarks.metadata,
        group: bookmarks.group,
        total_pages: bookmarks.total_pages,
    }))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct GetGroupsForChunksData {
    pub chunk_ids: Vec<uuid::Uuid>,
}

/// Get Groups for Chunks
///
/// Route to get the groups that a chunk is in.

#[utoipa::path(
    post,
    path = "/chunk_group/chunks",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = GetGroupsForChunksData, description = "JSON request payload to get the groups that a chunk is in", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the groups that the chunk is in", body = Vec<BookmarkGroupResult>),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_groups_chunk_is_in(
    data: web::Json<GetGroupsForChunksData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _required_user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let chunk_ids = data.chunk_ids.clone();

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
    tag = "chunk_group",
    request_body(content = RemoveChunkFromGroupReqPayload, description = "JSON request payload to remove a chunk from a group", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunk was removed to the group"),
        (status = 400, description = "Service error relating to removing the chunk from the group", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("group_id" = Option<uuid::Uuid>, Path, description = "Id of the group you want to remove the chunk from."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn remove_chunk_from_group(
    group_id: web::Path<uuid::Uuid>,
    body: web::Json<RemoveChunkFromGroupReqPayload>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let group_id = group_id.into_inner();
    let chunk_id = body.chunk_id;
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    dataset_owns_group(UnifiedId::TrieveUuid(group_id), dataset_id, pool.clone()).await?;

    let qdrant_point_id = delete_chunk_from_group_query(chunk_id, group_id, pool).await?;

    if let Some(qdrant_point_id) = qdrant_point_id {
        remove_bookmark_from_qdrant_query(qdrant_point_id, group_id, server_dataset_config).await?;
    }

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateOffGroupData {
    pub group_id: uuid::Uuid,
    pub page: Option<u64>,
    pub query: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct RecommendGroupChunksRequest {
    /// The ids of the groups to be used as positive examples for the recommendation. The groups in this array will be used to find similar groups.
    pub positive_group_ids: Option<Vec<uuid::Uuid>>,
    /// The ids of the groups to be used as negative examples for the recommendation. The groups in this array will be used to filter out similar groups.
    pub negative_group_ids: Option<Vec<uuid::Uuid>>,
    /// The ids of the groups to be used as positive examples for the recommendation. The groups in this array will be used to find similar groups.
    pub positive_group_tracking_ids: Option<Vec<String>>,
    /// The ids of the groups to be used as negative examples for the recommendation. The groups in this array will be used to filter out similar groups.
    pub negative_group_tracking_ids: Option<Vec<String>>,
    /// Strategy to use for recommendations, either "average_vector" or "best_score". The default is "average_vector". The "average_vector" strategy will construct a single average vector from the positive and negative samples then use it to perform a pseudo-search. The "best_score" strategy is more advanced and navigates the HNSW with a heuristic of picking edges where the point is closer to the positive samples than it is the negatives.
    pub strategy: Option<String>,
    /// The type of recommendation to make. This lets you choose whether to recommend based off of `semantic` or `fulltext` similarity. The default is `semantic`.
    pub recommend_type: Option<String>,
    /// Filters to apply to the chunks to be recommended. This is a JSON object which contains the filters to apply to the chunks to be recommended. The default is None.
    pub filters: Option<ChunkFilter>,
    /// The number of groups to return. This is the number of groups which will be returned in the response. The default is 10.
    pub limit: Option<u64>,
    /// The number of chunks to fetch for each group. This is the number of chunks which will be returned in the response for each group. The default is 3. If this is set to a large number, we recommend setting slim_chunks to true to avoid returning the content and chunk_html of the chunks so as to reduce latency due to content download and serialization.
    pub group_size: Option<u32>,
    /// Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typicall 10-50ms). Default is false.
    pub slim_chunks: Option<bool>,
}

/// Get Recommended Groups
///
/// Route to get recommended groups. This route will return groups which are similar to the groups in the request body. You must provide at least one positive group id or group tracking id.
#[utoipa::path(
    post,
    path = "/chunk_group/recommend",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = RecommendGroupChunksRequest, description = "JSON request payload to get recommendations of chunks similar to the chunks in the request", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the groups which are similar to the positive groups and dissimilar to the negative ones", body = Vec<GroupScoreChunk>),
        (status = 400, description = "Service error relating to to getting similar chunks", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_recommended_groups(
    data: web::Json<RecommendGroupChunksRequest>,
    pool: web::Data<Pool>,
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
    let server_dataset_config =
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let mut timer = Timer::new();

    let mut positive_qdrant_ids = vec![];

    if let Some(positive_group_ids) = positive_group_ids {
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

    if let Some(positive_tracking_ids) = positive_tracking_ids {
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

    if let Some(negative_group_ids) = negative_group_ids {
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

    if let Some(negative_tracking_ids) = negative_tracking_ids {
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
        server_dataset_config,
        pool.clone(),
    )
    .await
    .map_err(|err| {
        ServiceError::BadRequest(format!("Could not get recommended groups: {}", err))
    })?;

    let group_qdrant_query_result = SearchOverGroupsQueryResult {
        search_results: recommended_groups_from_qdrant.clone(),
        total_chunk_pages: (recommended_groups_from_qdrant.len() as f64 / 10.0).ceil() as i64,
    };

    timer.add("recommend_qdrant_groups_query");

    let recommended_chunk_metadatas = get_metadata_from_groups(
        group_qdrant_query_result.clone(),
        Some(false),
        data.slim_chunks,
        pool,
    )
    .await?;

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

    Ok(HttpResponse::Ok()
        .insert_header((Timer::header_key(), timer.header_value()))
        .json(recommended_chunk_metadatas))
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct SearchWithinGroupData {
    /// The query is the search query. This can be any string. The query will be used to create an embedding vector and/or SPLADE vector which will be used to find the result set.
    pub query: String,
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
    /// Search_type can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using BAAI/bge-reranker-large. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE.
    pub search_type: String,
    /// Recency Bias lets you determine how much of an effect the recency of chunks will have on the search results. If not specified, this defaults to 0.0.
    pub recency_bias: Option<f32>,
    /// Set use_weights to true to use the weights of the chunks in the result set in order to sort them. If not specified, this defaults to true.
    pub use_weights: Option<bool>,
    /// Tag weights is a JSON object which can be used to boost the ranking of chunks with certain tags. This is useful for when you want to be able to bias towards chunks with a certain tag on the fly. The keys are the tag names and the values are the weights.
    pub tag_weights: Option<HashMap<String, f32>>,
    /// Set highlight_results to false for a slight latency improvement (1-10ms). If not specified, this defaults to true. This will add `<b><mark>` tags to the chunk_html of the chunks to highlight matching splits and return the highlights on each scored chunk in the response.
    pub highlight_results: Option<bool>,
    /// Set highlight_threshold to a lower or higher value to adjust the sensitivity of the highlights applied to the chunk html. If not specified, this defaults to 0.8. The range is 0.0 to 1.0.
    pub highlight_threshold: Option<f64>,
    /// Set highlight_delimiters to a list of strings to use as delimiters for highlighting. If not specified, this defaults to ["?", ",", ".", "!"]. These are the characters that will be used to split the chunk_html into splits for highlighting.
    pub highlight_delimiters: Option<Vec<String>>,
    /// Set highlight_max_length to control the maximum number of tokens (typically whitespace separated strings, but sometimes also word stems) which can be present within a single highlight. If not specified, this defaults to 8. This is useful to shorten large splits which may have low scores due to length compared to the query. Set to something very large like 100 to highlight entire splits.
    pub highlight_max_length: Option<u32>,
    /// Set highlight_max_num to control the maximum number of highlights per chunk. If not specified, this defaults to 3. It may be less than 3 if no snippets score above the highlight_threshold.
    pub highlight_max_num: Option<u32>,
    /// Set highlight_window to a number to control the amount of words that are returned around the matched phrases. If not specified, this defaults to 0. This is useful for when you want to show more context around the matched words. When specified, window/2 whitespace separated words are added before and after each highlight in the response's highlights array. If an extended highlight overlaps with another highlight, the overlapping words are only included once.
    pub highlight_window: Option<u32>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold.
    pub score_threshold: Option<f32>,
    /// Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typicall 10-50ms). Default is false.
    pub slim_chunks: Option<bool>,
}

impl From<SearchWithinGroupData> for SearchChunksReqPayload {
    fn from(search_within_group_data: SearchWithinGroupData) -> Self {
        Self {
            query: search_within_group_data.query,
            page: search_within_group_data.page,
            page_size: search_within_group_data.page_size,
            get_total_pages: search_within_group_data.get_total_pages,
            filters: search_within_group_data.filters,
            search_type: search_within_group_data.search_type,
            recency_bias: search_within_group_data.recency_bias,
            use_weights: search_within_group_data.use_weights,
            tag_weights: search_within_group_data.tag_weights,
            get_collisions: Some(false),
            highlight_results: search_within_group_data.highlight_results,
            highlight_threshold: search_within_group_data.highlight_threshold,
            highlight_delimiters: search_within_group_data.highlight_delimiters,
            highlight_max_length: search_within_group_data.highlight_max_length,
            highlight_max_num: search_within_group_data.highlight_max_num,
            highlight_window: search_within_group_data.highlight_window,
            score_threshold: search_within_group_data.score_threshold,
            slim_chunks: search_within_group_data.slim_chunks,
            content_only: Some(false),
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SearchWithinGroupResults {
    pub bookmarks: Vec<ScoreChunkDTO>,
    pub group: ChunkGroup,
    pub total_pages: i64,
}

/// Search Within Group
///
/// This route allows you to search only within a group. This is useful for when you only want search results to contain chunks which are members of a specific group. If choosing hybrid search, the results will be re-ranked using BAAI/bge-reranker-large.
#[utoipa::path(
    post,
    path = "/chunk_group/search",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = SearchWithinGroupData, description = "JSON request payload to semantically search a group", content_type = "application/json"),
    responses(
        (status = 200, description = "Group chunks which are similar to the embedding vector of the search query", body = SearchWithinGroupResults),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool))]
pub async fn search_within_group(
    data: web::Json<SearchWithinGroupData>,
    pool: web::Data<Pool>,
    _required_user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    //search over the links as well
    let group_id = data.group_id;
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let search_pool = pool.clone();

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

    let parsed_query = parse_query(data.query.clone());

    let result_chunks = match data.search_type.as_str() {
        "fulltext" => {
            if !server_dataset_config.FULLTEXT_ENABLED {
                return Err(ServiceError::BadRequest(
                    "Fulltext search is not enabled for this dataset".into(),
                )
                .into());
            }

            search_full_text_groups(
                data.clone(),
                parsed_query,
                group,
                search_pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
            )
            .await?
        }
        "hybrid" => {
            search_hybrid_groups(
                data.clone(),
                parsed_query,
                group,
                search_pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
            )
            .await?
        }
        _ => {
            search_semantic_groups(
                data.clone(),
                parsed_query,
                group,
                search_pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
            )
            .await?
        }
    };

    Ok(HttpResponse::Ok().json(result_chunks))
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct SearchOverGroupsData {
    /// Can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using BAAI/bge-reranker-large. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE.
    pub search_type: String,
    /// Query is the search query. This can be any string. The query will be used to create an embedding vector and/or SPLADE vector which will be used to find the result set.
    pub query: String,
    /// Page of group results to fetch. Page is 1-indexed.
    pub page: Option<u64>,
    /// Page size is the number of group results to fetch. The default is 10.
    pub page_size: Option<u32>,
    /// Get total page count for the query accounting for the applied filters. Defaults to false, but can be set to true when the latency penalty is acceptable (typically 50-200ms).
    pub get_total_pages: Option<bool>,
    /// Filters is a JSON object which can be used to filter chunks. The values on each key in the object will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Set get_collisions to true to get the collisions for each chunk. This will only apply if environment variable COLLISIONS_ENABLED is set to true.
    pub get_collisions: Option<bool>,
    /// Set highlight_results to false for a slight latency improvement (1-10ms). If not specified, this defaults to true. This will add `<b><mark>` tags to the chunk_html of the chunks to highlight matching splits and return the highlights on each scored chunk in the response.
    pub highlight_results: Option<bool>,
    /// Set highlight_threshold to a lower or higher value to adjust the sensitivity of the highlights applied to the chunk html. If not specified, this defaults to 0.8. The range is 0.0 to 1.0.
    pub highlight_threshold: Option<f64>,
    /// Set highlight_delimiters to a list of strings to use as delimiters for highlighting. If not specified, this defaults to ["?", ",", ".", "!"]. These are the characters that will be used to split the chunk_html into splits for highlighting.
    pub highlight_delimiters: Option<Vec<String>>,
    /// Set highlight_max_length to control the maximum number of tokens (typically whitespace separated strings, but sometimes also word stems) which can be present within a single highlight. If not specified, this defaults to 8. This is useful to shorten large splits which may have low scores due to length compared to the query. Set to something very large like 100 to highlight entire splits.
    pub highlight_max_length: Option<u32>,
    /// Set highlight_max_num to control the maximum number of highlights per chunk. If not specified, this defaults to 3. It may be less than 3 if no snippets score above the highlight_threshold.
    pub highlight_max_num: Option<u32>,
    /// Set highlight_window to a number to control the amount of words that are returned around the matched phrases. If not specified, this defaults to 0. This is useful for when you want to show more context around the matched words. When specified, window/2 whitespace separated words are added before and after each highlight in the response's highlights array. If an extended highlight overlaps with another highlight, the overlapping words are only included once.
    pub highlight_window: Option<u32>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold.
    pub score_threshold: Option<f32>,
    /// Group_size is the number of chunks to fetch for each group. The default is 3. If a group has less than group_size chunks, all chunks will be returned. If this is set to a large number, we recommend setting slim_chunks to true to avoid returning the content and chunk_html of the chunks so as to lower the amount of time required for content download and serialization.
    pub group_size: Option<u32>,
    /// Set slim_chunks to true to avoid returning the content and chunk_html of the chunks. This is useful for when you want to reduce amount of data over the wire for latency improvement (typicall 10-50ms). Default is false.
    pub slim_chunks: Option<bool>,
}

/// Search Over Groups
///
/// This route allows you to get groups as results instead of chunks. Each group returned will have the matching chunks sorted by similarity within the group. This is useful for when you want to get groups of chunks which are similar to the search query. If choosing hybrid search, the results will be re-ranked using BAAI/bge-reranker-large. Compatible with semantic, fulltext, or hybrid search modes.
#[utoipa::path(
    post,
    path = "/chunk_group/group_oriented_search",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = SearchOverGroupsData, description = "JSON request payload to semantically search over groups", content_type = "application/json"),
    responses(
        (status = 200, description = "Group chunks which are similar to the embedding vector of the search query", body = SearchOverGroupsResults),
        (status = 400, description = "Service error relating to searching over groups", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn search_over_groups(
    data: web::Json<SearchOverGroupsData>,
    pool: web::Data<Pool>,
    _required_user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    let parsed_query = parse_query(data.query.clone());

    let mut timer = Timer::new();

    let result_chunks = match data.search_type.as_str() {
        "fulltext" => {
            if !server_dataset_config.FULLTEXT_ENABLED {
                return Err(ServiceError::BadRequest(
                    "Fulltext search is not enabled for this dataset".into(),
                )
                .into());
            }

            full_text_search_over_groups(
                data.clone(),
                parsed_query,
                pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
                &mut timer,
            )
            .await?
        }
        "hybrid" => {
            hybrid_search_over_groups(
                data.clone(),
                parsed_query,
                pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
                &mut timer,
            )
            .await?
        }
        _ => {
            semantic_search_over_groups(
                data.clone(),
                parsed_query,
                pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
                &mut timer,
            )
            .await?
        }
    };

    Ok(HttpResponse::Ok()
        .insert_header((Timer::header_key(), timer.header_value()))
        .json(result_chunks))
}
