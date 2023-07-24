use std::sync::{Arc, Mutex};

use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::{
        CardCollection, CardCollectionBookmark, CardMetadataWithVotesWithoutScore, Pool, CardCollectionAndFile,
    },
    errors::ServiceError,
    operators::{card_operator::get_collided_cards_query, collection_operator::*},
};

use super::auth_handler::LoggedUser;
//new handler and operator to get collections a card is in

pub async fn user_owns_collection(
    user_id: uuid::Uuid,
    collection_id: uuid::Uuid,
    pool: Arc<Mutex<web::Data<r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>>>>,
) -> Result<CardCollection, actix_web::Error> {
    let collection =
        web::block(move || get_collection_by_id_query(collection_id, pool.lock().unwrap()))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if collection.author_id != user_id {
        return Err(ServiceError::Forbidden.into());
    }

    Ok(collection)
}

#[derive(Deserialize)]
pub struct CreateCardCollectionData {
    pub name: String,
    pub description: String,
    pub is_public: bool,
}

pub async fn create_card_collection(
    body: web::Json<CreateCardCollectionData>,
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let name = body.name.clone();
    let description = body.description.clone();
    let is_public = body.is_public;

    web::block(move || {
        create_collection_query(
            CardCollection::from_details(user.id, name, is_public, description),
            pool,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize)]
pub struct CollectionData {
    pub collections: Vec<CardCollectionAndFile>,
    pub total_pages: i64,
}

#[derive(Deserialize)]
pub struct UserCollectionQuery {
    pub user_id: uuid::Uuid,
    pub page: u64,
}

pub async fn get_specific_user_card_collections(
    user: Option<LoggedUser>,
    user_and_page: web::Path<UserCollectionQuery>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let accessing_user_id = user.map(|user| user.id);
    let page = user_and_page.page;
    let collections = web::block(move || {
        get_collections_for_specifc_user_query(user_and_page.user_id, accessing_user_id, user_and_page.page, pool)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(CollectionData{
        collections: collections.iter().map(|collection| CardCollectionAndFile {
            id: collection.id,
            author_id: collection.author_id,
            name: collection.name.clone(),
            is_public: collection.is_public,
            description: collection.description.clone(),
            created_at: collection.created_at,
            updated_at: collection.updated_at,
            file_id: collection.file_id,
        }).collect(),
        total_pages: collections.get(0).map(|collection| collection.count / 10).unwrap_or(0),
    }))
}

pub async fn get_logged_in_user_card_collections(
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let collections = web::block(move || get_collections_for_logged_in_user_query(user.id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(collections))
}

#[derive(Debug, Deserialize)]
pub struct DeleteCollectionData {
    pub collection_id: uuid::Uuid,
}

pub async fn delete_card_collection(
    data: web::Json<DeleteCollectionData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let thread_safe_pool = Arc::new(Mutex::new(pool));
    let pool_two = thread_safe_pool.clone();
    let collection_id = data.collection_id;

    user_owns_collection(user.id, collection_id, thread_safe_pool).await?;

    web::block(move || delete_collection_by_id_query(collection_id, pool_two.lock().unwrap()))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
pub struct UpdateCardCollectionData {
    pub collection_id: uuid::Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

pub async fn update_card_collection(
    body: web::Json<UpdateCardCollectionData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let name = body.name.clone();
    let description = body.description.clone();
    let is_public = body.is_public;
    let collection_id = body.collection_id;
    let thread_safe_pool = Arc::new(Mutex::new(pool));

    let pool_two = thread_safe_pool.clone();

    let collection = user_owns_collection(user.id, collection_id, thread_safe_pool).await?;

    web::block(move || {
        update_card_collection_query(
            collection,
            name,
            description,
            is_public,
            pool_two.lock().unwrap(),
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize)]
pub struct AddCardToCollectionData {
    pub card_metadata_id: uuid::Uuid,
}

pub async fn add_bookmark(
    body: web::Json<AddCardToCollectionData>,
    collection_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let thread_safe_pool = Arc::new(Mutex::new(pool));
    let pool2 = thread_safe_pool.clone();
    let card_metadata_id = body.card_metadata_id;
    let collection_id = collection_id.into_inner();

    user_owns_collection(user.id, collection_id, thread_safe_pool).await?;

    web::block(move || {
        create_card_bookmark_query(
            pool2.lock().unwrap(),
            CardCollectionBookmark::from_details(collection_id, card_metadata_id),
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}
#[derive(Deserialize, Serialize)]
pub struct BookmarkData {
    pub bookmarks: Vec<BookmarkCards>,
    pub collection: CardCollection,
    pub total_pages: i64,
}

#[derive(Serialize, Deserialize, Debug)]

pub struct GetAllBookmarksData {
    pub collection_id: uuid::Uuid,
    pub page: Option<u64>,
}
#[derive(Deserialize, Serialize)]
pub struct BookmarkCards {
    pub metadata: Vec<CardMetadataWithVotesWithoutScore>,
}

pub async fn get_all_bookmarks(
    path_data: web::Path<GetAllBookmarksData>,
    pool: web::Data<Pool>,
    user: Option<LoggedUser>,
) -> Result<HttpResponse, actix_web::Error> {
    let collection_id = path_data.collection_id;
    let page = path_data.page.unwrap_or(1);
    let thread_safe_pool = Arc::new(Mutex::new(pool));
    let pool_two = thread_safe_pool.clone();
    let pool_three = thread_safe_pool.clone();
    let current_user_id = user.map(|user| user.id);

    let collection = web::block(move || {
        get_collection_by_id_query(collection_id, thread_safe_pool.lock().unwrap())
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    if !collection.is_public && current_user_id.is_none() {
        return Err(ServiceError::Unauthorized.into());
    }
    if !collection.is_public && Some(collection.author_id) != current_user_id {
        return Err(ServiceError::Forbidden.into());
    }

    let bookmarks = web::block(move || {
        get_bookmarks_for_collection_query(
            collection_id,
            page,
            current_user_id,
            pool_two.lock().unwrap(),
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let point_ids = bookmarks
        .metadata
        .iter()
        .map(|point| point.qdrant_point_id)
        .collect::<Vec<uuid::Uuid>>();

    let collided_cards = web::block(move || {
        get_collided_cards_query(point_ids, current_user_id, pool_three.lock().unwrap())
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let collection_cards = bookmarks
        .metadata
        .iter()
        .map(|search_result| {
            let mut collided_cards: Vec<CardMetadataWithVotesWithoutScore> = collided_cards
                .iter()
                .filter(|card| card.1 == search_result.qdrant_point_id)
                .map(|card| card.0.clone().into())
                .collect();

            collided_cards.insert(0, search_result.clone().into());

            // Move the card that was searched for to the front of the list
            let (matching, others): (Vec<_>, Vec<_>) = collided_cards
                .clone()
                .into_iter()
                .partition(|item| item.id == search_result.id);

            collided_cards.clear();
            collided_cards.extend(matching);
            collided_cards.extend(others);

            BookmarkCards {
                metadata: collided_cards,
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(BookmarkData {
        bookmarks: collection_cards,
        collection,
        total_pages: bookmarks.total_pages,
    }))
}

#[derive(Deserialize, Serialize)]
pub struct GetCollectionBookmarkData {
    pub collection_ids: String,
}

pub async fn get_collections_card_is_in(
    card_id: web::Json<GetCollectionBookmarkData>,
    pool: web::Data<Pool>,
    user: Option<LoggedUser>,
) -> Result<HttpResponse, actix_web::Error> {
    let collection_ids = card_id
        .collection_ids
        .split(',')
        .filter_map(|s| uuid::Uuid::parse_str(s.trim()).ok())
        .collect();

    let current_user_id = user.map(|user| user.id);
    if current_user_id.is_none() {
        return Err(ServiceError::Unauthorized.into());
    }

    let collections = web::block(move || {
        get_collections_for_bookmark_query(collection_ids, current_user_id, pool)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(collections))
}

#[derive(Deserialize)]
pub struct RemoveBookmarkData {
    pub card_metadata_id: uuid::Uuid,
}

pub async fn delete_bookmark(
    collection_id: web::Path<uuid::Uuid>,
    body: web::Json<RemoveBookmarkData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let collection_id = collection_id.into_inner();
    let bookmark_id = body.card_metadata_id;
    let thread_safe_pool = Arc::new(Mutex::new(pool));

    let pool_two = thread_safe_pool.clone();

    user_owns_collection(user.id, collection_id, thread_safe_pool).await?;

    web::block(move || delete_bookmark_query(bookmark_id, collection_id, pool_two.lock().unwrap()))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}
