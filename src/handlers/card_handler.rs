use std::collections::HashSet;
use std::process::Command;
use std::str::FromStr;

use crate::data::models::{
    CardCollection, CardMetadata, CardMetadataWithVotesAndFiles, CardMetadataWithVotesWithScore,
    Pool, UserDTO,
};
use crate::errors::{DefaultError, ServiceError};
use crate::operators::card_operator::*;
use crate::operators::card_operator::{
    get_metadata_from_id_query, get_qdrant_connection, search_card_query,
};
use crate::operators::collection_operator::get_collection_by_id_query;
use crate::operators::qdrant_operator::{
    create_new_qdrant_point_query, delete_qdrant_point_id_query, update_qdrant_point_private_query,
};
use crate::AppMutexStore;
use actix_web::{web, HttpResponse};
use difference::{Changeset, Difference};
use qdrant_client::qdrant::points_selector::PointsSelectorOneOf;
use qdrant_client::qdrant::{PointsIdsList, PointsSelector};
use redis::Commands;
use serde::{Deserialize, Serialize};
use serde_json::json;

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
    pub card_html: Option<String>,
    pub link: Option<String>,
    pub oc_file_path: Option<String>,
    pub private: Option<bool>,
    pub file_uuid: Option<uuid::Uuid>,
}

pub fn convert_html(html: &str) -> String {
    let html_parse_result = Command::new("node")
        .arg("./vault-nodejs/scripts/html-converter.js")
        .arg("-html")
        .arg(html)
        .output();

    let content = match html_parse_result {
        Ok(result) => {
            if result.status.success() {
                Some(
                    String::from_utf8(result.stdout)
                        .unwrap()
                        .lines()
                        .collect::<Vec<&str>>()
                        .join(" ")
                        .trim_end()
                        .to_string(),
                )
            } else {
                return "".to_string();
            }
        }
        Err(_) => {
            return "".to_string();
        }
    };

    match content {
        Some(content) => content,
        None => "".to_string(),
    }
}
#[derive(Serialize, Deserialize, Clone)]
pub struct ReturnCreatedCard {
    pub card_metadata: CardMetadata,
    pub duplicate: bool,
}

pub async fn create_card(
    card: web::Json<CreateCardData>,
    pool: web::Data<Pool>,
    mutex_store: web::Data<AppMutexStore>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let private = card.private.unwrap_or(false);
    let card_oc_file_path = card.oc_file_path.clone();
    let mut collision: Option<uuid::Uuid> = None;
    let mut embedding_vector: Option<Vec<f32>> = None;

    if card_oc_file_path.unwrap_or("".to_string()) != "" {
        let admin_uuid: String = std::env::var("ADMIN_UUID").expect("ADMIN_UUID must be set");

        if user.id != uuid::Uuid::from_str(&admin_uuid).unwrap_or(uuid::Uuid::nil()) {
            return Ok(HttpResponse::Forbidden().json(DefaultError {
                message: "Only admin can create cards with oc_file_path",
            }));
        }
    }

    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let pool3 = pool.clone();

    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let client = redis::Client::open(redis_url)
        .map_err(|err| ServiceError::BadRequest(format!("Could not connect to redis: {}", err)))?;
    let mut con = client
        .get_connection()
        .map_err(|err| ServiceError::BadRequest(format!("Could not connect to redis: {}", err)))?;

    let content = convert_html(card.card_html.as_ref().unwrap_or(&"".to_string()));
    // Card content can be at most 29000 characters long
    if cfg!(feature = "minimum-length") && content.len() > 29000 {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Card content must be at most 29000 characters long",
        })));
    }

    let words_in_content = content.split(' ').collect::<Vec<&str>>().len();
    if cfg!(feature = "minimum-length") && words_in_content < 70 {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Card content must be at least 70 words long",
        })));
    }
    if cfg!(feature = "minimum-length") && words_in_content > 5000 {
        return Ok(HttpResponse::BadRequest().json(json!({
            "message": "Card content must be at most 5000 words long",
        })));
    }

    // // text based similarity check to avoid paying for openai api call if not necessary
    let card_content_1 = content.clone();
    let first_text_result =
        web::block(move || global_top_full_text_card_query(card_content_1, pool))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if let Some(score_card) = first_text_result {
        if score_card.score >= Some(4.99) {
            //Sets collision to collided card id
            collision = Some(score_card.qdrant_point_id);

            if score_card.card_html.is_none() {
                let score_card_1 = score_card.clone();
                let card_metadata = CardMetadata::from_details_with_id(
                    score_card_1.id,
                    &content,
                    &card.card_html,
                    &card.link,
                    &card.oc_file_path,
                    score_card_1
                        .author
                        .clone()
                        .unwrap_or(UserDTO {
                            id: uuid::Uuid::default(),
                            email: None,
                            website: None,
                            username: None,
                            visible_email: false,
                            created_at: chrono::NaiveDateTime::default(),
                        })
                        .id,
                    Some(score_card_1.qdrant_point_id),
                    card.private.unwrap_or(score_card_1.private),
                );
                let metadata_1 = card_metadata.clone();
                web::block(move || {
                    update_card_metadata_query(card_metadata, card.file_uuid, pool3)
                })
                .await?
                .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

                con.set(format!("Verify: {}", metadata_1.id), true)
                    .map_err(|err| {
                        ServiceError::BadRequest(format!("Could not set redis key: {}", err))
                    })?;

                return Ok(HttpResponse::Ok().json(ReturnCreatedCard {
                    card_metadata: metadata_1,
                    duplicate: true,
                }));
            }
        }
    }

    // only check for embedding similarity if no text based collision was found
    if collision.is_none() {
        let openai_embedding_vector = create_embedding(&content, mutex_store).await?;
        embedding_vector = Some(openai_embedding_vector.clone());

        let first_semantic_result =
            global_unfiltered_top_match_query(openai_embedding_vector.clone())
                .await
                .map_err(|err| {
                    ServiceError::BadRequest(format!(
                        "Could not get semantic similarity for collision check: {}",
                        err.message
                    ))
                })?;

        let mut similarity_threshold = 0.95;
        if content.len() < 200 {
            similarity_threshold = 0.92;
        }

        if first_semantic_result.score >= similarity_threshold {
            //Sets collision to collided card id
            collision = Some(first_semantic_result.point_id);

            let score_card_result = web::block(move || {
                get_metadata_from_point_ids(
                    vec![first_semantic_result.point_id],
                    Some(user.id),
                    pool2,
                )
            })
            .await?;

            let top_score_card = match score_card_result {
                Ok(card_results) => {
                    if card_results.is_empty() {
                        delete_qdrant_point_id_query(first_semantic_result.point_id)
                            .await
                            .map_err(|_| {
                                ServiceError::BadRequest(
                                    "Could not delete qdrant point id. Please try again.".into(),
                                )
                            })?;

                        return Err(ServiceError::BadRequest(
                            "There was a data inconsistency issue. Please try again.".into(),
                        )
                        .into());
                    }
                    card_results.get(0).unwrap().clone()
                }
                Err(err) => {
                    return Err(ServiceError::BadRequest(err.message.into()).into());
                }
            };

            let top_score_card_author_id = match top_score_card.author.clone() {
                Some(author) => author.id,
                None => {
                    return Err(ServiceError::BadRequest(
                        "Could not find card with matching point id".into(),
                    )
                    .into())
                }
            };

            if top_score_card.card_html.is_none() {
                let card_metadata = CardMetadata::from_details_with_id(
                    top_score_card.id,
                    &content,
                    &card.card_html,
                    &card.link,
                    &card.oc_file_path,
                    top_score_card_author_id,
                    Some(top_score_card.qdrant_point_id),
                    top_score_card.private,
                );
                let metadata_1 = card_metadata.clone();

                web::block(move || {
                    update_card_metadata_query(card_metadata, card.file_uuid, pool3)
                })
                .await?
                .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

                con.set(format!("Verify: {}", metadata_1.id), true)
                    .map_err(|err| {
                        ServiceError::BadRequest(format!("Could not set redis key: {}", err))
                    })?;

                return Ok(HttpResponse::Ok().json(ReturnCreatedCard {
                    card_metadata: metadata_1,
                    duplicate: true,
                }));
            }
        }
    }

    let mut card_metadata: CardMetadata;
    let mut duplicate: bool = false;

    //if collision is not nil, insert card with collision
    if collision.is_some() {
        update_qdrant_point_private_query(
            collision.expect("Collision must be some"),
            private,
            Some(user.id),
        )
        .await?;

        card_metadata = CardMetadata::from_details(
            &content,
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
                collision.expect("Collision should must be some"),
                card.file_uuid,
                pool1,
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

        let qdrant_point_id = uuid::Uuid::new_v4();
        card_metadata = CardMetadata::from_details(
            &content,
            &card.card_html,
            &card.link,
            &card.oc_file_path,
            user.id,
            Some(qdrant_point_id),
            private,
        );
        card_metadata =
            web::block(move || insert_card_metadata_query(card_metadata, card.file_uuid, pool1))
                .await?
                .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

        create_new_qdrant_point_query(
            qdrant_point_id,
            ensured_embedding_vector,
            private,
            Some(user.id),
        )
        .await?;
    }

    con.set(format!("Verify: {}", card_metadata.id), true)
        .map_err(|err| ServiceError::BadRequest(format!("Could not set redis key: {}", err)))?;

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

    let card_metadata = user_owns_card(user.id, card_id_inner, pool).await?;

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

    web::block(move || delete_card_metadata_query(card_id_inner, pool1))
        .await?
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let qdrant_collection = std::env::var("QDRANT_COLLECTION").unwrap_or("debate_cards".to_owned());
    qdrant
        .delete_points_blocking(qdrant_collection, &deleted_values, None)
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
    let pool2 = pool.clone();
    let card_metadata = user_owns_card(user.id, card.card_uuid, pool).await?;

    let link = card
        .link
        .clone()
        .unwrap_or_else(|| card_metadata.link.unwrap_or_default());

    let html_parse_result = Command::new("node")
        .arg("./vault-nodejs/scripts/html-converter.js")
        .arg("-html")
        .arg(card.card_html.as_ref().unwrap_or(&"".to_string()))
        .output();

    let content = match html_parse_result {
        Ok(result) => {
            if result.status.success() {
                Some(
                    String::from_utf8(result.stdout)
                        .unwrap()
                        .lines()
                        .collect::<Vec<&str>>()
                        .join(" ")
                        .trim_end()
                        .to_string(),
                )
            } else {
                return Err(ServiceError::BadRequest(format!(
                    "Could not run html-converter.js: {:?}",
                    String::from_utf8(result.stderr).unwrap()
                ))
                .into());
            }
        }
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "message": "Could not parse html",
            })))
        }
    };

    let new_content = match content {
        Some(content) => content,
        None => {
            return Ok(HttpResponse::BadRequest().json(json!({
                "message": "Could not parse html",
            })))
        }
    };

    if new_content.replace(' ', "") != card_metadata.content.replace(' ', "") {
        let Changeset { diffs, .. } = Changeset::new(&card_metadata.content, &new_content, " ");
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

    let private = card.private.unwrap_or(card_metadata.private);
    let card_id1 = card.card_uuid;
    let qdrant_point_id = web::block(move || get_qdrant_id_from_card_id_query(card_id1, pool1))
        .await?
        .map_err(|_| ServiceError::BadRequest("Card not found".into()))?;

    update_qdrant_point_private_query(qdrant_point_id, private, Some(user.id)).await?;

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
            None,
            pool2,
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
    metadata: Vec<CardMetadataWithVotesWithScore>,
    score: f64,
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
    mutex_store: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    let current_user_id = user.map(|user| user.id);
    let page = page.map(|page| page.into_inner()).unwrap_or(1);
    let embedding_vector = create_embedding(&data.content, mutex_store).await?;
    let pool1 = pool.clone();

    let search_card_query_results = search_card_query(
        embedding_vector,
        page,
        pool1,
        data.filter_oc_file_path.clone(),
        data.filter_link_url.clone(),
        current_user_id,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let point_ids = search_card_query_results
        .search_results
        .iter()
        .map(|point| point.point_id)
        .collect::<Vec<_>>();

    let (metadata_cards, collided_cards) = web::block(move || {
        get_metadata_and_collided_cards_from_point_ids_query(point_ids, current_user_id, pool)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let score_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let card: CardMetadataWithVotesWithScore =
                <CardMetadataWithVotesAndFiles as Into<CardMetadataWithVotesWithScore>>::into(
                    match metadata_cards.iter().find(|metadata_card| {
                        metadata_card.qdrant_point_id == search_result.point_id
                    }) {
                        Some(metadata_card) => metadata_card.clone(),
                        None => CardMetadataWithVotesAndFiles {
                            id: uuid::Uuid::default(),
                            author: None,
                            qdrant_point_id: uuid::Uuid::default(),
                            total_upvotes: 0,
                            total_downvotes: 0,
                            vote_by_current_user: None,
                            created_at: chrono::Utc::now().naive_local(),
                            updated_at: chrono::Utc::now().naive_local(),
                            private: false,
                            score: Some(0.0),
                            file_id: None,
                            file_name: None,
                            verification_score: None,
                            content: "".to_string(),
                            card_html: Some("".to_string()),
                            link: Some("".to_string()),
                            oc_file_path: None,
                        },
                    },
                );

            let mut collided_cards: Vec<CardMetadataWithVotesWithScore> = collided_cards
                .iter()
                .filter(|card| card.qdrant_id == search_result.point_id)
                .map(|card| card.metadata.clone().into())
                .collect();

            if !card.private
                || card
                    .clone()
                    .author
                    .is_some_and(|author| Some(author.id) == current_user_id)
            {
                collided_cards.insert(0, card);
            }

            collided_cards.sort_by(|a, b| a.id.cmp(&b.id));
            collided_cards.dedup_by(|a, b| {
                a.oc_file_path.clone().unwrap_or_default().replace('/', "")
                    == b.oc_file_path.clone().unwrap_or_default().replace('/', "")
                    || a.clone().card_html.unwrap_or_default()
                        == b.clone().card_html.unwrap_or_default()
            });

            ScoreCardDTO {
                metadata: collided_cards,
                score: search_result.score.into(),
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
    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let search_card_query_results = web::block(move || {
        search_full_text_card_query(
            data.content.clone(),
            page,
            pool1,
            current_user_id,
            data.filter_oc_file_path.clone(),
            data.filter_link_url.clone(),
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let point_ids = search_card_query_results
        .search_results
        .iter()
        .map(|point| point.qdrant_point_id)
        .collect::<Vec<uuid::Uuid>>();

    let collided_cards =
        web::block(move || get_collided_cards_query(point_ids, current_user_id, pool2))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let mut full_text_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut collided_cards: Vec<CardMetadataWithVotesWithScore> = collided_cards
                .iter()
                .filter(|card| {
                    card.1 == search_result.qdrant_point_id && card.0.id != search_result.id
                })
                .map(|card| card.0.clone().into())
                .collect();

            collided_cards.insert(0, search_result.clone().into());

            collided_cards.sort_by(|a, b| a.id.cmp(&b.id));
            collided_cards.dedup_by(|a, b| {
                a.oc_file_path.clone().unwrap_or_default().replace('/', "")
                    == b.oc_file_path.clone().unwrap_or_default().replace('/', "")
                    || a.clone().card_html.unwrap_or_default()
                        == b.clone().card_html.unwrap_or_default()
            });

            ScoreCardDTO {
                metadata: collided_cards,
                score: search_result.score.unwrap_or(0.0),
            }
        })
        .collect();

    // order full_text_cards by score desc
    full_text_cards.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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
#[derive(Serialize, Deserialize)]
pub struct SearchCollectionsResult {
    pub bookmarks: Vec<ScoreCardDTO>,
    pub collection: CardCollection,
    pub total_pages: i64,
}
pub async fn search_collections(
    data: web::Json<SearchCollectionsData>,
    page: Option<web::Path<u64>>,
    user: Option<LoggedUser>,
    pool: web::Data<Pool>,
    mutex_store: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    //search over the links as well
    let page = page.map(|page| page.into_inner()).unwrap_or(1);
    let embedding_vector = create_embedding(&data.content, mutex_store).await?;
    let collection_id = data.collection_id;
    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let pool3 = pool.clone();
    let current_user_id = user.map(|user| user.id);

    let collection = web::block(move || get_collection_by_id_query(collection_id, pool))
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
        pool2,
        data.filter_oc_file_path.clone(),
        data.filter_link_url.clone(),
        data.collection_id,
        current_user_id,
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
        web::block(move || get_metadata_from_point_ids(point_ids, current_user_id, pool3))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let collided_cards =
        web::block(move || get_collided_cards_query(point_ids_1, current_user_id, pool1))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let score_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let card: CardMetadataWithVotesWithScore =
                <CardMetadataWithVotesAndFiles as Into<CardMetadataWithVotesWithScore>>::into(
                    match metadata_cards.iter().find(|metadata_card| {
                        metadata_card.qdrant_point_id == search_result.point_id
                    }) {
                        Some(metadata_card) => metadata_card.clone(),
                        None => CardMetadataWithVotesAndFiles {
                            id: uuid::Uuid::default(),
                            author: None,
                            qdrant_point_id: uuid::Uuid::default(),
                            total_upvotes: 0,
                            total_downvotes: 0,
                            vote_by_current_user: None,
                            created_at: chrono::Utc::now().naive_local(),
                            updated_at: chrono::Utc::now().naive_local(),
                            private: false,
                            score: Some(0.0),
                            file_id: None,
                            file_name: None,
                            verification_score: None,
                            content: "".to_string(),
                            card_html: Some("".to_string()),
                            link: Some("".to_string()),
                            oc_file_path: None,
                        },
                    },
                );

            let mut collided_cards: Vec<CardMetadataWithVotesWithScore> = collided_cards
                .iter()
                .filter(|card| card.1 == search_result.point_id)
                .map(|card| card.0.clone().into())
                .collect();

            collided_cards.insert(0, card);
            // remove duplicates from collided cards
            let mut seen_ids = HashSet::new();
            let mut i = 0;
            while i < collided_cards.len() {
                if seen_ids.contains(&collided_cards[i].id) {
                    collided_cards.remove(i);
                } else {
                    seen_ids.insert(collided_cards[i].id);
                    i += 1;
                }
            }

            ScoreCardDTO {
                metadata: collided_cards,
                score: search_result.score.into(),
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(SearchCollectionsResult {
        bookmarks: score_cards,
        collection,
        total_pages: search_card_query_results.total_card_pages,
    }))
}

pub async fn search_full_text_collections(
    data: web::Json<SearchCollectionsData>,
    page: Option<web::Path<u64>>,
    user: Option<LoggedUser>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    //search over the links as well
    let page = page.map(|page| page.into_inner()).unwrap_or(1);
    let collection_id = data.collection_id;
    let pool1 = pool.clone();
    let pool3 = pool.clone();
    let current_user_id = user.map(|user| user.id);

    let collection = web::block(move || get_collection_by_id_query(collection_id, pool))
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if !collection.is_public && current_user_id.is_none() {
        return Err(ServiceError::Unauthorized.into());
    }

    if !collection.is_public && Some(collection.author_id) != current_user_id {
        return Err(ServiceError::Forbidden.into());
    }

    let search_card_query_results = web::block(move || {
        search_full_text_collection_query(
            data.content.clone(),
            page,
            pool3,
            current_user_id,
            data.filter_oc_file_path.clone(),
            data.filter_link_url.clone(),
            data.collection_id,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let point_ids = search_card_query_results
        .search_results
        .iter()
        .map(|point| point.qdrant_point_id)
        .collect::<Vec<uuid::Uuid>>();

    let collided_cards =
        web::block(move || get_collided_cards_query(point_ids, current_user_id, pool1))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let full_text_cards: Vec<ScoreCardDTO> = search_card_query_results
        .search_results
        .iter()
        .map(|search_result| {
            let mut collided_cards: Vec<CardMetadataWithVotesWithScore> = collided_cards
                .iter()
                .filter(|card| {
                    card.1 == search_result.qdrant_point_id && card.0.id != search_result.id
                })
                .map(|card| card.0.clone().into())
                .collect();

            collided_cards.insert(0, search_result.clone().into());

            ScoreCardDTO {
                metadata: collided_cards,
                score: search_result.score.unwrap_or(0.0),
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(SearchCollectionsResult {
        bookmarks: full_text_cards,
        collection,
        total_pages: search_card_query_results.total_card_pages,
    }))
}

pub async fn get_most_recent_cards(
    page: Option<web::Path<u64>>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let page = page.map(|page| page.into_inner()).unwrap_or(1);

    let recent_cards = web::block(move || get_recently_created_cards_query(page, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(recent_cards))
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
    if card.private
        && Some(
            card.clone()
                .author
                .unwrap_or(UserDTO {
                    id: uuid::Uuid::default(),
                    email: None,
                    website: None,
                    username: None,
                    visible_email: false,
                    created_at: chrono::NaiveDateTime::default(),
                })
                .id,
        ) != current_user_id
    {
        return Err(ServiceError::Forbidden.into());
    }
    Ok(HttpResponse::Ok().json(card))
}

pub async fn get_total_card_count(pool: web::Data<Pool>) -> Result<HttpResponse, actix_web::Error> {
    let total_count = web::block(move || get_card_count_query(pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(json!({ "total_count": total_count })))
}
