use crate::data::models::SlimUser;
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
