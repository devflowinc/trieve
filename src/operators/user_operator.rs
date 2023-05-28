use crate::data::models::{SlimUser, UserDTOWithVotesAndCards};
use crate::diesel::prelude::*;
use crate::handlers::user_handler::UpdateUserData;
use crate::{
    data::models::{Pool, User},
    errors::DefaultError,
};
use actix_web::web;

pub fn get_user_by_email_query(
    user_email: &String,
    pool: &web::Data<Pool>,
) -> Result<User, DefaultError> {
    use crate::data::schema::users::dsl::*;

    let mut conn = pool.get().unwrap();

    let user: Option<User> = users
        .filter(email.eq(user_email))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| DefaultError {
            message: "Error loading user",
        })?;
    match user {
        Some(user) => Ok(user),
        None => Err(DefaultError {
            message: "User not found",
        }),
    }
}

pub fn get_user_by_username_query(
    user_name: &String,
    pool: &web::Data<Pool>,
) -> Result<User, DefaultError> {
    use crate::data::schema::users::dsl::*;

    let mut conn = pool.get().unwrap();

    let user: Option<User> = users
        .filter(username.eq(user_name))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| DefaultError {
            message: "Error loading user",
        })?;
    match user {
        Some(user) => Ok(user),
        None => Err(DefaultError {
            message: "User not found",
        }),
    }
}

pub fn get_user_by_id_query(
    user_id: &uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<User, DefaultError> {
    use crate::data::schema::users::dsl::*;

    let mut conn = pool.get().unwrap();

    let user: Option<User> = users
        .filter(id.eq(user_id))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| DefaultError {
            message: "Error loading user",
        })?;
    match user {
        Some(user) => Ok(user),
        None => Err(DefaultError {
            message: "User not found",
        }),
    }
}

pub fn get_user_with_votes_and_cards_by_id_query(
    user_id: &uuid::Uuid,
    pool: &web::Data<Pool>,
) -> Result<UserDTOWithVotesAndCards, DefaultError> {
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;
    use crate::data::schema::card_votes::dsl as card_votes_columns;
    use crate::data::schema::users::dsl as user_columns;

    let mut conn = pool.get().unwrap();

    let user_result: Option<User> = user_columns::users
        .filter(user_columns::id.eq(user_id))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| DefaultError {
            message: "Error loading user",
        })?;
    let user = match user_result {
        Some(user) => Ok(user),
        None => Err(DefaultError {
            message: "User not found",
        }),
    }?;

    let user_card_metadatas = card_metadata_columns::card_metadata
        .filter(card_metadata_columns::author_id.eq(user.id))
        .load::<crate::data::models::CardMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error loading user cards",
        })?;

    let user_card_votes = card_votes_columns::card_votes
        .filter(
            card_votes_columns::card_metadata_id.eq_any(
                user_card_metadatas
                    .iter()
                    .map(|metadata| metadata.id)
                    .collect::<Vec<uuid::Uuid>>(),
            ),
        )
        .load::<crate::data::models::CardVote>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load upvotes",
        })?;
    let total_upvotes_received = user_card_votes
        .iter()
        .filter(|card_vote| card_vote.vote)
        .count() as i32;
    let total_downvotes_received = user_card_votes
        .iter()
        .filter(|card_vote| !card_vote.vote)
        .count() as i32;

    let total_votes_cast = card_votes_columns::card_votes
        .filter(card_votes_columns::voted_user_id.eq(user.id))
        .count()
        .get_result::<i64>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load total votes cast",
        })? as i32;

    Ok(UserDTOWithVotesAndCards {
        id: user.id,
        email: if user.visible_email {
            Some(user.email)
        } else {
            None
        },
        username: user.username,
        website: user.website,
        visible_email: user.visible_email,
        created_at: user.created_at,
        cards: user_card_metadatas,
        total_upvotes_received,
        total_downvotes_received,
        total_votes_cast,
    })
}

pub fn update_user_query(
    user_id: &uuid::Uuid,
    new_user: &UpdateUserData,
    pool: &web::Data<Pool>,
) -> Result<SlimUser, DefaultError> {
    use crate::data::schema::users::dsl::*;

    let mut conn = pool.get().unwrap();

    if new_user.username.clone().unwrap_or("".to_string()) != "" {
        let user_by_username =
            get_user_by_username_query(&new_user.username.clone().unwrap(), pool);

        match user_by_username {
            Ok(old_user) => {
                if !(old_user.username.is_some()
                    && old_user.username.unwrap() == new_user.username.clone().unwrap())
                {
                    return Err(DefaultError {
                        message: "That username is already taken",
                    });
                }
            }
            Err(_) => {}
        }
    }

    let new_user_name: Option<String> = match new_user.username.clone() {
        Some(user_name) => {
            if user_name != "" {
                Some(user_name)
            } else {
                None
            }
        }
        None => None,
    };
    let new_user_website: Option<String> = match new_user.website.clone() {
        Some(user_website) => {
            if user_website != "" {
                Some(user_website)
            } else {
                None
            }
        }
        None => None,
    };

    let user: User = diesel::update(users.filter(id.eq(user_id)))
        .set((
            username.eq(&new_user_name),
            website.eq(&new_user_website),
            visible_email.eq(&new_user.visible_email),
        ))
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error updating user",
        })?;

    Ok(SlimUser::from(user))
}
