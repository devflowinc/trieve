use super::auth_handler::{LoggedUser, RequireAuth};
use super::dataset_handler::SlimDataset;
use crate::data::models::{
    CardCollection, CardCollectionBookmark, CardMetadata, CardMetadataWithVotesWithScore,
    ChatMessageProxy, Pool, UserDTO,
};
use crate::errors::ServiceError;
use crate::operators::card_operator::get_metadata_from_id_query;
use crate::operators::card_operator::*;
use crate::operators::collection_operator::{
    create_card_bookmark_query, get_collection_by_id_query,
};
use crate::operators::dataset_operator::get_dataset_by_id_query;
use crate::operators::qdrant_operator::{create_embedding, update_qdrant_point_query};
use crate::operators::qdrant_operator::{
    create_new_qdrant_point_query, delete_qdrant_point_id_query, recommend_qdrant_query,
};
use crate::operators::search_operator::{
    global_unfiltered_top_match_query, search_full_text_cards, search_full_text_collections,
    search_hybrid_cards, search_semantic_cards, search_semantic_collections,
};
use crate::operators::tantivy_operator::TantivyIndexMap;
use crate::{get_env, AppMutexStore, CrossEncoder};
use actix_web::web::Bytes;
use actix_web::{web, HttpResponse};
use chrono::NaiveDateTime;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::chat_completion::{ChatCompletionParameters, ChatMessage, Role};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tokio::sync::RwLock;
use tokio_stream::StreamExt;
use utoipa::{IntoParams, ToSchema};

pub async fn user_owns_card(
    user_id: uuid::Uuid,
    card_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<CardMetadata, actix_web::Error> {
    let cards = web::block(move || get_metadata_from_id_query(card_id, dataset_id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if cards.author_id != user_id {
        return Err(ServiceError::Forbidden.into());
    }

    Ok(cards)
}

pub async fn user_owns_card_tracking_id(
    user_id: uuid::Uuid,
    tracking_id: String,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<CardMetadata, actix_web::Error> {
    let cards =
        web::block(move || get_metadata_from_tracking_id_query(tracking_id, dataset_id, pool))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if cards.author_id != user_id {
        return Err(ServiceError::Forbidden.into());
    }

    Ok(cards)
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
pub struct CreateCardData {
    pub card_html: Option<String>,
    pub link: Option<String>,
    pub tag_set: Option<String>,
    pub private: Option<bool>,
    pub file_uuid: Option<uuid::Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub collection_id: Option<uuid::Uuid>,
    pub time_stamp: Option<String>,
}

pub fn convert_html(html: &str) -> String {
    let html_parse_result = Command::new("./server-python/html-converter.py")
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
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct ReturnCreatedCard {
    pub card_metadata: CardMetadata,
    pub duplicate: bool,
}

#[utoipa::path(
    post,
    path = "/card",
    context_path = "/api",
    tag = "card",
    request_body(content = CreateCardData, description = "JSON request payload to create a new card (chunk)", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON response payload containing the created card", body = [ReturnCreatedCard]),
        (status = 400, description = "Service error relating to to creating a card, likely due to conflicting tracking_id", body = [DefaultError]),
    )
)]
pub async fn create_card(
    card: web::Json<CreateCardData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    app_mutex: web::Data<AppMutexStore>,
    dataset: SlimDataset,
) -> Result<HttpResponse, actix_web::Error> {
    let only_admin_can_create_cards =
        std::env::var("ONLY_ADMIN_CAN_CREATE_CARDS").unwrap_or("off".to_string());

    if only_admin_can_create_cards == "on" {
        let admin_email = std::env::var("ADMIN_USER_EMAIL").unwrap_or("".to_string());
        if admin_email != user.email {
            return Err(ServiceError::Forbidden.into());
        }
    }

    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let pool3 = pool.clone();

    let dataset1 = dataset.clone();
    let dataset = web::block(move || get_dataset_by_id_query(dataset1.id, pool)).await??;
    if user.organization_id != dataset.organization_id {
        return Err(ServiceError::Forbidden.into());
    }

    let private = card.private.unwrap_or(false);
    let card_tracking_id = card
        .tracking_id
        .clone()
        .filter(|card_tracking| !card_tracking.is_empty());
    let card_collection_id = card.collection_id;

    let mut collision: Option<uuid::Uuid> = None;

    let content = convert_html(card.card_html.as_ref().unwrap_or(&"".to_string()));

    let embedding_vector = create_embedding(&content, app_mutex).await?;

    let first_semantic_result =
        global_unfiltered_top_match_query(embedding_vector.clone(), dataset.id)
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not get semantic similarity for collision check: {}",
                    err.message
                ))
            })?;

    let duplicate_distance_threshold = std::env::var("DUPLICATE_DISTANCE_THRESHOLD")
        .unwrap_or("0.95".to_string())
        .parse::<f32>()
        .unwrap_or(0.95);

    if first_semantic_result.score >= duplicate_distance_threshold {
        //Sets collision to collided card id
        collision = Some(first_semantic_result.point_id);

        let score_card_result = web::block(move || {
            get_metadata_from_point_ids(vec![first_semantic_result.point_id], Some(user.id), pool2)
        })
        .await?;

        match score_card_result {
            Ok(card_results) => {
                if card_results.is_empty() {
                    delete_qdrant_point_id_query(first_semantic_result.point_id, dataset.id)
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
    }

    let mut card_metadata: CardMetadata;
    let mut duplicate: bool = false;

    //if collision is not nil, insert card with collision
    if collision.is_some() {
        update_qdrant_point_query(
            None,
            private,
            collision.expect("Collision must be some"),
            Some(user.id),
            None,
            dataset.id,
        )
        .await?;

        card_metadata = CardMetadata::from_details(
            &content,
            &card.card_html,
            &card.link,
            &card.tag_set,
            user.id,
            None,
            private,
            card.metadata.clone(),
            card_tracking_id,
            card.time_stamp
                .clone()
                .map(|ts| -> Result<NaiveDateTime, ServiceError> {
                    NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S").map_err(|e| {
                        ServiceError::BadRequest(format!("Invalid timestamp format {}", e))
                    })
                })
                .transpose()?,
            dataset.id,
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
        let qdrant_point_id = uuid::Uuid::new_v4();

        card_metadata = CardMetadata::from_details(
            &content,
            &card.card_html,
            &card.link,
            &card.tag_set,
            user.id,
            Some(qdrant_point_id),
            private,
            card.metadata.clone(),
            card_tracking_id,
            card.time_stamp
                .clone()
                .map(|ts| -> Result<NaiveDateTime, ServiceError> {
                    NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S").map_err(|e| {
                        ServiceError::BadRequest(format!("Invalid timestamp format {}", e))
                    })
                })
                .transpose()?,
            dataset.id,
        );

        card_metadata = insert_card_metadata_query(
            card_metadata,
            card.file_uuid,
            tantivy_index_map,
            dataset.id,
            pool1,
        )
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

        create_new_qdrant_point_query(
            qdrant_point_id,
            embedding_vector,
            private,
            card_metadata.clone(),
            Some(user.id),
            dataset.id,
        )
        .await?;
    }

    if let Some(collection_id_to_bookmark) = card_collection_id {
        let card_collection_bookmark = CardCollectionBookmark::from_details(
            collection_id_to_bookmark,
            card_metadata.id,
            dataset.id,
        );

        let _ =
            web::block(move || create_card_bookmark_query(pool3, card_collection_bookmark)).await?;
    }

    Ok(HttpResponse::Ok().json(ReturnCreatedCard {
        card_metadata,
        duplicate,
    }))
}

#[utoipa::path(
    delete,
    path = "/card/{card_id}}",
    context_path = "/api",
    tag = "card",
    responses(
        (status = 204, description = "Confirmation that the card with the id specified was deleted", body = [CardMetadataWithVotesWithScore]),
        (status = 400, description = "Service error relating to finding a card by tracking_id", body = [DefaultError]),
    ),
    params(
        ("card_id" = Option<uuid>, Path, description = "id of the card you want to delete")
    ),
)]
pub async fn delete_card(
    card_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    app_mutex: web::Data<AppMutexStore>,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    user: LoggedUser,
    dataset: SlimDataset,
) -> Result<HttpResponse, actix_web::Error> {
    let card_id_inner = card_id.into_inner();
    let dataset_id = dataset.id;
    let pool1 = pool.clone();

    let card_metadata = user_owns_card(user.id, card_id_inner, dataset_id, pool).await?;
    let qdrant_point_id = card_metadata.qdrant_point_id;

    delete_card_metadata_query(
        card_id_inner,
        qdrant_point_id,
        tantivy_index_map,
        app_mutex,
        dataset_id,
        pool1,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    delete,
    path = "/card/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "card",
    responses(
        (status = 204, description = "Confirmation that the card with the tracking_id specified was deleted", body = [CardMetadataWithVotesWithScore]),
        (status = 400, description = "Service error relating to finding a card by tracking_id", body = [DefaultError]),
    ),
    params(
        ("tracking_id" = Option<String>, Path, description = "tracking_id of the card you want to delete")
    ),
)]
pub async fn delete_card_by_tracking_id(
    tracking_id: web::Path<String>,
    pool: web::Data<Pool>,
    app_mutex: web::Data<AppMutexStore>,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    user: LoggedUser,
    dataset: SlimDataset,
) -> Result<HttpResponse, actix_web::Error> {
    let tracking_id_inner = tracking_id.into_inner();
    let pool1 = pool.clone();
    let dataset_id = dataset.id;

    let card_metadata =
        user_owns_card_tracking_id(user.id, tracking_id_inner, dataset_id, pool).await?;

    let qdrant_point_id = card_metadata.qdrant_point_id;

    delete_card_metadata_query(
        card_metadata.id,
        qdrant_point_id,
        tantivy_index_map,
        app_mutex,
        dataset_id,
        pool1,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateCardData {
    card_uuid: uuid::Uuid,
    link: Option<String>,
    card_html: Option<String>,
    private: Option<bool>,
    metadata: Option<serde_json::Value>,
    tracking_id: Option<String>,
    time_stamp: Option<String>,
}
#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct CardHtmlUpdateError {
    pub message: String,
    changed_content: String,
}

#[utoipa::path(
    put,
    path = "/card/update",
    context_path = "/api",
    tag = "card",
    request_body(content = UpdateCardData, description = "JSON request payload to update a card (chunk)", content_type = "application/json"),
    responses(
        (status = 204, description = "No content Ok response indicating the card was updated as requested",),
        (status = 400, description = "Service error relating to to updating card, likely due to conflicting tracking_id", body = [DefaultError]),
    )
)]
pub async fn update_card(
    card: web::Json<UpdateCardData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    app_mutex: web::Data<AppMutexStore>,
    dataset: SlimDataset,
) -> Result<HttpResponse, actix_web::Error> {
    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let dataset_id = dataset.id;
    let card_metadata = user_owns_card(user.id, card.card_uuid, dataset_id, pool).await?;

    let link = card
        .link
        .clone()
        .unwrap_or_else(|| card_metadata.link.clone().unwrap_or_default());
    let card_tracking_id = card
        .tracking_id
        .clone()
        .filter(|card_tracking| !card_tracking.is_empty());

    let new_content = convert_html(card.card_html.as_ref().unwrap_or(&"".to_string()));

    let embedding_vector = create_embedding(&new_content, app_mutex).await?;

    let card_html = match card.card_html.clone() {
        Some(card_html) => Some(card_html),
        None => card_metadata.card_html,
    };

    let private = card.private.unwrap_or(card_metadata.private);
    let card_id1 = card.card_uuid;
    let qdrant_point_id = web::block(move || get_qdrant_id_from_card_id_query(card_id1, pool1))
        .await?
        .map_err(|_| ServiceError::BadRequest("Card not found".into()))?;

    let metadata = CardMetadata::from_details_with_id(
        card.card_uuid,
        &new_content,
        &card_html,
        &Some(link),
        &card_metadata.tag_set,
        user.id,
        card_metadata.qdrant_point_id,
        private,
        card.metadata.clone(),
        card_tracking_id,
        card.time_stamp
            .clone()
            .map(|ts| -> Result<NaiveDateTime, ServiceError> {
                NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S")
                    .map_err(|_| ServiceError::BadRequest("Invalid timestamp format".to_string()))
            })
            .transpose()?,
        dataset_id,
    );
    let metadata1 = metadata.clone();
    update_card_metadata_query(metadata, None, tantivy_index_map, dataset_id, pool2)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    update_qdrant_point_query(
        // If the card is a collision, we don't want to update the qdrant point
        if card_metadata.qdrant_point_id.is_none() {
            None
        } else {
            Some(metadata1)
        },
        private,
        qdrant_point_id,
        Some(user.id),
        Some(embedding_vector),
        dataset_id,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateCardByTrackingIdData {
    card_uuid: Option<uuid::Uuid>,
    link: Option<String>,
    card_html: Option<String>,
    private: Option<bool>,
    metadata: Option<serde_json::Value>,
    tracking_id: String,
    time_stamp: Option<String>,
}

#[utoipa::path(
    put,
    path = "/card/tracking_id/update",
    context_path = "/api",
    tag = "card",
    request_body(content = UpdateCardByTrackingIdData, description = "JSON request payload to update a card by tracking_id (chunks)", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the card has been updated as per your request",),
        (status = 400, description = "Service error relating to to updating card", body = [DefaultError]),
    ),
)]
pub async fn update_card_by_tracking_id(
    card: web::Json<UpdateCardByTrackingIdData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    app_mutex: web::Data<AppMutexStore>,
    dataset: SlimDataset,
) -> Result<HttpResponse, actix_web::Error> {
    let pool1 = pool.clone();
    let dataset = web::block(move || get_dataset_by_id_query(dataset.id, pool1)).await??;
    if user.organization_id != dataset.organization_id {
        return Err(ServiceError::Forbidden.into());
    };

    if card.tracking_id.is_empty() {
        return Err(ServiceError::BadRequest(
            "Tracking id must be provided to update by tracking_id".into(),
        )
        .into());
    }
    let tracking_id = card.tracking_id.clone();
    let tracking_id1 = tracking_id.clone();

    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let card_metadata = user_owns_card_tracking_id(user.id, tracking_id, dataset.id, pool).await?;

    let link = card
        .link
        .clone()
        .unwrap_or_else(|| card_metadata.link.clone().unwrap_or_default());

    let new_content = convert_html(card.card_html.as_ref().unwrap_or(&"".to_string()));

    let embedding_vector = create_embedding(&new_content, app_mutex).await?;

    let card_html = match card.card_html.clone() {
        Some(card_html) => Some(card_html),
        None => card_metadata.card_html,
    };

    let private = card.private.unwrap_or(card_metadata.private);
    let card_id1 = card_metadata.id;
    let qdrant_point_id = web::block(move || get_qdrant_id_from_card_id_query(card_id1, pool1))
        .await?
        .map_err(|_| ServiceError::BadRequest("Card not found".into()))?;

    let metadata = CardMetadata::from_details_with_id(
        card_metadata.id,
        &new_content,
        &card_html,
        &Some(link),
        &card_metadata.tag_set,
        user.id,
        card_metadata.qdrant_point_id,
        private,
        card.metadata.clone(),
        Some(tracking_id1),
        card.time_stamp
            .clone()
            .map(|ts| -> Result<NaiveDateTime, ServiceError> {
                NaiveDateTime::parse_from_str(&ts, "%Y-%m-%d %H:%M:%S")
                    .map_err(|_| ServiceError::BadRequest("Invalid timestamp format".to_string()))
            })
            .transpose()?,
        dataset.id,
    );
    let metadata1 = metadata.clone();

    update_card_metadata_query(metadata, None, tantivy_index_map, dataset_id, pool2)
        .await
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    update_qdrant_point_query(
        // If the card is a collision, we don't want to update the qdrant point
        if card_metadata.qdrant_point_id.is_none() {
            None
        } else {
            Some(metadata1)
        },
        private,
        qdrant_point_id,
        Some(user.id),
        Some(embedding_vector),
        dataset.id,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Clone, ToSchema)]
pub struct SearchCardData {
    pub search_type: String,
    pub content: String,
    pub link: Option<Vec<String>>,
    pub tag_set: Option<Vec<String>>,
    pub time_range: Option<(String, String)>,
    pub filters: Option<serde_json::Value>,
    pub cross_encoder: Option<bool>,
    pub weights: Option<(f64, f64)>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
pub struct ScoreCardDTO {
    pub metadata: Vec<CardMetadataWithVotesWithScore>,
    pub score: f64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SearchCardQueryResponseBody {
    pub score_cards: Vec<ScoreCardDTO>,
    pub total_card_pages: i64,
}

#[derive(Clone)]
pub struct ParsedQuery {
    pub quote_words: Option<Vec<String>>,
    pub negated_words: Option<Vec<String>>,
}
fn parse_query(query: String) -> ParsedQuery {
    let re = Regex::new(r#""(.*?)""#).unwrap();
    let quote_words: Vec<String> = re
        .captures_iter(&query.replace('\\', ""))
        .map(|capture| capture[1].to_string())
        .filter(|word| !word.is_empty())
        .collect::<Vec<String>>();

    let quote_words = if quote_words.is_empty() {
        None
    } else {
        Some(quote_words)
    };

    let negated_words: Vec<String> = query
        .split_whitespace()
        .filter(|word| word.starts_with('-'))
        .map(|word| word.strip_prefix('-').unwrap().to_string())
        .collect::<Vec<String>>();

    let negated_words = if negated_words.is_empty() {
        None
    } else {
        Some(negated_words)
    };

    ParsedQuery {
        quote_words,
        negated_words,
    }
}

#[utoipa::path(
    post,
    path = "/card/search",
    context_path = "/api",
    tag = "card",
    request_body(content = SearchCardData, description = "JSON request payload to semantically search for cards (chunks)", content_type = "application/json"),
    responses(
        (status = 200, description = "Cards which are similar to the embedding vector of the search query", body = [SearchCardQueryResponseBody]),
        (status = 400, description = "Service error relating to searching", body = [DefaultError]),
    ),
    params(
        ("page" = Option<u64>, Path, description = "Page number of the search results")
    ),
)]
#[allow(clippy::too_many_arguments)]
pub async fn search_card(
    data: web::Json<SearchCardData>,
    page: Option<web::Path<u64>>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    cross_encoder_init: web::Data<CrossEncoder>,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    app_mutex: web::Data<AppMutexStore>,
    dataset: SlimDataset,
) -> Result<HttpResponse, actix_web::Error> {
    let pool1 = pool.clone();
    let dataset = web::block(move || get_dataset_by_id_query(dataset.id, pool1)).await??;
    if user.organization_id != dataset.organization_id {
        return Err(ServiceError::Forbidden.into());
    };

    let current_user_id = Some(user.id);
    let page = page.map(|page| page.into_inner()).unwrap_or(1);
    let dataset_id = dataset.id;
    let parsed_query = parse_query(data.content.clone());

    let result_cards = match data.search_type.as_str() {
        "fulltext" => {
            search_full_text_cards(
                data,
                parsed_query,
                page,
                pool,
                tantivy_index_map,
                current_user_id,
                dataset_id,
            )
            .await?
        }
        "hybrid" => {
            search_hybrid_cards(
                data,
                parsed_query,
                page,
                pool,
                current_user_id,
                cross_encoder_init,
                tantivy_index_map,
                dataset_id,
                app_mutex,
            )
            .await?
        }
        _ => {
            search_semantic_cards(
                data,
                parsed_query,
                page,
                pool,
                current_user_id,
                dataset_id,
                app_mutex,
            )
            .await?
        }
    };

    Ok(HttpResponse::Ok().json(result_cards))
}

#[derive(Serialize, Deserialize, Clone, ToSchema, IntoParams)]
#[into_params(style = Form, parameter_in = Query)]
pub struct SearchCollectionsData {
    pub content: String,
    pub link: Option<Vec<String>>,
    pub tag_set: Option<Vec<String>>,
    pub filters: Option<serde_json::Value>,
    pub collection_id: uuid::Uuid,
    #[param(inline)]
    pub search_type: String,
}

impl From<SearchCollectionsData> for SearchCardData {
    fn from(data: SearchCollectionsData) -> Self {
        Self {
            content: data.content,
            link: data.link,
            tag_set: data.tag_set,
            time_range: None,
            filters: data.filters,
            cross_encoder: None,
            weights: None,
            search_type: data.search_type,
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SearchCollectionsResult {
    pub bookmarks: Vec<ScoreCardDTO>,
    pub collection: CardCollection,
    pub total_pages: i64,
}

#[utoipa::path(
    post,
    path = "/card_collection/search",
    context_path = "/api",
    tag = "card_collection",
    request_body(content = SearchCollectionsData, description = "JSON request payload to semantically search a collection", content_type = "application/json"),
    responses(
        (status = 200, description = "Collection cards which are similar to the embedding vector of the search query", body = [SearchCollectionsResult]),
        (status = 400, description = "Service error relating to getting the collections that the card is in", body = [DefaultError]),
    ),
    params(
        ("page" = u64, description = "The page of search results to get"),
    ),
)]
#[allow(clippy::too_many_arguments)]
pub async fn search_collections(
    data: web::Json<SearchCollectionsData>,
    page: Option<web::Path<u64>>,
    user: Option<LoggedUser>,
    pool: web::Data<Pool>,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    app_mutex: web::Data<AppMutexStore>,
    _required_user: RequireAuth,
    dataset: SlimDataset,
) -> Result<HttpResponse, actix_web::Error> {
    //search over the links as well
    let page = page.map(|page| page.into_inner()).unwrap_or(1);
    let collection_id = data.collection_id;
    let dataset_id = dataset.id;

    let current_user_id = user.map(|user| user.id);
    let pool1 = pool.clone();

    let collection = {
        let dataset_id = dataset_id;
        web::block(move || get_collection_by_id_query(collection_id, dataset_id, pool))
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?
    };

    if !collection.is_public && current_user_id.is_none() {
        return Err(ServiceError::Unauthorized.into());
    }

    if !collection.is_public && Some(collection.author_id) != current_user_id {
        return Err(ServiceError::Forbidden.into());
    }
    let parsed_query = parse_query(data.content.clone());

    let result_cards = match data.search_type.as_str() {
        "fulltext" => {
            search_full_text_collections(
                data,
                parsed_query,
                collection,
                page,
                pool1,
                tantivy_index_map,
                current_user_id,
                dataset_id,
            )
            .await?
        }
        _ => {
            search_semantic_collections(
                data,
                parsed_query,
                collection,
                page,
                pool1,
                current_user_id,
                dataset_id,
                app_mutex,
            )
            .await?
        }
    };

    Ok(HttpResponse::Ok().json(result_cards))
}

#[utoipa::path(
    get,
    path = "/top_cards",
    context_path = "/api",
    tag = "top_cards",
    responses(
        (status = 200, description = "JSON body representing the top cards by collected votes", body = [Vec<CardMetadataWithVotes>]),
        (status = 400, description = "Service error relating to fetching the top cards by collected votes", body = [DefaultError]),
    ),
    params(
        ("page" = u64, description = "The page of top cards to fetch"),
    ),
)]
pub async fn get_top_cards(
    page: Option<web::Path<u64>>,
    pool: web::Data<Pool>,
    _required_user: RequireAuth,
    dataset: SlimDataset,
) -> Result<HttpResponse, actix_web::Error> {
    let page = page.map(|page| page.into_inner()).unwrap_or(1);

    let top_cards = web::block(move || get_top_cards_query(page, dataset.id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(top_cards))
}

#[utoipa::path(
    get,
    path = "/card/{card_id}",
    context_path = "/api",
    tag = "card",
    responses(
        (status = 200, description = "Card with the id that you were searching for", body = [CardMetadataWithVotesWithScore]),
        (status = 400, description = "Service error relating to fidning a card by tracking_id", body = [DefaultError]),
    ),
    params(
        ("card_id" = Option<uuid>, Path, description = "id of the card you want to fetch")
    ),
)]
pub async fn get_card_by_id(
    card_id: web::Path<uuid::Uuid>,
    user: Option<LoggedUser>,
    pool: web::Data<Pool>,
    _required_user: RequireAuth,
    dataset: SlimDataset,
) -> Result<HttpResponse, actix_web::Error> {
    let current_user_id = user.map(|user| user.id);
    let card = web::block(move || {
        get_metadata_and_votes_from_id_query(
            card_id.into_inner(),
            current_user_id,
            dataset.id,
            pool,
        )
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
                    organization_id: uuid::Uuid::default(),
                })
                .id,
        ) != current_user_id
    {
        return Err(ServiceError::Forbidden.into());
    }
    Ok(HttpResponse::Ok().json(card))
}

#[utoipa::path(
    get,
    path = "/card/tracking_id/{tracking_id}",
    context_path = "/api",
    tag = "card",
    responses(
        (status = 200, description = "Card with the tracking_id that you were searching for", body = [CardMetadataWithVotesWithScore]),
        (status = 400, description = "Service error relating to fidning a card by tracking_id", body = [DefaultError]),
    ),
    params(
        ("tracking_id" = Option<String>, Path, description = "tracking_id of the card you want to fetch")
    ),
)]
pub async fn get_card_by_tracking_id(
    tracking_id: web::Path<String>,
    user: LoggedUser,
    pool: web::Data<Pool>,
    _required_user: RequireAuth,
    dataset: SlimDataset,
) -> Result<HttpResponse, actix_web::Error> {
    let pool1 = pool.clone();
    let dataset = web::block(move || get_dataset_by_id_query(dataset.id, pool1)).await??;
    if user.organization_id != dataset.organization_id {
        return Err(ServiceError::Forbidden.into());
    }

    let current_user_id = Some(user.id);
    let card = web::block(move || {
        get_metadata_and_votes_from_tracking_id_query(
            tracking_id.into_inner(),
            current_user_id,
            dataset.id,
            pool,
        )
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
                    organization_id: uuid::Uuid::default(),
                })
                .id,
        ) != current_user_id
    {
        return Err(ServiceError::Forbidden.into());
    }
    Ok(HttpResponse::Ok().json(card))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RecommendCardsRequest {
    pub positive_card_ids: Vec<uuid::Uuid>,
}

#[utoipa::path(
    post,
    path = "/card/recommend",
    context_path = "/api",
    tag = "card",
    request_body(content = RecommendCardsRequest, description = "JSON request payload to get recommendations of cards similar to the cards in the request", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON response payload containing cards with scores which are similar to those in the request body", body = [Vec<CardMetadataWithVotesWithScore>]),
        (status = 400, description = "Service error relating to to getting similar cards", body = [DefaultError]),
    )
)]
pub async fn get_recommended_cards(
    data: web::Json<RecommendCardsRequest>,
    pool: web::Data<Pool>,
    user: LoggedUser,
    dataset: SlimDataset,
) -> Result<HttpResponse, actix_web::Error> {
    let pool1 = pool.clone();
    let dataset = web::block(move || get_dataset_by_id_query(dataset.id, pool1)).await??;
    if user.organization_id != dataset.organization_id {
        return Err(ServiceError::Forbidden.into());
    }

    let positive_card_ids = data.positive_card_ids.clone();

    let recommended_qdrant_point_ids = recommend_qdrant_query(positive_card_ids, dataset.id)
        .await
        .map_err(|err| {
            ServiceError::BadRequest(format!("Could not get recommended cards: {}", err))
        })?;

    let recommended_card_metadatas =
        web::block(move || get_metadata_from_point_ids(recommended_qdrant_point_ids, None, pool))
            .await?
            .map_err(|err| {
                ServiceError::BadRequest(format!(
                    "Could not get recommended card_metadas from qdrant_point_ids: {}",
                    err
                ))
            })?;

    Ok(HttpResponse::Ok().json(recommended_card_metadatas))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GenerateCardsRequest {
    pub prev_messages: Vec<ChatMessageProxy>,
    pub card_ids: Vec<uuid::Uuid>,
}

#[utoipa::path(
    post,
    path = "/card/generate",
    context_path = "/api",
    tag = "card",
    request_body(content = GenerateCardsRequest, description = "JSON request payload to perform RAG on some cards (chunks)", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream, check the chat or search UI for an example how to process this",),
        (status = 400, description = "Service error relating to to updating card, likely due to conflicting tracking_id", body = [DefaultError]),
    ),
)]
pub async fn generate_off_cards(
    data: web::Json<GenerateCardsRequest>,
    pool: web::Data<Pool>,
    user: LoggedUser,
    dataset: SlimDataset,
) -> Result<HttpResponse, actix_web::Error> {
    let pool1 = pool.clone();
    let datset = web::block(move || get_dataset_by_id_query(dataset.id, pool1)).await??;
    if user.organization_id != datset.organization_id {
        return Err(ServiceError::Forbidden.into());
    }

    let prev_messages = data.prev_messages.clone();
    let card_ids = data.card_ids.clone();
    let user_id = user.id;
    let cards =
        web::block(move || get_metadata_from_ids_query(card_ids, user_id, dataset.id, pool))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    let openai_api_key = get_env!("OPENAI_API_KEY", "OPENAI_API_KEY should be set").into();

    let client = Client {
        api_key: openai_api_key,
        http_client: reqwest::Client::new(),
        base_url: std::env::var("OPENAI_BASE_URL")
            .map(|url| {
                if url.is_empty() {
                    "https://api.openai.com/v1".to_string()
                } else {
                    url
                }
            })
            .unwrap_or("https://api.openai.com/v1".into()),
    };

    let mut messages: Vec<ChatMessage> = prev_messages
        .iter()
        .map(|message| ChatMessage::from(message.clone()))
        .collect();
    messages.truncate(prev_messages.len() - 1);
    messages.push(ChatMessage {
        role: Role::User,
        content: "I am going to provide several pieces of information for you to use in response to a request or question. You will not respond until I ask you to.".to_string(),
        name: None,
    });
    messages.push(ChatMessage {
        role: Role::Assistant,
        content: "Understood, I will not reply until I receive a direct request or question."
            .to_string(),
        name: None,
    });
    cards.iter().enumerate().for_each(|(idx, bookmark)| {
        let first_240_words = bookmark
            .content
            .split_whitespace()
            .take(240)
            .collect::<Vec<_>>()
            .join(" ");

        messages.push(ChatMessage {
            role: Role::User,
            content: format!("Doc {}: {}", idx + 1, first_240_words),
            name: None,
        });
        messages.push(ChatMessage {
            role: Role::Assistant,
            content: "".to_string(),
            name: None,
        });
    });
    messages.push(ChatMessage {
        role: Role::User,
        content: format!("Respond to this question and include the doc numbers that you used in square brackets at the end of the sentences that you used the docs for.: {}",prev_messages
            .last()
            .expect("There needs to be at least 1 prior message")
            .content
            .clone()),
        name: None,
    });

    let parameters = ChatCompletionParameters {
        model: "gpt-3.5-turbo".into(),
        messages,
        temperature: None,
        top_p: None,
        n: None,
        stop: None,
        max_tokens: None,
        presence_penalty: Some(0.8),
        frequency_penalty: Some(0.8),
        logit_bias: None,
        user: None,
    };

    let stream = client.chat().create_stream(parameters).await.unwrap();

    Ok(HttpResponse::Ok().streaming(stream.map(
        move |response| -> Result<Bytes, actix_web::Error> {
            if let Ok(response) = response {
                let chat_content = response.choices[0].delta.content.clone();
                return Ok(Bytes::from(chat_content.unwrap_or("".to_string())));
            }
            Err(ServiceError::InternalServerError.into())
        },
    )))
}
