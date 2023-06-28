use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::{
    data::models::{CardCollection, CardCollectionBookmark, CardMetadataWithVotesAndFiles, Pool},
    errors::ServiceError,
    operators::collection_operator::*,
};

use super::auth_handler::LoggedUser;
//new handler and operator to get collections a card is in

pub async fn user_owns_collection(
    user_id: uuid::Uuid,
    collection_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<CardCollection, actix_web::Error> {
    let collection = web::block(move || get_collection_by_id_query(collection_id, pool))
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

pub async fn get_specific_user_card_collections(
    user: Option<LoggedUser>,
    user_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let accessing_user_id = user.map(|user| user.id);
    let user_id = user_id.into_inner();
    let collections = web::block(move || {
        get_collections_for_specifc_user_query(user_id, accessing_user_id, pool)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(collections))
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
    let pool_two = pool.clone();
    let collection_id = data.collection_id;

    user_owns_collection(user.id, collection_id, pool_two).await?;

    web::block(move || delete_collection_by_id_query(collection_id, pool))
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

    let pool_two = pool.clone();

    let collection = user_owns_collection(user.id, collection_id, pool_two).await?;

    web::block(move || {
        update_card_collection_query(collection, name, description, is_public, pool)
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
    let card_metadata_id = body.card_metadata_id;
    let collection_id = collection_id.into_inner();

    user_owns_collection(user.id, collection_id, pool.clone()).await?;

    web::block(move || {
        create_card_bookmark_query(
            pool,
            CardCollectionBookmark::from_details(collection_id, card_metadata_id),
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}
#[derive(Deserialize, Serialize)]
pub struct BookmarkData {
    pub bookmarks: Vec<CardMetadataWithVotesAndFiles>,
    pub collection: CardCollection,
}

pub async fn get_all_bookmarks(
    collection_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: Option<LoggedUser>,
) -> Result<HttpResponse, actix_web::Error> {
    let collection_id = collection_id.into_inner();
    let pool_two = pool.clone();
    let current_user_id = user.map(|user| user.id);

    let collection = web::block(move || get_collection_by_id_query(collection_id, pool_two))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    if !collection.is_public && current_user_id.is_none() {
        return Err(ServiceError::Unauthorized.into());
    }
    if !collection.is_public && Some(collection.author_id) != current_user_id {
        return Err(ServiceError::Forbidden.into());
    }

    let bookmarks = web::block(move || {
        get_bookmarks_for_collection_query(collection_id, current_user_id, pool)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(BookmarkData {
        bookmarks,
        collection,
    }))
}

pub async fn get_collections_card_is_in(
    card_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: Option<LoggedUser>,
) -> Result<HttpResponse, actix_web::Error> {
    let collection_id = card_id.into_inner();
    let pool_two = pool.clone();
    let current_user_id = user.map(|user| user.id);
    if current_user_id.is_none() {
        return Err(ServiceError::Unauthorized.into());
    }

    let collections = web::block(move || {
        get_collections_for_bookmark_query(collection_id, current_user_id, pool_two)
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

    let pool_two = pool.clone();

    user_owns_collection(user.id, collection_id, pool_two).await?;

    web::block(move || delete_bookmark_query(bookmark_id, collection_id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}
