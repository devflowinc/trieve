use actix_web::{web, HttpResponse};
use qdrant_client::qdrant::PointStruct;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::data::models::{
    CardMetadata, CardMetadataWithVotes, CardMetadataWithVotesWithoutScore, Pool,
};
use crate::errors::DefaultError;
use crate::operators::card_operator::{
    create_openai_embedding, get_card_count_query, get_metadata_from_point_ids,
    insert_card_metadata_query, search_full_text_card_query,
    update_card_html_by_qdrant_point_id_query,
};
use crate::operators::card_operator::{
    get_metadata_from_id_query, get_qdrant_connection, search_card_query,
};

use super::auth_handler::LoggedUser;

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateCardData {
    pub content: String,
    pub card_html: Option<String>,
    pub link: Option<String>,
    pub(crate) oc_file_path: Option<String>,
}

pub async fn create_card(
    card: web::Json<CreateCardData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let content_clone = card.content.clone();
    let content_clone_two = card.content.clone();
    let pool_one = pool.clone();
    let pool_two = pool.clone();
    let pool_three = pool.clone();

    let words_in_content = card.content.split(' ').collect::<Vec<&str>>();
    if words_in_content.len() < 70 {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Card content must be at least 70 words long",
        })));
    }

    let check_similarity = |qdrant_point_id: uuid::Uuid, similarity_score: f64, threshold: f64| {
        let lambda_card_content_clone = card.content.clone();
        let card_html_clone = card.card_html.clone();
        let pool_clone = pool.clone();

        let mut similarity_threshold = threshold;
        if lambda_card_content_clone.len() < 200 {
            similarity_threshold -= 0.05;
        }

        if similarity_score >= similarity_threshold {
            let _ = web::block(move || {
                update_card_html_by_qdrant_point_id_query(
                    &qdrant_point_id,
                    &card_html_clone,
                    &pool_clone,
                )
            });

            return Err(DefaultError {
                message: "Card already exists",
            });
        }

        Ok(())
    };

    let pg_similiarity_result = web::block(move || {
        search_full_text_card_query(content_clone, 1, pool_one.clone(), None, None, None)
    })
    .await?
    .map_err(|e| actix_web::error::ErrorBadRequest(e.message))?;

    match pg_similiarity_result.search_results.get(0) {
        Some(result_ref) => {
            match check_similarity(
                result_ref.qdrant_point_id.clone(),
                result_ref.score.unwrap_or(0.0).into(),
                0.9,
            ) {
                Ok(_) => {}
                Err(e) => {
                    return Ok(HttpResponse::BadRequest().json(e));
                }
            }
        }
        None => {}
    }

    let embedding_vector = create_openai_embedding(&content_clone_two).await?;

    let cards = search_card_query(embedding_vector.clone(), 1, pool_two, None, None)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.message))?;

    match cards.search_results.get(0) {
        Some(result_ref) => {
            match check_similarity(result_ref.point_id.clone(), result_ref.score.into(), 0.95) {
                Ok(_) => {}
                Err(e) => {
                    return Ok(HttpResponse::BadRequest().json(e));
                }
            }
        }
        None => {}
    }

    let qdrant = get_qdrant_connection()
        .await
        .map_err(|err| actix_web::error::ErrorBadRequest(err.message))?;

    let payload: qdrant_client::prelude::Payload = json!({}).try_into().unwrap();

    let point_id = uuid::Uuid::new_v4();
    let point = PointStruct::new(point_id.clone().to_string(), embedding_vector, payload);
    let card_clone = card.clone();

    web::block(move || {
        insert_card_metadata_query(
            CardMetadata::from_details(
                &card_clone.content,
                &card_clone.card_html,
                &card_clone.link,
                &card_clone.oc_file_path,
                user.id,
                point_id,
            ),
            &pool_three,
        )
    })
    .await?
    .map_err(actix_web::error::ErrorBadRequest)?;

    qdrant
        .upsert_points_blocking("debate_cards".to_string(), vec![point], None)
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

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
    metadata: CardMetadataWithVotesWithoutScore,
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
            .map_err(actix_web::error::ErrorBadRequest)?;

    let score_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let card = metadata_cards
                .iter()
                .find(|metadata_card| metadata_card.qdrant_point_id == search_result.point_id)
                .unwrap();

            ScoreCardDTO {
                metadata: <CardMetadataWithVotes as Into<CardMetadataWithVotesWithoutScore>>::into(
                    (*card).clone(),
                ),
                score: search_result.score,
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(SearchCardQueryResponseBody {
        score_cards,
        total_card_pages: search_card_query_results.total_card_pages,
    }))
}

pub async fn search_full_text_card(
    data: web::Json<SearchCardData>,
    page: Option<web::Path<u64>>,
    user: Option<LoggedUser>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    //search over the links as well
    let page = page.map(|page| page.into_inner()).unwrap_or(1);
    let current_user_id = user.map(|user| user.id);
    let search_results_result = search_full_text_card_query(
        data.content.clone(),
        page,
        pool,
        current_user_id,
        data.filter_oc_file_path.clone(),
        data.filter_link_url.clone(),
    );

    let search_card_query_results = match search_results_result {
        Ok(results) => results,
        Err(err) => return Ok(HttpResponse::BadRequest().json(err)),
    };

    let full_text_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| ScoreCardDTO {
            metadata: <CardMetadataWithVotes as Into<CardMetadataWithVotesWithoutScore>>::into(
                search_result.clone(),
            ),
            score: search_result.score.unwrap_or(0.0),
        })
        .collect();

    Ok(HttpResponse::Ok().json(SearchCardQueryResponseBody {
        score_cards: full_text_cards,
        total_card_pages: search_card_query_results.total_card_pages,
    }))
}

pub async fn get_card_by_id(
    card_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let card = web::block(|| get_metadata_from_id_query(card_id.into_inner(), pool))
        .await?
        .map_err(actix_web::error::ErrorBadRequest)?;

    Ok(HttpResponse::Ok().json(card))
}

pub async fn get_total_card_count(pool: web::Data<Pool>) -> Result<HttpResponse, actix_web::Error> {
    let total_count = web::block(move || get_card_count_query(&pool))
        .await?
        .map_err(actix_web::error::ErrorBadRequest)?;

    Ok(HttpResponse::Ok().json(json!({ "total_count": total_count })))
}
