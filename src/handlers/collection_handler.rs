use actix_web::{web, HttpResponse};
use serde::Deserialize;

use crate::{
    data::models::{CardCollection, Pool},
    operators::collection_operator::create_collection_operation,
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
        create_collection_operation(
            CardCollection::from_details(user.id, name, is_public, description),
            pool,
        )
    })
    .await?.map_err(actix_web::error::ErrorBadRequest)?;

    Ok(HttpResponse::NoContent().finish())
}
