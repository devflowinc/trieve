use crate::data::models::{
    ChunkFileWithName, ChunkMetadata, ChunkMetadataWithFileData, SlimUser, UserDTOWithChunks,
    UserOrganization, UserRole,
};
use crate::diesel::prelude::*;
use crate::handlers::auth_handler::LoggedUser;
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
) -> Result<(User, UserOrganization), DefaultError> {
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let mut conn = pool.get().unwrap();

    let user: Option<(User, UserOrganization)> = users_columns::users
        .inner_join(user_organizations_columns::user_organizations)
        .filter(users_columns::id.eq(user_id))
        .select((User::as_select(), UserOrganization::as_select()))
        .first::<(User, UserOrganization)>(&mut conn)
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

pub fn get_user_with_chunks_by_id_query(
    user_id: uuid::Uuid,
    dataset_id: uuid::Uuid,
    page: &i64,
    pool: web::Data<Pool>,
) -> Result<UserDTOWithChunks, DefaultError> {
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
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

    let total_chunks_created_by_user = chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::author_id.eq(user.id))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
        .into_boxed();

    let user_chunk_metadatas = chunk_metadata_columns::chunk_metadata
        .filter(chunk_metadata_columns::author_id.eq(user.id))
        .filter(chunk_metadata_columns::dataset_id.eq(dataset_id))
        .into_boxed();

    let total_chunks_created_by_user = total_chunks_created_by_user
        .count()
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
        message: "Error loading user chunks count",
    })?;

    let user_chunk_metadatas = user_chunk_metadatas
        .order(chunk_metadata_columns::updated_at.desc())
        .select(ChunkMetadata::as_select())
        .limit(10)
        .offset((page - 1) * 10)
        .load::<ChunkMetadata>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error loading user chunks",
        })?;

    let file_ids: Vec<ChunkFileWithName> = chunk_files_columns::chunk_files
        .filter(
            chunk_files_columns::chunk_id.eq_any(
                user_chunk_metadatas
                    .iter()
                    .map(|chunk| chunk.id)
                    .collect::<Vec<uuid::Uuid>>()
                    .as_slice(),
            ),
        )
        .inner_join(files_columns::files)
        .filter(files_columns::dataset_id.eq(dataset_id))
        .select((
            chunk_files_columns::chunk_id,
            chunk_files_columns::file_id,
            files_columns::file_name,
        ))
        .load::<ChunkFileWithName>(&mut conn)
        .map_err(|_| DefaultError {
            message: "Failed to load metadata",
        })?;

    let chunk_metadata_with_score_and_file: Vec<ChunkMetadataWithFileData> = (user_chunk_metadatas)
        .iter()
        .map(|metadata| {
            let author = None;
            let chunk_with_file_name = file_ids.iter().find(|file| file.chunk_id == metadata.id);

            ChunkMetadataWithFileData {
                id: metadata.id,
                content: metadata.content.clone(),
                link: metadata.link.clone(),
                author,
                qdrant_point_id: metadata.qdrant_point_id.unwrap_or(uuid::Uuid::nil()),
                chunk_html: metadata.chunk_html.clone(),
                created_at: metadata.created_at,
                updated_at: metadata.updated_at,
                tag_set: metadata.tag_set.clone(),
                file_name: chunk_with_file_name.map(|file| file.file_name.clone()),
                file_id: chunk_with_file_name.map(|file| file.file_id),
                metadata: metadata.metadata.clone(),
                tracking_id: metadata.tracking_id.clone(),
                time_stamp: metadata.time_stamp,
                weight: metadata.weight,
            }
        })
        .collect();

    Ok(UserDTOWithChunks {
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
        total_chunks_created: total_chunks_created_by_user,
        chunks: chunk_metadata_with_score_and_file,
    })
}

pub fn update_user_query(
    user: &LoggedUser,
    username: &Option<String>,
    website: &Option<String>,
    visible_email: bool,
    pool: web::Data<Pool>,
) -> Result<User, DefaultError> {
    use crate::data::schema::users::dsl as user_columns;

    let mut conn = pool.get().unwrap();

    if username.clone().is_some()
        && username.clone().unwrap() != user.username.clone().unwrap_or("".to_string())
    {
        let user_by_username = get_user_by_username_query(&username.clone().unwrap(), pool);

        if let Ok(old_user) = user_by_username {
            if !(old_user.username.is_some()
                && old_user.username.unwrap() == username.clone().unwrap())
            {
                return Err(DefaultError {
                    message: "That username is already taken",
                });
            }
        }
    }

    let user: User = diesel::update(user_columns::users.filter(user_columns::id.eq(user.id)))
        .set((
            user_columns::username.eq(username),
            user_columns::website.eq(website),
            user_columns::visible_email.eq(&visible_email),
        ))
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
            message: "Error updating user",
        })?;

    Ok(user)
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
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let api_key_hash = hash_password(api_key)?;

    let mut conn = pool.get().unwrap();

    let user: Option<(User, UserOrganization)> = users_columns::users
        .inner_join(user_organizations_columns::user_organizations)
        .filter(users_columns::api_key_hash.eq(api_key_hash))
        .first::<(User, UserOrganization)>(&mut conn)
        .optional()
        .map_err(|_| DefaultError {
            message: "Error loading user",
        })?;
    match user {
        Some(user) => Ok(SlimUser::from_details(user.0, user.1)),
        None => Err(DefaultError {
            message: "User not found",
        }),
    }
}

pub fn create_user_query(
    user_id: uuid::Uuid,
    email: String,
    name: Option<String>,
    owner: bool,
    org_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(User, UserOrganization), DefaultError> {
    use crate::data::schema::user_organizations::dsl as user_organizations_columns;
    use crate::data::schema::users::dsl as users_columns;

    let user = User::from_details_with_id(user_id, email, name);
    let user_org = if owner {
        UserOrganization::from_details(user_id, org_id, UserRole::Owner)
    } else {
        UserOrganization::from_details(user_id, org_id, UserRole::User)
    };
    let mut conn = pool.get().unwrap();

    let user_org = conn
        .transaction::<_, diesel::result::Error, _>(|conn| {
            let user = diesel::insert_into(users_columns::users)
                .values(&user)
                .get_result::<User>(conn)?;

            let user_org = diesel::insert_into(user_organizations_columns::user_organizations)
                .values(&user_org)
                .get_result::<UserOrganization>(conn)?;

            Ok((user, user_org))
        })
        .map_err(|_| DefaultError {
            message: "Failed to create user, likely that organization_id is invalid",
        })?;

    Ok(user_org)
}
