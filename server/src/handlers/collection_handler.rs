use super::auth_handler::{LoggedUser, RequireAuth};
use crate::{
    data::models::{
        CardCollection, CardCollectionAndFile, CardCollectionBookmark, CardMetadataWithFileData,
        Dataset, Pool,
    },
    errors::ServiceError,
    operators::{card_operator::get_collided_cards_query, collection_operator::*},
};
use actix_web::{
    web::{self, Bytes},
    HttpResponse,
};
use crossbeam_channel::unbounded;
use futures_util::StreamExt;
use openai_dive::v1::{
    api::Client,
    resources::chat::{ChatCompletionParameters, ChatMessage, Role},
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use utoipa::ToSchema;

pub async fn user_owns_collection(
    user_id: uuid::Uuid,
    collection_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<CardCollection, actix_web::Error> {
    let collection =
        web::block(move || get_collection_by_id_query(collection_id, dataset_id, pool))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if collection.author_id != user_id {
        return Err(ServiceError::Forbidden.into());
    }

    Ok(collection)
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateCardCollectionData {
    pub name: String,
    pub description: String,
    pub is_public: bool,
}

#[utoipa::path(
    post,
    path = "/card_collection",
    context_path = "/api",
    tag = "card_collection",
    request_body(content = CreateCardCollectionData, description = "JSON request payload to cretea a CardCollection", content_type = "application/json"),
    responses(
        (status = 200, description = "Returns the created CardCollection", body = [CardCollection]),
        (status = 400, description = "Service error relating to creating the CardCollection", body = [DefaultError]),
    ),
)]
pub async fn create_card_collection(
    body: web::Json<CreateCardCollectionData>,
    user: LoggedUser,
    dataset: Dataset,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let name = body.name.clone();
    let description = body.description.clone();
    let is_public = body.is_public;

    let collection =
        CardCollection::from_details(user.id, name, is_public, description, dataset.id);
    {
        let collection = collection.clone();
        web::block(move || create_collection_query(collection, pool))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    }

    Ok(HttpResponse::Ok().json(collection))
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CollectionData {
    pub collections: Vec<CardCollectionAndFile>,
    pub total_pages: i64,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UserCollectionQuery {
    pub user_id: uuid::Uuid,
    pub page: u64,
}

#[utoipa::path(
    get,
    path = "/user/collections/{user_id}/{page}",
    context_path = "/api",
    tag = "user",
    responses(
        (status = 200, description = "JSON body representing the collections created by the given user", body = [CollectionData]),
        (status = 400, description = "Service error relating to getting the collections created by the given user", body = [DefaultError]),
    ),
    params(
        ("user_id" = uuid::Uuid, description = "The id of the user to fetch collections for"),
        ("page" = i64, description = "The page of collections to fetch"),
    ),
)]
pub async fn get_specific_user_card_collections(
    user: Option<LoggedUser>,
    user_and_page: web::Path<UserCollectionQuery>,
    dataset: Dataset,
    pool: web::Data<Pool>,
    _required_user: RequireAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let accessing_user_id = user.map(|user| user.id);
    let collections = web::block(move || {
        get_collections_for_specifc_user_query(
            user_and_page.user_id,
            accessing_user_id,
            user_and_page.page,
            dataset.id,
            pool,
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(CollectionData {
        collections: collections
            .iter()
            .map(|collection| CardCollectionAndFile {
                id: collection.id,
                author_id: collection.author_id,
                name: collection.name.clone(),
                is_public: collection.is_public,
                description: collection.description.clone(),
                created_at: collection.created_at,
                updated_at: collection.updated_at,
                file_id: collection.file_id,
            })
            .collect(),
        total_pages: collections
            .first()
            .map(|collection| {
                (collection.collection_count.unwrap_or(10) as f64 / 10.0).ceil() as i64
            })
            .unwrap_or(1),
    }))
}

#[utoipa::path(
    get,
    path = "/card_collection/{page_or_card_collection_id}",
    context_path = "/api",
    tag = "card_collection",
    responses(
        (status = 200, description = "The page of collections for the auth'ed user", body = [CollectionData]),
        (status = 400, description = "Service error relating to getting the collections for the auth'ed user", body = [DefaultError]),
    ),
    params(
        ("page_or_card_collection_id" = u64, description = "The page of collections to fetch"),
    ),
)]
pub async fn get_logged_in_user_card_collections(
    user: LoggedUser,
    page: web::Path<u64>,
    dataset: Dataset,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, actix_web::Error> {
    let collections = web::block(move || {
        get_collections_for_logged_in_user_query(user.id, page.into_inner(), dataset.id, pool)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(CollectionData {
        collections: collections
            .iter()
            .map(|collection| CardCollectionAndFile {
                id: collection.id,
                author_id: collection.author_id,
                name: collection.name.clone(),
                is_public: collection.is_public,
                description: collection.description.clone(),
                created_at: collection.created_at,
                updated_at: collection.updated_at,
                file_id: collection.file_id,
            })
            .collect(),
        total_pages: collections
            .first()
            .map(|collection| (collection.collection_count.unwrap_or(5) as f64 / 5.0).ceil() as i64)
            .unwrap_or(1),
    }))
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct DeleteCollectionData {
    pub collection_id: uuid::Uuid,
}

#[utoipa::path(
    delete,
    path = "/card_collection",
    context_path = "/api",
    tag = "card_collection",
    request_body(content = DeleteCollectionData, description = "JSON request payload to delete a CardCollection", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the CardCollection was deleted"),
        (status = 400, description = "Service error relating to deleting the CardCollection", body = [DefaultError]),
    ),
)]
pub async fn delete_card_collection(
    data: web::Json<DeleteCollectionData>,
    pool: web::Data<Pool>,
    dataset: Dataset,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let pool2 = pool.clone();
    let collection_id = data.collection_id;
    let dataset_id = dataset.id;

    user_owns_collection(user.id, collection_id, dataset_id, pool).await?;

    web::block(move || delete_collection_by_id_query(collection_id, dataset_id, pool2))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateCardCollectionData {
    pub collection_id: uuid::Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[utoipa::path(
    put,
    path = "/card_collection",
    context_path = "/api",
    tag = "card_collection",
    request_body(content = UpdateCardCollectionData, description = "JSON request payload to update a CardCollection", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the CardCollection was updated"),
        (status = 400, description = "Service error relating to updating the CardCollection", body = [DefaultError]),
    ),
)]
pub async fn update_card_collection(
    body: web::Json<UpdateCardCollectionData>,
    pool: web::Data<Pool>,
    dataset: Dataset,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let name = body.name.clone();
    let description = body.description.clone();
    let is_public = body.is_public;
    let collection_id = body.collection_id;
    let dataset_id = dataset.id;

    let pool2 = pool.clone();

    let collection = user_owns_collection(user.id, collection_id, dataset_id, pool).await?;

    web::block(move || {
        update_card_collection_query(collection, name, description, is_public, dataset_id, pool2)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct AddCardToCollectionData {
    pub card_metadata_id: uuid::Uuid,
}

#[utoipa::path(
    post,
    path = "/card_collection/{page_or_card_collection_id}",
    context_path = "/api",
    tag = "card_collection",
    request_body(content = AddCardToCollectionData, description = "JSON request payload to add a card to a collection (bookmark it)", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the card was added to the collection"),
        (status = 400, description = "Service error relating to getting the collections that the card is in", body = [DefaultError]),
    ),
    params(
        ("page_or_card_collection_id" = uuid::Uuid, description = "The id of the collection to add the card to"),
    ),
)]
pub async fn add_bookmark(
    body: web::Json<AddCardToCollectionData>,
    collection_id: web::Path<uuid::Uuid>,
    dataset: Dataset,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let pool2 = pool.clone();
    let card_metadata_id = body.card_metadata_id;
    let collection_id = collection_id.into_inner();
    let dataset_id = dataset.id;

    user_owns_collection(user.id, collection_id, dataset_id, pool).await?;

    web::block(move || {
        create_card_bookmark_query(
            pool2,
            CardCollectionBookmark::from_details(collection_id, card_metadata_id),
        )
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::NoContent().finish())
}
#[derive(Deserialize, Serialize, ToSchema)]
pub struct BookmarkData {
    pub bookmarks: Vec<BookmarkCards>,
    pub collection: CardCollection,
    pub total_pages: i64,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GetAllBookmarksData {
    pub collection_id: uuid::Uuid,
    pub page: Option<u64>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct BookmarkCards {
    pub metadata: Vec<CardMetadataWithFileData>,
}

#[utoipa::path(
    get,
    path = "/card_collection/{collection_id}/{page}",
    context_path = "/api",
    tag = "card_collection",
    responses(
        (status = 200, description = "Bookmark'ed cards present within the specified collection", body = [BookmarkData]),
        (status = 400, description = "Service error relating to getting the collections that the card is in", body = [DefaultError]),
    ),
    params(
        ("collection_id" = uuid::Uuid, description = "The id of the collection to get the cards from"),
        ("page" = u64, description = "The page of cards to get from the collection"),
    ),
)]
pub async fn get_all_bookmarks(
    path_data: web::Path<GetAllBookmarksData>,
    pool: web::Data<Pool>,
    user: Option<LoggedUser>,
    dataset: Dataset,
    _required_user: RequireAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let collection_id = path_data.collection_id;
    let page = path_data.page.unwrap_or(1);
    let pool1 = pool.clone();
    let pool2 = pool.clone();
    let current_user_id = user.map(|user| user.id);
    let dataset_id = dataset.id;

    let bookmarks = {
        web::block(move || {
            get_bookmarks_for_collection_query(
                collection_id,
                page,
                None,
                current_user_id,
                dataset_id,
                pool2,
            )
        })
        .await?
        .map_err(<ServiceError as std::convert::Into<actix_web::Error>>::into)?
    };

    let point_ids = bookmarks
        .metadata
        .iter()
        .map(|point| point.qdrant_point_id)
        .collect::<Vec<uuid::Uuid>>();

    let collided_cards = {
        web::block(move || get_collided_cards_query(point_ids, current_user_id, dataset_id, pool1))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?
    };

    let collection_cards = bookmarks
        .metadata
        .iter()
        .map(|search_result| {
            let mut collided_cards: Vec<CardMetadataWithFileData> = collided_cards
                .iter()
                .filter(|card| {
                    card.1 == search_result.qdrant_point_id && card.0.id != search_result.id
                })
                .map(|card| card.0.clone())
                .collect();

            // de-duplicate collided cards by removing cards with the same metadata: Option<serde_json::Value>
            let mut seen_metadata = HashSet::new();
            let mut i = 0;
            while i < collided_cards.len() {
                let metadata_string = serde_json::to_string(&collided_cards[i].metadata).unwrap();

                if seen_metadata.contains(&metadata_string) {
                    collided_cards.remove(i);
                } else {
                    seen_metadata.insert(metadata_string);
                    i += 1;
                }
            }

            collided_cards.insert(0, search_result.clone());

            // Move the card that was searched for to the front of the list
            let (matching, others): (Vec<_>, Vec<_>) = collided_cards
                .clone()
                .into_iter()
                .partition(|item| item.id == search_result.id);

            collided_cards.clear();
            collided_cards.extend(matching);
            collided_cards.extend(others);

            BookmarkCards {
                metadata: collided_cards,
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(BookmarkData {
        bookmarks: collection_cards,
        collection: bookmarks.collection,
        total_pages: bookmarks.total_pages,
    }))
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct GetCollectionsForCardsData {
    pub card_ids: Vec<uuid::Uuid>,
}

#[utoipa::path(
    post,
    path = "/card_collection/bookmark",
    context_path = "/api",
    tag = "card_collection",
    request_body(content = GetCollectionsForCardsData, description = "JSON request payload to get the collections that a card is in", content_type = "application/json"),
    responses(
        (status = 200, description = "JSON body representing the collections that the card is in", body = [Vec<BookmarkCollectionResult>]),
        (status = 400, description = "Service error relating to getting the collections that the card is in", body = [DefaultError]),
    ),
)]
pub async fn get_collections_card_is_in(
    data: web::Json<GetCollectionsForCardsData>,
    pool: web::Data<Pool>,
    dataset: Dataset,
    user: Option<LoggedUser>,
    _required_user: RequireAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let card_ids = data.card_ids.clone();

    let dataset_id = dataset.id;
    let current_user_id = user.map(|user| user.id);

    let collections = web::block(move || {
        get_collections_for_bookmark_query(card_ids, current_user_id, dataset_id, pool)
    })
    .await?
    .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    Ok(HttpResponse::Ok().json(collections))
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct RemoveBookmarkData {
    pub card_metadata_id: uuid::Uuid,
}

#[utoipa::path(
    delete,
    path = "/card_collection/{page_or_card_collection_id}",
    context_path = "/api",
    tag = "card_collection",
    request_body(content = RemoveBookmarkData, description = "JSON request payload to remove a card to a collection (un-bookmark it)", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the card was removed to the collection"),
        (status = 400, description = "Service error relating to removing the card from the collection", body = [DefaultError]),
    ),
    params(
        ("page_or_card_collection_id" = uuid::Uuid, description = "The id of the collection to remove the bookmark'ed card from"),
    ),
)]
pub async fn delete_bookmark(
    collection_id: web::Path<uuid::Uuid>,
    body: web::Json<RemoveBookmarkData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
    dataset: Dataset,
) -> Result<HttpResponse, actix_web::Error> {
    let collection_id = collection_id.into_inner();
    let bookmark_id = body.card_metadata_id;
    let dataset_id = dataset.id;

    {
        let pool = pool.clone();
        user_owns_collection(user.id, collection_id, dataset_id, pool).await?;
    }

    {
        let pool = pool.clone();
        web::block(move || delete_bookmark_query(bookmark_id, collection_id, dataset_id, pool))
            .await?
            .map_err(|err| ServiceError::BadRequest(err.message.into()))?;
    }

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct GenerateOffCollectionData {
    pub collection_id: uuid::Uuid,
    pub page: Option<u64>,
    pub query: Option<String>,
}

#[utoipa::path(
    post,
    path = "/card_collection/generate",
    context_path = "/api",
    tag = "card_collection",
    request_body(content = GenerateOffCollectionData, description = "JSON request payload to generate off of a CardCollection", content_type = "application/json"),
    responses(
        (status = 200, description = "This will be a HTTP stream, check the chat or search UI for an example how to process this"),
        (status = 400, description = "Service error relating to generating off the CardCollection", body = [DefaultError]),
    ),
)]
pub async fn generate_off_collection(
    body: web::Json<GenerateOffCollectionData>,
    pool: web::Data<Pool>,
    dataset: Dataset,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let request_data = body.into_inner();
    let collection_id = request_data.collection_id;
    let page = request_data.page.unwrap_or(1);
    let dataset_id = dataset.id;

    let collection_bookmarks = {
        web::block(move || {
            get_bookmarks_for_collection_query(
                collection_id,
                page,
                Some(10),
                Some(user.id),
                dataset_id,
                pool,
            )
        })
        .await??
        .metadata
    };

    let query = request_data.query.unwrap_or(
        "Now, provide a multi paragraph summary of the information I provided.".to_string(),
    );

    let mut messages: Vec<ChatMessage> = Vec::new();
    messages.push(ChatMessage {
        role: Role::User,
        content: Some("I am going to provide several pieces of information for you to use in response to a request or question. You will not respond until I ask you to.".to_string()),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    });
    messages.push(ChatMessage {
        role: Role::Assistant,
        content: Some(
            "Understood, I will not reply until I receive a direct request or question."
                .to_string(),
        ),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    });
    collection_bookmarks.iter().for_each(|bookmark| {
        messages.push(ChatMessage {
            role: Role::User,
            content: Some(bookmark.content.clone()),
            tool_calls: None,
            name: None,
            tool_call_id: None,
        });
        messages.push(ChatMessage {
            role: Role::Assistant,
            content: Some("".to_string()),
            tool_calls: None,
            name: None,
            tool_call_id: None,
        });
    });
    messages.push(ChatMessage {
        role: Role::User,
        content: Some(query),
        tool_calls: None,
        name: None,
        tool_call_id: None,
    });

    let summary_completion_param = ChatCompletionParameters {
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
        respsonse_format: None,
        tools: None,
        tool_choice: None,
    };

    let open_ai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::new(open_ai_api_key);

    let (s, _r) = unbounded::<String>();
    let stream = client
        .chat()
        .create_stream(summary_completion_param)
        .await
        .expect("Failed to create chat");

    Ok(HttpResponse::Ok().streaming(stream.map(
        move |response| -> Result<Bytes, actix_web::Error> {
            if let Ok(response) = response {
                let chat_content = response.choices[0].delta.content.clone();
                if let Some(message) = chat_content.clone() {
                    let _ = s.send(message);
                }
                return Ok(Bytes::from(chat_content.unwrap_or("".to_string())));
            }
            Err(ServiceError::InternalServerError(
                "Model Response Error. Please try again later".into(),
            )
            .into())
        },
    )))
}
