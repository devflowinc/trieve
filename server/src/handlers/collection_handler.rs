use super::auth_handler::{AdminOnly, LoggedUser};
use crate::{
    data::models::{
        ChunkCollection, ChunkCollectionAndFile, ChunkCollectionBookmark,
        ChunkMetadataWithFileData, DatasetAndOrgWithSubAndPlan, Pool,
    },
    errors::ServiceError,
    operators::{chunk_operator::get_collided_chunks_query, collection_operator::*},
};
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use utoipa::ToSchema;

pub async fn user_owns_collection(
    user_id: uuid::Uuid,
    collection_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<ChunkCollection, actix_web::Error> {
    let collection =
        web::block(move || get_collection_by_id_query(collection_id, dataset_id, pool))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if collection.author_id != user_id {
        return Err(ServiceError::Forbidden.into());
    }

    Ok(collection)
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateChunkCollectionData {
    pub name: String,
    pub description: String,
}

#[utoipa::path(
    post,
    path = "/chunk_collection",
    context_path = "/api",
    tag = "chunk_collection",
    request_body(content = CreateChunkCollectionData, description = "JSON request payload to cretea a chunkCollection", content_type = "application/json"),
    responses(
        (status = 200, description = "Returns the created chunkCollection", body = [ChunkCollection]),
        (status = 400, description = "Service error relating to creating the chunkCollection", body = [DefaultError]),
    ),
)]
pub async fn create_chunk_collection(
    body: web::Json<CreateChunkCollectionData>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let name = body.name.clone();
    let description = body.description.clone();

    let collection = ChunkCollection::from_details(
        user.0.id,
        name,
        description,
        dataset_org_plan_sub.dataset.id,
    );
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
pub struct UserCollectionQuery {
    pub user_id: uuid::Uuid,
    pub page: u64,
}

#[utoipa::path(
    get,
    path = "/user/collections/{user_id}/{page}",
    context_path = "/api",
    tag = "user",
    responses(
        (status = 200, description = "JSON body representing the collections created by the given user", body = [CollectionData]),
        (status = 400, description = "Service error relating to getting the collections created by the given user", body = [DefaultError]),
    ),
    params(
        ("user_id" = uuid::Uuid, description = "The id of the user to fetch collections for"),
        ("page" = i64, description = "The page of collections to fetch"),
    ),
)]
pub async fn get_specific_user_chunk_collections(
    user_and_page: web::Path<UserCollectionQuery>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    _required_user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let collections = web::block(move || {
        get_collections_for_specific_user_query(
            user_and_page.user_id,
            user_and_page.page,
            dataset_org_plan_sub.dataset.id,
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
                author_id: collection.author_id,
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

#[utoipa::path(
    get,
    path = "/chunk_collection/{page_or_chunk_collection_id}",
    context_path = "/api",
    tag = "chunk_collection",
    responses(
        (status = 200, description = "The page of collections for the auth'ed user", body = [CollectionData]),
        (status = 400, description = "Service error relating to getting the collections for the auth'ed user", body = [DefaultError]),
    ),
    params(
        ("page_number" = u64, description = "The page of collections to fetch"),
    ),
)]
pub async fn get_logged_in_user_chunk_collections(
    user: LoggedUser,
    page: web::Path<u64>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let collections = web::block(move || {
        get_collections_for_logged_in_user_query(
            user.id,
            page.into_inner(),
            dataset_org_plan_sub.dataset.id,
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
                author_id: collection.author_id,
                name: collection.name.clone(),
                description: collection.description.clone(),
                created_at: collection.created_at,
                updated_at: collection.updated_at,
                file_id: collection.file_id,
            })
            .collect(),
        total_pages: collections
            .first()
            .map(|collection| (collection.collection_count.unwrap_or(5) as f64 / 5.0).ceil() as i64)
            .unwrap_or(1),
    }))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeleteCollectionData {
    pub collection_id: uuid::Uuid,
}

#[utoipa::path(
    delete,
    path = "/chunk_collection",
    context_path = "/api",
    tag = "chunk_collection",
    request_body(content = DeleteCollectionData, description = "JSON request payload to delete a chunkCollection", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunkCollection was deleted"),
        (status = 400, description = "Service error relating to deleting the chunkCollection", body = [DefaultError]),
    ),
)]
pub async fn delete_chunk_collection(
    data: web::Json<DeleteCollectionData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let pool2 = pool.clone();
    let collection_id = data.collection_id;

    user_owns_collection(
        user.0.id,
        collection_id,
        dataset_org_plan_sub.dataset.id,
        pool,
    )
    .await?;

    web::block(move || {
        delete_collection_by_id_query(collection_id, dataset_org_plan_sub.dataset.id, pool2)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateChunkCollectionData {
    pub collection_id: uuid::Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[utoipa::path(
    put,
    path = "/chunk_collection",
    context_path = "/api",
    tag = "chunk_collection",
    request_body(content = UpdateChunkCollectionData, description = "JSON request payload to update a chunkCollection", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunkCollection was updated"),
        (status = 400, description = "Service error relating to updating the chunkCollection", body = [DefaultError]),
    ),
)]
pub async fn update_chunk_collection(
    body: web::Json<UpdateChunkCollectionData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let name = body.name.clone();
    let description = body.description.clone();
    let collection_id = body.collection_id;

    let pool2 = pool.clone();

    let collection = user_owns_collection(
        user.0.id,
        collection_id,
        dataset_org_plan_sub.dataset.id,
        pool,
    )
    .await?;

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
    pub chunk_metadata_id: uuid::Uuid,
}

#[utoipa::path(
    post,
    path = "/chunk_collection/{page_or_chunk_collection_id}",
    context_path = "/api",
    tag = "chunk_collection",
    request_body(content = AddChunkToCollectionData, description = "JSON request payload to add a chunk to a collection (bookmark it)", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunk was added to the collection"),
        (status = 400, description = "Service error relating to getting the collections that the chunk is in", body = [DefaultError]),
    ),
    params(
        ("page_or_chunk_collection_id" = uuid::Uuid, description = "The id of the collection to add the chunk to"),
    ),
)]
pub async fn add_bookmark(
    body: web::Json<AddChunkToCollectionData>,
    collection_id: web::Path<uuid::Uuid>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    user: AdminOnly,
) -> Result<HttpResponse, actix_web::Error> {
    let pool2 = pool.clone();
    let chunk_metadata_id = body.chunk_metadata_id;
    let collection_id = collection_id.into_inner();
    let dataset_id = dataset_org_plan_sub.dataset.id;

    user_owns_collection(user.0.id, collection_id, dataset_id, pool).await?;

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
    pub metadata: Vec<ChunkMetadataWithFileData>,
}

#[utoipa::path(
    get,
    path = "/chunk_collection/{collection_id}/{page}",
    context_path = "/api",
    tag = "chunk_collection",
    responses(
        (status = 200, description = "Bookmark'ed chunks present within the specified collection", body = [BookmarkData]),
        (status = 400, description = "Service error relating to getting the collections that the chunk is in", body = [DefaultError]),
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
    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let bookmarks = {
        web::block(move || {
            get_bookmarks_for_collection_query(collection_id, page, None, dataset_id, pool2)
        })
        .await?
        .map_err(<ServiceError as std::convert::Into<actix_web::Error>>::into)?
    };

    let point_ids = bookmarks
        .metadata
        .iter()
        .map(|point| point.qdrant_point_id)
        .collect::<Vec<uuid::Uuid>>();

    let collided_chunks = {
        web::block(move || get_collided_chunks_query(point_ids, dataset_id, pool1))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?
    };

    let collection_chunks = bookmarks
        .metadata
        .iter()
        .map(|search_result| {
            let mut collided_chunks: Vec<ChunkMetadataWithFileData> = collided_chunks
                .iter()
                .filter(|chunk| {
                    chunk.1 == search_result.qdrant_point_id && chunk.0.id != search_result.id
                })
                .map(|chunk| chunk.0.clone())
                .collect();

            // de-duplicate collided chunks by removing chunks with the same metadata: Option<serde_json::Value>
            let mut seen_metadata = HashSet::new();
            let mut i = 0;
            while i < collided_chunks.len() {
                let metadata_string = serde_json::to_string(&collided_chunks[i].metadata).unwrap();

                if seen_metadata.contains(&metadata_string) {
                    collided_chunks.remove(i);
                } else {
                    seen_metadata.insert(metadata_string);
                    i += 1;
                }
            }

            collided_chunks.insert(0, search_result.clone());

            // Move the chunk that was searched for to the front of the list
            let (matching, others): (Vec<_>, Vec<_>) = collided_chunks
                .clone()
                .into_iter()
                .partition(|item| item.id == search_result.id);

            collided_chunks.clear();
            collided_chunks.extend(matching);
            collided_chunks.extend(others);

            BookmarkChunks {
                metadata: collided_chunks,
            }
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
        (status = 200, description = "JSON body representing the collections that the chunk is in", body = [Vec<BookmarkCollectionResult>]),
        (status = 400, description = "Service error relating to getting the collections that the chunk is in", body = [DefaultError]),
    ),
)]
pub async fn get_collections_chunk_is_in(
    data: web::Json<GetCollectionsForChunksData>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    user: Option<LoggedUser>,
    _required_user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let chunk_ids = data.chunk_ids.clone();

    let dataset_id = dataset_org_plan_sub.dataset.id;
    let current_user_id = user.map(|user| user.id);

    let collections = web::block(move || {
        get_collections_for_bookmark_query(chunk_ids, current_user_id, dataset_id, pool)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(collections))
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct RemoveBookmarkData {
    pub chunk_metadata_id: uuid::Uuid,
}

#[utoipa::path(
    delete,
    path = "/chunk_collection/{page_or_chunk_collection_id}",
    context_path = "/api",
    tag = "chunk_collection",
    request_body(content = RemoveBookmarkData, description = "JSON request payload to remove a chunk to a collection (un-bookmark it)", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the chunk was removed to the collection"),
        (status = 400, description = "Service error relating to removing the chunk from the collection", body = [DefaultError]),
    ),
    params(
        ("page_or_chunk_collection_id" = uuid::Uuid, description = "The id of the collection to remove the bookmark'ed chunk from"),
    ),
)]
pub async fn delete_bookmark(
    collection_id: web::Path<uuid::Uuid>,
    body: web::Json<RemoveBookmarkData>,
    pool: web::Data<Pool>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let pool1 = pool.clone();
    let collection_id = collection_id.into_inner();
    let bookmark_id = body.chunk_metadata_id;
    let dataset_id = dataset_org_plan_sub.dataset.id;

    let pool = pool.clone();
    user_owns_collection(user.0.id, collection_id, dataset_id, pool1).await?;

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
