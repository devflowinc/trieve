#![allow(clippy::extra_unused_lifetimes)]

use diesel::{r2d2::ConnectionManager, PgConnection};
use openai_dive::v1::resources::chat_completion::{ChatMessage, Role};
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
    pub username: Option<String>,
    pub website: Option<String>,
    pub visible_email: bool,
}

impl User {
    pub fn from_details<S: Into<String>, T: Into<String>>(email: S, pwd: T) -> Self {
        User {
            id: uuid::Uuid::new_v4(),
            email: email.into(),
            hash: pwd.into(),
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
            username: None,
            website: None,
            visible_email: true,
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
    pub referral_tokens: Option<String>,
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
            referral_tokens: None,
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
    pub normal_chat: bool,
}

impl Topic {
    pub fn from_details<S: Into<String>, T: Into<uuid::Uuid>>(
        resolution: S,
        user_id: T,
        normal_chat: Option<bool>,
    ) -> Self {
        Topic {
            id: uuid::Uuid::new_v4(),
            user_id: user_id.into(),
            resolution: resolution.into(),
            side: false,
            deleted: false,
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
            normal_chat: normal_chat.unwrap_or(false),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone)]
#[diesel(table_name = messages)]
pub struct Message {
    pub id: uuid::Uuid,
    pub topic_id: uuid::Uuid,
    pub sort_order: i32,
    pub content: String,
    pub role: String,
    pub deleted: bool,
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<Message> for ChatMessage {
    fn from(message: Message) -> Self {
        let role = match message.role.as_str() {
            "system" => Role::System,
            "user" => Role::User,
            _ => Role::Assistant,
        };

        ChatMessage {
            role,
            content: message.content,
            name: None,
        }
    }
}

impl Message {
    pub fn from_details<S: Into<String>, T: Into<uuid::Uuid>>(
        content: S,
        topic_id: T,
        sort_order: i32,
        role: String,
        prompt_tokens: Option<i32>,
        completion_tokens: Option<i32>,
    ) -> Self {
        Message {
            id: uuid::Uuid::new_v4(),
            topic_id: topic_id.into(),
            sort_order,
            content: content.into(),
            role,
            deleted: false,
            prompt_tokens,
            completion_tokens,
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = stripe_customers)]
pub struct StripeCustomer {
    pub id: uuid::Uuid,
    pub stripe_id: String,
    pub email: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl StripeCustomer {
    pub fn from_details<S: Into<String>, T: Into<String>>(stripe_id: S, email: Option<T>) -> Self {
        StripeCustomer {
            id: uuid::Uuid::new_v4(),
            stripe_id: stripe_id.into(),
            email: email.map(|e| e.into()),
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = user_plans)]
pub struct UserPlan {
    pub id: uuid::Uuid,
    pub stripe_customer_id: String,
    pub stripe_subscription_id: String,
    pub plan: String,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl UserPlan {
    pub fn from_details(
        stripe_customer_id: String,
        plan: String,
        subscription_id: String,
        status: Option<String>,
    ) -> Self {
        UserPlan {
            id: uuid::Uuid::new_v4(),
            stripe_customer_id,
            plan,
            status: status.unwrap_or("active".to_string()),
            stripe_subscription_id: subscription_id,
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone)]
#[diesel(table_name = card_metadata)]
pub struct CardMetadata {
    pub id: uuid::Uuid,
    pub content: String,
    pub author_id: uuid::Uuid,
    pub qdrant_point_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl CardMetadata {
    pub fn from_details<S: Into<String>, T: Into<uuid::Uuid>>(
        content: S,
        author_id: T,
        qdrant_point_id: T,
    ) -> Self {
        CardMetadata {
            id: uuid::Uuid::new_v4(),
            content: content.into(),
            author_id: author_id.into(),
            qdrant_point_id: qdrant_point_id.into(),
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = card_votes)]
pub struct CardVote {
    pub id: uuid::Uuid,
    pub voted_user_id: uuid::Uuid,
    pub card_metadata_id: uuid::Uuid,
    pub vote: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl CardVote {
    pub fn from_details<T: Into<uuid::Uuid>>(
        voted_user_id: T,
        card_metadata_id: T,
        vote: bool,
    ) -> Self {
        CardVote {
            id: uuid::Uuid::new_v4(),
            voted_user_id: voted_user_id.into(),
            card_metadata_id: card_metadata_id.into(),
            vote,
            created_at: chrono::Local::now().naive_local(),
            updated_at: chrono::Local::now().naive_local(),
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CardMetadataWithVotes {
    pub id: uuid::Uuid,
    pub author: Option<UserDTO>,
    pub content: String,
    pub qdrant_point_id: uuid::Uuid,
    pub total_upvotes: i64,
    pub total_downvotes: i64,
    pub vote_by_current_user: Option<bool>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlimUser {
    pub id: uuid::Uuid,
    pub email: String,
    pub username: Option<String>,
    pub website: Option<String>,
    pub visible_email: bool,
}

impl From<User> for SlimUser {
    fn from(user: User) -> Self {
        SlimUser {
            id: user.id,
            email: user.email,
            username: user.username,
            website: user.website,
            visible_email: user.visible_email,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDTO {
    pub id: uuid::Uuid,
    pub email: Option<String>,
    pub username: Option<String>,
    pub website: Option<String>,
    pub visible_email: bool,
    pub created_at: chrono::NaiveDateTime,
}

impl From<User> for UserDTO {
    fn from(user: User) -> Self {
        UserDTO {
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
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDTOWithVotesAndCards {
    pub id: uuid::Uuid,
    pub email: Option<String>,
    pub username: Option<String>,
    pub website: Option<String>,
    pub visible_email: bool,
    pub created_at: chrono::NaiveDateTime,
    pub cards: Vec<CardMetadata>,
    pub total_upvotes_received: i32,
    pub total_downvotes_received: i32,
    pub total_votes_cast: i32,
}
