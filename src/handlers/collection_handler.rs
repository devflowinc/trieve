use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::{
    data::models::{CardCollection, Pool},
    operators::collection_operator::{create_collection_query, get_collections_for_user_query},
};

use super::auth_handler::LoggedUser;

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
    .await?.map_err(actix_web::error::ErrorBadRequest)?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn get_collections(
    user: LoggedUser,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let collections = web::block(move || {
        get_collections_for_user_query(user.id, pool)
    })
    .await?
    .map_err(actix_web::error::ErrorBadRequest)?;

    Ok(HttpResponse::Ok().json(collections))
}
