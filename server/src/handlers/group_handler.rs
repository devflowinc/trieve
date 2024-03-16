use super::{
    auth_handler::{AdminOnly, LoggedUser},
    chunk_handler::{parse_query, ChunkFilter, ScoreChunkDTO, SearchChunkData},
};
use crate::{
    data::models::{
        ChunkGroup, ChunkGroupAndFile, ChunkGroupBookmark, ChunkMetadataWithFileData,
        DatasetAndOrgWithSubAndPlan, GetReqParams, Pool, ServerDatasetConfiguration, UnifiedId,
    },
    errors::ServiceError,
    operators::{
        group_operator::*,
        qdrant_operator::{
            add_bookmark_to_qdrant_query, recommend_qdrant_groups_query,
            remove_bookmark_from_qdrant_query,
        },
        search_operator::{
            full_text_search_over_groups, get_metadata_from_groups, hybrid_search_over_groups,
            search_full_text_groups, search_hybrid_groups, search_semantic_groups,
            semantic_search_over_groups, SearchOverGroupsQueryResult,
        },
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[tracing::instrument(skip(pool))]
pub async fn dataset_owns_group(
    unified_group_id: UnifiedId,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkGroup, actix_web::Error> {
    let group = match unified_group_id {
        UnifiedId::TrieveUuid(group_id) => get_group_by_id_query(group_id, dataset_id, pool)
            .await
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?,
        UnifiedId::TrackingId(tracking_id) => {
            get_group_from_tracking_id_query(tracking_id, dataset_id, pool)
                .await
                .map_err(|err| ServiceError::BadRequest(err.message.into()))?
        }
    };

    if group.dataset_id != dataset_id {
        return Err(ServiceError::Forbidden.into());
    }

    Ok(group)
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct CreateChunkGroupData {
    /// Name to assign to the chunk_group. Does not need to be unique.
    pub name: String,
    /// Description to assign to the chunk_group. Convenience field for you to avoid having to remember what the group is for.
    pub description: String,
    /// Optional tracking id to assign to the chunk_group. This is a unique identifier for the chunk_group.
    pub tracking_id: Option<String>,
}

/// create_chunk_group
///
/// Create a new chunk_group.
#[utoipa::path(
    post,
    path = "/chunk_group",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = CreateChunkGroupData, description = "JSON request payload to cretea a chunkGroup", content_type = "application/json"),
    responses(
        (status = 200, description = "Returns the created chunkGroup", body = ChunkGroup),
        (status = 400, description = "Service error relating to creating the chunkGroup", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
        ("Cookie" = ["admin"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn create_chunk_group(
    body: web::Json<CreateChunkGroupData>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let name = body.name.clone();
    let description = body.description.clone();

    let group = ChunkGroup::from_details(
        name,
        description,
        dataset_org_plan_sub.dataset.id,
        body.tracking_id.clone(),
    );
    {
        let group = group.clone();
        create_group_query(group, pool).await?;
    }

    Ok(HttpResponse::Ok().json(group))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GroupData {
    pub groups: Vec<ChunkGroupAndFile>,
    pub total_pages: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatasetGroupQuery {
    pub dataset_id: uuid::Uuid,
    pub page: u64,
}

/// get_dataset_groups
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
        ("page" = i64, description = "The page of groups to fetch. Each page contains 10 groups. Support for custom page size is coming soon."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_specific_dataset_chunk_groups(
    dataset_and_page: web::Path<DatasetGroupQuery>,
    _dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    _required_user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let groups = get_groups_for_specific_dataset_query(
        dataset_and_page.page,
        dataset_and_page.dataset_id,
        pool,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(GroupData {
        groups: groups
            .iter()
            .map(|group| ChunkGroupAndFile {
                id: group.id,
                dataset_id: group.dataset_id,
                name: group.name.clone(),
                description: group.description.clone(),
                created_at: group.created_at,
                updated_at: group.updated_at,
                file_id: group.file_id,
                tracking_id: group.tracking_id.clone(),
            })
            .collect(),
        total_pages: groups
            .first()
            .map(|group| (group.group_count.unwrap_or(10) as f64 / 10.0).ceil() as i64)
            .unwrap_or(1),
    }))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetGroupByTrackingIDData {
    pub tracking_id: String,
}

#[utoipa::path(
    get,
    path = "/chunk_group/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "chunk_group",
    responses(
        (status = 200, description = "JSON body representing the group with the given tracking id", body = ChunkGroup),
        (status = 400, description = "Service error relating to getting the group with the given tracking id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("tracking_id" = String, description = "The tracking id of the group to fetch."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
#[deprecated]
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
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(group))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetGroupData {
    pub group_id: Option<uuid::Uuid>,
    pub tracking_id: Option<String>,
}

#[utoipa::path(
    get,
    path = "/chunk_group/{id}",
    context_path = "/api",
    tag = "chunk_group",
    responses(
        (status = 200, description = "JSON body representing the group with the given tracking id", body = ChunkGroup),
        (status = 400, description = "Service error relating to getting the group with the given tracking id", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("tracking_id" = String, description = "The tracking id of the group to fetch."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
/// get_group
#[tracing::instrument(skip(pool))]
pub async fn get_chunk_group(
    chunk_id: GetReqParams,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let group = match chunk_id.id {
        UnifiedId::TrieveUuid(group_id) => {
            get_group_by_id_query(group_id, dataset_org_plan_sub.dataset.id, pool)
                .await
                .map_err(|err| ServiceError::BadRequest(err.message.into()))?
        }
        UnifiedId::TrackingId(tracking_id) => {
            get_group_from_tracking_id_query(tracking_id, dataset_org_plan_sub.dataset.id, pool)
                .await
                .map_err(|err| ServiceError::BadRequest(err.message.into()))?
        }
    };

    Ok(HttpResponse::Ok().json(group))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct UpdateGroupByTrackingIDData {
    /// Tracking Id of the chunk_group to update.
    pub tracking_id: String,
    /// Name to assign to the chunk_group. Does not need to be unique. If not provided, the name will not be updated.
    pub name: Option<String>,
    /// Description to assign to the chunk_group. Convenience field for you to avoid having to remember what the group is for. If not provided, the description will not be updated.
    pub description: Option<String>,
}

#[utoipa::path(
    put,
    path = "/chunk_group/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = UpdateGroupByTrackingIDData, description = "JSON request payload to update a chunkGroup", content_type = "application/json"),
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
        ("Cookie" = ["admin"])
    )
)]
#[deprecated]
#[tracing::instrument(skip(pool))]
pub async fn update_group_by_tracking_id(
    data: web::Json<UpdateGroupByTrackingIDData>,
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

    update_chunk_group_query(
        group,
        data.name.clone(),
        data.description.clone(),
        dataset_org_plan_sub.dataset.id,
        pool,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteGroupByTrackingIDData {
    pub delete_chunks: Option<bool>,
}

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
    ),
    security(
        ("ApiKey" = ["admin"]),
        ("Cookie" = ["admin"])
    )
)]
#[deprecated]
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
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteGroupData {
    pub delete_chunks: Option<bool>,
}

/// delete_chunk_group
///
/// This will delete a chunk_group. This will not delete the chunks that are in the group. We will soon support deleting a chunk_group along with its member chunks.
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
        ("group_id" = uuid::Uuid, description = "Id of the chunk_group to delete"),
        ("delete_chunks" = bool, Query, description = "Delete the chunks within the group"),
    ),
    security(
        ("ApiKey" = ["admin"]),
        ("Cookie" = ["admin"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn delete_chunk_group(
    group_id: GetReqParams,
    data: web::Query<DeleteGroupData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let delete_group_pool = pool.clone();
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    match group_id.id {
        UnifiedId::TrieveUuid(group_id) => {
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
            .await
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
        }
        UnifiedId::TrackingId(tracking_id) => {
            let group = dataset_owns_group(
                UnifiedId::TrackingId(tracking_id),
                dataset_org_plan_sub.dataset.id,
                pool.clone(),
            )
            .await?;
            delete_group_by_id_query(
                group.id,
                dataset_org_plan_sub.dataset,
                data.delete_chunks,
                delete_group_pool,
                server_dataset_config,
            )
            .await
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
        }
    }

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
}

/// update_chunk_group
///
/// Update a chunk_group.
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
        ("Cookie" = ["admin"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn update_chunk_group(
    body: web::Json<UpdateChunkGroupData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let name = body.name.clone();
    let description = body.description.clone();
    let group_id = body.group_id;

    let group = if let Some(group_id) = group_id {
        dataset_owns_group(
            UnifiedId::TrieveUuid(group_id),
            dataset_org_plan_sub.dataset.id,
            pool.clone(),
        )
        .await?
    } else if let Some(tracking_id) = body.tracking_id.clone() {
        dataset_owns_group(
            UnifiedId::TrackingId(tracking_id),
            dataset_org_plan_sub.dataset.id,
            pool.clone(),
        )
        .await?
    } else {
        return Err(ServiceError::BadRequest("No group id or tracking id provided".into()).into());
    };

    update_chunk_group_query(
        group,
        name,
        description,
        dataset_org_plan_sub.dataset.id,
        pool,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct AddChunkToGroupData {
    /// Id of the chunk to make a member of the group. Think of this as "bookmark"ing a chunk.
    pub chunk_id: uuid::Uuid,
}

/// add_chunk_to_group
///
/// Route to add a chunk to a group
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
        ("Cookie" = ["admin"])
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
    let chunk_metadata_id = body.chunk_id;
    let group_id = group_id.into_inner();
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );

    dataset_owns_group(UnifiedId::TrieveUuid(group_id), dataset_id, pool.clone()).await?;

    let qdrant_point_id = create_chunk_bookmark_query(
        pool,
        ChunkGroupBookmark::from_details(group_id, chunk_metadata_id),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if let Some(qdrant_point_id) = qdrant_point_id {
        add_bookmark_to_qdrant_query(qdrant_point_id, group_id, server_dataset_config)
            .await
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    }

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct AddChunkToGroupByTrackingIdData {
    /// Id of the chunk to make a member of the group. Think of this as "bookmark"ing a chunk.
    pub chunk_id: uuid::Uuid,
}

/// add_chunk_to_group_by_tracking_id
///
/// Route to add a chunk to a group by tracking id. Think of a bookmark as a chunk which is a member of a group.
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
        ("Cookie" = ["admin"])
    )
)]
#[tracing::instrument(skip(pool))]
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
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if let Some(qdrant_point_id) = qdrant_point_id {
        add_bookmark_to_qdrant_query(qdrant_point_id, group_id, server_dataset_config)
            .await
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    }

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
pub struct BookmarkData {
    pub chunks: Vec<ChunkMetadataWithFileData>,
    pub group: ChunkGroup,
    pub total_pages: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAllBookmarksData {
    pub group_id: Option<uuid::Uuid>,
    pub tracking_id: Option<String>,
    pub page: Option<u64>,
}

/// get_chunks_in_group
///
/// Route to get all chunks for a group. The response is paginated, with each page containing 10 chunks. Support for custom page size is coming soon.
#[utoipa::path(
    get,
    path = "/chunk_group/{group_id}/{page}",
    context_path = "/api",
    tag = "chunk_group",
    responses(
        (status = 200, description = "Chunks present within the specified group", body = BookmarkData),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("group_id" = uuid::Uuid, description = "The id of the group to get the chunks from"),
        ("page" = u64, description = "The page of chunks to get from the group"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_chunks_in_group(
    path_data: web::Path<GetAllBookmarksData>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let group_id = path_data.group_id;
    let page = path_data.page.unwrap_or(1);
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let bookmarks = if let Some(group_id) = group_id {
        get_bookmarks_for_group_query(
            UnifiedId::TrieveUuid(group_id),
            page,
            None,
            dataset_id,
            pool,
        )
        .await
        .map_err(<ServiceError as std::convert::Into<actix_web::Error>>::into)?
    } else if let Some(tracking_id) = path_data.tracking_id.clone() {
        get_bookmarks_for_group_query(
            UnifiedId::TrackingId(tracking_id),
            page,
            None,
            dataset_id,
            pool,
        )
        .await
        .map_err(<ServiceError as std::convert::Into<actix_web::Error>>::into)?
    } else {
        return Err(ServiceError::BadRequest("No group id or tracking id provided".into()).into());
    };

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

/// get_chunks_in_group_by_tracking_id
///
/// Route to get all chunks for a group. The response is paginated, with each page containing 10 chunks. Support for custom page size is coming soon.
#[utoipa::path(
    get,
    path = "/chunk_group/tracking_id/{group_tracking_id}/{page}",
    context_path = "/api",
    tag = "chunk_group",
    responses(
        (status = 200, description = "Chunks present within the specified group", body = BookmarkData),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("group_tracking_id" = uuid::Uuid, description = "The id of the group to get the chunks from"),
        ("page" = u64, description = "The page of chunks to get from the group"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
#[deprecated]
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
        .await
        .map_err(<ServiceError as std::convert::Into<actix_web::Error>>::into)?
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
        ("Cookie" = ["readonly"])
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

    let groups = get_groups_for_bookmark_query(chunk_ids, dataset_id, pool)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(groups))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteBookmarkPathData {
    pub chunk_id: uuid::Uuid,
}

/// remove_chunk_from_group
///
/// Route to remove a chunk from a group.
#[utoipa::path(
    delete,
    path = "/chunk_group/chunk/{group_id}",
    context_path = "/api",
    tag = "chunk_group",
    responses(
        (status = 204, description = "Confirmation that the chunk was removed to the group"),
        (status = 400, description = "Service error relating to removing the chunk from the group", body = ErrorResponseBody),
    ),
    request_body(content = DeleteBookmarkPathData, description = "JSON request payload to send an invitation", content_type = "application/json"),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("group_id" = uuid::Uuid, description = "Id of the group to remove the bookmark'ed chunk from"),
    ),
    request_body(content = CreateChunkGroupData, description = "JSON request payload to cretea a chunkGroup", content_type = "application/json"),
    security(
        ("ApiKey" = ["admin"]),
        ("Cookie" = ["admin"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn remove_chunk_from_group(
    group_id: web::Path<uuid::Uuid>,
    body: web::Json<DeleteBookmarkPathData>,
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

    let qdrant_point_id = delete_chunk_from_group_query(chunk_id, group_id, pool)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if let Some(qdrant_point_id) = qdrant_point_id {
        remove_bookmark_from_qdrant_query(qdrant_point_id, group_id, server_dataset_config)
            .await
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    }

    Ok(HttpResponse::NoContent().finish())
}

#[tracing::instrument(skip(pool))]
pub async fn group_unique_search(
    group_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkGroup, actix_web::Error> {
    let group = get_group_by_id_query(group_id, dataset_id, pool)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(group)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GenerateOffGroupData {
    pub group_id: uuid::Uuid,
    pub page: Option<u64>,
    pub query: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct ReccomendGroupChunksRequest {
    /// The  ids of the groups to be used as positive examples for the recommendation. The groups in this array will be used to find similar groups.
    pub positive_group_ids: Option<Vec<uuid::Uuid>>,
    /// The  ids of the groups to be used as negative examples for the recommendation. The groups in this array will be used to filter out similar groups.
    pub negative_group_ids: Option<Vec<uuid::Uuid>>,
    /// The  ids of the groups to be used as positive examples for the recommendation. The groups in this array will be used to find similar groups.
    pub positive_group_tracking_ids: Option<Vec<String>>,
    /// The  ids of the groups to be used as negative examples for the recommendation. The groups in this array will be used to filter out similar groups.
    pub negative_group_tracking_ids: Option<Vec<String>>,
    /// Filters to apply to the chunks to be recommended. This is a JSON object which contains the filters to apply to the chunks to be recommended. The default is None.
    pub filters: Option<ChunkFilter>,
    /// The number of groups to return. This is the number of groups which will be returned in the response. The default is 10.
    pub limit: Option<u64>,
    /// The number of chunks to fetch for each group. This is the number of chunks which will be returned in the response for each group. The default is 10.
    pub group_size: Option<u32>,
}

#[utoipa::path(
    post,
    path = "/chunk_group/recommend",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = ReccomendGroupChunksRequest, description = "JSON request payload to get recommendations of chunks similar to the chunks in the request", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON response payload containing chunks with scores which are similar to those in the request body", body = Vec<GroupScoreChunkDTO>),
        (status = 400, description = "Service error relating to to getting similar chunks", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_recommended_groups(
    data: web::Json<ReccomendGroupChunksRequest>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let positive_group_ids = data.positive_group_ids.clone();
    let negative_group_ids = data.negative_group_ids.clone();
    let positive_tracking_ids = data.positive_group_tracking_ids.clone();
    let negative_tracking_ids = data.negative_group_tracking_ids.clone();

    let limit = data.limit.unwrap_or(10);
    let server_dataset_config =
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);

    let positive_qdrant_ids = if positive_group_ids.is_some() {
        get_point_ids_from_unified_group_ids(
            positive_group_ids
                .clone()
                .unwrap()
                .into_iter()
                .map(UnifiedId::TrieveUuid)
                .collect(),
            pool.clone(),
        )
        .await
        .map_err(|err| {
            ServiceError::BadRequest(format!("Could not get positive qdrant_point_ids: {}", err))
        })?
    } else if positive_group_ids.is_none() && positive_tracking_ids.is_some() {
        get_point_ids_from_unified_group_ids(
            positive_tracking_ids
                .clone()
                .unwrap()
                .into_iter()
                .map(UnifiedId::TrackingId)
                .collect(),
            pool.clone(),
        )
        .await
        .map_err(|err| {
            ServiceError::BadRequest(format!(
                "Could not get positive qdrant_point_ids from tracking_ids: {}",
                err
            ))
        })?
    } else {
        return Err(ServiceError::BadRequest(
            "You must provide either positive_group_ids or positive_group_tracking_ids".into(),
        )
        .into());
    };

    log::info!("positive: {:?}", positive_qdrant_ids);

    let negative_qdrant_ids = if negative_group_ids.is_some() {
        get_point_ids_from_unified_group_ids(
            negative_group_ids
                .clone()
                .unwrap()
                .into_iter()
                .map(UnifiedId::TrieveUuid)
                .collect(),
            pool.clone(),
        )
        .await
        .map_err(|err| {
            ServiceError::BadRequest(format!("Could not get negative qdrant_point_ids: {}", err))
        })?
    } else if negative_group_ids.is_none() && negative_tracking_ids.is_some() {
        get_point_ids_from_unified_group_ids(
            negative_tracking_ids
                .clone()
                .unwrap()
                .into_iter()
                .map(UnifiedId::TrackingId)
                .collect(),
            pool.clone(),
        )
        .await
        .map_err(|err| {
            ServiceError::BadRequest(format!(
                "Could not get negative qdrant_point_ids from tracking_ids: {}",
                err
            ))
        })?
    } else {
        vec![]
    };

    log::info!("negative: {:?}", negative_qdrant_ids);

    let recommended_qdrant_point_ids = recommend_qdrant_groups_query(
        positive_qdrant_ids,
        negative_qdrant_ids,
        data.filters.clone(),
        limit,
        data.group_size.unwrap_or(10),
        dataset_org_plan_sub.dataset.id,
        server_dataset_config,
    )
    .await
    .map_err(|err| {
        ServiceError::BadRequest(format!("Could not get recommended groups: {}", err))
    })?;

    let group_query_result = SearchOverGroupsQueryResult {
        search_results: recommended_qdrant_point_ids.clone(),
        total_chunk_pages: (recommended_qdrant_point_ids.len() as f64 / 10.0).ceil() as i64,
    };

    let recommended_chunk_metadatas =
        get_metadata_from_groups(group_query_result, Some(false), pool).await?;

    Ok(HttpResponse::Ok().json(recommended_chunk_metadatas))
}

#[derive(Serialize, Deserialize, Clone, Debug, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct SearchWithinGroupData {
    /// The query is the search query. This can be any string. The query will be used to create an embedding vector and/or SPLADE vector which will be used to find the result set.
    pub query: String,
    /// The page of chunks to fetch. Each page is 10 chunks. Support for custom page size is coming soon.
    pub page: Option<u64>,
    /// The page size is the number of chunks to fetch. This can be used to fetch more than 10 chunks at a time.
    pub page_size: Option<u64>,
    /// Filters is a JSON object which can be used to filter chunks. The values on each key in the object will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Group specifies the group to search within. Results will only consist of chunks which are bookmarks within the specified group.
    pub group_id: Option<uuid::Uuid>,
    /// Group_tracking_id specifies the group to search within by tracking id. Results will only consist of chunks which are bookmarks within the specified group. If both group_id and group_tracking_id are provided, group_id will be used.
    pub group_tracking_id: Option<String>,
    /// Search_type can be either "semantic", "fulltext", or "hybrid". "hybrid" will pull in one page (10 chunks) of both semantic and full-text results then re-rank them using BAAI/bge-reranker-large. "semantic" will pull in one page (10 chunks) of the nearest cosine distant vectors. "fulltext" will pull in one page (10 chunks) of full-text results based on SPLADE.
    pub search_type: String,
    /// Set date_bias to true to bias search results towards more recent chunks. This will work best in hybrid search mode.
    pub date_bias: Option<bool>,
    /// Set use_weights to true to use the weights of the chunks in the result set in order to sort them. If not specified, this defaults to true.
    pub use_weights: Option<bool>,
    /// Set highlight_results to true to highlight the results. If not specified, this defaults to true.
    pub highlight_results: Option<bool>,
    /// Set highlight_delimiters to a list of strings to use as delimiters for highlighting. If not specified, this defaults to ["?", ",", ".", "!"].
    pub highlight_delimiters: Option<Vec<String>>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold.
    pub score_threshold: Option<f32>,
}

impl From<SearchWithinGroupData> for SearchChunkData {
    fn from(data: SearchWithinGroupData) -> Self {
        Self {
            query: data.query,
            page: data.page,
            page_size: data.page_size,
            filters: data.filters,
            search_type: data.search_type,
            date_bias: data.date_bias,
            use_weights: data.use_weights,
            get_collisions: Some(false),
            highlight_results: data.highlight_results,
            highlight_delimiters: data.highlight_delimiters,
            score_threshold: data.score_threshold,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SearchGroupsResult {
    pub bookmarks: Vec<ScoreChunkDTO>,
    pub group: ChunkGroup,
    pub total_pages: i64,
}

/// search_within_group
///
/// This route allows you to search only within a group. This is useful for when you only want search results to contain chunks which are members of a specific group. If choosing hybrid search, the results will be re-ranked using BAAI/bge-reranker-large.
#[utoipa::path(
    post,
    path = "/chunk_group/search",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = SearchWithinGroupData, description = "JSON request payload to semantically search a group", content_type = "application/json"),
    responses(
        (status = 200, description = "Group chunks which are similar to the embedding vector of the search query", body = SearchGroupsResult),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
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
    let page = data.page.unwrap_or(1);
    let group_id = data.group_id;
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let search_pool = pool.clone();

    let group = {
        if let Some(group_id) = group_id {
            get_group_by_id_query(group_id, dataset_id, pool)
                .await
                .map_err(|err| ServiceError::BadRequest(err.message.into()))?
        } else if let Some(group_tracking_id) = data.group_tracking_id.clone() {
            get_group_from_tracking_id_query(group_tracking_id, dataset_id, pool)
                .await
                .map_err(|err| ServiceError::BadRequest(err.message.into()))?
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
                data,
                parsed_query,
                group,
                page,
                search_pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
            )
            .await?
        }
        "hybrid" => {
            search_hybrid_groups(
                data,
                parsed_query,
                group,
                page,
                search_pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
            )
            .await?
        }
        _ => {
            search_semantic_groups(
                data,
                parsed_query,
                group,
                page,
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
    /// Page of chunks to fetch. Each page is 10 chunks. Support for custom page size is coming soon.
    pub page: Option<u64>,
    /// Page size is the number of chunks to fetch. This can be used to fetch more than 10 chunks at a time.
    pub page_size: Option<u32>,
    /// Filters is a JSON object which can be used to filter chunks. The values on each key in the object will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata.
    pub filters: Option<ChunkFilter>,
    /// Set get_collisions to true to get the collisions for each chunk. This will only apply if environment variable COLLISIONS_ENABLED is set to true.
    pub get_collisions: Option<bool>,
    /// Set highlight_results to true to highlight the results. If not specified, this defaults to true.
    pub highlight_results: Option<bool>,
    /// Set highlight_delimiters to a list of strings to use as delimiters for highlighting. If not specified, this defaults to ["?", ",", ".", "!"].
    pub highlight_delimiters: Option<Vec<String>>,
    /// Set score_threshold to a float to filter out chunks with a score below the threshold.
    pub score_threshold: Option<f32>,
    // Group_size is the number of chunks to fetch for each group.
    pub group_size: Option<u32>,
}

/// group_oriented_search
///
/// This route allows you to get groups as results instead of chunks. Each group returned will have the matching chunks sorted by similarity within the group. This is useful for when you want to get groups of chunks which are similar to the search query. If choosing hybrid search, the results will be re-ranked using BAAI/bge-reranker-large. Compatible with semantic, fulltext, or hybrid search modes.
#[utoipa::path(
    post,
    path = "/chunk_group/group_oriented_search",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = SearchOverGroupsData, description = "JSON request payload to semantically search over groups", content_type = "application/json"),
    responses(
        (status = 200, description = "Group chunks which are similar to the embedding vector of the search query", body = SearchOverGroupsResponseBody),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = ErrorResponseBody),
    ),
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

    //search over the links as well
    let page = data.page.unwrap_or(1);

    let parsed_query = parse_query(data.query.clone());

    let result_chunks = match data.search_type.as_str() {
        "fulltext" => {
            if !server_dataset_config.FULLTEXT_ENABLED {
                return Err(ServiceError::BadRequest(
                    "Fulltext search is not enabled for this dataset".into(),
                )
                .into());
            }

            full_text_search_over_groups(
                data,
                parsed_query,
                page,
                pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
            )
            .await?
        }
        "hybrid" => {
            hybrid_search_over_groups(
                data,
                parsed_query,
                page,
                pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
            )
            .await?
        }
        _ => {
            semantic_search_over_groups(
                data,
                parsed_query,
                page,
                pool,
                dataset_org_plan_sub.dataset,
                server_dataset_config,
            )
            .await?
        }
    };

    Ok(HttpResponse::Ok().json(result_chunks))
}
