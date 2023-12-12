use crate::data::models::{
    CardFileWithName, CardMetadata, CardMetadataWithFileData, SlimUser, UserDTOWithCards,
};
use crate::diesel::prelude::*;
use crate::handlers::user_handler::UpdateUserData;
use crate::{
    data::models::{Pool, User},
    errors::DefaultError,
};
use actix_web::web;
use argon2::Config;
use once_cell::sync::Lazy;
use rand::distributions::Alphanumeric;
use rand::Rng;

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

pub fn get_user_with_cards_by_id_query(
    user_id: uuid::Uuid,
    accessing_user_id: Option<uuid::Uuid>,
    dataset_id: uuid::Uuid,
    page: &i64,
    pool: web::Data<Pool>,
) -> Result<UserDTOWithCards, DefaultError> {
    use crate::data::schema::card_files::dsl as card_files_columns;
    use crate::data::schema::card_metadata::dsl as card_metadata_columns;
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
        .filter(card_metadata_columns::dataset_id.eq(dataset_id))
        .into_boxed();

    //Ensure only user can see their own private cards on their page
    let mut user_card_metadatas = card_metadata_columns::card_metadata
        .filter(card_metadata_columns::author_id.eq(user.id))
        .filter(card_metadata_columns::dataset_id.eq(dataset_id))
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
            card_metadata_columns::tracking_id,
            card_metadata_columns::time_stamp,
            card_metadata_columns::dataset_id,
        ))
        .limit(10)
        .offset((page - 1) * 10)
        .load::<CardMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error loading user cards",
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
        .filter(files_columns::dataset_id.eq(dataset_id))
        .select((
            card_files_columns::card_id,
            card_files_columns::file_id,
            files_columns::file_name,
        ))
        .load::<CardFileWithName>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    let card_metadata_with_score_and_file: Vec<CardMetadataWithFileData> = (user_card_metadatas)
        .iter()
        .map(|metadata| {
            let author = None;
            let card_with_file_name = file_ids.iter().find(|file| file.card_id == metadata.id);

            CardMetadataWithFileData {
                id: metadata.id,
                content: metadata.content.clone(),
                link: metadata.link.clone(),
                author,
                qdrant_point_id: metadata.qdrant_point_id.unwrap_or(uuid::Uuid::nil()),
                card_html: metadata.card_html.clone(),
                created_at: metadata.created_at,
                updated_at: metadata.updated_at,
                tag_set: metadata.tag_set.clone(),
                private: metadata.private,
                file_name: card_with_file_name.map(|file| file.file_name.clone()),
                file_id: card_with_file_name.map(|file| file.file_id),
                metadata: metadata.metadata.clone(),
                tracking_id: metadata.tracking_id.clone(),
                time_stamp: metadata.time_stamp,
            }
        })
        .collect();

    Ok(UserDTOWithCards {
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
        cards: card_metadata_with_score_and_file,
        organization_id: user.organization_id,
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

pub fn generate_api_key() -> String {
    let rng = rand::thread_rng();
    let api_key: String = format!(
        "af-{}",
        rng.sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect::<String>()
    );

    api_key
}

pub static SECRET_KEY: Lazy<String> =
    Lazy::new(|| std::env::var("SECRET_KEY").unwrap_or_else(|_| "0123".repeat(16)));

pub static SALT: Lazy<String> =
    Lazy::new(|| std::env::var("SALT").unwrap_or_else(|_| "supersecuresalt".to_string()));

pub fn hash_password(password: &str) -> Result<String, DefaultError> {
    let config = Config {
        secret: SECRET_KEY.as_bytes(),
        ..Config::original()
    };
    argon2::hash_encoded(password.as_bytes(), SALT.as_bytes(), &config).map_err(|_err| {
        DefaultError {
            message: "Error processing password, try again",
        }
    })
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
    api_key: &str,
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
