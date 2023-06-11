use actix_web::{web, HttpResponse};
use qdrant_client::qdrant::PointStruct;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::data::models::{CardMetadata, CardMetadataWithVotes, Pool};
use crate::errors::ServiceError;
use crate::operators::card_operator::{
    create_openai_embedding, get_card_count_query, get_metadata_from_point_ids,
    insert_card_metadata_query,
};
use crate::operators::card_operator::{
    get_metadata_from_id_query, get_qdrant_connection, search_card_query,
};

use super::auth_handler::LoggedUser;

#[derive(Serialize, Deserialize)]
pub struct CreateCardData {
    content: String,
    card_html: Option<String>,
    link: Option<String>,
    oc_file_path: Option<String>,
}

pub async fn create_card(
    card: web::Json<CreateCardData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let words_in_content = card.content.split(' ').collect::<Vec<&str>>().len();
    if words_in_content < 70 {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Card content must be at least 70 words long",
        })));
    }

    let embedding_vector = create_openai_embedding(&card.content).await?;

    let cards = search_card_query(embedding_vector.clone(), 1, pool.clone(), None, None)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    let first_result = cards.search_results.get(0);

    if let Some(score_card) = first_result {
        let mut similarity_threashold = 0.95;
        if card.content.len() < 200 {
            similarity_threashold = 0.9;
        }

        if score_card.score >= similarity_threashold {
            return Ok(HttpResponse::BadRequest().json(json!({
                "message": "Card already exists",
                "card": score_card,
            })));
        }
    }

    let qdrant = get_qdrant_connection()
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let payload: qdrant_client::prelude::Payload = json!({}).try_into().unwrap();

    let point_id = uuid::Uuid::new_v4();
    let point = PointStruct::new(point_id.clone().to_string(), embedding_vector, payload);

    web::block(move || {
        insert_card_metadata_query(
            CardMetadata::from_details(
                &card.content,
                &card.card_html,
                &card.link,
                &card.oc_file_path,
                user.id,
                point_id,
            ),
            &pool,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    qdrant
        .upsert_points_blocking("debate_cards".to_string(), vec![point], None)
        .await
        .map_err(|err| ServiceError::BadRequest("Failed inserting card to qdrant".into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize)]
pub struct SearchCardData {
    content: String,
    filter_oc_file_path: Option<Vec<String>>,
    filter_link_url: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct ScoreCardDTO {
    metadata: CardMetadataWithVotes,
    score: f32,
}

#[derive(Serialize, Deserialize)]
pub struct SearchCardQueryResponseBody {
    score_cards: Vec<ScoreCardDTO>,
    total_card_pages: i64,
}

pub async fn search_card(
    data: web::Json<SearchCardData>,
    page: Option<web::Path<u64>>,
    user: Option<LoggedUser>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    //search over the links as well
    let page = page.map(|page| page.into_inner()).unwrap_or(1);
    let embedding_vector = create_openai_embedding(&data.content).await?;
    let pool2 = pool.clone();
    let search_results_result = search_card_query(
        embedding_vector,
        page,
        pool,
        data.filter_oc_file_path.clone(),
        data.filter_link_url.clone(),
    )
    .await;

    let search_card_query_results = match search_results_result {
        Ok(results) => results,
        Err(err) => return Ok(HttpResponse::BadRequest().json(err)),
    };

    let point_ids = search_card_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();

    let current_user_id = user.map(|user| user.id);
    let metadata_cards =
        web::block(move || get_metadata_from_point_ids(point_ids, current_user_id, pool2))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let score_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let card = metadata_cards
                .iter()
                .find(|metadata_card| metadata_card.qdrant_point_id == search_result.point_id)
                .unwrap();

            ScoreCardDTO {
                metadata: (*card).clone(),
                score: search_result.score,
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(SearchCardQueryResponseBody {
        score_cards,
        total_card_pages: search_card_query_results.total_card_pages,
    }))
}

pub async fn get_card_by_id(
    card_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let card = web::block(|| get_metadata_from_id_query(card_id.into_inner(), pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(card))
}

pub async fn get_total_card_count(pool: web::Data<Pool>) -> Result<HttpResponse, actix_web::Error> {
    let total_count = web::block(move || get_card_count_query(&pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(json!({ "total_count": total_count })))
}
