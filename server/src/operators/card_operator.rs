use super::tantivy_operator::TantivyIndexMap;
use crate::data::models::{
    CardCollisions, CardFile, CardMetadataWithVotes, CardMetadataWithVotesWithScore,
    FullTextSearchResult,
};
use crate::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use crate::operators::qdrant_operator::{create_embedding, get_qdrant_connection};
use crate::operators::search_operator::get_metadata_query;
use crate::AppMutexStore;
use crate::{
    data::models::{CardMetadata, Pool},
    errors::DefaultError,
};
use actix_web::web;
use diesel::dsl::sql;
use diesel::sql_types::BigInt;
use diesel::sql_types::Text;
use diesel::{
    BoolExpressionMethods, Connection, JoinOnDsl, NullableExpressionMethods, SelectableHelper,
};
use itertools::Itertools;
use qdrant_client::qdrant::{PointId, PointVectors};
use serde::{Deserialize, Serialize};
use simsearch::SimSearch;
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize)]
pub struct ScoredCardDTO {
    pub metadata: CardMetadata,
    pub score: f32,
}

pub fn get_metadata_from_point_ids(
    point_ids: Vec<uuid::Uuid>,
    current_user_id: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<CardMetadataWithVotesWithScore>, DefaultError> {
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;

    let mut conn = pool.get().unwrap();

    let card_metadata: Vec<CardMetadata> = card_metadata_columns::card_metadata
        .filter(card_metadata_columns::qdrant_point_id.eq_any(&point_ids))
        .select((
            card_metadata_columns::id,
            card_metadata_columns::content,
            card_metadata_columns::link,
            card_metadata_columns::author_id,
            card_metadata_columns::qdrant_point_id,
            card_metadata_columns::created_at,
            card_metadata_columns::updated_at,
            card_metadata_columns::tag_set,
            card_metadata_columns::card_html,
            card_metadata_columns::private,
            card_metadata_columns::metadata,
            card_metadata_columns::tracking_id,
            card_metadata_columns::time_stamp,
            card_metadata_columns::dataset,
        ))
        .load::<CardMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    let converted_cards: Vec<FullTextSearchResult> = card_metadata
        .iter()
        .map(|card| <CardMetadata as Into<FullTextSearchResult>>::into(card.clone()))
        .collect::<Vec<FullTextSearchResult>>();

    let card_metadata_with_upvotes_and_file_id =
        get_metadata_query(converted_cards, current_user_id, conn).map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    //combine card_metadata_with vote with the file_ids that was loaded

    Ok(card_metadata_with_upvotes_and_file_id)
}

pub struct CardMetadataWithQdrantId {
    pub metadata: CardMetadataWithVotesWithScore,
    pub qdrant_id: uuid::Uuid,
}

pub fn get_metadata_and_collided_cards_from_point_ids_query(
    point_ids: Vec<uuid::Uuid>,
    current_user_id: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<
    (
        Vec<CardMetadataWithVotesWithScore>,
        Vec<CardMetadataWithQdrantId>,
    ),
    DefaultError,
> {
    use crate::data::schema::card_collisions::dsl as card_collisions_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;

    let card_search_result = {
        let mut conn = pool.get().unwrap();
        let card_metadata: Vec<CardMetadata> = card_metadata_columns::card_metadata
            .filter(card_metadata_columns::qdrant_point_id.eq_any(&point_ids))
            .select((
                card_metadata_columns::id,
                card_metadata_columns::content,
                card_metadata_columns::link,
                card_metadata_columns::author_id,
                card_metadata_columns::qdrant_point_id,
                card_metadata_columns::created_at,
                card_metadata_columns::updated_at,
                card_metadata_columns::tag_set,
                card_metadata_columns::card_html,
                card_metadata_columns::private,
                card_metadata_columns::metadata,
                card_metadata_columns::tracking_id,
                card_metadata_columns::time_stamp,
                card_metadata_columns::dataset,
            ))
            .limit(500)
            .load::<CardMetadata>(&mut conn)
            .map_err(|_| DefaultError {
                message: "Failed to load metadata",
            })?;

        card_metadata
            .iter()
            .map(|card| <CardMetadata as Into<FullTextSearchResult>>::into(card.clone()))
            .collect::<Vec<FullTextSearchResult>>()
    };

    let (collided_search_result, collided_qdrant_ids) = {
        let mut conn = pool.get().unwrap();
        let card_metadata: Vec<(CardMetadata, uuid::Uuid)> =
            card_collisions_columns::card_collisions
                .inner_join(
                    card_metadata_columns::card_metadata
                        .on(card_metadata_columns::id.eq(card_collisions_columns::card_id)),
                )
                .select((
                    (
                        card_metadata_columns::id,
                        card_metadata_columns::content,
                        card_metadata_columns::link,
                        card_metadata_columns::author_id,
                        card_metadata_columns::qdrant_point_id,
                        card_metadata_columns::created_at,
                        card_metadata_columns::updated_at,
                        card_metadata_columns::tag_set,
                        card_metadata_columns::card_html,
                        card_metadata_columns::private,
                        card_metadata_columns::metadata,
                        card_metadata_columns::tracking_id,
                        card_metadata_columns::time_stamp,
            card_metadata_columns::dataset,
                    ),
                    (card_collisions_columns::collision_qdrant_id.assume_not_null()),
                ))
                .filter(card_collisions_columns::collision_qdrant_id.eq_any(point_ids))
                .filter(
                    card_metadata_columns::private
                        .eq(false)
                        .or(card_metadata_columns::author_id
                            .eq(current_user_id.unwrap_or(uuid::Uuid::nil()))),
                )
                // TODO: Properly handle this and remove the arbitrary limit
                .limit(500)
                .load::<(CardMetadata, uuid::Uuid)>(&mut conn)
                .map_err(|_| DefaultError {
                    message: "Failed to load metadata",
                })?;

        let collided_qdrant_ids = card_metadata
            .iter()
            .map(|(_, qdrant_id)| *qdrant_id)
            .collect::<Vec<uuid::Uuid>>();

        let converted_cards: Vec<FullTextSearchResult> = card_metadata
            .iter()
            .map(|card| <CardMetadata as Into<FullTextSearchResult>>::into(card.0.clone()))
            .collect::<Vec<FullTextSearchResult>>();

        (converted_cards, collided_qdrant_ids)
    };

    let (card_metadata_with_upvotes_and_file_id, collided_card_metadata_with_upvotes_and_file_id) = {
        let conn = pool.get().unwrap();
        // Assuming that get_metadata will maintain the order of the Vec<> returned
        let split_index = card_search_result.len();
        let all_cards = card_search_result
            .iter()
            .chain(collided_search_result.iter())
            .cloned()
            .collect::<Vec<FullTextSearchResult>>();

        let all_metadata =
            get_metadata_query(all_cards, current_user_id, conn).map_err(|_| DefaultError {
                message: "Failed to load metadata",
            })?;

        let meta_cards = all_metadata
            .iter()
            .take(split_index)
            .cloned()
            .collect::<Vec<CardMetadataWithVotesWithScore>>();

        let meta_collided = all_metadata
            .iter()
            .skip(split_index)
            .cloned()
            .collect::<Vec<CardMetadataWithVotesWithScore>>();

        (meta_cards, meta_collided)
    };

    let card_metadatas_with_collided_qdrant_ids = collided_card_metadata_with_upvotes_and_file_id
        .iter()
        .zip(collided_qdrant_ids.iter())
        .map(|(card, qdrant_id)| CardMetadataWithQdrantId {
            metadata: card.clone(),
            qdrant_id: *qdrant_id,
        })
        .collect::<Vec<CardMetadataWithQdrantId>>();

    Ok((
        card_metadata_with_upvotes_and_file_id,
        card_metadatas_with_collided_qdrant_ids,
    ))
}

pub fn get_collided_cards_query(
    point_ids: Vec<uuid::Uuid>,
    current_user_id: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<Vec<(CardMetadataWithVotesWithScore, uuid::Uuid)>, DefaultError> {
    use crate::data::schema::card_collisions::dsl as card_collisions_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;

    let mut conn = pool.get().unwrap();

    let card_metadata: Vec<CardMetadata> = card_metadata_columns::card_metadata
        .left_outer_join(
            card_collisions_columns::card_collisions
                .on(card_metadata_columns::id.eq(card_collisions_columns::card_id)),
        )
        .select((
            card_metadata_columns::id,
            card_metadata_columns::content,
            card_metadata_columns::link,
            card_metadata_columns::author_id,
            card_metadata_columns::qdrant_point_id,
            card_metadata_columns::created_at,
            card_metadata_columns::updated_at,
            card_metadata_columns::tag_set,
            card_metadata_columns::card_html,
            card_metadata_columns::private,
            card_metadata_columns::metadata,
            card_metadata_columns::tracking_id,
            card_metadata_columns::time_stamp,
            card_metadata_columns::dataset,
        ))
        .filter(
            card_collisions_columns::collision_qdrant_id
                .eq_any(point_ids.clone())
                .or(card_metadata_columns::qdrant_point_id.eq_any(point_ids)),
        )
        .filter(
            card_metadata_columns::private
                .eq(false)
                .or(card_metadata_columns::author_id.eq(current_user_id.unwrap_or_default())),
        )
        // TODO: Properly handle this and remove the arbitrary limit
        .limit(500)
        .load::<CardMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    let converted_cards: Vec<FullTextSearchResult> = card_metadata
        .iter()
        .map(|card| <CardMetadata as Into<FullTextSearchResult>>::into(card.clone()))
        .collect::<Vec<FullTextSearchResult>>();

    let card_metadata_with_upvotes_and_file_id =
        get_metadata_query(converted_cards, current_user_id, conn).map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    let card_metadatas_with_collided_qdrant_ids = card_metadata_with_upvotes_and_file_id
        .iter()
        .map(|card| (card.clone(), card.qdrant_point_id))
        .collect::<Vec<(CardMetadataWithVotesWithScore, uuid::Uuid)>>();

    Ok(card_metadatas_with_collided_qdrant_ids)
}

pub fn get_metadata_from_id_query(
    card_id: uuid::Uuid,
    dataset: String,
    pool: web::Data<Pool>,
) -> Result<CardMetadata, DefaultError> {
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;
    let mut conn = pool.get().unwrap();

    card_metadata_columns::card_metadata
        .filter(card_metadata_columns::id.eq(card_id))
        .filter(card_metadata_columns::dataset.eq(dataset))
        .select(CardMetadata::as_select())
        .first::<CardMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })
}

pub fn get_metadata_from_tracking_id_query(
    tracking_id: String,
    dataset: String,
    pool: web::Data<Pool>,
) -> Result<CardMetadata, DefaultError> {
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;

    let mut conn = pool.get().unwrap();

    card_metadata_columns::card_metadata
        .filter(card_metadata_columns::tracking_id.eq(tracking_id))
        .filter(card_metadata_columns::dataset.eq(dataset))
        .select(CardMetadata::as_select())
        .first::<CardMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })
}

pub fn get_metadata_from_ids_query(
    card_ids: Vec<uuid::Uuid>,
    user_id: uuid::Uuid,
    dataset_name: String,
    pool: web::Data<Pool>,
) -> Result<Vec<CardMetadataWithVotesWithScore>, DefaultError> {
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;

    let mut conn = pool.get().unwrap();

    let metadatas: Vec<CardMetadata> = card_metadata_columns::card_metadata
        .filter(card_metadata_columns::id.eq_any(card_ids))
        .filter(card_metadata_columns::dataset.eq(dataset_name))
        .select((
            card_metadata_columns::id,
            card_metadata_columns::content,
            card_metadata_columns::link,
            card_metadata_columns::author_id,
            card_metadata_columns::qdrant_point_id,
            card_metadata_columns::created_at,
            card_metadata_columns::updated_at,
            card_metadata_columns::tag_set,
            card_metadata_columns::card_html,
            card_metadata_columns::private,
            card_metadata_columns::metadata,
            card_metadata_columns::tracking_id,
            card_metadata_columns::time_stamp,
            card_metadata_columns::dataset,
        ))
        .load::<CardMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;
    let full_text_metadatas = metadatas
        .iter()
        .map_into::<FullTextSearchResult>()
        .collect_vec();

    Ok(get_metadata_query(full_text_metadatas, Some(user_id), conn).unwrap_or_default())
}

pub fn get_metadata_and_votes_from_id_query(
    card_id: uuid::Uuid,
    current_user_id: Option<uuid::Uuid>,
    dataset_name: String,
    pool: web::Data<Pool>,
) -> Result<CardMetadataWithVotesWithScore, DefaultError> {
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;

    let mut conn = pool.get().unwrap();

    let card_metadata = card_metadata_columns::card_metadata
        .filter(card_metadata_columns::id.eq(card_id))
        .filter(card_metadata_columns::dataset.eq(dataset_name))
        .select(CardMetadata::as_select())
        .first::<CardMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;
    let converted_card: FullTextSearchResult =
        <CardMetadata as Into<FullTextSearchResult>>::into(card_metadata);

    let card_metadata_with_upvotes_and_file_id =
        get_metadata_query(vec![converted_card], current_user_id, conn).map_err(|_| {
            DefaultError {
                message: "Failed to load metadata",
            }
        })?;
    Ok(card_metadata_with_upvotes_and_file_id
        .first()
        .expect("card_metadata_with_upvotes_and_file_id should have at least one element")
        .clone())
}

pub fn get_metadata_and_votes_from_tracking_id_query(
    tracking_id: String,
    current_user_id: Option<uuid::Uuid>,
    dataset_name: String,
    pool: web::Data<Pool>,
) -> Result<CardMetadataWithVotesWithScore, DefaultError> {
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;

    let mut conn = pool.get().unwrap();

    let card_metadata = card_metadata_columns::card_metadata
        .filter(card_metadata_columns::tracking_id.eq(tracking_id))
        .filter(card_metadata_columns::dataset.eq(dataset_name.clone()))
        .select(CardMetadata::as_select())
        .first::<CardMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;
    let converted_card: FullTextSearchResult =
        <CardMetadata as Into<FullTextSearchResult>>::into(card_metadata);

    let card_metadata_with_upvotes_and_file_id =
        get_metadata_query(vec![converted_card], current_user_id, conn).map_err(|_| {
            DefaultError {
                message: "Failed to load metadata",
            }
        })?;
    Ok(card_metadata_with_upvotes_and_file_id
        .first()
        .expect("card_metadata_with_upvotes_and_file_id should have at least one element")
        .clone())
}

pub async fn insert_card_metadata_query(
    card_data: CardMetadata,
    file_uuid: Option<uuid::Uuid>,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    dataset_name: String,
    pool: web::Data<Pool>,
) -> Result<CardMetadata, DefaultError> {
    use crate::data::schema::card_files::dsl as card_files_columns;
    use crate::data::schema::card_metadata::dsl::*;

    let mut conn = pool.get().unwrap();

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::insert_into(card_metadata)
            .values(&card_data)
            .execute(conn)?;

        if file_uuid.is_some() {
            diesel::insert_into(card_files_columns::card_files)
                .values(&CardFile::from_details(
                    card_data.id,
                    file_uuid.expect("file_uuid should be Some"),
                ))
                .execute(conn)?;
        }

        Ok(())
    });

    let tantivy_index_map = tantivy_index_map.read().await;

    match transaction_result {
        Ok(_) => tantivy_index_map
            .add_card(Some(dataset_name.as_str()), card_data.clone())
            .map_err(|e| {
                log::info!("Failed to add card to index: {:?}", e);
                DefaultError {
                    message: "Failed to add card to index",
                }
            })?,
        Err(_) => {
            return Err(DefaultError {
                message: "Failed to insert card metadata, likely due to duplicate tracking_id",
            })
        }
    };

    Ok(card_data)
}

pub fn insert_duplicate_card_metadata_query(
    card_data: CardMetadata,
    duplicate_card: uuid::Uuid,
    file_uuid: Option<uuid::Uuid>,
    pool: web::Data<Pool>,
) -> Result<CardMetadata, DefaultError> {
    use crate::data::schema::card_collisions::dsl::*;
    use crate::data::schema::card_files::dsl as card_files_columns;
    use crate::data::schema::card_metadata::dsl::*;

    let mut conn = pool.get().unwrap();

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::insert_into(card_metadata)
            .values(&card_data)
            .execute(conn)?;

        //insert duplicate into card_collisions
        diesel::insert_into(card_collisions)
            .values(&CardCollisions::from_details(card_data.id, duplicate_card))
            .execute(conn)?;

        if file_uuid.is_some() {
            diesel::insert_into(card_files_columns::card_files)
                .values(&CardFile::from_details(
                    card_data.id,
                    file_uuid.expect("file_uuid should be some"),
                ))
                .execute(conn)?;
        }

        Ok(())
    });

    match transaction_result {
        Ok(_) => (),
        Err(_) => {
            return Err(DefaultError {
                message: "Failed to insert duplicate card metadata",
            })
        }
    };
    Ok(card_data)
}

pub async fn update_card_metadata_query(
    card_data: CardMetadata,
    file_uuid: Option<uuid::Uuid>,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::card_files::dsl as card_files_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;
    use crate::data::schema::card_votes::dsl as card_votes_columns;

    let mut conn = pool.get().unwrap();
    let card_data_1 = card_data.clone();

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::update(
            card_metadata_columns::card_metadata.filter(card_metadata_columns::id.eq(card_data.id)),
        )
        .set((
            card_metadata_columns::link.eq(card_data.link),
            card_metadata_columns::card_html.eq(card_data.card_html),
            card_metadata_columns::private.eq(card_data.private),
            card_metadata_columns::metadata.eq(card_data.metadata),
        ))
        .execute(conn)?;

        diesel::update(
            card_votes_columns::card_votes
                .filter(card_votes_columns::card_metadata_id.eq(card_data.id)),
        )
        .set(card_votes_columns::deleted.eq(card_data.private))
        .execute(conn)?;

        if file_uuid.is_some() {
            diesel::insert_into(card_files_columns::card_files)
                .values(&CardFile::from_details(
                    card_data.id,
                    file_uuid.expect("file_uuid should be some"),
                ))
                .execute(conn)?;
        }
        Ok(())
    });

    match transaction_result {
        Ok(_) => {
            let tantivy_index_map = tantivy_index_map.read().await;
            tantivy_index_map
                .update_card(None, card_data_1)
                .map_err(|_e| DefaultError {
                    message: "Failed to add card to index",
                })?
        }
        Err(_) => {
            return Err(DefaultError {
                message: "Failed to update card metadata",
            })
        }
    };

    Ok(())
}

enum TransactionResult {
    CardCollisionDetected(CardMetadata),
    CardCollisionNotDetected,
}

pub async fn delete_card_metadata_query(
    card_uuid: uuid::Uuid,
    qdrant_point_id: Option<uuid::Uuid>,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    app_mutex: web::Data<AppMutexStore>,
    dataset_name: String,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::card_collection_bookmarks::dsl as card_collection_bookmarks_columns;
    use crate::data::schema::card_collisions::dsl as card_collisions_columns;
    use crate::data::schema::card_files::dsl as card_files_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;
    let mut conn = pool.get().unwrap();

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        {
            diesel::delete(
                card_files_columns::card_files.filter(card_files_columns::card_id.eq(card_uuid)),
            )
            .execute(conn)?;

            diesel::delete(
                card_collection_bookmarks_columns::card_collection_bookmarks
                    .filter(card_collection_bookmarks_columns::card_metadata_id.eq(card_uuid)),
            )
            .execute(conn)?;

            let deleted_card_collision_count = diesel::delete(
                card_collisions_columns::card_collisions
                    .filter(card_collisions_columns::card_id.eq(card_uuid)),
            )
            .execute(conn)?;

            if deleted_card_collision_count > 0 {
                // there cannot be collisions for a collision, just delete the card_metadata without issue
                diesel::delete(
                    card_metadata_columns::card_metadata
                        .filter(card_metadata_columns::id.eq(card_uuid)),
                )
                .execute(conn)?;

                return Ok(TransactionResult::CardCollisionNotDetected);
            }

            let card_collisions: Vec<(CardCollisions, CardMetadata)> =
                card_collisions_columns::card_collisions
                    .inner_join(
                        card_metadata_columns::card_metadata
                            .on(card_metadata_columns::qdrant_point_id
                                .eq(card_collisions_columns::collision_qdrant_id)),
                    )
                    .filter(card_metadata_columns::id.eq(card_uuid))
                    .select((CardCollisions::as_select(), CardMetadata::as_select()))
                    .order_by(card_collisions_columns::created_at.asc())
                    .load::<(CardCollisions, CardMetadata)>(conn)?;

            if !card_collisions.is_empty() {
                // get the first collision that is public or the first collision if all are private
                let latest_collision = match card_collisions.iter().find(|x| !x.1.private) {
                    Some(x) => x.0.clone(),
                    None => card_collisions[0].0.clone(),
                };

                let latest_collision_metadata = match card_collisions.iter().find(|x| !x.1.private)
                {
                    Some(x) => x.1.clone(),
                    None => card_collisions[0].1.clone(),
                };

                // update all collisions except latest_collision to point to a qdrant_id of None
                diesel::update(
                    card_collisions_columns::card_collisions.filter(
                        card_collisions_columns::id.eq_any(
                            card_collisions
                                .iter()
                                .filter(|x| x.0.id != latest_collision.id)
                                .map(|x| x.0.id)
                                .collect::<Vec<uuid::Uuid>>(),
                        ),
                    ),
                )
                .set(card_collisions_columns::collision_qdrant_id.eq::<Option<uuid::Uuid>>(None))
                .execute(conn)?;

                // delete latest_collision from card_collisions
                diesel::delete(
                    card_collisions_columns::card_collisions
                        .filter(card_collisions_columns::id.eq(latest_collision.id)),
                )
                .execute(conn)?;

                // delete the original card_metadata
                diesel::delete(
                    card_metadata_columns::card_metadata
                        .filter(card_metadata_columns::id.eq(card_uuid))
                        .filter(card_metadata_columns::dataset.eq(dataset_name.clone())),
                )
                .execute(conn)?;

                // set the card_metadata of latest_collision to have the qdrant_point_id of the original card_metadata
                diesel::update(
                    card_metadata_columns::card_metadata
                        .filter(card_metadata_columns::id.eq(latest_collision.card_id)),
                )
                .set((
                    card_metadata_columns::qdrant_point_id.eq(latest_collision.collision_qdrant_id),
                ))
                .execute(conn)?;

                // set the collision_qdrant_id of all other collisions to be the same as they were to begin with
                diesel::update(
                    card_collisions_columns::card_collisions.filter(
                        card_collisions_columns::id.eq_any(
                            card_collisions
                                .iter()
                                .skip(1)
                                .map(|x| x.0.id)
                                .collect::<Vec<uuid::Uuid>>(),
                        ),
                    ),
                )
                .set((card_collisions_columns::collision_qdrant_id
                    .eq(latest_collision.collision_qdrant_id),))
                .execute(conn)?;

                return Ok(TransactionResult::CardCollisionDetected(
                    latest_collision_metadata,
                ));
            }

            // if there were no collisions, just delete the card_metadata without issue
            diesel::delete(
                card_metadata_columns::card_metadata
                    .filter(card_metadata_columns::id.eq(card_uuid))
                    .filter(card_metadata_columns::dataset.eq(dataset_name.clone())),
            )
            .execute(conn)?;

            Ok(TransactionResult::CardCollisionNotDetected)
        }
    });

    let qdrant_collection = std::env::var("QDRANT_COLLECTION").unwrap_or("debate_cards".to_owned());
    match transaction_result {
        Ok(result) => match result {
            TransactionResult::CardCollisionNotDetected => {
                let qdrant = get_qdrant_connection().await?;
                let _ = qdrant
                    .delete_points(
                        qdrant_collection,
                        &vec![<String as Into<PointId>>::into(
                            qdrant_point_id.unwrap_or_default().to_string(),
                        )]
                        .into(),
                        None,
                    )
                    .await
                    .map_err(|_e| {
                        Err::<(), DefaultError>(DefaultError {
                            message: "Failed to delete card from qdrant",
                        })
                    });

                let tantivy_index_map = tantivy_index_map.read().await;

                tantivy_index_map
                    .delete_card(Some(dataset_name.as_str()), card_uuid)
                    .map_err(|_e| DefaultError {
                        message: "Failed to delete card from index",
                    })?;
            }
            TransactionResult::CardCollisionDetected(latest_collision_metadata) => {
                let qdrant = get_qdrant_connection().await?;
                let collision_content = latest_collision_metadata
                    .card_html
                    .clone()
                    .unwrap_or(latest_collision_metadata.content.clone());

                let new_embedding_vector = create_embedding(collision_content.as_str(), app_mutex)
                    .await
                    .map_err(|_e| DefaultError {
                        message: "Failed to create embedding for card",
                    })?;

                let _ = qdrant
                    .update_vectors_blocking(
                        qdrant_collection,
                        &[PointVectors {
                            id: Some(<String as Into<PointId>>::into(
                                qdrant_point_id.unwrap_or_default().to_string(),
                            )),
                            vectors: Some(new_embedding_vector.into()),
                        }],
                        None,
                    )
                    .await
                    .map_err(|_e| {
                        Err::<(), DefaultError>(DefaultError {
                            message: "Failed to update card in qdrant",
                        })
                    });

                let tantivy_index_map = tantivy_index_map.read().await;

                tantivy_index_map
                    .update_card(None, latest_collision_metadata)
                    .map_err(|_e| DefaultError {
                        message: "Failed to update card in index",
                    })?;
            }
        },

        Err(_) => {
            return Err(DefaultError {
                message: "Failed to delete card data",
            })
        }
    };

    Ok(())
}

pub async fn get_card_count_query(pool: web::Data<Pool>) -> Result<i64, DefaultError> {
    use crate::data::schema::card_metadata_count::dsl as card_metadata_count_columns;

    let mut conn = pool.get().unwrap();

    card_metadata_count_columns::card_metadata_count
        .select(card_metadata_count_columns::total_rows)
        .first(&mut conn)
        .map_err(|_err| DefaultError {
            message: "Failed to get card count",
        })
}

pub fn get_qdrant_id_from_card_id_query(
    card_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<uuid::Uuid, DefaultError> {
    use crate::data::schema::card_collisions::dsl as card_collisions_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;

    let mut conn = pool.get().unwrap();

    let qdrant_point_ids: Vec<(Option<uuid::Uuid>, Option<uuid::Uuid>)> =
        card_metadata_columns::card_metadata
            .left_outer_join(
                card_collisions_columns::card_collisions
                    .on(card_metadata_columns::id.eq(card_collisions_columns::card_id)),
            )
            .select((
                card_metadata_columns::qdrant_point_id,
                card_collisions_columns::collision_qdrant_id.nullable(),
            ))
            .filter(card_metadata_columns::id.eq(card_id))
            .load(&mut conn)
            .map_err(|_err| DefaultError {
                message: "Failed to get qdrant_point_id and collision_qdrant_id",
            })?;

    match qdrant_point_ids.first() {
        Some(x) => match x.0 {
            Some(y) => Ok(y),
            None => match x.1 {
                Some(y) => Ok(y),
                None => Err(DefaultError {
                    message: "Both qdrant_point_id and collision_qdrant_id are None",
                }),
            },
        },
        None => Err(DefaultError {
            message: "Failed to get qdrant_point_id for card_id",
        }),
    }
}

pub fn get_top_cards_query(
    page: u64,
    dataset_name: String,
    pool: web::Data<Pool>,
) -> Result<Vec<CardMetadataWithVotes>, DefaultError> {
    let page = if page == 0 { 1 } else { page };
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;
    use crate::data::schema::card_votes::dsl as card_votes_columns;

    let mut conn = pool.get().unwrap();

    let top_10_cards = card_metadata_columns::card_metadata
        .left_outer_join(
            card_votes_columns::card_votes
                .on(card_metadata_columns::id.eq(card_votes_columns::card_metadata_id)),
        )
        .select((CardMetadata::as_select(), diesel::dsl::sql::<BigInt>("(SUM(case when vote = true then 1 else 0 end) - SUM(case when vote = false then 1 else 0 end)) as score")))
        .filter(card_metadata_columns::private.eq(false))
        .filter(card_metadata_columns::dataset.eq(dataset_name))
        .group_by(card_metadata_columns::id)
        .order(sql::<Text>("score DESC"))
        .limit(5)
        .offset(
            ((page - 1) * 5)
                .try_into()
                .expect("Failed to convert u64 to i64"),
        )
        .load::<(CardMetadata, i64)>(&mut conn)
        .map_err(|err| {
            log::info!("Failed to get recently created cards: {:?}", err);
            DefaultError {
            message: "Failed to get recently created cards",
        }})?;

    let recent_ten_full_text_results: Vec<CardMetadataWithVotes> = top_10_cards
        .iter()
        .map(|x| CardMetadataWithVotes::from(x.clone()))
        .collect();

    Ok(recent_ten_full_text_results)
}

pub fn find_relevant_sentence(
    input: CardMetadataWithVotesWithScore,
    query: String,
) -> Result<CardMetadataWithVotesWithScore, DefaultError> {
    let content = &input.card_html.clone().unwrap_or(input.content.clone());
    let mut engine: SimSearch<String> = SimSearch::new();
    let mut split_content = content
        .split(". ")
        .map(|x| x.split(',').map(|y| y.to_string()).collect::<Vec<String>>())
        .collect::<Vec<Vec<String>>>();
    //insert all sentences into the engine
    split_content
        .iter()
        .enumerate()
        .for_each(|(idx, sentence)| {
            sentence.iter().enumerate().for_each(|(idy, phrase)| {
                engine.insert(
                    format!("{:?},{:?},{}", idx, idy, &phrase.clone()),
                    &phrase.clone(),
                );
            })
        });

    let mut new_output = input;

    //search for the query
    let results = engine.search(&query);
    let amount = if split_content.len() < 5 { 2 } else { 3 };
    for x in results.iter().take(amount) {
        let split_x: Vec<&str> = x.split(',').collect();
        if split_x.len() < 3 {
            continue;
        }
        let sentence_index = split_x[0].parse::<usize>().unwrap();
        let phrase_index = split_x[1].parse::<usize>().unwrap();
        let highlighted_sentence = format!("{}{}{}", "<mark>", split_x[2], "</mark>");
        split_content[sentence_index][phrase_index] = highlighted_sentence;
    }
    new_output.card_html = Some(
        split_content
            .iter()
            .map(|x| x.join(", "))
            .collect::<Vec<String>>()
            .join(". "),
    );
    Ok(new_output)
}
