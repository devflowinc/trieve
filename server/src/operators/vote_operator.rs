use crate::diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use crate::{
    data::models::{CardVote, Pool},
    errors::DefaultError,
};
use actix_web::web;

pub fn create_vote_query(
    voted_user_id: &uuid::Uuid,
    card_metadata_id: &uuid::Uuid,
    vote: &bool,
    pool: web::Data<Pool>,
) -> Result<CardVote, DefaultError> {
    use crate::data::schema::card_votes::dsl as card_votes_columns;

    let pool1 = pool.clone();

    let _ = delete_vote_query(voted_user_id, card_metadata_id, pool);

    let mut conn = pool1.get().unwrap();

    let new_vote = CardVote::from_details(voted_user_id, card_metadata_id, vote);

    let created_vote: CardVote = diesel::insert_into(card_votes_columns::card_votes)
        .values(&new_vote)
        .get_result::<CardVote>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to create vote",
        })?;

    Ok(created_vote)
}

pub fn delete_vote_query(
    voted_user_id: &uuid::Uuid,
    card_metadata_id: &uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), DefaultError> {
    use crate::data::schema::card_votes::dsl as card_votes_columns;

    let mut conn = pool.get().unwrap();

    diesel::delete(
        card_votes_columns::card_votes
            .filter(card_votes_columns::voted_user_id.eq(voted_user_id))
            .filter(card_votes_columns::card_metadata_id.eq(card_metadata_id)),
    )
    .execute(&mut conn)
    .map_err(|_| DefaultError {
        message: "Failed to delete vote",
    })?;

    Ok(())
}
