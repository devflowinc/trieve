use actix_web::{web, HttpResponse};
use qdrant_client::qdrant::point_id::PointIdOptions;
use qdrant_client::qdrant::value::Kind;
use qdrant_client::qdrant::with_payload_selector::SelectorOptions;
use qdrant_client::qdrant::{PayloadIncludeSelector, PointStruct};
use qdrant_client::{prelude::*, qdrant::WithPayloadSelector};
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
}

pub async fn create_card(
    card: web::Json<CreateCardData>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let embedding_vector = create_openai_embedding(&card.content).await?;

    let qdrant = get_qdrant_connection()
        .await
        .map_err(|err| actix_web::error::ErrorBadRequest(err.message))?;

    // add some evidence card in CSV form to the vector db (Create route) (Route takes the card [side, topic, author, date, content], creates an embedding for only content, puts content's vector into the db]

    let payload: Payload = json!(
        {
            "content": card.content.clone(),
            "topic": card.topic.clone(),
            "side": card.side.clone(),
            "user_id": user.id.to_string(),
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

#[derive(Serialize, Deserialize)]
pub struct CardDTO {
    id: String,
    content: String,
    side: String,
    topic: String,
    score: f32,
}

pub async fn search_card(
    data: web::Json<SearchCardData>,
) -> Result<HttpResponse, actix_web::Error> {
    let embedding_vector = create_openai_embedding(&data.content).await?;

    let qdrant = get_qdrant_connection()
        .await
        .map_err(|err| actix_web::error::ErrorBadRequest(err.message))?;

    let data = qdrant
        .search_points(&SearchPoints {
            collection_name: "debate_cards".to_string(),
            vector: embedding_vector,
            filter: None,
            limit: 10,
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(SelectorOptions::Include(PayloadIncludeSelector {
                    fields: vec![
                        "content".to_string(),
                        "side".to_string(),
                        "topic".to_string(),
                        "user_id".to_string(),
                    ],
                })),
            }),
            with_vectors: None,
            params: None,
            score_threshold: None,
            offset: None,
            ..Default::default()
        })
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    let cards: Vec<CardDTO> = data
        .result
        .iter()
        .filter_map(|point| {
            let id = match point.id.clone()?.point_id_options? {
                PointIdOptions::Num(n) => n.to_string(),
                PointIdOptions::Uuid(s) => s,
            };
            let content = point.payload.get("content")?;
            let side = point.payload.get("side")?;
            let topic = point.payload.get("topic")?;
            let score = point.score;

            match (&content.kind, &side.kind, &topic.kind) {
                (
                    Some(Kind::StringValue(content)),
                    Some(Kind::StringValue(side)),
                    Some(Kind::StringValue(topic)),
                ) => Some(CardDTO {
                    id,
                    content: content.clone(),
                    side: side.clone(),
                    topic: topic.clone(),
                    score,
                }),
                (_, _, _) => None,
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(cards))
}
