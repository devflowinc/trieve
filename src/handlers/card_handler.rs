use actix_web::{web, HttpResponse};
use qdrant_client::prelude::*;
use qdrant_client::qdrant::PointStruct;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{errors::DefaultError, operators::card_operator::create_openai_embedding};

use super::auth_handler::LoggedUser;

pub async fn get_qdrant_connection() -> Result<QdrantClient, DefaultError> {
    let qdrant_url = std::env::var("QDRANT_URL").expect("QDRANT_URL must be set");
    QdrantClient::new(Some(QdrantClientConfig::from_url(qdrant_url.as_str())))
        .await
        .map_err(|_err| DefaultError {
            message: "Failed to connect to Qdrant",
        })
}

#[derive(Serialize, Deserialize)]
pub struct CreateCardData {
    content: String,
    side: String,
    topic: String,
    date: chrono::NaiveDateTime,
}

pub async fn create_card(
    card: web::Json<CreateCardData>,
    user: LoggedUser
) -> Result<HttpResponse, actix_web::Error> {
    
    let embedding_vector = create_openai_embedding(&card.content).await?;

    let qdrant = get_qdrant_connection()
        .await
        .map_err(|err| actix_web::error::ErrorBadRequest(err.message))?;

    // add some evidence card in CSV form to the vector db (Create route) (Route takes the card [side, topic, author, date, content], creates an embedding for only content, puts content's vector into the db]

    let payload: Payload = json!(
        {
            "content": card.content.clone(),
            "topic": card.topic,
            "side": card.side,
            "date": card.date,
            "user_id": user.id.to_string(),
        }
    )
    .try_into()
    .map_err(actix_web::error::ErrorBadRequest)?;

    let point = PointStruct::new(0, embedding_vector, payload);
    qdrant
        .upsert_points_blocking("debate_cards".to_string(), vec![point], None)
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    Ok(HttpResponse::NoContent().finish())
}
