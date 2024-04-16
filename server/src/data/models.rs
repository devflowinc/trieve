#![allow(clippy::extra_unused_lifetimes)]

use crate::get_env;

use super::schema::*;
use crate::handlers::chunk_handler::ScoreChunkDTO;
use crate::handlers::file_handler::UploadFileData;
use chrono::{DateTime, NaiveDateTime};
use dateparser::DateTimeUtc;
use diesel::expression::ValidGrouping;
use openai_dive::v1::resources::chat::{ChatMessage, ChatMessageContent, Role};
use qdrant_client::{prelude::Payload, qdrant::RetrievedPoint};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

// type alias to use in multiple places
pub type Pool = diesel_async::pooled_connection::deadpool::Pool<diesel_async::AsyncPgConnection>;
pub type RedisPool = bb8_redis::bb8::Pool<bb8_redis::RedisConnectionManager>;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "email": "developers@trieve.ai",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "name": "Trieve",
}))]
#[diesel(table_name = users)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub name: Option<String>,
}

impl User {
    pub fn from_details<S: Into<String>>(email: S, name: Option<S>) -> Self {
        User {
            id: uuid::Uuid::new_v4(),
            email: email.into(),
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            name: name.map(|n| n.into()),
        }
    }

    pub fn from_details_with_id<S: Into<String>, T: Into<uuid::Uuid>>(
        id: T,
        email: S,
        name: Option<S>,
    ) -> Self {
        User {
            id: id.into(),
            email: email.into(),
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            name: name.map(|n| n.into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, ValidGrouping, Clone, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "user_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "name": "Trieve",
    "deleted": false,
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
}))]
#[diesel(table_name = topics)]
pub struct Topic {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub name: String,
    pub deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub dataset_id: uuid::Uuid,
}

impl Topic {
    pub fn from_details<S: Into<String>, T: Into<uuid::Uuid>>(
        name: S,
        user_id: T,
        dataset_id: uuid::Uuid,
    ) -> Self {
        Topic {
            id: uuid::Uuid::new_v4(),
            user_id: user_id.into(),
            name: name.into(),
            deleted: false,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            dataset_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Clone, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "topic_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "sort_order": 1,
    "content": "Hello, world!",
    "role": "user",
    "deleted": false,
    "prompt_tokens": 300,
    "completion_tokens": 300,
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
}))]
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
    pub dataset_id: uuid::Uuid,
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
            content: ChatMessageContent::Text(message.content),
            tool_calls: None,
            name: None,
            tool_call_id: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example=json!({
    "role": "user",
    "content": "Hello, world!"
}))]
pub struct ChatMessageProxy {
    pub role: String,
    pub content: String,
}

impl From<ChatMessageProxy> for ChatMessage {
    fn from(message: ChatMessageProxy) -> Self {
        let role = match message.role.as_str() {
            "system" => Role::System,
            "user" => Role::User,
            _ => Role::Assistant,
        };

        ChatMessage {
            role,
            content: ChatMessageContent::Text(message.content),
            tool_calls: None,
            name: None,
            tool_call_id: None,
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
        dataset_id: T,
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
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            dataset_id: dataset_id.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable)]
pub struct ChunkMetadataWithCount {
    pub id: uuid::Uuid,
    pub content: String,
    pub link: Option<String>,
    pub qdrant_point_id: Option<uuid::Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub tag_set: Option<String>,
    pub chunk_html: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub dataset_id: uuid::Uuid,
    pub weight: f64,
    pub count: i64,
}

#[derive(
    Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone, ToSchema, AsChangeset,
)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "content": "Hello, world!",
    "link": "https://trieve.ai",
    "qdrant_point_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "tag_set": "tag1,tag2",
    "chunk_html": "<p>Hello, world!</p>",
    "metadata": {"key": "value"},
    "tracking_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "time_stamp": "2021-01-01T00:00:00",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "weight": 0.5,
}))]
#[diesel(table_name = chunk_metadata)]
pub struct ChunkMetadata {
    pub id: uuid::Uuid,
    pub content: String,
    pub link: Option<String>,
    pub qdrant_point_id: Option<uuid::Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub tag_set: Option<String>,
    pub chunk_html: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub dataset_id: uuid::Uuid,
    pub weight: f64,
}

impl ChunkMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn from_details<S: Into<String>>(
        content: S,
        chunk_html: &Option<String>,
        link: &Option<String>,
        tag_set: &Option<String>,
        qdrant_point_id: Option<uuid::Uuid>,
        metadata: Option<serde_json::Value>,
        tracking_id: Option<String>,
        time_stamp: Option<NaiveDateTime>,
        dataset_id: uuid::Uuid,
        weight: f64,
    ) -> Self {
        ChunkMetadata {
            id: uuid::Uuid::new_v4(),
            content: content.into(),
            chunk_html: chunk_html.clone(),
            link: link.clone(),
            qdrant_point_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            tag_set: tag_set.clone(),
            metadata,
            tracking_id,
            time_stamp,
            dataset_id,
            weight,
        }
    }
}

impl ChunkMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn from_details_with_id<S: Into<String>, T: Into<uuid::Uuid>>(
        id: T,
        content: S,
        chunk_html: &Option<String>,
        link: &Option<String>,
        tag_set: &Option<String>,
        qdrant_point_id: Option<uuid::Uuid>,
        metadata: Option<serde_json::Value>,
        tracking_id: Option<String>,
        time_stamp: Option<NaiveDateTime>,
        dataset_id: uuid::Uuid,
        weight: f64,
    ) -> Self {
        ChunkMetadata {
            id: id.into(),
            content: content.into(),
            chunk_html: chunk_html.clone(),
            link: link.clone(),
            qdrant_point_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            tag_set: tag_set.clone(),
            metadata,
            tracking_id,
            time_stamp,
            dataset_id,
            weight,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IngestSpecificChunkMetadata {
    pub id: uuid::Uuid,
    pub qdrant_point_id: Option<uuid::Uuid>,
    pub dataset_id: uuid::Uuid,
    pub attempt_number: usize,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Insertable, Clone)]
#[diesel(table_name = chunk_collisions)]
pub struct ChunkCollision {
    pub id: uuid::Uuid,
    pub chunk_id: uuid::Uuid,
    pub collision_qdrant_id: Option<uuid::Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl ChunkCollision {
    pub fn from_details<T: Into<uuid::Uuid>>(chunk_id: T, collision_id: T) -> Self {
        ChunkCollision {
            id: uuid::Uuid::new_v4(),
            chunk_id: chunk_id.into(),
            collision_qdrant_id: Some(collision_id.into()),
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "content": "Hello, world!",
    "link": "https://trieve.ai",
    "qdrant_point_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "tag_set": "tag1,tag2",
    "chunk_html": "<p>Hello, world!</p>",
    "metadata": {"key": "value"},
    "tracking_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "time_stamp": "2021-01-01T00:00:00",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "weight": 0.5,
    "score": 0.9,
}))]
pub struct ChunkMetadataWithScore {
    pub id: uuid::Uuid,
    pub content: String,
    pub link: Option<String>,
    pub qdrant_point_id: Option<uuid::Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub tag_set: Option<String>,
    pub chunk_html: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub dataset_id: uuid::Uuid,
    pub weight: f64,
    pub score: f32,
}

impl From<(ChunkMetadata, f32)> for ChunkMetadataWithScore {
    fn from((chunk, score): (ChunkMetadata, f32)) -> Self {
        ChunkMetadataWithScore {
            id: chunk.id,
            content: chunk.content,
            link: chunk.link,
            qdrant_point_id: chunk.qdrant_point_id,
            created_at: chunk.created_at,
            updated_at: chunk.updated_at,
            tag_set: chunk.tag_set,
            chunk_html: chunk.chunk_html,
            metadata: chunk.metadata,
            tracking_id: chunk.tracking_id,
            time_stamp: chunk.time_stamp,
            dataset_id: chunk.dataset_id,
            weight: chunk.weight,
            score,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "link": "https://trieve.ai",
    "qdrant_point_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "tag_set": "tag1,tag2",
    "metadata": {"key": "value"},
    "tracking_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "time_stamp": "2021-01-01T00:00:00",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "weight": 0.5,
}))]
pub struct SlimChunkMetadata {
    pub id: uuid::Uuid,
    pub link: Option<String>,
    pub qdrant_point_id: Option<uuid::Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub tag_set: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub weight: f64,
}

impl From<ChunkMetadata> for SlimChunkMetadata {
    fn from(chunk: ChunkMetadata) -> Self {
        SlimChunkMetadata {
            id: chunk.id,
            link: chunk.link,
            qdrant_point_id: chunk.qdrant_point_id,
            created_at: chunk.created_at,
            updated_at: chunk.updated_at,
            tag_set: chunk.tag_set,
            metadata: chunk.metadata,
            tracking_id: chunk.tracking_id,
            time_stamp: chunk.time_stamp,
            weight: chunk.weight,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "link": "https://trieve.ai",
    "qdrant_point_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "tag_set": "tag1,tag2",
    "metadata": {"key": "value"},
    "tracking_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "time_stamp": "2021-01-01T00:00:00",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "weight": 0.5,
    "score": 0.9,
}))]
pub struct SlimChunkMetadataWithScore {
    pub id: uuid::Uuid,
    pub link: Option<String>,
    pub qdrant_point_id: Option<uuid::Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub tag_set: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub weight: f64,
    pub score: f32,
}

impl From<ChunkMetadataWithScore> for SlimChunkMetadataWithScore {
    fn from(chunk: ChunkMetadataWithScore) -> Self {
        SlimChunkMetadataWithScore {
            id: chunk.id,
            link: chunk.link,
            qdrant_point_id: chunk.qdrant_point_id,
            created_at: chunk.created_at,
            updated_at: chunk.updated_at,
            tag_set: chunk.tag_set,
            metadata: chunk.metadata,
            tracking_id: chunk.tracking_id,
            time_stamp: chunk.time_stamp,
            weight: chunk.weight,
            score: chunk.score,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, ToSchema)]
#[schema(
    example = json!({
        "metadata": [
            {
                "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
                "link": "https://trieve.ai",
                "qdrant_point_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
                "created_at": "2021-01-01T00:00:00",
                "updated_at": "2021-01-01T00:00:00",
                "tag_set": "tag1,tag2",
                "metadata": {"key": "value"},
                "tracking_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
                "time_stamp": "2021-01-01T00:00:00",
                "weight": 0.5,
            }
        ],
        "score": 0.5,
    })

)]
pub struct ScoreSlimChunks {
    pub metadata: Vec<SlimChunkMetadata>,
    pub score: f64,
}

impl From<ScoreChunkDTO> for ScoreSlimChunks {
    fn from(score: ScoreChunkDTO) -> Self {
        ScoreSlimChunks {
            metadata: score.metadata.into_iter().map(|m| m.into()).collect(),
            score: score.score,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SearchSlimChunkQueryResponseBody {
    pub score_chunks: Vec<ScoreSlimChunks>,
    pub total_chunk_pages: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "group_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "metadata": [
        {
            "metadata": [
                {
                    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
                    "link": "https://trieve.ai",
                    "qdrant_point_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
                    "created_at": "2021-01-01T00:00:00",
                    "updated_at": "2021-01-01T00:00:00",
                    "tag_set": "tag1,tag2",
                    "metadata": {"key": "value"},
                    "tracking_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
                    "time_stamp": "2021-01-01T00:00:00",
                    "weight": 0.5,
                }
            ],
            "score": 0.5,
        }
    ],
}))]
pub struct GroupScoreSlimChunks {
    pub group_id: uuid::Uuid,
    pub metadata: Vec<ScoreSlimChunks>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SearchWithinGroupSlimResults {
    pub bookmarks: Vec<ScoreSlimChunks>,
    pub group: ChunkGroup,
    pub total_pages: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct SearchOverGroupsSlimResults {
    pub group_chunks: Vec<GroupScoreSlimChunks>,
    pub total_chunk_pages: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(example=json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "name": "Trieve",
    "email": "developers@trieve.ai",
    "user_orgs": [
        {
            "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
            "user_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
            "org_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
            "role": 0,
            "created_at": "2021-01-01T00:00:00",
            "updated_at": "2021-01-01T00:00:00",
        }
    ],
    "orgs": [
        {
            "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
            "name": "Trieve",
            "created_at": "2021-01-01T00:00:00",
            "updated_at": "2021-01-01T00:00:00",
            "registerable": true,
        }
    ],
}))]
pub struct SlimUser {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub email: String,
    pub user_orgs: Vec<UserOrganization>,
    pub orgs: Vec<Organization>,
}

impl SlimUser {
    pub fn from_details(
        user: User,
        user_orgs: Vec<UserOrganization>,
        orgs: Vec<Organization>,
    ) -> Self {
        SlimUser {
            id: user.id,
            name: user.name,
            email: user.email,
            user_orgs,
            orgs,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDTO {
    pub id: uuid::Uuid,
    pub email: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(
    Debug, Default, Serialize, Deserialize, Selectable, Queryable, Insertable, Clone, ToSchema,
)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "name": "Trieve",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "tracking_id": "3",
}))]
#[diesel(table_name = chunk_group)]
pub struct ChunkGroup {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub dataset_id: uuid::Uuid,
    pub tracking_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tag_set: Option<String>,
}

impl ChunkGroup {
    pub fn from_details(
        name: String,
        description: Option<String>,
        dataset_id: uuid::Uuid,
        tracking_id: Option<String>,
        metadata: Option<serde_json::Value>,
        tag_set: Option<String>,
    ) -> Self {
        ChunkGroup {
            id: uuid::Uuid::new_v4(),
            name,
            description: description.unwrap_or_default(),
            dataset_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            tracking_id,
            metadata,
            tag_set,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "name": "Trieve",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "of_current_dataset": true,
}))]
pub struct SlimGroup {
    pub id: uuid::Uuid,
    pub name: String,
    pub dataset_id: uuid::Uuid,
    pub of_current_dataset: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Queryable, ToSchema)]
#[schema(example=json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "name": "Trieve",
    "description": "A group of chunks",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "file_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "tracking_id": "3",
}))]
pub struct ChunkGroupAndFile {
    pub id: uuid::Uuid,
    pub dataset_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub file_id: Option<uuid::Uuid>,
    pub tracking_id: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Queryable)]
pub struct ChunkGroupAndFileWithCount {
    pub id: uuid::Uuid,
    pub dataset_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub file_id: Option<uuid::Uuid>,
    pub group_count: Option<i32>,
    pub tracking_id: Option<String>,
}

impl From<ChunkGroupAndFileWithCount> for ChunkGroupAndFile {
    fn from(group: ChunkGroupAndFileWithCount) -> Self {
        ChunkGroupAndFile {
            id: group.id,
            dataset_id: group.dataset_id,
            name: group.name,
            description: group.description,
            created_at: group.created_at,
            updated_at: group.updated_at,
            file_id: group.file_id,
            tracking_id: group.tracking_id,
        }
    }
}

#[derive(
    Debug, Default, Serialize, Deserialize, Selectable, Queryable, Insertable, Clone, ToSchema,
)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "group_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "chunk_metadata_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
}))]
#[diesel(table_name = chunk_group_bookmarks)]
pub struct ChunkGroupBookmark {
    pub id: uuid::Uuid,
    pub group_id: uuid::Uuid,
    pub chunk_metadata_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl ChunkGroupBookmark {
    pub fn from_details(group_id: uuid::Uuid, chunk_metadata_id: uuid::Uuid) -> Self {
        ChunkGroupBookmark {
            id: uuid::Uuid::new_v4(),
            group_id,
            chunk_metadata_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        }
    }
}

#[derive(
    Debug, Default, Serialize, Deserialize, Selectable, Queryable, Insertable, Clone, ToSchema,
)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "file_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "group_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
}))]
#[diesel(table_name = groups_from_files)]
pub struct FileGroup {
    pub id: uuid::Uuid,
    pub file_id: uuid::Uuid,
    pub group_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl FileGroup {
    pub fn from_details(file_id: uuid::Uuid, group_id: uuid::Uuid) -> Self {
        FileGroup {
            id: uuid::Uuid::new_v4(),
            file_id,
            group_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDTOWithChunks {
    pub id: uuid::Uuid,
    pub email: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub total_chunks_created: i64,
    pub chunks: Vec<ChunkMetadata>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Default)]
pub struct FullTextSearchResult {
    pub id: uuid::Uuid,
    pub content: String,
    pub link: Option<String>,
    pub qdrant_point_id: Option<uuid::Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub tag_set: Option<String>,
    pub chunk_html: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub dataset_id: uuid::Uuid,
    pub weight: f64,
    pub score: Option<f64>,
    pub count: i64,
}

impl From<ChunkMetadata> for FullTextSearchResult {
    fn from(chunk: ChunkMetadata) -> Self {
        FullTextSearchResult {
            id: chunk.id,
            content: chunk.content,
            link: chunk.link,
            qdrant_point_id: chunk.qdrant_point_id,
            created_at: chunk.created_at,
            updated_at: chunk.updated_at,
            tag_set: chunk.tag_set,
            chunk_html: chunk.chunk_html,
            metadata: chunk.metadata,
            tracking_id: chunk.tracking_id,
            time_stamp: chunk.time_stamp,
            dataset_id: chunk.dataset_id,
            weight: chunk.weight,
            score: None,
            count: 0,
        }
    }
}

impl From<&ChunkMetadata> for FullTextSearchResult {
    fn from(chunk: &ChunkMetadata) -> Self {
        FullTextSearchResult {
            id: chunk.id,
            content: chunk.content.clone(),
            link: chunk.link.clone(),
            qdrant_point_id: chunk.qdrant_point_id,
            created_at: chunk.created_at,
            updated_at: chunk.updated_at,
            tag_set: chunk.tag_set.clone(),
            chunk_html: chunk.chunk_html.clone(),
            tracking_id: chunk.tracking_id.clone(),
            time_stamp: chunk.time_stamp,
            metadata: chunk.metadata.clone(),
            dataset_id: chunk.dataset_id,
            weight: chunk.weight,
            score: None,
            count: 0,
        }
    }
}

impl From<ChunkMetadataWithCount> for FullTextSearchResult {
    fn from(chunk: ChunkMetadataWithCount) -> Self {
        FullTextSearchResult {
            id: chunk.id,
            content: chunk.content,
            link: chunk.link,
            qdrant_point_id: chunk.qdrant_point_id,
            created_at: chunk.created_at,
            updated_at: chunk.updated_at,
            tag_set: chunk.tag_set,
            chunk_html: chunk.chunk_html,
            metadata: chunk.metadata,
            tracking_id: chunk.tracking_id,
            time_stamp: chunk.time_stamp,
            dataset_id: chunk.dataset_id,
            weight: chunk.weight,
            score: None,
            count: chunk.count,
        }
    }
}

#[derive(
    Debug, Default, Serialize, Deserialize, Selectable, Queryable, Insertable, Clone, ToSchema,
)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "file_name": "file.txt",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "size": 1000,
    "tag_set": "tag1,tag2",
    "metadata": {"key": "value"},
    "link": "https://trieve.ai",
    "time_stamp": "2021-01-01T00:00:00",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
}))]
#[diesel(table_name = files)]
pub struct File {
    pub id: uuid::Uuid,
    pub file_name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub size: i64,
    pub tag_set: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub link: Option<String>,
    pub time_stamp: Option<chrono::NaiveDateTime>,
    pub dataset_id: uuid::Uuid,
}

impl File {
    #[allow(clippy::too_many_arguments)]
    pub fn from_details(
        file_id: Option<uuid::Uuid>,
        file_name: &str,
        size: i64,
        tag_set: Option<String>,
        metadata: Option<serde_json::Value>,
        link: Option<String>,
        time_stamp: Option<String>,
        dataset_id: uuid::Uuid,
    ) -> Self {
        File {
            id: file_id.unwrap_or(uuid::Uuid::new_v4()),
            file_name: file_name.to_string(),
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            size,
            tag_set,
            metadata,
            link,
            time_stamp: time_stamp.map(|ts| {
                ts.parse::<DateTimeUtc>()
                    .unwrap_or(DateTimeUtc(DateTime::default()))
                    .0
                    .with_timezone(&chrono::Local)
                    .naive_local()
            }),
            dataset_id,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct FileAndGroupId {
    pub file: File,
    pub group_id: Option<uuid::Uuid>,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
#[schema(example=json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "file_name": "file.txt",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "size": 1000,
    "s3_url": "https://trieve.ai",
    "metadata": {"key": "value"},
    "link": "https://trieve.ai",
}))]
pub struct FileDTO {
    pub id: uuid::Uuid,
    pub file_name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub size: i64,
    pub s3_url: String,
    pub metadata: Option<serde_json::Value>,
    pub link: Option<String>,
}

impl From<File> for FileDTO {
    fn from(file: File) -> Self {
        FileDTO {
            id: file.id,
            file_name: file.file_name,
            created_at: file.created_at,
            updated_at: file.updated_at,
            size: file.size,
            s3_url: "".to_string(),
            metadata: file.metadata,
            link: file.link,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Selectable, Queryable, Insertable, Clone)]
#[diesel(table_name = chunk_files)]
pub struct ChunkFile {
    pub id: uuid::Uuid,
    pub chunk_id: uuid::Uuid,
    pub file_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl ChunkFile {
    pub fn from_details(chunk_id: uuid::Uuid, file_id: uuid::Uuid) -> Self {
        ChunkFile {
            id: uuid::Uuid::new_v4(),
            chunk_id,
            file_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Queryable, Clone)]
pub struct ChunkFileWithName {
    pub chunk_id: uuid::Uuid,
    pub file_id: uuid::Uuid,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Selectable, ToSchema)]
#[schema(example=json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "event_type": "file_uploaded",
    "event_data": {"group_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3", "file_name": "file.txt"},
}))]
#[diesel(table_name = events)]
pub struct Event {
    pub id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub dataset_id: uuid::Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
}

#[derive(Debug, Deserialize, Clone)]
pub enum EventType {
    FileUploaded {
        file_id: uuid::Uuid,
        file_name: String,
    },
    FileUploadFailed {
        file_id: uuid::Uuid,
        error: String,
    },
    ChunkUploaded {
        chunk_id: uuid::Uuid,
    },
    ChunkActionFailed {
        chunk_id: uuid::Uuid,
        error: String,
    },
    ChunkUpdated {
        chunk_id: uuid::Uuid,
    },
}

impl EventType {
    pub fn as_str(&self) -> String {
        match self {
            EventType::FileUploaded { .. } => "file_uploaded".to_string(),
            EventType::FileUploadFailed { .. } => "file_upload_failed".to_string(),
            EventType::ChunkUploaded { .. } => "chunk_uploaded".to_string(),
            EventType::ChunkActionFailed { .. } => "chunk_action_failed".to_string(),
            EventType::ChunkUpdated { .. } => "chunk_updated".to_string(),
        }
    }

    pub fn get_all_event_types() -> Vec<String> {
        vec![
            "file_uploaded".to_string(),
            "chunk_uploaded".to_string(),
            "chunk_action_failed".to_string(),
            "chunk_updated".to_string(),
        ]
    }
}

impl From<EventType> for serde_json::Value {
    fn from(val: EventType) -> Self {
        match val {
            EventType::FileUploaded { file_id, file_name } => {
                json!({"file_id": file_id, "file_name": file_name})
            }
            EventType::FileUploadFailed { file_id, error } => {
                json!({"file_id": file_id, "error": error})
            }
            EventType::ChunkUploaded { chunk_id } => json!({"chunk_id": chunk_id}),
            EventType::ChunkActionFailed { chunk_id, error } => {
                json!({"chunk_id": chunk_id, "error": error})
            }
            EventType::ChunkUpdated { chunk_id } => json!({"chunk_id": chunk_id}),
        }
    }
}

impl Event {
    pub fn from_details(dataset_id: uuid::Uuid, event_type: EventType) -> Self {
        Event {
            id: uuid::Uuid::new_v4(),
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            dataset_id,
            event_type: event_type.as_str(),
            event_data: event_type.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, ValidGrouping)]
#[diesel(table_name = dataset_group_counts)]
pub struct DatasetGroupCount {
    pub id: uuid::Uuid,
    pub group_count: i32,
    pub dataset_id: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, ValidGrouping)]
#[diesel(table_name = dataset_event_counts)]
pub struct DatasetEventCount {
    pub id: uuid::Uuid,
    pub dataset_uuid: uuid::Uuid,
    pub notification_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone, ToSchema)]
#[schema(example=json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "name": "Trieve",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "organization_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "tracking_id": "3",
    "server_configuration": {"key": "value"},
    "client_configuration": {"key": "value"},
}))]
#[diesel(table_name = datasets)]
pub struct Dataset {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub organization_id: uuid::Uuid,
    pub server_configuration: serde_json::Value,
    pub client_configuration: serde_json::Value,
    pub tracking_id: Option<String>,
}

impl Dataset {
    pub fn from_details(
        name: String,
        organization_id: uuid::Uuid,
        tracking_id: Option<String>,
        server_configuration: serde_json::Value,
        client_configuration: serde_json::Value,
    ) -> Self {
        Dataset {
            id: uuid::Uuid::new_v4(),
            name,
            organization_id,
            tracking_id,
            server_configuration,
            client_configuration,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Clone, ToSchema)]
#[schema(example=json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "name": "Trieve",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "tracking_id": "3",
    "organization_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "client_configuration": {"key": "value"},
}))]
pub struct DatasetDTO {
    pub id: uuid::Uuid,
    pub name: String,
    pub tracking_id: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub organization_id: uuid::Uuid,
    pub client_configuration: serde_json::Value,
}

impl From<Dataset> for DatasetDTO {
    fn from(dataset: Dataset) -> Self {
        DatasetDTO {
            id: dataset.id,
            name: dataset.name,
            created_at: dataset.created_at,
            updated_at: dataset.updated_at,
            tracking_id: dataset.tracking_id,
            organization_id: dataset.organization_id,
            client_configuration: dataset.client_configuration,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "chunk_count": 100,
}))]
#[diesel(table_name = dataset_usage_counts)]
pub struct DatasetUsageCount {
    pub id: uuid::Uuid,
    pub dataset_id: uuid::Uuid,
    pub chunk_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "dataset": {
        "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "name": "Trieve",
        "created_at": "2021-01-01T00:00:00",
        "updated_at": "2021-01-01T00:00:00",
        "organization_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "client_configuration": {"key": "value"},
    },
    "dataset_usage": {
        "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "chunk_count": 100,
    }
}))]
pub struct DatasetAndUsage {
    pub dataset: DatasetDTO,
    pub dataset_usage: DatasetUsageCount,
}

impl DatasetAndUsage {
    pub fn from_components(dataset: DatasetDTO, dataset_usage: DatasetUsageCount) -> Self {
        DatasetAndUsage {
            dataset,
            dataset_usage,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(example=json!({
    "DOCUMENT_UPLOAD_FEATURE": true,
    "DOCUMENT_DOWNLOAD_FEATURE": true,
    "LLM_BASE_URL": "https://api.openai.com/v1",
    "EMBEDDING_BASE_URL": "https://api.openai.com/v1",
    "EMBEDDING_MODEL_NAME": "text-embedding-3-small",
    "QDRANT_URL": "http://localhost:6333",
    "QDRANT_API_KEY": "api_key",
    "QDRANT_COLLECTION_NAME": "collection",
    "RAG_PROMPT": "Write a 1-2 sentence semantic search query along the lines of a hypothetical response to: \n\n",
    "N_RETRIEVALS_TO_INCLUDE": 5,
    "DUPLICATE_DISTANCE_THRESHOLD": 1.1,
    "COLLISIONS_ENABLED": false,
    "EMBEDDING_SIZE": 1536,
    "LLM_DEFAULT_MODEL": "gpt-3.5-turbo-1106",
    "FULLTEXT_ENABLED": true,
    "EMBEDDING_QUERY_PREFIX": "Search for",
}))]
#[allow(non_snake_case)]
pub struct ServerDatasetConfiguration {
    pub DOCUMENT_UPLOAD_FEATURE: bool,
    pub DOCUMENT_DOWNLOAD_FEATURE: bool,
    pub LLM_BASE_URL: String,
    pub EMBEDDING_BASE_URL: String,
    pub EMBEDDING_MODEL_NAME: String,
    pub QDRANT_URL: String,
    pub QDRANT_API_KEY: String,
    pub QDRANT_COLLECTION_NAME: String,
    pub RAG_PROMPT: String,
    pub N_RETRIEVALS_TO_INCLUDE: usize,
    pub DUPLICATE_DISTANCE_THRESHOLD: f64,
    pub COLLISIONS_ENABLED: bool,
    pub EMBEDDING_SIZE: usize,
    pub LLM_DEFAULT_MODEL: String,
    pub FULLTEXT_ENABLED: bool,
    pub EMBEDDING_QUERY_PREFIX: String,
}

impl ServerDatasetConfiguration {
    pub fn from_json(configuration: serde_json::Value) -> Self {
        let default_config = json!({});
        let configuration = configuration
            .as_object()
            .unwrap_or(default_config.as_object().unwrap());

        ServerDatasetConfiguration {
            DOCUMENT_UPLOAD_FEATURE: configuration
                .get("DOCUMENT_UPLOAD_FEATURE")
                .unwrap_or(&json!(true))
                .as_bool()
                .unwrap_or(true),
            DOCUMENT_DOWNLOAD_FEATURE: configuration
                .get("DOCUMENT_DOWNLOAD_FEATURE")
                .unwrap_or(&json!(true))
                .as_bool()
                .unwrap_or(true),
            LLM_BASE_URL: configuration
                .get("LLM_BASE_URL")
                .unwrap_or(&json!("https://api.openai.com/v1".to_string()))
                .as_str()
                .map(|s| {
                    if s.is_empty() {
                        "https://api.openai.com/v1".to_string()
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or("https://api.openai.com/v1".to_string()),
            EMBEDDING_BASE_URL: configuration
                .get("EMBEDDING_BASE_URL")
                .unwrap_or(&json!(get_env!("OPENAI_BASE_URL", "OPENAI_BASE_URL must be set").to_string()))
                .as_str()
                .map(|s| {
                    if s.is_empty() {
                        get_env!("OPENAI_BASE_URL", "OPENAI_BASE_URL must be set").to_string()
                    } else {
                        s.to_string()
                    }
                }).expect("EMBEDDING_BASE_URL should exist"),
            RAG_PROMPT: configuration
                .get("RAG_PROMPT")
                .unwrap_or(&json!("Write a 1-2 sentence semantic search query along the lines of a hypothetical response to: \n\n".to_string()))
                .as_str()
                .map(|s| {
                    if s.is_empty() {
                        "Write a 1-2 sentence semantic search query along the lines of a hypothetical response to: \n\n".to_string()
                    } else {
                        s.to_string()
                    }
                }).unwrap_or("Write a 1-2 sentence semantic search query along the lines of a hypothetical response to: \n\n".to_string()),
            N_RETRIEVALS_TO_INCLUDE: configuration
                .get("N_RETRIEVALS_TO_INCLUDE")
                .unwrap_or(&json!(5))
                .as_u64()
                .map(|u| u as usize)
                .unwrap_or(5),
            DUPLICATE_DISTANCE_THRESHOLD: configuration
                .get("DUPLICATE_DISTANCE_THRESHOLD")
                .unwrap_or(&json!(1.1))
                .as_f64()
                .unwrap_or(1.1),
            EMBEDDING_SIZE: configuration
                .get("EMBEDDING_SIZE")
                .unwrap_or(&json!(1536))
                .as_u64()
                .map(|u| u as usize)
                .unwrap_or(1536),
            EMBEDDING_MODEL_NAME: configuration
                .get("EMBEDDING_MODEL_NAME")
                .unwrap_or(&json!("text-embedding-3-small"))
                .as_str()
                .map(|s| {
                    if s.is_empty() {
                        "text-embedding-3-small".to_string()
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or("text-embedding-3-small".to_string()),
            LLM_DEFAULT_MODEL: configuration
                .get("LLM_DEFAULT_MODEL")
                .unwrap_or(&json!("gpt-3.5-turbo-1106"))
                .as_str()
                .map(|s| {
                    if s.is_empty() {
                        "gpt-3.5-turbo-1106".to_string()
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or("gpt-3.5-turbo-1106".to_string()),
            COLLISIONS_ENABLED: configuration
                .get("COLLISIONS_ENABLED")
                .unwrap_or(&json!(false))
                .as_bool()
                .unwrap_or(false),
            FULLTEXT_ENABLED: configuration
                .get("FULLTEXT_ENABLED")
                .unwrap_or(&json!(true))
                .as_bool()
                .unwrap_or(true),
            QDRANT_URL: configuration
                .get("QDRANT_URL")
                .unwrap_or(&json!(get_env!("QDRANT_URL", "Must provide QDRANT_URL")))
                .as_str()
                .map(|s| {
                    if s.is_empty() {
                        get_env!("QDRANT_URL", "Must provide QDRANT_URL").to_string()
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or(get_env!("QDRANT_URL", "Must provide QDRANT_URL").to_string()),
            QDRANT_API_KEY: configuration
                .get("QDRANT_API_KEY")
                .unwrap_or(&json!(get_env!("QDRANT_API_KEY", "Must provide QDRANT_API_KEY")))
                .as_str()
                .map(|s| {
                    if s.is_empty() {
                        get_env!("QDRANT_API_KEY", "Must provide QDRANT_API_KEY").to_string()
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or(get_env!("QDRANT_API_KEY", "Must provide QDRANT_API_KEY").to_string()),
            QDRANT_COLLECTION_NAME: configuration
                .get("QDRANT_COLLECTION_NAME")
                .unwrap_or(&json!(get_env!("QDRANT_COLLECTION", "Must provide QDRANT_COLLECTION")))
                .as_str()
                .map(|s| {
                    if s.is_empty() {
                        get_env!("QDRANT_COLLECTION", "Must provide QDRANT_COLLECTION").to_string()
                    } else {
                        s.to_string()
                    }
                })
                .unwrap_or(get_env!("QDRANT_COLLECTION", "Must provide QDRANT_COLLECTION").to_string()),
            EMBEDDING_QUERY_PREFIX: configuration
                .get("EMBEDDING_QUERY_PREFIX")
                .unwrap_or(&json!(""))
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or("".to_string())
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(example=json!({
    "CREATE_CHUNK_FEATURE": true,
    "SEARCH_QUERIES": "search queries",
    "FRONTMATTER_VALS": "frontmatter vals",
    "LINES_BEFORE_SHOW_MORE": 10,
    "DATE_RANGE_VALUE": "date range value",
    "FILTER_ITEMS": [],
    "SUGGESTED_QUERIES": "suggested queries",
    "IMAGE_RANGE_START_KEY": "image range start key",
    "IMAGE_RANGE_END_KEY": "image range end key",
    "DOCUMENT_UPLOAD_FEATURE": true,
    "FILE_NAME_KEY": "file_name_key",
}))]
#[allow(non_snake_case)]
pub struct ClientDatasetConfiguration {
    pub CREATE_CHUNK_FEATURE: Option<bool>,
    pub SEARCH_QUERIES: Option<String>,
    pub FRONTMATTER_VALS: Option<String>,
    pub LINES_BEFORE_SHOW_MORE: Option<usize>,
    pub DATE_RANGE_VALUE: Option<String>,
    pub FILTER_ITEMS: Option<serde_json::Value>,
    pub SUGGESTED_QUERIES: Option<String>,
    pub IMAGE_RANGE_START_KEY: Option<String>,
    pub IMAGE_RANGE_END_KEY: Option<String>,
    pub DOCUMENT_UPLOAD_FEATURE: Option<bool>,
    pub FILE_NAME_KEY: String,
}

impl ClientDatasetConfiguration {
    pub fn from_json(configuration: serde_json::Value) -> Self {
        let default_config = json!({});
        let configuration = configuration
            .as_object()
            .unwrap_or(default_config.as_object().unwrap());

        ClientDatasetConfiguration {
            CREATE_CHUNK_FEATURE: configuration
                .get("CREATE_CHUNK_FEATURE")
                .unwrap_or(&json!(true))
                .as_bool(),
            SEARCH_QUERIES: configuration
                .get("SEARCH_QUERIES")
                .unwrap_or(&json!(""))
                .as_str()
                .map(|s| s.to_string()),
            FRONTMATTER_VALS: configuration
                .get("FRONTMATTER_VALS")
                .unwrap_or(&json!(""))
                .as_str()
                .map(|s| s.to_string()),
            LINES_BEFORE_SHOW_MORE: configuration
                .get("LINES_BEFORE_SHOW_MORE")
                .unwrap_or(&json!(10))
                .as_u64()
                .map(|u| u as usize),
            DATE_RANGE_VALUE: configuration
                .get("DATE_RANGE_VALUE")
                .unwrap_or(&json!(""))
                .as_str()
                .map(|s| s.to_string()),
            FILTER_ITEMS: configuration
                .get("FILTER_ITEMS")
                .unwrap_or(&json!([]))
                .as_array()
                .map(|a| serde_json::Value::Array(a.clone())),
            SUGGESTED_QUERIES: configuration
                .get("SUGGESTED_QUERIES")
                .unwrap_or(&json!(""))
                .as_str()
                .map(|s| s.to_string()),
            IMAGE_RANGE_START_KEY: configuration
                .get("IMAGE_RANGE_START_KEY")
                .unwrap_or(&json!(""))
                .as_str()
                .map(|s| s.to_string()),
            IMAGE_RANGE_END_KEY: configuration
                .get("IMAGE_RANGE_END_KEY")
                .unwrap_or(&json!(""))
                .as_str()
                .map(|s| s.to_string()),
            DOCUMENT_UPLOAD_FEATURE: configuration
                .get("DOCUMENT_UPLOAD_FEATURE")
                .unwrap_or(&json!(true))
                .as_bool(),
            FILE_NAME_KEY: configuration
                .get("FILE_NAME_KEY")
                .unwrap_or(&json!(""))
                .as_str()
                .expect("FILE_NAME_KEY should exist")
                .to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatasetAndOrgWithSubAndPlan {
    pub dataset: Dataset,
    pub organization: OrganizationWithSubAndPlan,
}

impl DatasetAndOrgWithSubAndPlan {
    pub fn from_components(dataset: Dataset, organization: OrganizationWithSubAndPlan) -> Self {
        DatasetAndOrgWithSubAndPlan {
            dataset,
            organization,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "name": "Trieve",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "registerable": true,
}))]
#[diesel(table_name = organizations)]
pub struct Organization {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub registerable: Option<bool>,
}

impl Organization {
    pub fn from_details(name: String) -> Self {
        Organization {
            id: uuid::Uuid::new_v4(),
            name,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            registerable: Some(true),
        }
    }

    pub fn from_org_with_plan_sub(org_plan_sub: OrganizationWithSubAndPlan) -> Self {
        org_plan_sub.organization.clone()
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, ValidGrouping)]
#[diesel(table_name = invitations)]
pub struct Invitation {
    pub id: uuid::Uuid,
    pub email: String,
    pub organization_id: uuid::Uuid,
    pub used: bool,
    pub expires_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub role: i32,
}

// any type that implements Into<String> can be used to create Invitation
impl Invitation {
    pub fn from_details(email: String, organization_id: uuid::Uuid, role: i32) -> Self {
        Invitation {
            id: uuid::Uuid::new_v4(),
            email,
            organization_id,
            used: false,
            expires_at: chrono::Utc::now().naive_local() + chrono::Duration::days(3),
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            role,
        }
    }
    pub fn expired(&self) -> bool {
        self.expires_at < chrono::Utc::now().naive_local()
    }
}

#[derive(
    Debug, Serialize, Deserialize, Selectable, Clone, Queryable, Insertable, ValidGrouping, ToSchema,
)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "stripe_id": "plan_123",
    "chunk_count": 1000,
    "file_storage": 512,
    "user_count": 5,
    "dataset_count": 1,
    "message_count": 1000,
    "amount": 1000,
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "name": "Free",
}))]
#[diesel(table_name = stripe_plans)]
pub struct StripePlan {
    pub id: uuid::Uuid,
    pub stripe_id: String,
    pub chunk_count: i32,
    pub file_storage: i64,
    pub user_count: i32,
    pub dataset_count: i32,
    pub message_count: i32,
    pub amount: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub name: String,
}

impl StripePlan {
    #[allow(clippy::too_many_arguments)]
    pub fn from_details(
        stripe_id: String,
        chunk_count: i32,
        file_storage: i64,
        user_count: i32,
        dataset_count: i32,
        message_count: i32,
        amount: i64,
        name: String,
    ) -> Self {
        StripePlan {
            id: uuid::Uuid::new_v4(),
            stripe_id,
            chunk_count,
            file_storage,
            user_count,
            dataset_count,
            message_count,
            amount,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            name,
        }
    }
}

impl Default for StripePlan {
    fn default() -> Self {
        let unlimited = std::env::var("UNLIMITED").unwrap_or("false".to_string());
        if unlimited == "true" {
            return StripePlan {
                id: uuid::Uuid::default(),
                stripe_id: "".to_string(),
                chunk_count: i32::MAX,
                file_storage: i64::MAX,
                user_count: i32::MAX,
                dataset_count: i32::MAX,
                message_count: i32::MAX,
                amount: 0,
                created_at: chrono::Utc::now().naive_local(),
                updated_at: chrono::Utc::now().naive_local(),
                name: "Unlimited".to_string(),
            };
        }

        StripePlan {
            id: uuid::Uuid::default(),
            stripe_id: "".to_string(),
            chunk_count: 1000,
            file_storage: 512,
            user_count: 5,
            dataset_count: 1,
            message_count: 1000,
            amount: 0,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            name: "Free".to_string(),
        }
    }
}
#[derive(
    Debug, Serialize, Deserialize, Selectable, Clone, Queryable, Insertable, ValidGrouping, ToSchema,
)]
#[schema(example=json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "stripe_id": "sub_123",
    "plan_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "organization_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "current_period_end": "2021-01-01T00:00:00",
}))]
#[diesel(table_name = stripe_subscriptions)]
pub struct StripeSubscription {
    pub id: uuid::Uuid,
    pub stripe_id: String,
    pub plan_id: uuid::Uuid,
    pub organization_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub current_period_end: Option<chrono::NaiveDateTime>,
}

impl StripeSubscription {
    pub fn from_details(
        stripe_id: String,
        plan_id: uuid::Uuid,
        organization_id: uuid::Uuid,
        current_period_end: Option<chrono::NaiveDateTime>,
    ) -> Self {
        StripeSubscription {
            id: uuid::Uuid::new_v4(),
            stripe_id,
            plan_id,
            organization_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            current_period_end,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "organization": {
        "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "name": "Trieve",
        "created_at": "2021-01-01T00:00:00",
        "updated_at": "2021-01-01T00:00:00",
        "registerable": true,
    },
    "plan": {
        "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "stripe_id": "plan_123",
        "chunk_count": 1000,
        "file_storage": 512,
        "user_count": 5,
        "dataset_count": 1,
        "message_count": 1000,
        "amount": 1000,
        "created_at": "2021-01-01T00:00:00",
        "updated_at": "2021-01-01T00:00:00",
        "name": "Free",
    },
    "subscription": {
        "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "stripe_id": "sub_123",
        "plan_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "organization_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
        "created_at": "2021-01-01T00:00:00",
        "updated_at": "2021-01-01T00:00:00",
        "current_period_end": "2021-01-01T00:00:00",
    }
}))]
pub struct OrganizationWithSubAndPlan {
    pub organization: Organization,
    pub plan: Option<StripePlan>,
    pub subscription: Option<StripeSubscription>,
}

impl OrganizationWithSubAndPlan {
    pub fn from_components(
        organization: Organization,
        plan: Option<StripePlan>,
        subscription: Option<StripeSubscription>,
    ) -> Self {
        OrganizationWithSubAndPlan {
            organization: organization.clone(),
            plan,
            subscription,
        }
    }

    pub fn with_defaults(&self) -> Self {
        OrganizationWithSubAndPlan {
            organization: self.organization.clone(),
            plan: Some(self.plan.clone().unwrap_or_default()),
            subscription: self.subscription.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Ord, PartialOrd)]
pub enum UserRole {
    Owner = 2,
    Admin = 1,
    User = 0,
}

impl From<i32> for UserRole {
    fn from(role: i32) -> Self {
        match role {
            2 => UserRole::Owner,
            1 => UserRole::Admin,
            _ => UserRole::User,
        }
    }
}

impl From<UserRole> for i32 {
    fn from(role: UserRole) -> Self {
        match role {
            UserRole::Owner => 2,
            UserRole::Admin => 1,
            UserRole::User => 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "user_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "organization_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "role": 2,
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
}))]
#[diesel(table_name = user_organizations)]
pub struct UserOrganization {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub organization_id: uuid::Uuid,
    pub role: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl UserOrganization {
    pub fn from_details(user_id: uuid::Uuid, organization_id: uuid::Uuid, role: UserRole) -> Self {
        UserOrganization {
            id: uuid::Uuid::new_v4(),
            user_id,
            organization_id,
            role: role.into(),
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "org_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "dataset_count": 1,
    "user_count": 5,
    "file_storage": 512,
    "message_count": 1000,
}))]
#[diesel(table_name = organization_usage_counts)]
pub struct OrganizationUsageCount {
    pub id: uuid::Uuid,
    pub org_id: uuid::Uuid,
    pub dataset_count: i32,
    pub user_count: i32,
    pub file_storage: i64,
    pub message_count: i32,
}

#[derive(Debug)]
pub enum ApiKeyRole {
    Read = 0,
    ReadAndWrite = 1,
}

impl From<i32> for ApiKeyRole {
    fn from(role: i32) -> Self {
        match role {
            1 => ApiKeyRole::ReadAndWrite,
            _ => ApiKeyRole::Read,
        }
    }
}

impl From<ApiKeyRole> for i32 {
    fn from(role: ApiKeyRole) -> Self {
        match role {
            ApiKeyRole::ReadAndWrite => 1,
            ApiKeyRole::Read => 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "user_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3",
    "api_key_hash": "hash",
    "name": "Trieve",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "role": 1,
    "blake3_hash": "hash",
}))]
#[diesel(table_name = user_api_key)]
pub struct UserApiKey {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub api_key_hash: Option<String>,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub role: i32,
    pub blake3_hash: Option<String>,
}

impl UserApiKey {
    pub fn from_details(
        user_id: uuid::Uuid,
        blake3_hash: String,
        name: String,
        role: ApiKeyRole,
    ) -> Self {
        UserApiKey {
            id: uuid::Uuid::new_v4(),
            user_id,
            api_key_hash: None,
            name,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            role: role.into(),
            blake3_hash: Some(blake3_hash),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "user_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "name": "Trieve",
    "role": 1,
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
}))]
pub struct ApiKeyDTO {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub name: String,
    pub role: i32,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<UserApiKey> for ApiKeyDTO {
    fn from(api_key: UserApiKey) -> Self {
        ApiKeyDTO {
            id: api_key.id,
            user_id: api_key.user_id,
            name: api_key.name,
            role: api_key.role,
            created_at: api_key.created_at,
            updated_at: api_key.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum UnifiedId {
    TrackingId(String),
    TrieveUuid(uuid::Uuid),
}

impl UnifiedId {
    pub fn as_uuid(&self) -> Option<uuid::Uuid> {
        match self {
            UnifiedId::TrackingId(_) => None,
            UnifiedId::TrieveUuid(uuid) => Some(*uuid),
        }
    }

    pub fn as_tracking_id(&self) -> Option<String> {
        match self {
            UnifiedId::TrackingId(tracking_id) => Some(tracking_id.clone()),
            UnifiedId::TrieveUuid(_) => None,
        }
    }
}

impl From<uuid::Uuid> for UnifiedId {
    fn from(uuid: uuid::Uuid) -> Self {
        UnifiedId::TrieveUuid(uuid)
    }
}

impl From<String> for UnifiedId {
    fn from(tracking_id: String) -> Self {
        UnifiedId::TrackingId(tracking_id)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct QdrantPayload {
    pub tag_set: Vec<String>,
    pub link: String,
    pub metadata: serde_json::Value,
    pub time_stamp: i64,
    pub dataset_id: uuid::Uuid,
    pub content: String,
    pub group_ids: Vec<uuid::Uuid>,
}

impl Into<Payload> for QdrantPayload {
    fn into(self) -> Payload {
        let value = json!(self);
        value
            .try_into()
            .expect("Failed to convert QdrantPayload to Payload")
    }
}

impl QdrantPayload {
    pub fn new(
        chunk_metadata: ChunkMetadata,
        group_ids: Option<Vec<uuid::Uuid>>,
        dataset_id: Option<uuid::Uuid>,
    ) -> Self {
        QdrantPayload {
            tag_set: chunk_metadata
                .tag_set
                .unwrap_or("".to_string())
                .split(',')
                .map(|tag| tag.to_string())
                .collect(),
            link: chunk_metadata.link.unwrap_or("".to_string()),
            metadata: chunk_metadata.metadata.unwrap_or_default(),
            time_stamp: chunk_metadata.time_stamp.unwrap_or_default().timestamp(),
            dataset_id: dataset_id.unwrap_or(chunk_metadata.dataset_id),
            content: chunk_metadata.content,
            group_ids: group_ids.unwrap_or_default(),
        }
    }

    pub fn new_from_point(point: RetrievedPoint, group_ids: Option<Vec<uuid::Uuid>>) -> Self {
        QdrantPayload {
            tag_set: point
                .payload
                .get("tag_set")
                .cloned()
                .unwrap_or_default()
                .as_list()
                .expect("tag_set should be a list")
                .iter()
                .map(|value| value.to_string())
                .collect(),
            link: point
                .payload
                .get("link")
                .cloned()
                .unwrap_or_default()
                .to_string(),
            metadata: point
                .payload
                .get("metadata")
                .cloned()
                .unwrap_or_default()
                .into(),
            time_stamp: point
                .payload
                .get("time_stamp")
                .cloned()
                .unwrap_or_default()
                .as_integer()
                .expect("time_stamp should be an integer"),
            dataset_id: point
                .payload
                .get("dataset_id")
                .cloned()
                .unwrap_or_default()
                .as_str()
                .map(|s| uuid::Uuid::parse_str(s).unwrap())
                .unwrap_or_default(),
            group_ids: group_ids.unwrap_or_default(),
            content: point
                .payload
                .get("content")
                .cloned()
                .unwrap_or_default()
                .to_string(),
        }
    }
}

impl From<RetrievedPoint> for QdrantPayload {
    fn from(current_point: RetrievedPoint) -> Self {
        QdrantPayload {
            tag_set: current_point
                .payload
                .get("tag_set")
                .cloned()
                .unwrap_or_default()
                .as_list()
                .expect("tag_set should be a list")
                .iter()
                .map(|value| value.to_string())
                .collect(),
            link: current_point
                .payload
                .get("link")
                .cloned()
                .unwrap_or_default()
                .to_string(),
            metadata: current_point
                .payload
                .get("metadata")
                .cloned()
                .unwrap_or_default()
                .into(),
            time_stamp: current_point
                .payload
                .get("time_stamp")
                .cloned()
                .unwrap_or_default()
                .as_integer()
                .expect("time_stamp should be an integer"),
            dataset_id: current_point
                .payload
                .get("dataset_id")
                .cloned()
                .unwrap_or_default()
                .as_str()
                .map(|s| uuid::Uuid::parse_str(s).unwrap())
                .unwrap_or_default(),
            group_ids: current_point
                .payload
                .get("group_ids")
                .cloned()
                .unwrap_or_default()
                .as_list()
                .expect("group_ids should be a list")
                .iter()
                .map(|value| value.to_string().parse().expect("Failed to parse group_id"))
                .collect(),
            content: current_point
                .payload
                .get("content")
                .cloned()
                .unwrap_or_default()
                .to_string(),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct FileWorkerMessage {
    pub file_id: uuid::Uuid,
    pub dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pub upload_file_data: FileDataDTO,
    pub attempt_number: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FileDataDTO {
    /// Name of the file being uploaded, including the extension.
    pub file_name: String,
    /// Tag set is a comma separated list of tags which will be passed down to the chunks made from the file. Tags are used to filter chunks when searching. HNSW indices are created for each tag such that there is no performance loss when filtering on them.
    pub tag_set: Option<Vec<String>>,
    /// Description is an optional convience field so you do not have to remember what the file contains or is about. It will be included on the group resulting from the file which will hold its chunk.
    pub description: Option<String>,
    /// Link to the file. This can also be any string. This can be used to filter when searching for the file's resulting chunks. The link value will not affect embedding creation.
    pub link: Option<String>,
    /// Time stamp should be an ISO 8601 combined date and time without timezone. Time_stamp is used for time window filtering and recency-biasing search results. Will be passed down to the file's chunks.
    pub time_stamp: Option<String>,
    /// Metadata is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata. Will be passed down to the file's chunks.
    pub metadata: Option<serde_json::Value>,
    /// Create chunks is a boolean which determines whether or not to create chunks from the file. If false, you can manually chunk the file and send the chunks to the create_chunk endpoint with the file_id to associate chunks with the file. Meant mostly for advanced users.
    pub create_chunks: Option<bool>,
    /// Group tracking id is an optional field which allows you to specify the tracking id of the group that is created from the file. Chunks created will be created with the tracking id of `group_tracking_id|<index of chunk>`
    pub group_tracking_id: Option<String>,
}

impl From<UploadFileData> for FileDataDTO {
    fn from(upload_file_data: UploadFileData) -> Self {
        FileDataDTO {
            file_name: upload_file_data.file_name,
            tag_set: upload_file_data.tag_set,
            description: upload_file_data.description,
            link: upload_file_data.link,
            time_stamp: upload_file_data.time_stamp,
            metadata: upload_file_data.metadata,
            create_chunks: upload_file_data.create_chunks,
            group_tracking_id: upload_file_data.group_tracking_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PointStructData {
    pub point_id: uuid::Uuid,
    pub splade_vector: Vec<(u32, f32)>,
    pub dense_vector: Vec<f32>,
    pub payload: QdrantPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantMessage {
    pub point_struct_data: PointStructData,
    pub chunk_id: uuid::Uuid,
    pub upsert_by_tracking_id: bool,
    pub dataset_id: uuid::Uuid
}
