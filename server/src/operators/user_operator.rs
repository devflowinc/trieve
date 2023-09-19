use crate::data::models::{
    CardFileWithName, CardMetadata, CardMetadataWithVotesAndFiles, CardVerifications, CardVote,
    SlimUser, UserDTOWithScore, UserDTOWithVotesAndCards, UserScore,
};
use crate::diesel::prelude::*;
use crate::handlers::register_handler::{generate_api_key, hash_password};
use crate::handlers::user_handler::UpdateUserData;
use crate::{
    data::models::{Pool, User},
    errors::DefaultError,
};
use actix_web::web;
use diesel::sql_types::{BigInt, Text};

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
    pool: web::Data<Pool>,
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
    pool: web::Data<Pool>,
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
    user_id: uuid::Uuid,
    accessing_user_id: Option<uuid::Uuid>,
    page: &i64,
    pool: web::Data<Pool>,
) -> Result<UserDTOWithVotesAndCards, DefaultError> {
    use crate::data::schema::card_files::dsl as card_files_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;
    use crate::data::schema::card_verification::dsl as card_verification_columns;
    use crate::data::schema::card_votes::dsl as card_votes_columns;
    use crate::data::schema::files::dsl as files_columns;
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

    let mut total_cards_created_by_user = card_metadata_columns::card_metadata
        .filter(card_metadata_columns::author_id.eq(user.id))
        .into_boxed();

    //Ensure only user can see their own private cards on their page
    let mut user_card_metadatas = card_metadata_columns::card_metadata
        .filter(card_metadata_columns::author_id.eq(user.id))
        .into_boxed();

    match accessing_user_id {
        Some(accessing_user_uuid) => {
            if user_id != accessing_user_uuid {
                user_card_metadatas =
                    user_card_metadatas.filter(card_metadata_columns::private.eq(false));
                total_cards_created_by_user =
                    total_cards_created_by_user.filter(card_metadata_columns::private.eq(false));
            }
        }
        None => {
            user_card_metadatas =
                user_card_metadatas.filter(card_metadata_columns::private.eq(false));
            total_cards_created_by_user =
                total_cards_created_by_user.filter(card_metadata_columns::private.eq(false));
        }
    }

    let total_cards_created_by_user = total_cards_created_by_user
        .count()
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error loading user cards",
        })?;

    let user_card_metadatas = user_card_metadatas
        .order(card_metadata_columns::updated_at.desc())
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
        ))
        .limit(10)
        .offset((page - 1) * 10)
        .load::<CardMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error loading user cards",
        })?;

    let card_votes: Vec<CardVote> = card_votes_columns::card_votes
        .filter(
            card_votes_columns::card_metadata_id.eq_any(
                user_card_metadatas
                    .iter()
                    .map(|metadata| metadata.id)
                    .collect::<Vec<uuid::Uuid>>(),
            ),
        )
        .load::<CardVote>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load upvotes",
        })?;

    let file_ids: Vec<CardFileWithName> = card_files_columns::card_files
        .filter(
            card_files_columns::card_id.eq_any(
                user_card_metadatas
                    .iter()
                    .map(|card| card.id)
                    .collect::<Vec<uuid::Uuid>>()
                    .as_slice(),
            ),
        )
        .inner_join(files_columns::files)
        .filter(files_columns::private.eq(false))
        .select((
            card_files_columns::card_id,
            card_files_columns::file_id,
            files_columns::file_name,
        ))
        .load::<CardFileWithName>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    let card_verifications: Vec<CardVerifications> = card_verification_columns::card_verification
        .filter(
            card_verification_columns::card_id.eq_any(
                user_card_metadatas
                    .iter()
                    .map(|card| card.id)
                    .collect::<Vec<uuid::Uuid>>()
                    .as_slice(),
            ),
        )
        .load::<CardVerifications>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load verification metadata",
        })?;

    let card_metadata_with_upvotes: Vec<CardMetadataWithVotesAndFiles> = (user_card_metadatas)
        .iter()
        .map(|metadata| {
            let votes = card_votes
                .iter()
                .filter(|upvote| upvote.card_metadata_id == metadata.id)
                .collect::<Vec<&CardVote>>();
            let total_upvotes = votes.iter().filter(|upvote| upvote.vote).count() as i64;
            let total_downvotes = votes.iter().filter(|upvote| !upvote.vote).count() as i64;
            let vote_by_current_user = None;

            let author = None;
            let card_with_file_name = file_ids.iter().find(|file| file.card_id == metadata.id);

            let verification_score = card_verifications
                .iter()
                .find(|verification| verification.card_id == metadata.id)
                .map(|verification| verification.similarity_score);

            CardMetadataWithVotesAndFiles {
                id: metadata.id,
                content: metadata.content.clone(),
                link: metadata.link.clone(),
                author,
                qdrant_point_id: metadata.qdrant_point_id.unwrap_or(uuid::Uuid::nil()),
                total_upvotes,
                total_downvotes,
                card_html: metadata.card_html.clone(),
                vote_by_current_user,
                created_at: metadata.created_at,
                updated_at: metadata.updated_at,
                tag_set: metadata.tag_set.clone(),
                private: metadata.private,
                score: None,
                file_name: card_with_file_name.map(|file| file.file_name.clone()),
                file_id: card_with_file_name.map(|file| file.file_id),
                verification_score,
                metadata: metadata.metadata.clone(),
            }
        })
        .collect();

    let user_card_votes = card_votes_columns::card_votes
        .inner_join(
            card_metadata_columns::card_metadata
                .on(card_votes_columns::card_metadata_id.eq(card_metadata_columns::id)),
        )
        .select((
            card_votes_columns::id,
            card_votes_columns::voted_user_id,
            card_votes_columns::card_metadata_id,
            card_votes_columns::vote,
            card_votes_columns::created_at,
            card_votes_columns::updated_at,
            card_votes_columns::deleted,
        ))
        .filter(card_metadata_columns::author_id.eq(user.id))
        .load::<CardVote>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load upvotes",
        })?;
    let total_upvotes_received = user_card_votes
        .iter()
        .filter(|card_vote| card_vote.vote)
        .count() as i32;
    let total_downvotes_received = user_card_votes.len() as i32 - total_upvotes_received;

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
        total_cards_created: total_cards_created_by_user,
        cards: card_metadata_with_upvotes,
        total_upvotes_received,
        total_downvotes_received,
        total_votes_cast,
    })
}

pub fn update_user_query(
    user_id: &uuid::Uuid,
    new_user: &UpdateUserData,
    pool: web::Data<Pool>,
) -> Result<SlimUser, DefaultError> {
    use crate::data::schema::users::dsl::*;

    let mut conn = pool.get().unwrap();

    if new_user.username.clone().unwrap_or("".to_string()) != "" {
        let user_by_username =
            get_user_by_username_query(&new_user.username.clone().unwrap(), pool);

        if let Ok(old_user) = user_by_username {
            if !(old_user.username.is_some()
                && old_user.username.unwrap() == new_user.username.clone().unwrap())
            {
                return Err(DefaultError {
                    message: "That username is already taken",
                });
            }
        }
    }

    let new_user_name: Option<String> = new_user
        .username
        .clone()
        .filter(|user_name| !user_name.is_empty());
    let new_user_website: Option<String> = new_user
        .website
        .clone()
        .filter(|user_website| !user_website.is_empty());

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

pub fn get_top_users_query(
    page: &i64,
    pool: web::Data<Pool>,
) -> Result<Vec<UserDTOWithScore>, DefaultError> {
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;
    use crate::data::schema::card_votes::dsl as card_votes_columns;
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().unwrap();

    let query = card_metadata_columns::card_metadata
        .inner_join(
            card_votes_columns::card_votes
                .on(card_metadata_columns::id.eq(card_votes_columns::card_metadata_id))
        )
        .select((
            card_metadata_columns::author_id,
            diesel::dsl::sql::<BigInt>("(SUM(case when vote = true then 1 else 0 end) - SUM(case when vote = false then 1 else 0 end)) as score"),
        ))
        .group_by((
            card_metadata_columns::author_id,
        ))
        .order(diesel::dsl::sql::<Text>("score desc"))
        .limit(10)
        .offset((page - 1) * 10);

    let user_scores: Vec<UserScore> =
        query
            .load::<UserScore>(&mut conn)
            .map_err(|_| DefaultError {
                message: "Failed to load top users",
            })?;

    let users_with_scores = users_columns::users
        .filter(
            users_columns::id.eq_any(
                user_scores
                    .iter()
                    .map(|user_score| user_score.author_id)
                    .collect::<Vec<uuid::Uuid>>(),
            ),
        )
        .load::<User>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load top users",
        })?;

    let user_scores_with_users = user_scores
        .iter()
        .map(|user_score| {
            let user = users_with_scores
                .iter()
                .find(|user| user.id == user_score.author_id)
                .unwrap();

            UserDTOWithScore {
                id: user_score.author_id,
                email: if user.visible_email {
                    Some(user.email.clone())
                } else {
                    None
                },
                username: user.username.clone(),
                website: user.website.clone(),
                visible_email: user.visible_email,
                created_at: user.created_at,
                score: user_score.score,
            }
        })
        .collect::<Vec<UserDTOWithScore>>();

    Ok(user_scores_with_users)
}

pub fn get_total_users_query(pool: web::Data<Pool>) -> Result<i64, DefaultError> {
    use crate::data::schema::users::dsl::*;

    let mut conn = pool.get().unwrap();

    let total_users = users
        .count()
        .get_result::<i64>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load total users",
        })?;

    Ok(total_users)
}

pub fn set_user_api_key_query(
    user_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<String, DefaultError> {
    use crate::data::schema::users::dsl as users_columns;

    let raw_api_key = generate_api_key();
    let hashed_api_key = hash_password(&raw_api_key)?;

    let mut conn = pool.get().unwrap();

    diesel::update(users_columns::users.filter(users_columns::id.eq(user_id)))
        .set(users_columns::api_key_hash.eq(hashed_api_key))
        .execute(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to set api key",
        })?;

    Ok(raw_api_key)
}

pub fn get_user_from_api_key_query(
    api_key: &String,
    pool: &web::Data<Pool>,
) -> Result<SlimUser, DefaultError> {
    use crate::data::schema::users::dsl as users_columns;

    let api_key_hash = hash_password(api_key)?;

    let mut conn = pool.get().unwrap();

    let user: Option<User> = users_columns::users
        .filter(users_columns::api_key_hash.eq(api_key_hash))
        .first::<User>(&mut conn)
        .optional()
        .map_err(|_| DefaultError {
            message: "Error loading user",
        })?;
    match user {
        Some(user) => Ok(SlimUser::from(user)),
        None => Err(DefaultError {
            message: "User not found",
        }),
    }
}
