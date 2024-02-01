use super::auth_handler::{AdminOnly, LoggedUser};
use crate::{
    data::models::{
        ChunkGroup, ChunkGroupAndFile, ChunkGroupBookmark, ChunkMetadataWithFileData,
        DatasetAndOrgWithSubAndPlan, Pool,
    },
    errors::ServiceError,
    operators::{
        group_operator::*,
        qdrant_operator::{add_bookmark_to_qdrant_query, remove_bookmark_from_qdrant_query},
    },
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub async fn dataset_owns_group(
    group_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkGroup, actix_web::Error> {
    let group = web::block(move || get_group_by_id_query(group_id, dataset_id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if group.dataset_id != dataset_id {
        return Err(ServiceError::Forbidden.into());
    }

    Ok(group)
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateChunkGroupData {
    /// Name to assign to the chunk_group. Does not need to be unique.
    pub name: String,
    /// Description to assign to the chunk_group. Convenience field for you to avoid having to remember what the group is for.
    pub description: String,
}

/// create_chunk_group
///
/// Create a new chunk_group. Think of this as analogous to a bookmark folder.
#[utoipa::path(
    post,
    path = "/chunk_group",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = CreateChunkGroupData, description = "JSON request payload to cretea a chunkGroup", content_type = "application/json"),
    responses(
        (status = 200, description = "Returns the created chunkGroup", body = ChunkGroup),
        (status = 400, description = "Service error relating to creating the chunkGroup", body = DefaultError),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
        ("Cookie" = ["admin"])
    )
)]
pub async fn create_chunk_group(
    body: web::Json<CreateChunkGroupData>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let name = body.name.clone();
    let description = body.description.clone();

    let group = ChunkGroup::from_details(name, description, dataset_org_plan_sub.dataset.id);
    {
        let group = group.clone();
        web::block(move || create_group_query(group, pool))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    }

    Ok(HttpResponse::Ok().json(group))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GroupData {
    pub groups: Vec<ChunkGroupAndFile>,
    pub total_pages: i64,
}

#[derive(Deserialize, Serialize, ToSchema)]
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
        (status = 400, description = "Service error relating to getting the groups created by the given dataset", body = DefaultError),
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
pub async fn get_specific_dataset_chunk_groups(
    dataset_and_page: web::Path<DatasetGroupQuery>,
    _dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    _required_user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let groups = web::block(move || {
        get_groups_for_specific_dataset_query(
            dataset_and_page.page,
            dataset_and_page.dataset_id,
            pool,
        )
    })
    .await?
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
            })
            .collect(),
        total_pages: groups
            .first()
            .map(|group| (group.group_count.unwrap_or(10) as f64 / 10.0).ceil() as i64)
            .unwrap_or(1),
    }))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
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
        (status = 400, description = "Service error relating to deleting the chunkGroup", body = DefaultError),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("group_id" = uuid, description = "Id of the chunk_group to delete"),
    ),
    security(
        ("ApiKey" = ["admin"]),
        ("Cookie" = ["admin"])
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
    let group_id = group_id.into_inner();

    dataset_owns_group(group_id, dataset_org_plan_sub.dataset.id, pool).await?;

    delete_group_by_id_query(
        group_id,
        dataset_org_plan_sub.dataset,
        data.delete_chunks,
        delete_group_pool,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateChunkGroupData {
    /// Id of the chunk_group to update.
    pub group_id: uuid::Uuid,
    /// Name to assign to the chunk_group. Does not need to be unique. If not provided, the name will not be updated.
    pub name: Option<String>,
    /// Description to assign to the chunk_group. Convenience field for you to avoid having to remember what the group is for. If not provided, the description will not be updated.
    pub description: Option<String>,
}

/// update_chunk_group
///
/// Update a chunk_group. Think of this as analogous to a bookmark folder.
#[utoipa::path(
    put,
    path = "/chunk_group",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = UpdateChunkGroupData, description = "JSON request payload to update a chunkGroup", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunkGroup was updated"),
        (status = 400, description = "Service error relating to updating the chunkGroup", body = DefaultError),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
        ("Cookie" = ["admin"])
    )
)]
pub async fn update_chunk_group(
    body: web::Json<UpdateChunkGroupData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let name = body.name.clone();
    let description = body.description.clone();
    let group_id = body.group_id;

    let pool2 = pool.clone();

    let group = dataset_owns_group(group_id, dataset_org_plan_sub.dataset.id, pool).await?;

    web::block(move || {
        update_chunk_group_query(
            group,
            name,
            description,
            dataset_org_plan_sub.dataset.id,
            pool2,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct AddChunkToGroupData {
    /// Id of the chunk to make a member of the group. Think of this as "bookmark"ing a chunk.
    pub chunk_id: uuid::Uuid,
}

/// add_bookmark
///
/// Route to add a bookmark. Think of a bookmark as a chunk which is a member of a group.
#[utoipa::path(
    post,
    path = "/chunk_group/{group_id}",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = AddChunkToGroupData, description = "JSON request payload to add a chunk to a group (bookmark it)", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunk was added to the group (bookmark'ed)."),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in.", body = DefaultError),
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
pub async fn add_bookmark(
    body: web::Json<AddChunkToGroupData>,
    group_id: web::Path<uuid::Uuid>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let pool2 = pool.clone();
    let chunk_metadata_id = body.chunk_id;
    let group_id = group_id.into_inner();
    let dataset_id = dataset_org_plan_sub.dataset.id;

    dataset_owns_group(group_id, dataset_id, pool).await?;

    let qdrant_point_id = web::block(move || {
        create_chunk_bookmark_query(
            pool2,
            ChunkGroupBookmark::from_details(group_id, chunk_metadata_id),
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if let Some(qdrant_point_id) = qdrant_point_id {
        add_bookmark_to_qdrant_query(qdrant_point_id, group_id)
            .await
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    }

    Ok(HttpResponse::NoContent().finish())
}
#[derive(Deserialize, Serialize, ToSchema)]
pub struct BookmarkData {
    pub bookmarks: Vec<BookmarkChunks>,
    pub group: ChunkGroup,
    pub total_pages: i64,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GetAllBookmarksData {
    pub group_id: uuid::Uuid,
    pub page: Option<u64>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct BookmarkChunks {
    pub metadata: ChunkMetadataWithFileData,
}

/// get_all_bookmarks
///
/// Route to get all bookmarks for a group. Think of a bookmark as a chunk which is a member of a group. The response is paginated, with each page containing 10 chunks (bookmarks). Support for custom page size is coming soon.
#[utoipa::path(
    get,
    path = "/chunk_group/{group_id}/{page}",
    context_path = "/api",
    tag = "chunk_group",
    responses(
        (status = 200, description = "Bookmark'ed chunks present within the specified group", body = BookmarkData),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = DefaultError),
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
pub async fn get_all_bookmarks(
    path_data: web::Path<GetAllBookmarksData>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let group_id = path_data.group_id;
    let page = path_data.page.unwrap_or(1);
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let bookmarks = {
        web::block(move || get_bookmarks_for_group_query(group_id, page, None, dataset_id, pool))
            .await?
            .map_err(<ServiceError as std::convert::Into<actix_web::Error>>::into)?
    };

    let group_chunks = bookmarks
        .metadata
        .iter()
        .map(|search_result| BookmarkChunks {
            metadata: search_result.clone(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(BookmarkData {
        bookmarks: group_chunks,
        group: bookmarks.group,
        total_pages: bookmarks.total_pages,
    }))
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GetGroupsForChunksData {
    pub chunk_ids: Vec<uuid::Uuid>,
}

#[utoipa::path(
    post,
    path = "/chunk_group/bookmark",
    context_path = "/api",
    tag = "chunk_group",
    request_body(content = GetGroupsForChunksData, description = "JSON request payload to get the groups that a chunk is in", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the groups that the chunk is in", body = Vec<BookmarkGroupResult>),
        (status = 400, description = "Service error relating to getting the groups that the chunk is in", body = DefaultError),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
pub async fn get_groups_chunk_is_in(
    data: web::Json<GetGroupsForChunksData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _required_user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let chunk_ids = data.chunk_ids.clone();

    let dataset_id = dataset_org_plan_sub.dataset.id;

    let groups = web::block(move || get_groups_for_bookmark_query(chunk_ids, dataset_id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(groups))
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct DeleteBookmarkPathData {
    pub group_id: uuid::Uuid,
    pub bookmark_id: uuid::Uuid,
}

/// delete_bookmark
///
/// Route to delete a bookmark. Think of a bookmark as a chunk which is a member of a group.
#[utoipa::path(
    delete,
    path = "/bookmark/{group_id}/{bookmark_id}",
    context_path = "/api",
    tag = "chunk_group",
    responses(
        (status = 204, description = "Confirmation that the chunk was removed to the group"),
        (status = 400, description = "Service error relating to removing the chunk from the group", body = DefaultError),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("group_id" = uuid::Uuid, description = "Id of the group to remove the bookmark'ed chunk from"),
        ("bookmark_id" = uuid::Uuid, description = "Id of the bookmark to remove"),
    ),
    security(
        ("ApiKey" = ["admin"]),
        ("Cookie" = ["admin"])
    )
)]
pub async fn delete_bookmark(
    path_data: web::Path<DeleteBookmarkPathData>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let pool1 = pool.clone();
    let group_id = path_data.group_id;
    let bookmark_id = path_data.bookmark_id;
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let pool = pool.clone();
    dataset_owns_group(group_id, dataset_id, pool1).await?;

    let qdrant_point_id = web::block(move || delete_bookmark_query(bookmark_id, group_id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if let Some(qdrant_point_id) = qdrant_point_id {
        remove_bookmark_from_qdrant_query(qdrant_point_id, group_id)
            .await
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    }

    Ok(HttpResponse::NoContent().finish())
}

pub async fn group_unique_search(
    group_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkGroup, actix_web::Error> {
    let group = web::block(move || get_group_by_id_query(group_id, dataset_id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(group)
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GenerateOffGroupData {
    pub group_id: uuid::Uuid,
    pub page: Option<u64>,
    pub query: Option<String>,
}
