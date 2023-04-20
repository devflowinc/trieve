#![allow(clippy::extra_unused_lifetimes)]

use diesel::{r2d2::ConnectionManager, PgConnection};
use serde::{Deserialize, Serialize};

use super::schema::*;

// type alias to use in multiple places
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub hash: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl User {
    pub fn from_details<S: Into<String>, T: Into<String>>(email: S, pwd: T) -> Self {
        User {
            id: uuid::Uuid::new_v4(),
            email: email.into(),
            hash: pwd.into(),
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = invitations)]
pub struct Invitation {
    pub id: uuid::Uuid,
    pub email: String,
    pub expires_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

// any type that implements Into<String> can be used to create Invitation
impl<T> From<T> for Invitation
where
    T: Into<String>,
{
    fn from(email: T) -> Self {
        Invitation {
            id: uuid::Uuid::new_v4(),
            email: email.into(),
            expires_at: chrono::Local::now().naive_local() + chrono::Duration::minutes(5),
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = password_resets)]
pub struct PasswordReset {
    pub id: uuid::Uuid,
    pub email: String,
    pub expires_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

// any type that implements Into<String> can be used to create PasswordReset
impl<T> From<T> for PasswordReset
where
    T: Into<String>,
{
    fn from(email: T) -> Self {
        PasswordReset {
            id: uuid::Uuid::new_v4(),
            email: email.into(),
            expires_at: chrono::Local::now().naive_local() + chrono::Duration::minutes(5),
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = topics)]
pub struct Topic {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub resolution: String,
    pub side: bool,
    pub deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl Topic {
    pub fn from_details<S: Into<String>, T: Into<uuid::Uuid>>(
        resolution: S,
        user_id: T,
        side: bool,
    ) -> Self {
        Topic {
            id: uuid::Uuid::new_v4(),
            user_id: user_id.into(),
            resolution: resolution.into(),
            side,
            deleted: false,
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SlimUser {
    pub id: uuid::Uuid,
    pub email: String,
}

impl From<User> for SlimUser {
    fn from(user: User) -> Self {
        SlimUser {
            id: user.id,
            email: user.email,
        }
    }
}
