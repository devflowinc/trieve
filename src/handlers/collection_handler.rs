use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::{
    data::models::{CardCollection, CardCollectionBookmark, Pool},
    operators::collection_operator::*,
};

use super::auth_handler::LoggedUser;

pub async fn user_owns_collection(
    user_id: uuid::Uuid,
    collection_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<CardCollection, actix_web::Error> {
    let collection = web::block(move || get_collection_by_id_query(collection_id, pool))
        .await?
        .map_err(actix_web::error::ErrorBadRequest)?;

    if collection.author_id != user_id {
        return Err(actix_web::error::ErrorBadRequest(
            "You are not the author of this collection",
        ));
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
    .map_err(actix_web::error::ErrorBadRequest)?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn get_card_collections(
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let collections = web::block(move || get_collections_for_user_query(user.id, pool))
        .await?
        .map_err(actix_web::error::ErrorBadRequest)?;

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
        .map_err(actix_web::error::ErrorBadRequest)?;

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
    .map_err(actix_web::error::ErrorBadRequest)?;

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
            CardCollectionBookmark {
                id: uuid::Uuid::new_v4(),
                collection_id,
                card_metadata_id,
                ..Default::default()
            },
        )
    })
    .await?
    .map_err(actix_web::error::ErrorBadRequest)?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn get_all_bookmarks(
    collection_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let collection_id = collection_id.into_inner();
    let pool_two = pool.clone();

    let collection = web::block(move || get_collection_by_id_query(collection_id, pool_two))
        .await?
        .map_err(actix_web::error::ErrorBadRequest)?;

    if !collection.is_public && collection.author_id != user.id {
        return Err(actix_web::error::ErrorUnauthorized(
            "You are not authorized to view this collection",
        ));
    }

    let bookmarks = web::block(move || {
        get_bookmarks_for_collection_query(collection_id, pool)
    }).await?.map_err(actix_web::error::ErrorBadRequest)?;

    Ok(HttpResponse::Ok().json(bookmarks))
}

#[derive(Deserialize)]
pub struct RemoveBookmarkData {
    pub bookmark_id: uuid::Uuid,
}

pub async fn delete_bookmark(
    collection_id: web::Path<uuid::Uuid>,
    body: web::Json<RemoveBookmarkData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let collection_id = collection_id.into_inner();
    let bookmark_id = body.bookmark_id;

    let pool_two = pool.clone();

    user_owns_collection(user.id, collection_id, pool_two).await?;

    web::block(move || {
        delete_bookmark_query(
            bookmark_id,
            pool,
        )
    })
    .await?
    .map_err(actix_web::error::ErrorBadRequest)?;

    Ok(HttpResponse::NoContent().finish())
}
