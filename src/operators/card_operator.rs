use crate::data::models::CardMetadataWithVotes;
use crate::data::models::CardVote;
use crate::data::models::User;
use crate::data::models::UserDTO;
use crate::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use crate::{
    data::models::{CardMetadata, Pool},
    errors::DefaultError,
};
use actix_web::web;
use openai_dive::v1::{api::Client, resources::embedding::EmbeddingParameters};
use qdrant_client::{
    prelude::{QdrantClient, QdrantClientConfig},
    qdrant::{point_id::PointIdOptions, SearchPoints},
};
use serde::{Deserialize, Serialize};

pub async fn get_qdrant_connection() -> Result<QdrantClient, DefaultError> {
    let qdrant_url = std::env::var("QDRANT_URL").expect("QDRANT_URL must be set");
    QdrantClient::new(Some(QdrantClientConfig::from_url(qdrant_url.as_str())))
        .await
        .map_err(|_err| DefaultError {
            message: "Failed to connect to Qdrant",
        })
}

pub async fn create_openai_embedding(message: &str) -> Result<Vec<f32>, actix_web::Error> {
    let open_ai_api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
    let client = Client::new(open_ai_api_key);

    // Vectorize
    let parameters = EmbeddingParameters {
        model: "text-embedding-ada-002".to_string(),
        input: message.to_string(),
    };

    let embeddings = client
        .embeddings()
        .create(parameters)
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    let vector = embeddings.data.get(0).unwrap().embedding.clone();
    Ok(vector.iter().map(|&x| x as f32).collect())
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    pub score: f32,
    pub point_id: uuid::Uuid,
}

pub async fn search_card_query(
    embedding_vector: Vec<f32>,
    page: u64,
) -> Result<Vec<SearchResult>, actix_web::Error> {
    let page = if page == 0 { 1 } else { page };

    let qdrant = get_qdrant_connection()
        .await
        .map_err(|err| actix_web::error::ErrorBadRequest(err.message))?;

    let data = qdrant
        .search_points(&SearchPoints {
            collection_name: "debate_cards".to_string(),
            vector: embedding_vector,
            limit: 25,
            offset: Some((page - 1) * 25),
            with_payload: None,
            ..Default::default()
        })
        .await
        .map_err(actix_web::error::ErrorBadRequest)?;

    let point_ids: Vec<SearchResult> = data
        .result
        .iter()
        .filter_map(|point| match point.clone().id?.point_id_options? {
            PointIdOptions::Uuid(id) => Some(SearchResult {
                score: point.score,
                point_id: uuid::Uuid::parse_str(&id).ok()?,
            }),
            PointIdOptions::Num(_) => None,
        })
        .collect();

    Ok(point_ids)
}

#[derive(Serialize, Deserialize)]
pub struct ScoredCardDTO {
    pub metadata: CardMetadata,
    pub score: f32,
}

pub fn get_metadata_from_point_ids(
    point_ids: Vec<uuid::Uuid>,
    current_user_id: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<CardMetadataWithVotes>, DefaultError> {
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;
    use crate::data::schema::card_votes::dsl as card_votes_columns;
    use crate::data::schema::users::dsl as user_columns;

    let mut conn = pool.get().unwrap();

    let card_metadata: Vec<CardMetadata> = card_metadata_columns::card_metadata
        .filter(card_metadata_columns::qdrant_point_id.eq_any(point_ids))
        .load::<CardMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    let card_creators: Vec<User> = user_columns::users
        .filter(
            user_columns::id.eq_any(
                card_metadata
                    .iter()
                    .map(|metadata| metadata.author_id)
                    .collect::<Vec<uuid::Uuid>>(),
            ),
        )
        .load::<User>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load card creators",
        })?;

    let card_votes: Vec<CardVote> = card_votes_columns::card_votes
        .filter(
            card_votes_columns::card_metadata_id.eq_any(
                card_metadata
                    .iter()
                    .map(|metadata| metadata.id)
                    .collect::<Vec<uuid::Uuid>>(),
            ),
        )
        .load::<CardVote>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load upvotes",
        })?;

    let card_metadata_with_upvotes: Vec<CardMetadataWithVotes> = card_metadata
        .into_iter()
        .map(|metadata| {
            let votes = card_votes
                .iter()
                .filter(|upvote| upvote.card_metadata_id == metadata.id)
                .collect::<Vec<&CardVote>>();
            let total_upvotes = votes.iter().filter(|upvote| upvote.vote).count() as i64;
            let total_downvotes = votes.iter().filter(|upvote| !upvote.vote).count() as i64;
            let vote_by_current_user = match current_user_id {
                Some(user_id) => votes
                    .iter()
                    .find(|upvote| upvote.voted_user_id == user_id)
                    .map(|upvote| upvote.vote),
                None => None,
            };

            let author = card_creators
                .iter()
                .find(|user| user.id == metadata.author_id)
                .map(|user| UserDTO {
                    id: user.id,
                    username: user.username.clone(),
                    email: if user.visible_email {
                        Some(user.email.clone())
                    } else {
                        None
                    },
                    website: user.website.clone(),
                    visible_email: user.visible_email,
                    created_at: user.created_at,
                });

            CardMetadataWithVotes {
                id: metadata.id,
                content: metadata.content,
                link: metadata.link,
                oc_file_path: metadata.oc_file_path,
                author,
                qdrant_point_id: metadata.qdrant_point_id,
                total_upvotes,
                total_downvotes,
                vote_by_current_user,
                created_at: metadata.created_at,
                updated_at: metadata.updated_at,
            }
        })
        .collect();

    Ok(card_metadata_with_upvotes)
}

pub fn get_metadata_from_id_query(
    card_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<CardMetadata, DefaultError> {
    use crate::data::schema::card_metadata::dsl::*;

    let mut conn = pool.get().unwrap();

    card_metadata
        .filter(id.eq(card_id))
        .first::<CardMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })
}

pub fn insert_card_metadata_query(
    card_data: CardMetadata,
    pool: &web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::card_metadata::dsl::*;

    let mut conn = pool.get().unwrap();

    diesel::insert_into(card_metadata)
        .values(&card_data)
        .execute(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Failed to insert card metadata",
        })?;

    Ok(())
}
