use super::auth_handler::{AdminOnly, LoggedUser};
use crate::{
    data::models::{
        ChunkCollection, ChunkCollectionAndFile, ChunkCollectionBookmark,
        ChunkMetadataWithFileData, DatasetAndOrgWithSubAndPlan, Pool,
    },
    errors::ServiceError,
    operators::collection_operator::*,
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub async fn dataset_owns_collection(
    collection_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkCollection, actix_web::Error> {
    let collection =
        web::block(move || get_collection_by_id_query(collection_id, dataset_id, pool))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if collection.dataset_id != dataset_id {
        return Err(ServiceError::Forbidden.into());
    }

    Ok(collection)
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateChunkCollectionData {
    /// Name to assign to the chunk_collection. Does not need to be unique.
    pub name: String,
    /// Description to assign to the chunk_collection. Convenience field for you to avoid having to remember what the collection is for.
    pub description: String,
}

/// create_chunk_collection
///
/// Create a new chunk_collection. Think of this as analogous to a bookmark folder.
#[utoipa::path(
    post,
    path = "/chunk_collection",
    context_path = "/api",
    tag = "chunk_collection",
    request_body(content = CreateChunkCollectionData, description = "JSON request payload to cretea a chunkCollection", content_type = "application/json"),
    responses(
        (status = 200, description = "Returns the created chunkCollection", body = ChunkCollection),
        (status = 400, description = "Service error relating to creating the chunkCollection", body = DefaultError),
    ),
)]
pub async fn create_chunk_collection(
    body: web::Json<CreateChunkCollectionData>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let name = body.name.clone();
    let description = body.description.clone();

    let collection =
        ChunkCollection::from_details(name, description, dataset_org_plan_sub.dataset.id);
    {
        let collection = collection.clone();
        web::block(move || create_collection_query(collection, pool))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    }

    Ok(HttpResponse::Ok().json(collection))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CollectionData {
    pub collections: Vec<ChunkCollectionAndFile>,
    pub total_pages: i64,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct DatasetCollectionQuery {
    pub dataset_id: uuid::Uuid,
    pub page: u64,
}

/// get_dataset_collections
///
/// Fetch the collections which belong to a dataset specified by its id.
#[utoipa::path(
    get,
    path = "/dataset/collections/{dataset_id}/{page}",
    context_path = "/api",
    tag = "chunk_collection",
    responses(
        (status = 200, description = "JSON body representing the collections created by the given dataset", body = CollectionData),
        (status = 400, description = "Service error relating to getting the collections created by the given dataset", body = DefaultError),
    ),
    params(
        ("user_id" = uuid::Uuid, description = "The id of the dataset to fetch collections for."),
        ("page" = i64, description = "The page of collections to fetch. Each page contains 10 collections. Support for custom page size is coming soon."),
    ),
)]
pub async fn get_specific_dataset_chunk_collections(
    dataset_and_page: web::Path<DatasetCollectionQuery>,
    _dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    _required_user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let collections = web::block(move || {
        get_collections_for_specific_dataset_query(
            dataset_and_page.page,
            dataset_and_page.dataset_id,
            pool,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(CollectionData {
        collections: collections
            .iter()
            .map(|collection| ChunkCollectionAndFile {
                id: collection.id,
                dataset_id: collection.dataset_id,
                name: collection.name.clone(),
                description: collection.description.clone(),
                created_at: collection.created_at,
                updated_at: collection.updated_at,
                file_id: collection.file_id,
            })
            .collect(),
        total_pages: collections
            .first()
            .map(|collection| {
                (collection.collection_count.unwrap_or(10) as f64 / 10.0).ceil() as i64
            })
            .unwrap_or(1),
    }))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeleteCollectionData {
    pub collection_id: uuid::Uuid,
}

/// delete_chunk_collection
///
/// This will delete a chunk_collection. This will not delete the chunks that are in the collection. We will soon support deleting a chunk_collection along with its member chunks.
#[utoipa::path(
    delete,
    path = "/chunk_collection/{collection_id}",
    context_path = "/api",
    tag = "chunk_collection",
    responses(
        (status = 204, description = "Confirmation that the chunkCollection was deleted"),
        (status = 400, description = "Service error relating to deleting the chunkCollection", body = DefaultError),
    ),
    params(
        ("collection_id" = uuid, description = "Id of the chunk_collection to delete"),
    ),
)]
pub async fn delete_chunk_collection(
    collection_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let delete_collection_pool = pool.clone();
    let collection_id = collection_id.into_inner();

    dataset_owns_collection(collection_id, dataset_org_plan_sub.dataset.id, pool).await?;

    web::block(move || {
        delete_collection_by_id_query(
            collection_id,
            dataset_org_plan_sub.dataset.id,
            delete_collection_pool,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateChunkCollectionData {
    /// Id of the chunk_collection to update.
    pub collection_id: uuid::Uuid,
    /// Name to assign to the chunk_collection. Does not need to be unique. If not provided, the name will not be updated.
    pub name: Option<String>,
    /// Description to assign to the chunk_collection. Convenience field for you to avoid having to remember what the collection is for. If not provided, the description will not be updated.
    pub description: Option<String>,
}

/// update_chunk_collection
///
/// Update a chunk_collection. Think of this as analogous to a bookmark folder.
#[utoipa::path(
    put,
    path = "/chunk_collection",
    context_path = "/api",
    tag = "chunk_collection",
    request_body(content = UpdateChunkCollectionData, description = "JSON request payload to update a chunkCollection", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunkCollection was updated"),
        (status = 400, description = "Service error relating to updating the chunkCollection", body = DefaultError),
    ),
)]
pub async fn update_chunk_collection(
    body: web::Json<UpdateChunkCollectionData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let name = body.name.clone();
    let description = body.description.clone();
    let collection_id = body.collection_id;

    let pool2 = pool.clone();

    let collection =
        dataset_owns_collection(collection_id, dataset_org_plan_sub.dataset.id, pool).await?;

    web::block(move || {
        update_chunk_collection_query(
            collection,
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
pub struct AddChunkToCollectionData {
    /// Id of the chunk to make a member of the collection. Think of this as "bookmark"ing a chunk.
    pub chunk_id: uuid::Uuid,
}

/// add_bookmark
///
/// Route to add a bookmark. Think of a bookmark as a chunk which is a member of a collection.
#[utoipa::path(
    post,
    path = "/chunk_collection/{collection_id}",
    context_path = "/api",
    tag = "chunk_collection",
    request_body(content = AddChunkToCollectionData, description = "JSON request payload to add a chunk to a collection (bookmark it)", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunk was added to the collection (bookmark'ed)."),
        (status = 400, description = "Service error relating to getting the collections that the chunk is in.", body = DefaultError),
    ),
    params(
        ("collection_id" = uuid, description = "Id of the collection to add the chunk to as a bookmark"),
    ),
)]
pub async fn add_bookmark(
    body: web::Json<AddChunkToCollectionData>,
    collection_id: web::Path<uuid::Uuid>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    _user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let pool2 = pool.clone();
    let chunk_metadata_id = body.chunk_id;
    let collection_id = collection_id.into_inner();
    let dataset_id = dataset_org_plan_sub.dataset.id;

    dataset_owns_collection(collection_id, dataset_id, pool).await?;

    web::block(move || {
        create_chunk_bookmark_query(
            pool2,
            ChunkCollectionBookmark::from_details(collection_id, chunk_metadata_id),
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}
#[derive(Deserialize, Serialize, ToSchema)]
pub struct BookmarkData {
    pub bookmarks: Vec<BookmarkChunks>,
    pub collection: ChunkCollection,
    pub total_pages: i64,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GetAllBookmarksData {
    pub collection_id: uuid::Uuid,
    pub page: Option<u64>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct BookmarkChunks {
    pub metadata: ChunkMetadataWithFileData,
}

/// get_all_bookmarks
///
/// Route to get all bookmarks for a collection. Think of a bookmark as a chunk which is a member of a collection. The response is paginated, with each page containing 10 chunks (bookmarks). Support for custom page size is coming soon.
#[utoipa::path(
    get,
    path = "/chunk_collection/{collection_id}/{page}",
    context_path = "/api",
    tag = "chunk_collection",
    responses(
        (status = 200, description = "Bookmark'ed chunks present within the specified collection", body = BookmarkData),
        (status = 400, description = "Service error relating to getting the collections that the chunk is in", body = DefaultError),
    ),
    params(
        ("collection_id" = uuid::Uuid, description = "The id of the collection to get the chunks from"),
        ("page" = u64, description = "The page of chunks to get from the collection"),
    ),
)]
pub async fn get_all_bookmarks(
    path_data: web::Path<GetAllBookmarksData>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let collection_id = path_data.collection_id;
    let page = path_data.page.unwrap_or(1);
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let bookmarks = {
        web::block(move || {
            get_bookmarks_for_collection_query(collection_id, page, None, dataset_id, pool)
        })
        .await?
        .map_err(<ServiceError as std::convert::Into<actix_web::Error>>::into)?
    };

    let collection_chunks = bookmarks
        .metadata
        .iter()
        .map(|search_result| BookmarkChunks {
            metadata: search_result.clone(),
        })
        .collect();

    Ok(HttpResponse::Ok().json(BookmarkData {
        bookmarks: collection_chunks,
        collection: bookmarks.collection,
        total_pages: bookmarks.total_pages,
    }))
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GetCollectionsForChunksData {
    pub chunk_ids: Vec<uuid::Uuid>,
}

#[utoipa::path(
    post,
    path = "/chunk_collection/bookmark",
    context_path = "/api",
    tag = "chunk_collection",
    request_body(content = GetCollectionsForChunksData, description = "JSON request payload to get the collections that a chunk is in", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the collections that the chunk is in", body = Vec<BookmarkCollectionResult>),
        (status = 400, description = "Service error relating to getting the collections that the chunk is in", body = DefaultError),
    ),
)]
pub async fn get_collections_chunk_is_in(
    data: web::Json<GetCollectionsForChunksData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _required_user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let chunk_ids = data.chunk_ids.clone();

    let dataset_id = dataset_org_plan_sub.dataset.id;

    let collections =
        web::block(move || get_collections_for_bookmark_query(chunk_ids, dataset_id, pool))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(collections))
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct DeleteBookmarkPathData {
    pub collection_id: uuid::Uuid,
    pub bookmark_id: uuid::Uuid,
}

/// delete_bookmark
///
/// Route to delete a bookmark. Think of a bookmark as a chunk which is a member of a collection.
#[utoipa::path(
    delete,
    path = "/bookmark/{collection_id}/{bookmark_id}",
    context_path = "/api",
    tag = "chunk_collection",
    responses(
        (status = 204, description = "Confirmation that the chunk was removed to the collection"),
        (status = 400, description = "Service error relating to removing the chunk from the collection", body = DefaultError),
    ),
    params(
        ("collection_id" = uuid::Uuid, description = "Id of the collection to remove the bookmark'ed chunk from"),
        ("bookmark_id" = uuid::Uuid, description = "Id of the bookmark to remove"),
    ),
)]
pub async fn delete_bookmark(
    path_data: web::Path<DeleteBookmarkPathData>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let pool1 = pool.clone();
    let collection_id = path_data.collection_id;
    let bookmark_id = path_data.bookmark_id;
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let pool = pool.clone();
    dataset_owns_collection(collection_id, dataset_id, pool1).await?;

    web::block(move || delete_bookmark_query(bookmark_id, collection_id, dataset_id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GenerateOffCollectionData {
    pub collection_id: uuid::Uuid,
    pub page: Option<u64>,
    pub query: Option<String>,
}
