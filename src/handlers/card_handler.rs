use crate::data::models::{
    CardMetadata, CardMetadataWithVotesAndFiles, CardMetadataWithVotesWithoutScore, Pool,
};
use crate::errors::ServiceError;
use crate::operators::card_operator::*;
use crate::operators::card_operator::{
    get_metadata_from_id_query, get_qdrant_connection, search_card_query,
};
use crate::operators::collection_operator::get_collection_by_id_query;
use actix_web::{web, HttpResponse};
use difference::{Changeset, Difference};
use qdrant_client::qdrant::points_selector::PointsSelectorOneOf;
use qdrant_client::qdrant::{PointStruct, PointsIdsList, PointsSelector};
use serde::{Deserialize, Serialize};
use serde_json::json;
use soup::Soup;

use super::auth_handler::LoggedUser;

pub async fn user_owns_card(
    user_id: uuid::Uuid,
    card_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<CardMetadata, actix_web::Error> {
    let cards = web::block(move || get_metadata_from_id_query(card_id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if cards.author_id != user_id {
        return Err(ServiceError::Forbidden.into());
    }

    Ok(cards)
}

#[derive(Serialize, Deserialize)]
pub struct CreateCardData {
    pub content: String,
    pub card_html: Option<String>,
    pub link: Option<String>,
    pub oc_file_path: Option<String>,
    pub private: Option<bool>,
    pub file_uuid: Option<uuid::Uuid>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReturnCreatedCard {
    pub card_metadata: CardMetadata,
    pub duplicate: bool,
}

pub async fn create_card(
    card: web::Json<CreateCardData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let private = card.private.unwrap_or(false);
    let mut collision: Option<uuid::Uuid> = None;
    let mut embedding_vector: Option<Vec<f32>> = None;

    let pool1 = pool.clone();

    let words_in_content = card.content.split(' ').collect::<Vec<&str>>().len();
    if words_in_content < 70 {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Card content must be at least 70 words long",
        })));
    }

    // text based similarity check to avoid paying for openai api call if not necessary
    let card_content_1 = card.content.clone();
    let text_based_similarity_results = web::block(move || {
        search_full_text_card_query(card_content_1, 1, pool.clone(), Some(user.id), None, None)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    let first_text_result = text_based_similarity_results.search_results.get(0);

    if let Some(score_card) = first_text_result {
        if score_card.score >= Some(0.95) {
            //Sets collision to collided card id
            collision = Some(score_card.qdrant_point_id);
        }
    }

    // only check for embedding similarity if no text based collision was found
    if collision.is_none() {
        let openai_embedding_vector = create_openai_embedding(&card.content).await?;
        embedding_vector = Some(openai_embedding_vector.clone());

        let cards = search_card_query(
            openai_embedding_vector.clone(),
            1,
            pool1.clone(),
            None,
            None,
        )
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
        let first_result = cards.search_results.get(0);

        if let Some(score_card) = first_result {
            let mut similarity_threashold = 0.95;
            if card.content.len() < 200 {
                similarity_threashold = 0.92;
            }

            if score_card.score >= similarity_threashold {
                //Sets collision to collided card id
                collision = Some(score_card.point_id);
            }
        }
    }

    let mut card_metadata: CardMetadata;
    let mut duplicate: bool = false;

    //if collision is not nil, insert card with collision
    if collision.is_some() {
        card_metadata = CardMetadata::from_details(
            &card.content,
            &card.card_html,
            &card.link,
            &card.oc_file_path,
            user.id,
            None,
            private,
        );
        card_metadata = web::block(move || {
            insert_duplicate_card_metadata_query(
                card_metadata,
                collision.unwrap(),
                card.file_uuid,
                &pool1,
            )
        })
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

        duplicate = true;
    }
    //if collision is nil and embedding vector is some, insert card with no collision
    else {
        // if this statement is reached, the embedding vector must be some
        let ensured_embedding_vector = match embedding_vector {
            Some(embedding_vector) => embedding_vector,
            None => {
                return Ok(HttpResponse::BadRequest().json(json!({
                    "message": "Could not create embedding vector",
                })))
            }
        };

        let qdrant = get_qdrant_connection()
            .await
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
        //if private is true, set payload to private
        let payload = match private {
            true => json!({"private": true}).try_into().unwrap(),
            false => json!({}).try_into().unwrap(),
        };

        let point_id = uuid::Uuid::new_v4();
        let point = PointStruct::new(
            point_id.clone().to_string(),
            ensured_embedding_vector,
            payload,
        );
        card_metadata = CardMetadata::from_details(
            &card.content,
            &card.card_html,
            &card.link,
            &card.oc_file_path,
            user.id,
            Some(point_id),
            private,
        );
        card_metadata =
            web::block(move || insert_card_metadata_query(card_metadata, card.file_uuid, &pool1))
                .await?
                .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

        qdrant
            .upsert_points_blocking("debate_cards".to_string(), vec![point], None)
            .await
            .map_err(|_err| ServiceError::BadRequest("Failed inserting card to qdrant".into()))?;
    }

    Ok(HttpResponse::Ok().json(ReturnCreatedCard {
        card_metadata,
        duplicate,
    }))
}

pub async fn delete_card(
    card_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let card_id_inner = card_id.into_inner();
    let pool1 = pool.clone();

    let card_metadata = user_owns_card(user.id, card_id_inner, pool1).await?;

    let qdrant = get_qdrant_connection()
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let deleted_values = PointsSelector {
        points_selector_one_of: Some(PointsSelectorOneOf::Points(PointsIdsList {
            ids: vec![card_metadata
                .qdrant_point_id
                .unwrap_or(uuid::Uuid::nil())
                .to_string()
                .into()],
        })),
    };

    web::block(move || delete_card_metadata_query(&card_id_inner, &pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    qdrant
        .delete_points_blocking("debate_cards".to_string(), &deleted_values, None)
        .await
        .map_err(|_err| ServiceError::BadRequest("Failed deleting card from qdrant".into()))?;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UpdateCardData {
    card_uuid: uuid::Uuid,
    link: Option<String>,
    card_html: Option<String>,
    private: Option<bool>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct CardHtmlUpdateError {
    pub message: String,
    changed_content: String,
}
pub async fn update_card(
    card: web::Json<UpdateCardData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let pool1 = pool.clone();
    let card_metadata = user_owns_card(user.id, card.card_uuid, pool1).await?;

    let link = card
        .link
        .clone()
        .unwrap_or_else(|| card_metadata.link.unwrap_or_default());

    let soup = Soup::new(card.card_html.as_ref().unwrap_or(&"".to_string()).as_str());
    if soup.text() != card_metadata.content && card_metadata.card_html.is_some() {
        let soup_text_ref = soup.text();
        let Changeset { diffs, .. } = Changeset::new(&card_metadata.content, &soup_text_ref, " ");
        let mut ret: String = Default::default();
        for diff in diffs {
            match diff {
                Difference::Same(ref x) => {
                    ret += format!(" {}", x).as_str();
                }
                Difference::Add(ref x) => {
                    ret += format!("++++{}", x).as_str();
                }
                Difference::Rem(ref x) => {
                    ret += format!("----{}", x).as_str();
                }
            }
        }

        return Ok(HttpResponse::BadRequest().json(CardHtmlUpdateError {
            message: "Card content has changed".to_string(),
            changed_content: ret,
        }));
    }

    let card_html = match card.card_html.clone() {
        Some(card_html) => Some(card_html),
        None => card_metadata.card_html,
    };

    if card_metadata.private
        && !card.private.unwrap_or(true)
        && card_metadata.qdrant_point_id.is_none()
    {
        return Err(ServiceError::BadRequest("Cannot make a duplicate card public".into()).into());
    }
    let private = card.private.unwrap_or(card_metadata.private);

    web::block(move || {
        update_card_metadata_query(
            CardMetadata::from_details_with_id(
                card.card_uuid,
                &card_metadata.content,
                &card_html,
                &Some(link),
                &card_metadata.oc_file_path,
                user.id,
                card_metadata.qdrant_point_id,
                private,
            ),
            &pool,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

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
    metadata: Vec<CardMetadataWithVotesWithoutScore>,
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
    let pool3 = pool.clone();

    let search_card_query_results = search_card_query(
        embedding_vector,
        page,
        pool,
        data.filter_oc_file_path.clone(),
        data.filter_link_url.clone(),
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let point_ids = search_card_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();
    let point_ids_1 = point_ids.clone();

    let current_user_id = user.map(|user| user.id);
    let metadata_cards =
        web::block(move || get_metadata_from_point_ids(point_ids, current_user_id, pool2))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let collided_cards =
        web::block(move || get_collided_cards_query(point_ids_1, current_user_id, pool3))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let score_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let card: CardMetadataWithVotesWithoutScore = <CardMetadataWithVotesAndFiles as Into<
                CardMetadataWithVotesWithoutScore,
            >>::into(
                metadata_cards
                    .iter()
                    .find(|metadata_card| metadata_card.qdrant_point_id == search_result.point_id)
                    .unwrap()
                    .clone(),
            );

            let mut collided_cards: Vec<CardMetadataWithVotesWithoutScore> = collided_cards
                .iter()
                .filter(|card| card.1 == search_result.point_id)
                .map(|card| card.0.clone().into())
                .collect();

            collided_cards.insert(0, card);

            ScoreCardDTO {
                metadata: collided_cards,
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
    let pool2 = pool.clone();
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

    let point_ids = search_card_query_results
        .search_results
        .iter()
        .map(|point| point.qdrant_point_id)
        .collect::<Vec<uuid::Uuid>>();

    let collided_cards =
        web::block(move || get_collided_cards_query(point_ids, current_user_id, pool2))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let full_text_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut collided_cards: Vec<CardMetadataWithVotesWithoutScore> = collided_cards
                .iter()
                .filter(|card| card.1 == search_result.qdrant_point_id)
                .map(|card| card.0.clone().into())
                .collect();

            collided_cards.insert(0, search_result.clone().into());

            ScoreCardDTO {
                metadata: collided_cards,
                score: search_result.score.unwrap_or(0.0),
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(SearchCardQueryResponseBody {
        score_cards: full_text_cards,
        total_card_pages: search_card_query_results.total_card_pages,
    }))
}

#[derive(Serialize, Deserialize)]
pub struct SearchCollectionsData {
    content: String,
    filter_oc_file_path: Option<Vec<String>>,
    filter_link_url: Option<Vec<String>>,
    collection_id: uuid::Uuid,
}

pub async fn search_collections(
    data: web::Json<SearchCollectionsData>,
    page: Option<web::Path<u64>>,
    user: Option<LoggedUser>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    //search over the links as well
    let page = page.map(|page| page.into_inner()).unwrap_or(1);
    let embedding_vector = create_openai_embedding(&data.content).await?;
    let collection_id = data.collection_id;
    let pool2 = pool.clone();
    let pool3 = pool.clone();
    let pool4 = pool.clone();
    let current_user_id = user.map(|user| user.id);
    let collection = web::block(move || get_collection_by_id_query(collection_id, pool3))
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if !collection.is_public && current_user_id.is_none() {
        return Err(ServiceError::Unauthorized.into());
    }

    if !collection.is_public && Some(collection.author_id) != current_user_id {
        return Err(ServiceError::Forbidden.into());
    }

    let search_card_query_results = search_card_collections_query(
        embedding_vector,
        page,
        pool,
        data.filter_oc_file_path.clone(),
        data.filter_link_url.clone(),
        data.collection_id,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let point_ids = search_card_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();

    let point_ids_1 = point_ids.clone();

    let metadata_cards =
        web::block(move || get_metadata_from_point_ids(point_ids, current_user_id, pool2))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let collided_cards =
        web::block(move || get_collided_cards_query(point_ids_1, current_user_id, pool4))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let score_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let card: CardMetadataWithVotesWithoutScore = <CardMetadataWithVotesAndFiles as Into<
                CardMetadataWithVotesWithoutScore,
            >>::into(
                metadata_cards
                    .iter()
                    .find(|metadata_card| metadata_card.qdrant_point_id == search_result.point_id)
                    .unwrap()
                    .clone(),
            );

            let mut collided_cards: Vec<CardMetadataWithVotesWithoutScore> = collided_cards
                .iter()
                .filter(|card| card.1 == search_result.point_id)
                .map(|card| card.0.clone().into())
                .collect();

            collided_cards.insert(0, card);

            ScoreCardDTO {
                metadata: collided_cards,
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
    user: Option<LoggedUser>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let current_user_id = user.map(|user| user.id);
    let card = web::block(move || {
        get_metadata_and_votes_from_id_query(card_id.into_inner(), current_user_id, pool)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    if card.private && current_user_id.is_none() {
        return Err(ServiceError::Unauthorized.into());
    }
    if card.private && Some(card.clone().author.unwrap().id) != current_user_id {
        return Err(ServiceError::Forbidden.into());
    }
    Ok(HttpResponse::Ok().json(card))
}

pub async fn get_total_card_count(pool: web::Data<Pool>) -> Result<HttpResponse, actix_web::Error> {
    let total_count = web::block(move || get_card_count_query(&pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(json!({ "total_count": total_count })))
}
