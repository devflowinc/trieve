use actix_web::{web, HttpResponse};
use qdrant_client::qdrant::PointStruct;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::operators::card_operator::{get_qdrant_connection, search_card_query};
use crate::operators::card_operator::create_openai_embedding;

use super::auth_handler::LoggedUser;

#[derive(Serialize, Deserialize)]
pub struct CreateCardData {
    content: String,
    side: String,
    topic: String,
    link: Option<String>,
}

pub async fn create_card(
    card: web::Json<CreateCardData>,
    user: Option<LoggedUser>,
) -> Result<HttpResponse, actix_web::Error> {
    let embedding_vector = create_openai_embedding(&card.content).await?;

    let qdrant = get_qdrant_connection()
        .await
        .map_err(|err| actix_web::error::ErrorBadRequest(err.message))?;

    let id_str = user.map(|user| user.id.to_string()).unwrap_or("".to_string());

    let payload: qdrant_client::prelude::Payload = json!(
        {
            "content": card.content.clone(),
            "topic": card.topic.clone(),
            "side": card.side.clone(),
            "user_id": id_str,
            "link": card.link,
            "upvotes": 0,
            "downvotes": 0,
            "created_at": chrono::Utc::now().to_rfc3339(),
        }
    )
    .try_into()
    .map_err(actix_web::error::ErrorBadRequest)?;

    let point = PointStruct::new(uuid::Uuid::new_v4().to_string(), embedding_vector, payload);
    qdrant
        .upsert_points_blocking("debate_cards".to_string(), vec![point], None)
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize)]
pub struct SearchCardData {
    content: String,
}

pub async fn search_card(
    data: web::Json<SearchCardData>,
) -> Result<HttpResponse, actix_web::Error> {
    let embedding_vector = create_openai_embedding(&data.content).await?;

    let cards = search_card_query(embedding_vector).await?;


    Ok(HttpResponse::Ok().json(cards))
}
