#![allow(clippy::extra_unused_lifetimes)]

use crate::errors::ServiceError;
use crate::get_env;
use crate::operators::analytics_operator::{
    HeadQueryResponse, LatencyGraphResponse, QueryCountResponse, RPSGraphResponse,
    RagQueryResponse, RecommendationsEventResponse, SearchClusterResponse, SearchQueryResponse,
};
use crate::operators::chunk_operator::get_metadata_from_ids_query;
use crate::operators::clickhouse_operator::{CHSlimResponse, CHSlimResponseGroup};
use crate::operators::parse_operator::convert_html_to_text;
use std::io::Write;

use super::schema::*;
use crate::handlers::chunk_handler::{BoostPhrase, DistancePhrase};
use crate::handlers::file_handler::UploadFileReqPayload;
use crate::operators::search_operator::{
    get_group_metadata_filter_condition, get_group_tag_set_filter_condition,
    get_metadata_filter_condition, GroupScoreChunk,
};
use actix_web::web;
use chrono::{DateTime, NaiveDateTime};
use clickhouse::Row;
use dateparser::DateTimeUtc;
use derive_more::Display;
use diesel::expression::ValidGrouping;
use diesel::{
    deserialize::{self as deserialize, FromSql},
    pg::sql_types::Jsonb,
    pg::Pg,
    pg::PgValue,
    serialize::{self as serialize, IsNull, Output, ToSql},
};
use itertools::Itertools;
use openai_dive::v1::resources::chat::{ChatMessage, ChatMessageContent, Role};
use qdrant_client::qdrant::{GeoBoundingBox, GeoLineString, GeoPoint, GeoPolygon, GeoRadius};
use qdrant_client::{prelude::Payload, qdrant, qdrant::RetrievedPoint};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::OffsetDateTime;
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

#[derive(
    Debug, Serialize, Deserialize, Queryable, Selectable, Insertable, ValidGrouping, Clone, ToSchema,
)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "owner_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "name": "Trieve",
    "deleted": false,
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
}))]
#[diesel(table_name = topics)]
pub struct Topic {
    pub id: uuid::Uuid,
    pub name: String,
    pub deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub dataset_id: uuid::Uuid,
    pub owner_id: String,
}

impl Topic {
    pub fn from_details<S: Into<String>>(name: S, owner_id: S, dataset_id: uuid::Uuid) -> Self {
        Topic {
            id: uuid::Uuid::new_v4(),
            name: name.into(),
            deleted: false,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            dataset_id,
            owner_id: owner_id.into(),
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
    pub role: Role,
    pub content: String,
}

impl From<ChatMessageProxy> for ChatMessage {
    fn from(message: ChatMessageProxy) -> Self {
        ChatMessage {
            role: message.role,
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

#[derive(Debug, Serialize, Deserialize, Clone, Copy, ToSchema)]
#[serde(untagged)]
pub enum GeoTypes {
    Int(i64),
    Float(f64),
}

impl From<GeoTypes> for f64 {
    fn from(val: GeoTypes) -> Self {
        match val {
            GeoTypes::Int(i) => i as f64,
            GeoTypes::Float(f) => f,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema, AsExpression)]
#[diesel(sql_type = Jsonb)]
pub struct GeoInfoWithBias {
    pub location: GeoInfo,
    pub bias: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, ToSchema, AsExpression)]
#[diesel(sql_type = Jsonb)]
pub struct GeoInfo {
    pub lat: GeoTypes,
    pub lon: GeoTypes,
}

impl GeoInfo {
    pub fn haversine_distance_to(&self, other: &GeoInfo) -> f64 {
        let lat1: f64 = self.lat.into();
        let lon1: f64 = self.lon.into();
        let lat2: f64 = other.lat.into();
        let lon2: f64 = other.lon.into();

        let r = 6371.0; // Earth radius in km

        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();
        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let a = (d_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        r * c
    }
}

impl FromSql<Jsonb, Pg> for GeoInfo {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        let bytes = bytes.as_bytes();

        if bytes[0] != 1 {
            return Err("Unsupported JSONB encoding version".into());
        }
        serde_json::from_slice(&bytes[1..]).map_err(Into::into)
    }
}

impl ToSql<Jsonb, Pg> for GeoInfo {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        out.write_all(&[1])?;
        serde_json::to_writer(out, self)
            .map(|_| IsNull::No)
            .map_err(Into::into)
    }
}

impl Default for GeoInfo {
    fn default() -> Self {
        GeoInfo {
            lat: GeoTypes::Float(0.0),
            lon: GeoTypes::Float(0.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "link": "https://trieve.ai",
    "qdrant_point_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "tag_set": "[tag1,tag2]",
    "chunk_html": "<p>Hello, world!</p>",
    "metadata": {"key": "value"},
    "tracking_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "time_stamp": "2021-01-01T00:00:00",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "weight": 0.5,
}))]
pub struct ChunkMetadata {
    pub id: uuid::Uuid,
    pub link: Option<String>,
    pub qdrant_point_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub chunk_html: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub dataset_id: uuid::Uuid,
    pub weight: f64,
    pub location: Option<GeoInfo>,
    pub image_urls: Option<Vec<Option<String>>>,
    pub tag_set: Option<Vec<Option<String>>>,
    pub num_value: Option<f64>,
}

impl Default for ChunkMetadata {
    fn default() -> Self {
        ChunkMetadata {
            id: uuid::Uuid::new_v4(),
            link: None,
            qdrant_point_id: uuid::Uuid::new_v4(),
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            chunk_html: None,
            metadata: None,
            tracking_id: None,
            time_stamp: None,
            dataset_id: uuid::Uuid::new_v4(),
            weight: 0.0,
            location: None,
            image_urls: None,
            tag_set: None,
            num_value: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Selectable, Queryable, Insertable, Clone)]
#[diesel(table_name = chunk_metadata)]
pub struct ChunkMetadataTable {
    pub id: uuid::Uuid,
    pub link: Option<String>,
    pub qdrant_point_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub chunk_html: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub dataset_id: uuid::Uuid,
    pub weight: f64,
    pub location: Option<GeoInfo>,
    pub image_urls: Option<Vec<Option<String>>>,
    pub num_value: Option<f64>,
}

impl From<ChunkMetadata> for ChunkMetadataTable {
    fn from(chunk_metadata: ChunkMetadata) -> Self {
        Self {
            id: chunk_metadata.id,
            link: chunk_metadata.link,
            qdrant_point_id: chunk_metadata.qdrant_point_id,
            created_at: chunk_metadata.created_at,
            updated_at: chunk_metadata.updated_at,
            chunk_html: chunk_metadata.chunk_html,
            metadata: chunk_metadata.metadata,
            tracking_id: chunk_metadata.tracking_id,
            time_stamp: chunk_metadata.time_stamp,
            dataset_id: chunk_metadata.dataset_id,
            weight: chunk_metadata.weight,
            location: chunk_metadata.location,
            image_urls: chunk_metadata.image_urls,
            num_value: chunk_metadata.num_value,
        }
    }
}

impl ChunkMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn from_details(
        chunk_html: &Option<String>,
        link: &Option<String>,
        tag_set: &Option<Vec<Option<String>>>,
        qdrant_point_id: uuid::Uuid,
        metadata: Option<serde_json::Value>,
        tracking_id: Option<String>,
        time_stamp: Option<NaiveDateTime>,
        location: Option<GeoInfo>,
        image_urls: Option<Vec<String>>,
        dataset_id: uuid::Uuid,
        weight: f64,
        num_value: Option<f64>,
    ) -> Self {
        ChunkMetadata {
            id: uuid::Uuid::new_v4(),
            chunk_html: chunk_html.clone(),
            link: link.clone(),
            qdrant_point_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            tag_set: tag_set.clone(),
            metadata,
            tracking_id,
            time_stamp,
            location,
            dataset_id,
            weight,
            image_urls: image_urls.map(|urls| urls.into_iter().map(Some).collect()),
            num_value,
        }
    }

    pub fn from_table_and_tag_set(
        chunk_metadata_table: ChunkMetadataTable,
        tag_set: Vec<String>,
    ) -> Self {
        Self {
            id: chunk_metadata_table.id,
            chunk_html: chunk_metadata_table.chunk_html,
            link: chunk_metadata_table.link,
            qdrant_point_id: chunk_metadata_table.qdrant_point_id,
            created_at: chunk_metadata_table.created_at,
            updated_at: chunk_metadata_table.updated_at,
            tag_set: Some(tag_set.into_iter().map(Some).collect()),
            metadata: chunk_metadata_table.metadata,
            tracking_id: chunk_metadata_table.tracking_id,
            time_stamp: chunk_metadata_table.time_stamp,
            location: chunk_metadata_table.location,
            dataset_id: chunk_metadata_table.dataset_id,
            weight: chunk_metadata_table.weight,
            image_urls: chunk_metadata_table.image_urls,
            num_value: chunk_metadata_table.num_value,
        }
    }

    pub fn from_table_and_tag_set_option_string(
        chunk_metadata_table: ChunkMetadataTable,
        tag_set: Option<Vec<Option<String>>>,
    ) -> Self {
        Self {
            id: chunk_metadata_table.id,
            chunk_html: chunk_metadata_table.chunk_html,
            link: chunk_metadata_table.link,
            qdrant_point_id: chunk_metadata_table.qdrant_point_id,
            created_at: chunk_metadata_table.created_at,
            updated_at: chunk_metadata_table.updated_at,
            tag_set,
            metadata: chunk_metadata_table.metadata,
            tracking_id: chunk_metadata_table.tracking_id,
            time_stamp: chunk_metadata_table.time_stamp,
            location: chunk_metadata_table.location,
            dataset_id: chunk_metadata_table.dataset_id,
            weight: chunk_metadata_table.weight,
            image_urls: chunk_metadata_table.image_urls,
            num_value: chunk_metadata_table.num_value,
        }
    }
}

impl ChunkMetadata {
    #[allow(clippy::too_many_arguments)]
    pub fn from_details_with_id<T: Into<uuid::Uuid>>(
        id: T,
        chunk_html: Option<String>,
        link: &Option<String>,
        tag_set: &Option<Vec<Option<String>>>,
        qdrant_point_id: uuid::Uuid,
        metadata: Option<serde_json::Value>,
        tracking_id: Option<String>,
        time_stamp: Option<NaiveDateTime>,
        location: Option<GeoInfo>,
        image_urls: Option<Vec<String>>,
        dataset_id: uuid::Uuid,
        weight: f64,
        num_value: Option<f64>,
    ) -> Self {
        ChunkMetadata {
            id: id.into(),
            chunk_html: chunk_html.clone(),
            link: link.clone(),
            qdrant_point_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            tag_set: tag_set.clone(),
            metadata,
            tracking_id,
            time_stamp,
            location,
            dataset_id,
            weight,
            image_urls: image_urls.map(|urls| urls.into_iter().map(Some).collect()),
            num_value,
        }
    }
}

impl From<ChunkMetadataStringTagSet> for ChunkMetadata {
    fn from(chunk_metadata_string_tag_set: ChunkMetadataStringTagSet) -> Self {
        ChunkMetadata {
            id: chunk_metadata_string_tag_set.id,
            link: chunk_metadata_string_tag_set.link,
            qdrant_point_id: chunk_metadata_string_tag_set.qdrant_point_id,
            created_at: chunk_metadata_string_tag_set.created_at,
            updated_at: chunk_metadata_string_tag_set.updated_at,
            chunk_html: chunk_metadata_string_tag_set.chunk_html,
            metadata: chunk_metadata_string_tag_set.metadata,
            tracking_id: chunk_metadata_string_tag_set.tracking_id,
            time_stamp: chunk_metadata_string_tag_set.time_stamp,
            dataset_id: chunk_metadata_string_tag_set.dataset_id,
            weight: chunk_metadata_string_tag_set.weight,
            location: chunk_metadata_string_tag_set.location,
            image_urls: chunk_metadata_string_tag_set.image_urls,
            tag_set: chunk_metadata_string_tag_set
                .tag_set
                .map(|tags| tags.split(',').map(|tag| Some(tag.to_string())).collect()),
            num_value: chunk_metadata_string_tag_set.num_value,
        }
    }
}

impl From<SlimChunkMetadata> for ChunkMetadata {
    fn from(slim_chunk: SlimChunkMetadata) -> Self {
        ChunkMetadata {
            id: slim_chunk.id,
            chunk_html: Some("".to_string()),
            link: slim_chunk.link,
            qdrant_point_id: slim_chunk.qdrant_point_id,
            created_at: slim_chunk.created_at,
            updated_at: slim_chunk.updated_at,
            tag_set: slim_chunk
                .tag_set
                .map(|tags| tags.split(',').map(|tag| Some(tag.to_string())).collect()),
            metadata: slim_chunk.metadata,
            tracking_id: slim_chunk.tracking_id,
            time_stamp: slim_chunk.time_stamp,
            location: slim_chunk.location,
            dataset_id: slim_chunk.dataset_id,
            weight: slim_chunk.weight,
            image_urls: slim_chunk.image_urls,
            num_value: slim_chunk.num_value,
        }
    }
}

impl From<ContentChunkMetadata> for ChunkMetadata {
    fn from(content_chunk: ContentChunkMetadata) -> Self {
        ChunkMetadata {
            id: content_chunk.id,
            chunk_html: content_chunk.chunk_html,
            link: None,
            qdrant_point_id: content_chunk.qdrant_point_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            tag_set: None,
            metadata: None,
            tracking_id: content_chunk.tracking_id,
            time_stamp: content_chunk.time_stamp,
            location: None,
            dataset_id: uuid::Uuid::new_v4(),
            weight: content_chunk.weight,
            image_urls: content_chunk.image_urls,
            num_value: content_chunk.num_value,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IngestSpecificChunkMetadata {
    pub id: uuid::Uuid,
    pub dataset_config: ServerDatasetConfiguration,
    pub dataset_id: uuid::Uuid,
    pub qdrant_point_id: uuid::Uuid,
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
    pub link: Option<String>,
    pub qdrant_point_id: uuid::Uuid,
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
            link: chunk.link,
            qdrant_point_id: chunk.qdrant_point_id,
            created_at: chunk.created_at,
            updated_at: chunk.updated_at,
            tag_set: chunk.tag_set.map(|tags| {
                tags.into_iter()
                    .map(|tag| tag.unwrap_or_default())
                    .join(",")
            }),
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

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
#[schema(example = json!({
    "metadata": [
        {
            "id": "d290f1ee-6c54-4b01-90e6-d701748f0851",
            "content": "Some content",
            "link": "https://example.com",
            "chunk_html": "<p>Some HTML content</p>",
            "metadata": {"key1": "value1", "key2": "value2"},
            "time_stamp": "2021-01-01T00:00:00",
            "weight": 0.5,
        }
    ],
    "highlights": ["highlight is two tokens: high, light", "whereas hello is only one token: hello"],
    "score": 0.5
}))]
pub struct ScoreChunkDTO {
    pub metadata: Vec<ChunkMetadataTypes>,
    pub highlights: Option<Vec<String>>,
    pub score: f64,
}

impl ScoreChunkDTO {
    pub fn slim(&self) -> ScoreChunkDTO {
        let mut slim_chunk_dto = self.clone();
        slim_chunk_dto.metadata = slim_chunk_dto
            .metadata
            .into_iter()
            .map(|metadata| match metadata {
                ChunkMetadataTypes::ID(slim_chunk_metadata) => {
                    ChunkMetadataTypes::ID(slim_chunk_metadata)
                }
                ChunkMetadataTypes::Metadata(chunk_metadata) => {
                    ChunkMetadataTypes::ID(chunk_metadata.into())
                }
                ChunkMetadataTypes::Content(content_chunk_metadata) => {
                    ChunkMetadataTypes::ID(content_chunk_metadata.into())
                }
            })
            .collect();
        slim_chunk_dto
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
#[serde(untagged)]
pub enum ChunkMetadataTypes {
    ID(SlimChunkMetadata),
    Metadata(ChunkMetadataStringTagSet),
    Content(ContentChunkMetadata),
}

impl From<ChunkMetadataTypes> for ChunkMetadata {
    fn from(val: ChunkMetadataTypes) -> Self {
        match val {
            ChunkMetadataTypes::ID(slim_chunk_metadata) => slim_chunk_metadata.into(),
            ChunkMetadataTypes::Metadata(chunk_metadata) => chunk_metadata.into(),
            ChunkMetadataTypes::Content(content_chunk_metadata) => content_chunk_metadata.into(),
        }
    }
}

impl From<ChunkMetadata> for ChunkMetadataTypes {
    fn from(chunk_metadata: ChunkMetadata) -> Self {
        ChunkMetadataTypes::Metadata(chunk_metadata.into())
    }
}

impl From<SlimChunkMetadata> for ChunkMetadataTypes {
    fn from(slim_chunk_metadata: SlimChunkMetadata) -> Self {
        ChunkMetadataTypes::ID(slim_chunk_metadata)
    }
}

impl From<ContentChunkMetadata> for ChunkMetadataTypes {
    fn from(content_chunk_metadata: ContentChunkMetadata) -> Self {
        ChunkMetadataTypes::Content(content_chunk_metadata)
    }
}

impl ChunkMetadataTypes {
    pub fn metadata(&self) -> ChunkMetadata {
        match self {
            ChunkMetadataTypes::Metadata(metadata) => metadata.clone().into(),
            ChunkMetadataTypes::ID(slim_metadata) => slim_metadata.clone().into(),
            ChunkMetadataTypes::Content(content_metadata) => content_metadata.clone().into(),
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
    pub qdrant_point_id: uuid::Uuid,
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
pub struct ChunkMetadataStringTagSet {
    pub id: uuid::Uuid,
    pub link: Option<String>,
    pub qdrant_point_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub chunk_html: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub dataset_id: uuid::Uuid,
    pub weight: f64,
    pub location: Option<GeoInfo>,
    pub image_urls: Option<Vec<Option<String>>>,
    pub tag_set: Option<String>,
    pub num_value: Option<f64>,
}

impl From<ChunkMetadata> for ChunkMetadataStringTagSet {
    fn from(chunk: ChunkMetadata) -> Self {
        ChunkMetadataStringTagSet {
            id: chunk.id,
            link: chunk.link,
            qdrant_point_id: chunk.qdrant_point_id,
            created_at: chunk.created_at,
            updated_at: chunk.updated_at,
            chunk_html: chunk.chunk_html,
            metadata: chunk.metadata,
            tracking_id: chunk.tracking_id,
            time_stamp: chunk.time_stamp,
            dataset_id: chunk.dataset_id,
            weight: chunk.weight,
            location: chunk.location,
            image_urls: chunk.image_urls,
            tag_set: chunk.tag_set.map(|tags| {
                tags.into_iter()
                    .map(|tag| tag.unwrap_or_default())
                    .join(",")
            }),
            num_value: chunk.num_value,
        }
    }
}

impl From<ContentChunkMetadata> for ChunkMetadataStringTagSet {
    fn from(chunk: ContentChunkMetadata) -> Self {
        ChunkMetadataStringTagSet {
            id: chunk.id,
            link: None,
            qdrant_point_id: chunk.qdrant_point_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            chunk_html: Some("".to_string()),
            metadata: None,
            tracking_id: chunk.tracking_id,
            time_stamp: None,
            dataset_id: uuid::Uuid::new_v4(),
            weight: chunk.weight,
            location: None,
            image_urls: chunk.image_urls,
            tag_set: None,
            num_value: chunk.num_value,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable)]
pub struct SlimChunkMetadataTable {
    pub id: uuid::Uuid,
    pub link: Option<String>,
    pub qdrant_point_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub location: Option<GeoInfo>,
    pub dataset_id: uuid::Uuid,
    pub weight: f64,
    pub image_urls: Option<Vec<Option<String>>>,
    pub num_value: Option<f64>,
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
    pub qdrant_point_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub tag_set: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub location: Option<GeoInfo>,
    pub dataset_id: uuid::Uuid,
    pub weight: f64,
    pub image_urls: Option<Vec<Option<String>>>,
    pub num_value: Option<f64>,
}

impl SlimChunkMetadata {
    pub fn from_table_and_tag_set(table: SlimChunkMetadataTable, tag_set: Vec<String>) -> Self {
        Self {
            id: table.id,
            link: table.link,
            qdrant_point_id: table.qdrant_point_id,
            created_at: table.created_at,
            updated_at: table.updated_at,
            tag_set: Some(tag_set.into_iter().join(",")),
            metadata: table.metadata,
            tracking_id: table.tracking_id,
            time_stamp: table.time_stamp,
            location: table.location,
            dataset_id: table.dataset_id,
            weight: table.weight,
            image_urls: table.image_urls,
            num_value: table.num_value,
        }
    }
}

impl From<ChunkMetadata> for SlimChunkMetadata {
    fn from(chunk: ChunkMetadata) -> Self {
        SlimChunkMetadata {
            id: chunk.id,
            link: chunk.link,
            qdrant_point_id: chunk.qdrant_point_id,
            created_at: chunk.created_at,
            updated_at: chunk.updated_at,
            tag_set: chunk.tag_set.map(|tags| {
                tags.into_iter()
                    .map(|tag| tag.unwrap_or_default())
                    .join(",")
            }),
            metadata: chunk.metadata,
            tracking_id: chunk.tracking_id,
            time_stamp: chunk.time_stamp,
            location: chunk.location,
            dataset_id: chunk.dataset_id,
            weight: chunk.weight,
            image_urls: chunk.image_urls,
            num_value: chunk.num_value,
        }
    }
}

impl From<ChunkMetadataStringTagSet> for SlimChunkMetadata {
    fn from(chunk: ChunkMetadataStringTagSet) -> Self {
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
            location: chunk.location,
            dataset_id: chunk.dataset_id,
            weight: chunk.weight,
            image_urls: chunk.image_urls,
            num_value: chunk.num_value,
        }
    }
}

impl From<ContentChunkMetadata> for SlimChunkMetadata {
    fn from(chunk: ContentChunkMetadata) -> Self {
        SlimChunkMetadata {
            id: chunk.id,
            link: None,
            qdrant_point_id: chunk.qdrant_point_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            tag_set: None,
            metadata: None,
            tracking_id: chunk.tracking_id,
            time_stamp: None,
            location: None,
            dataset_id: uuid::Uuid::new_v4(),
            weight: chunk.weight,
            image_urls: chunk.image_urls,
            num_value: chunk.num_value,
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
pub struct ContentChunkMetadata {
    pub id: uuid::Uuid,
    pub qdrant_point_id: uuid::Uuid,
    pub chunk_html: Option<String>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub weight: f64,
    pub image_urls: Option<Vec<Option<String>>>,
    pub num_value: Option<f64>,
}

impl From<ChunkMetadata> for ContentChunkMetadata {
    fn from(chunk: ChunkMetadata) -> Self {
        ContentChunkMetadata {
            id: chunk.id,
            qdrant_point_id: chunk.qdrant_point_id,
            chunk_html: chunk.chunk_html,
            tracking_id: chunk.tracking_id,
            time_stamp: chunk.time_stamp,
            weight: chunk.weight,
            image_urls: chunk.image_urls,
            num_value: chunk.num_value,
        }
    }
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
    pub tag_set: Option<Vec<Option<String>>>,
}

impl ChunkGroup {
    pub fn from_details(
        name: Option<String>,
        description: Option<String>,
        dataset_id: uuid::Uuid,
        tracking_id: Option<String>,
        metadata: Option<serde_json::Value>,
        tag_set: Option<Vec<Option<String>>>,
    ) -> Self {
        ChunkGroup {
            id: uuid::Uuid::new_v4(),
            name: name.unwrap_or_default(),
            description: description.unwrap_or_default(),
            dataset_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            tracking_id,
            metadata,
            tag_set,
        }
    }

    pub fn from_details_with_id(
        id: uuid::Uuid,
        name: String,
        description: Option<String>,
        dataset_id: uuid::Uuid,
        tracking_id: Option<String>,
        metadata: Option<serde_json::Value>,
        tag_set: Option<Vec<Option<String>>>,
    ) -> Self {
        ChunkGroup {
            id,
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
pub struct ChunkGroupAndFileId {
    pub id: uuid::Uuid,
    pub dataset_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub tracking_id: Option<String>,
    pub tag_set: Option<Vec<Option<String>>>,
    pub metadata: Option<serde_json::Value>,
    pub file_id: Option<uuid::Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl ChunkGroupAndFileId {
    pub fn from_group(group: ChunkGroup, file_id: Option<uuid::Uuid>) -> Self {
        Self {
            id: group.id,
            dataset_id: group.dataset_id,
            name: group.name,
            description: group.description,
            tracking_id: group.tracking_id,
            tag_set: group.tag_set,
            metadata: group.metadata,
            file_id,
            created_at: group.created_at,
            updated_at: group.updated_at,
        }
    }

    pub fn to_group(&self) -> ChunkGroup {
        ChunkGroup {
            id: self.id,
            dataset_id: self.dataset_id,
            name: self.name.clone(),
            description: self.description.clone(),
            tracking_id: self.tracking_id.clone(),
            tag_set: self.tag_set.clone(),
            metadata: self.metadata.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
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
    pub metadata: Option<serde_json::Value>,
    pub link: Option<String>,
    pub time_stamp: Option<chrono::NaiveDateTime>,
    pub dataset_id: uuid::Uuid,
    pub tag_set: Option<Vec<Option<String>>>,
}

impl File {
    #[allow(clippy::too_many_arguments)]
    pub fn from_details(
        file_id: Option<uuid::Uuid>,
        file_name: &str,
        size: i64,
        tag_set: Option<Vec<Option<String>>>,
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

#[derive(Debug, Serialize, Deserialize, Clone, Row, ToSchema)]
pub struct ClickhouseEvent {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: uuid::Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub dataset_id: uuid::Uuid,
    pub event_type: String,
    pub event_data: String,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(example=json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "event_type": "file_uploaded",
    "event_data": {"group_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3", "file_name": "file.txt"},
}))]
pub struct Event {
    pub id: uuid::Uuid,
    pub created_at: String,
    pub dataset_id: uuid::Uuid,
    pub event_type: String,
    pub event_data: String,
}

impl From<ClickhouseEvent> for Event {
    fn from(clickhouse_event: ClickhouseEvent) -> Self {
        Event {
            id: uuid::Uuid::from_bytes(*clickhouse_event.id.as_bytes()),
            created_at: clickhouse_event.created_at.to_string(),
            dataset_id: uuid::Uuid::from_bytes(*clickhouse_event.dataset_id.as_bytes()),
            event_type: clickhouse_event.event_type,
            event_data: clickhouse_event.event_data,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Display, ToSchema)]
#[serde(untagged)]
pub enum EventType {
    #[display(fmt = "file_uploaded")]
    FileUploaded {
        file_id: uuid::Uuid,
        file_name: String,
    },
    #[display(fmt = "file_upload_failed")]
    FileUploadFailed { file_id: uuid::Uuid, error: String },
    #[display(fmt = "chunks_uploaded")]
    ChunksUploaded { chunk_ids: Vec<uuid::Uuid> },
    #[display(fmt = "chunk_action_failed")]
    ChunkActionFailed { chunk_id: uuid::Uuid, error: String },
    #[display(fmt = "chunk_updated")]
    ChunkUpdated { chunk_id: uuid::Uuid },
    #[display(fmt = "bulk_chunks_deleted")]
    BulkChunksDeleted { message: String },
    #[display(fmt = "dataset_delete_failed")]
    DatasetDeleteFailed { error: String },
    #[display(fmt = "qdrant_index_failed")]
    QdrantUploadFailed {
        chunk_id: uuid::Uuid,
        qdrant_point_id: uuid::Uuid,
        error: String,
    },
    #[display(fmt = "bulk_chunk_upload_failed")]
    BulkChunkUploadFailed {
        chunk_ids: Vec<uuid::Uuid>,
        error: String,
    },
    #[display(fmt = "group_chunks_updated")]
    GroupChunksUpdated { group_id: uuid::Uuid },
    #[display(fmt = "group_chunks_action_failed")]
    GroupChunksActionFailed { group_id: uuid::Uuid, error: String },
}

impl EventType {
    pub fn get_all_event_types() -> Vec<EventTypeRequest> {
        vec![
            EventTypeRequest::FileUploaded,
            EventTypeRequest::FileUploadFailed,
            EventTypeRequest::ChunksUploaded,
            EventTypeRequest::ChunkActionFailed,
            EventTypeRequest::ChunkUpdated,
            EventTypeRequest::BulkChunksDeleted,
            EventTypeRequest::DatasetDeleteFailed,
            EventTypeRequest::QdrantUploadFailed,
            EventTypeRequest::BulkChunkUploadFailed,
            EventTypeRequest::GroupChunksUpdated,
            EventTypeRequest::GroupChunksActionFailed,
        ]
    }
}

impl Event {
    pub fn from_details(dataset_id: uuid::Uuid, event_type: EventType) -> Self {
        Event {
            id: uuid::Uuid::new_v4(),
            created_at: chrono::Utc::now().naive_local().to_string(),
            dataset_id,
            event_type: event_type.to_string(),
            event_data: serde_json::to_value(event_type).unwrap().to_string(),
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

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Queryable,
    Insertable,
    Selectable,
    Clone,
    ToSchema,
    QueryableByName,
)]
#[schema(example=json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "name": "Trieve",
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
    "organization_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "tracking_id": "3",
    "server_configuration": {"key": "value"},
}))]
#[diesel(table_name = datasets)]
pub struct Dataset {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub organization_id: uuid::Uuid,
    pub server_configuration: serde_json::Value,
    pub tracking_id: Option<String>,
    pub deleted: i32,
}

impl Dataset {
    pub fn from_details(
        name: String,
        organization_id: uuid::Uuid,
        tracking_id: Option<String>,
        server_configuration: serde_json::Value,
    ) -> Self {
        Dataset {
            id: uuid::Uuid::new_v4(),
            name,
            organization_id,
            tracking_id,
            server_configuration,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            deleted: 0,
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
}))]
pub struct DatasetDTO {
    pub id: uuid::Uuid,
    pub name: String,
    pub tracking_id: Option<String>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub organization_id: uuid::Uuid,
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
    "MESSAGE_TO_QUERY_PROMPT": "Write a 1-2 sentence semantic search query along the lines of a hypothetical response to: \n\n",
    "N_RETRIEVALS_TO_INCLUDE": 5,
    "DUPLICATE_DISTANCE_THRESHOLD": 1.1,
    "EMBEDDING_SIZE": 1536,
    "LLM_DEFAULT_MODEL": "gpt-3.5-turbo-1106",
    "FULLTEXT_ENABLED": true,
    "SEMANTIC_ENABLED": true,
    "EMBEDDING_QUERY_PREFIX": "Search for",
    "USE_MESSAGE_TO_QUERY_PROMPT": false,
    "FREQUENCY_PENALTY": 0.0,
    "TEMPERATURE": 0.5,
    "PRESENCE_PENALTY": 0.0,
    "STOP_TOKENS": ["\n\n", "\n"],
    "INDEXED_ONLY": false,
    "LOCKED": false,
}))]
#[allow(non_snake_case)]
pub struct ServerDatasetConfiguration {
    pub LLM_BASE_URL: String,
    pub EMBEDDING_BASE_URL: String,
    pub EMBEDDING_MODEL_NAME: String,
    pub RERANKER_BASE_URL: String,
    pub MESSAGE_TO_QUERY_PROMPT: String,
    pub RAG_PROMPT: String,
    pub N_RETRIEVALS_TO_INCLUDE: usize,
    pub EMBEDDING_SIZE: usize,
    pub LLM_DEFAULT_MODEL: String,
    pub FULLTEXT_ENABLED: bool,
    pub SEMANTIC_ENABLED: bool,
    pub EMBEDDING_QUERY_PREFIX: String,
    pub USE_MESSAGE_TO_QUERY_PROMPT: bool,
    pub FREQUENCY_PENALTY: Option<f64>,
    pub TEMPERATURE: Option<f64>,
    pub PRESENCE_PENALTY: Option<f64>,
    pub STOP_TOKENS: Option<Vec<String>>,
    pub INDEXED_ONLY: bool,
    pub LOCKED: bool,
    pub SYSTEM_PROMPT: Option<String>,
}

impl ServerDatasetConfiguration {
    pub fn from_json(configuration: serde_json::Value) -> Self {
        let default_config = json!({});
        let configuration = configuration
            .as_object()
            .unwrap_or(default_config.as_object().unwrap());

        ServerDatasetConfiguration {
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
            MESSAGE_TO_QUERY_PROMPT: configuration
                .get("MESSAGE_TO_QUERY_PROMPT")
                .unwrap_or(&json!("Write a 1-2 sentence semantic search query along the lines of a hypothetical response to: \n\n".to_string()))
                .as_str()
                .map(|s| {
                    if s.is_empty() {
                        "Write a 1-2 sentence semantic search query along the lines of a hypothetical response to: \n\n".to_string()
                    } else {
                        s.to_string()
                    }
                }).unwrap_or("Write a 1-2 sentence semantic search query along the lines of a hypothetical response to: \n\n".to_string()),
            RAG_PROMPT: configuration
                .get("RAG_PROMPT")
                .unwrap_or(&json!("Use the following retrieved documents in your response. Include footnotes in the format of the document number that you used for a sentence in square brackets at the end of the sentences like [^n] where n is the doc number. These are the docs:".to_string()))
                .as_str()
                .map(|s|
                    if s.is_empty() {
                        "Use the following retrieved documents in your response. Include footnotes in the format of the document number that you used for a sentence in square brackets at the end of the sentences like [^n] where n is the doc number. These are the docs:".to_string()
                    } else {
                        s.to_string()
                    }
                )
                .unwrap_or("Use the following retrieved documents in your response. Include footnotes in the format of the document number that you used for a sentence in square brackets at the end of the sentences like [^n] where n is the doc number. These are the docs:".to_string()),
            N_RETRIEVALS_TO_INCLUDE: configuration
                .get("N_RETRIEVALS_TO_INCLUDE")
                .unwrap_or(&json!(5))
                .as_u64()
                .map(|u| u as usize)
                .unwrap_or(5),
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
            RERANKER_BASE_URL: configuration
                .get("RERANKER_SERVER_ORIGIN")
                .unwrap_or(&json!(get_env!("RERANKER_SERVER_ORIGIN", "RERANKER_SERVER_ORIGIN must be set").to_string()))
                .as_str()
                .map(|s| {
                    if s.is_empty() {
                        get_env!("RERANKER_SERVER_ORIGIN", "RERANKER_BASE_URL must be set").to_string()
                    } else {
                        s.to_string()
                    }
                }).expect("RERANKER_SERVER_ORIGIN should exist"),
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
            FULLTEXT_ENABLED: configuration
                .get("FULLTEXT_ENABLED")
                .unwrap_or(&json!(true))
                .as_bool()
                .unwrap_or(true),
            SEMANTIC_ENABLED: configuration
                .get("SEMANTIC_ENABLED")
                .unwrap_or(&json!(true))
                .as_bool()
                .unwrap_or(true),
            EMBEDDING_QUERY_PREFIX: configuration
                .get("EMBEDDING_QUERY_PREFIX")
                .unwrap_or(&{
                    let model_name = configuration
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
                        .unwrap_or("text-embedding-3-small".to_string());
                    if model_name == "jina-base-en" {
                        json!("Search for")
                    } else {
                        json!("")
                    }
                })
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or("".to_string()),
            USE_MESSAGE_TO_QUERY_PROMPT: configuration
                .get("USE_MESSAGE_TO_QUERY_PROMPT")
                .unwrap_or(&json!(false))
                .as_bool()
                .unwrap_or(false),
            FREQUENCY_PENALTY: configuration
                .get("FREQUENCY_PENALTY")
                .and_then(|v| v.as_f64()),
            TEMPERATURE: configuration
                .get("TEMPERATURE")
                .and_then(|v| v.as_f64()),
            PRESENCE_PENALTY: configuration
                .get("PRESENCE_PENALTY")
                .and_then(|v| v.as_f64()),
            STOP_TOKENS: configuration
                .get("STOP_TOKENS")
                .and_then(|v| v.as_str())
                .map(|v| v.split(',').map(|s| s.to_string()).collect::<Vec<String>>()),
            INDEXED_ONLY: configuration
                .get("INDEXED_ONLY")
                .unwrap_or(&json!(false))
                .as_bool()
                .unwrap_or(false),
            LOCKED: configuration
                .get("LOCKED")
                .unwrap_or(&json!(false))
                .as_bool()
                .unwrap_or(false),
            SYSTEM_PROMPT: configuration
                .get("SYSTEM_PROMPT")
                .and_then(|v| v.as_str())
                .map(|s|
                    if s.is_empty() {
                        "".to_string()
                    } else {
                        s.to_string()
                    }
                )
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
    pub deleted: i32,
}

impl Organization {
    pub fn from_details(name: String) -> Self {
        Organization {
            id: uuid::Uuid::new_v4(),
            name,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            registerable: Some(true),
            deleted: 0,
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
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            role,
        }
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
    "chunk_count": 1000,
}))]
#[diesel(table_name = organization_usage_counts)]
pub struct OrganizationUsageCount {
    pub id: uuid::Uuid,
    pub org_id: uuid::Uuid,
    pub dataset_count: i32,
    pub user_count: i32,
    pub file_storage: i64,
    pub message_count: i32,
    pub chunk_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone)]
#[diesel(table_name = dataset_tags)]
pub struct DatasetTags {
    pub id: uuid::Uuid,
    pub dataset_id: uuid::Uuid,
    pub tag: String,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone)]
#[diesel(table_name = chunk_metadata_tags)]
pub struct ChunkMetadataTags {
    pub id: uuid::Uuid,
    pub chunk_metadata_id: uuid::Uuid,
    pub tag_id: uuid::Uuid,
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
    pub dataset_ids: Option<Vec<Option<String>>>,
    pub organization_ids: Option<Vec<Option<String>>>,
}

impl UserApiKey {
    pub fn from_details(
        user_id: uuid::Uuid,
        blake3_hash: String,
        name: String,
        role: ApiKeyRole,
        dataset_ids: Option<Vec<uuid::Uuid>>,
        organization_ids: Option<Vec<uuid::Uuid>>,
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
            dataset_ids: dataset_ids
                .map(|ids| ids.into_iter().map(|id| Some(id.to_string())).collect()),
            organization_ids: organization_ids
                .map(|ids| ids.into_iter().map(|id| Some(id.to_string())).collect()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(example = json!({
    "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "user_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
    "name": "Trieve",
    "role": 1,
    "dataset_ids": ["d0d0d0d0-d0d0-d0d0-d0d0-d0d0d0d0d0d0"],
    "organization_ids": ["o1o1o1o1-o1o1-o1o1-o1o1-o1o1o1o1o1o1"],
    "created_at": "2021-01-01T00:00:00",
    "updated_at": "2021-01-01T00:00:00",
}))]
pub struct ApiKeyRespBody {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub name: String,
    pub role: i32,
    pub dataset_ids: Option<Vec<String>>,
    pub organization_ids: Option<Vec<String>>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl From<UserApiKey> for ApiKeyRespBody {
    fn from(api_key: UserApiKey) -> Self {
        ApiKeyRespBody {
            id: api_key.id,
            user_id: api_key.user_id,
            name: api_key.name,
            role: api_key.role,
            dataset_ids: api_key
                .dataset_ids
                .map(|ids| ids.into_iter().flatten().collect()),
            organization_ids: api_key
                .organization_ids
                .map(|ids| ids.into_iter().flatten().collect()),
            created_at: api_key.created_at,
            updated_at: api_key.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum UnifiedId {
    TrieveUuid(uuid::Uuid),
    TrackingId(String),
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
    pub tag_set: Option<Vec<Option<String>>>,
    pub link: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub time_stamp: Option<i64>,
    pub dataset_id: uuid::Uuid,
    pub content: String,
    pub group_ids: Option<Vec<uuid::Uuid>>,
    pub location: Option<GeoInfo>,
    pub num_value: Option<f64>,
    pub group_tag_set: Option<Vec<Option<String>>>,
}

impl From<QdrantPayload> for Payload {
    fn from(val: QdrantPayload) -> Self {
        let value = json!(val);
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
        group_tag_set: Option<Vec<Option<String>>>,
    ) -> Self {
        QdrantPayload {
            tag_set: chunk_metadata.tag_set,
            link: chunk_metadata.link,
            metadata: chunk_metadata.metadata,
            time_stamp: chunk_metadata.time_stamp.map(|x| x.timestamp()),
            dataset_id: dataset_id.unwrap_or(chunk_metadata.dataset_id),
            content: convert_html_to_text(&chunk_metadata.chunk_html.unwrap_or_default()),
            group_ids,
            location: chunk_metadata.location,
            num_value: chunk_metadata.num_value,
            group_tag_set,
        }
    }

    pub fn new_from_point(point: RetrievedPoint, group_ids: Option<Vec<uuid::Uuid>>) -> Self {
        QdrantPayload {
            tag_set: point.payload.get("tag_set").cloned().map(|x| {
                x.as_list()
                    .unwrap_or_default()
                    .iter()
                    .map(|value| Some(value.to_string()))
                    .collect()
            }),
            link: point.payload.get("link").cloned().map(|x| x.to_string()),
            metadata: point
                .payload
                .get("metadata")
                .cloned()
                .map(|value| value.into()),
            time_stamp: point
                .payload
                .get("time_stamp")
                .cloned()
                .and_then(|x| x.as_integer()),
            dataset_id: point
                .payload
                .get("dataset_id")
                .cloned()
                .unwrap_or_default()
                .as_str()
                .map(|s| uuid::Uuid::parse_str(s).unwrap())
                .unwrap_or_default(),
            group_ids,
            content: point
                .payload
                .get("content")
                .cloned()
                .unwrap_or_default()
                .to_string(),
            location: point
                .payload
                .get("location")
                .cloned()
                .and_then(|value| serde_json::from_value(value.into()).ok()),
            num_value: point
                .payload
                .get("num_value")
                .cloned()
                .and_then(|x| x.as_double()),
            group_tag_set: point.payload.get("group_tag_set").cloned().map(|x| {
                x.as_list()
                    .unwrap_or_default()
                    .iter()
                    .map(|value| Some(value.to_string()))
                    .collect()
            }),
        }
    }
}

impl From<RetrievedPoint> for QdrantPayload {
    fn from(point: RetrievedPoint) -> Self {
        QdrantPayload {
            tag_set: point.payload.get("tag_set").cloned().map(|x| {
                x.as_list()
                    .unwrap_or_default()
                    .iter()
                    .map(|value| Some(value.to_string().replace(['"', '\\'], "")))
                    .collect()
            }),
            link: point
                .payload
                .get("link")
                .cloned()
                .map(|x| x.to_string().replace(['"', '\\'], "")),
            metadata: point
                .payload
                .get("metadata")
                .cloned()
                .map(|value| value.into()),
            time_stamp: point
                .payload
                .get("time_stamp")
                .cloned()
                .and_then(|x| x.as_integer()),
            dataset_id: point
                .payload
                .get("dataset_id")
                .cloned()
                .unwrap_or_default()
                .as_str()
                .and_then(|s| uuid::Uuid::parse_str(s).ok())
                .unwrap_or_default(),
            group_ids: point.payload.get("group_ids").cloned().map(|x| {
                x.as_list()
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|value| value.to_string().parse().ok())
                    .collect()
            }),
            content: point
                .payload
                .get("content")
                .cloned()
                .unwrap_or_default()
                .to_string()
                .replace(['"', '\\'], ""),
            location: point
                .payload
                .get("location")
                .cloned()
                .and_then(|value| serde_json::from_value(value.into()).ok())
                .unwrap_or_default(),
            num_value: point
                .payload
                .get("num_value")
                .cloned()
                .and_then(|x| x.as_double()),
            group_tag_set: point.payload.get("group_tag_set").cloned().map(|x| {
                x.as_list()
                    .unwrap_or_default()
                    .iter()
                    .map(|value| Some(value.to_string().replace(['"', '\\'], "")))
                    .collect()
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileWorkerMessage {
    pub file_id: uuid::Uuid,
    pub dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pub upload_file_data: UploadFileReqPayload,
    pub attempt_number: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(untagged)]
pub enum RangeCondition {
    Float(f64),
    Int(i64),
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "gte": 0.0,
    "lte": 1.0,
    "gt": 0.0,
    "lt": 1.0
}))]
pub struct Range {
    // gte is the lower bound of the range. This is inclusive.
    pub gte: Option<RangeCondition>,
    // lte is the upper bound of the range. This is inclusive.
    pub lte: Option<RangeCondition>,
    // gt is the lower bound of the range. This is exclusive.
    pub gt: Option<RangeCondition>,
    // lt is the upper bound of the range. This is exclusive.
    pub lt: Option<RangeCondition>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "gte": "2021-01-01T00:00:00",
    "lte": "2021-01-01T00:00:00",
    "gt": "2021-01-01T00:00:00",
    "lt": "2021-01-01T00:00:00"
}))]
pub struct DateRange {
    // gte is the lower bound of the range. This is inclusive.
    pub gte: Option<String>,
    // lte is the upper bound of the range. This is inclusive.
    pub lte: Option<String>,
    // gt is the lower bound of the range. This is exclusive.
    pub gt: Option<String>,
    // lt is the upper bound of the range. This is exclusive.
    pub lt: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(untagged)]
pub enum MatchCondition {
    Text(String),
    Integer(i64),
    Float(f64),
}

impl MatchCondition {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        match self {
            MatchCondition::Text(text) => text.clone(),
            MatchCondition::Integer(int) => int.to_string(),
            MatchCondition::Float(float) => float.to_string(),
        }
    }

    pub fn to_i64(&self) -> i64 {
        match self {
            MatchCondition::Text(text) => text.parse().unwrap(),
            MatchCondition::Integer(int) => *int,
            MatchCondition::Float(float) => *float as i64,
        }
    }

    pub fn to_f64(&self) -> f64 {
        match self {
            MatchCondition::Text(text) => text.parse().unwrap(),
            MatchCondition::Integer(int) => *int as f64,
            MatchCondition::Float(float) => *float,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct LocationBoundingBox {
    pub top_left: GeoInfo,
    pub bottom_right: GeoInfo,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct LocationRadius {
    pub center: GeoInfo,
    pub radius: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct LocationPolygon {
    pub exterior: Vec<GeoInfo>,
    pub interior: Option<Vec<Vec<GeoInfo>>>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[serde(untagged)]
pub enum ConditionType {
    Field(FieldCondition),
    HasID(HasIDCondition),
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct HasIDCondition {
    pub ids: Option<Vec<uuid::Uuid>>,
    pub tracking_ids: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
#[schema(example = json!({
    "field": "metadata.key1",
    "match": ["value1", "value2"],
    "range": {
        "gte": 0.0,
        "lte": 1.0,
        "gt": 0.0,
        "lt": 1.0
    }
}))]
pub struct FieldCondition {
    /// Field is the name of the field to filter on. The field value will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata. To access fields inside of the metadata that you provide with the card, prefix the field name with `metadata.`.
    pub field: String,
    /// Match is the value to match on the field. The match value will be used to check for an exact substring match on the metadata values for each existing chunk. This is useful for when you want to filter chunks by arbitrary metadata.
    pub r#match: Option<Vec<MatchCondition>>,
    /// Range is a JSON object which can be used to filter chunks by a range of values. This only works for numerical fields. You can specify this if you want values in a certain range.
    pub range: Option<Range>,
    /// Date range is a JSON object which can be used to filter chunks by a range of dates. This only works for date fields. You can specify this if you want values in a certain range. You must provide ISO 8601 combined date and time without timezone.
    pub date_range: Option<DateRange>,
    /// Geo Bounding Box search is useful for when you want to find points inside a rectangular area. This is useful for when you want to filter chunks by location. The bounding box is defined by two points: the top-left and bottom-right corners of the box.
    pub geo_bounding_box: Option<LocationBoundingBox>,
    /// Geo Radius search is useful for when you want to find points within a certain distance of a point. This is useful for when you want to filter chunks by location. The radius is in meters.
    pub geo_radius: Option<LocationRadius>,
    /// Geo Polygons search is useful for when you want to find points inside an irregularly shaped area, for example a country boundary or a forest boundary. A polygon always has an exterior ring and may optionally include interior rings. When defining a ring, you must pick either a clockwise or counterclockwise ordering for your points. The first and last point of the polygon must be the same.
    pub geo_polygon: Option<LocationPolygon>,
}

pub fn convert_to_date_time(time_stamp: Option<String>) -> Result<Option<f64>, ServiceError> {
    match time_stamp {
        Some(time_stamp) => Ok(Some(
            time_stamp
                .parse::<DateTimeUtc>()
                .map_err(|_| ServiceError::BadRequest("Invalid timestamp format".to_string()))?
                .0
                .with_timezone(&chrono::Local)
                .naive_local()
                .timestamp() as f64,
        )),
        None => Ok(None),
    }
}

fn get_date_range(date_range: DateRange) -> Result<qdrant::Range, ServiceError> {
    // Based on the determined type, process the values
    let gt = convert_to_date_time(date_range.gt)?;
    let gte = convert_to_date_time(date_range.gte)?;
    let lt = convert_to_date_time(date_range.lt)?;
    let lte = convert_to_date_time(date_range.lte)?;

    Ok(qdrant::Range { gt, gte, lt, lte })
}

pub fn get_range(range: Range) -> Result<qdrant::Range, ServiceError> {
    fn convert_range(range: Option<RangeCondition>) -> Result<Option<f64>, ServiceError> {
        match range {
            Some(RangeCondition::Float(val)) => Ok(Some(val)),
            Some(RangeCondition::Int(val)) => Ok(Some(val as f64)),
            None => Ok(None),
        }
    }

    // Based on the determined type, process the values

    let gt = convert_range(range.gt)?;
    let gte = convert_range(range.gte)?;
    let lt = convert_range(range.lt)?;
    let lte = convert_range(range.lte)?;

    Ok(qdrant::Range { gt, gte, lt, lte })
}

impl FieldCondition {
    pub async fn convert_to_qdrant_condition(
        &self,
        condition_type: &str,
        jsonb_prefilter: Option<bool>,
        dataset_id: uuid::Uuid,
        pool: web::Data<Pool>,
    ) -> Result<Option<qdrant::Condition>, ServiceError> {
        if self.r#match.is_some() && self.range.is_some() {
            return Err(ServiceError::BadRequest(
                "Cannot have both match and range conditions".to_string(),
            ));
        }

        if jsonb_prefilter.unwrap_or(true) && self.field.starts_with("metadata.") {
            return Ok(Some(
                get_metadata_filter_condition(self, dataset_id, pool)
                    .await?
                    .into(),
            ));
        }

        if self.field.starts_with("group_metadata.") {
            return Ok(Some(
                get_group_metadata_filter_condition(self, dataset_id, pool)
                    .await?
                    .into(),
            ));
        }

        if self.field == "group_tag_set" {
            return Ok(Some(
                get_group_tag_set_filter_condition(self, dataset_id, pool)
                    .await?
                    .into(),
            ));
        }

        if let Some(date_range) = self.date_range.clone() {
            let time_range = get_date_range(date_range)?;
            return Ok(Some(qdrant::Condition::range(
                self.field.as_str(),
                time_range,
            )));
        }

        if let Some(range) = self.range.clone() {
            let range = get_range(range)?;
            return Ok(Some(qdrant::Condition::range(self.field.as_str(), range)));
        };

        if let Some(geo_bounding_box) = self.geo_bounding_box.clone() {
            let top_left = geo_bounding_box.top_left;
            let bottom_right = geo_bounding_box.bottom_right;

            return Ok(Some(qdrant::Condition::geo_bounding_box(
                self.field.as_str(),
                GeoBoundingBox {
                    top_left: Some(GeoPoint {
                        lat: top_left.lat.into(),
                        lon: top_left.lon.into(),
                    }),
                    bottom_right: Some(GeoPoint {
                        lat: bottom_right.lat.into(),
                        lon: bottom_right.lon.into(),
                    }),
                },
            )));
        }

        if let Some(geo_radius) = self.geo_radius.clone() {
            let center = geo_radius.center;
            let radius = geo_radius.radius;
            return Ok(Some(qdrant::Condition::geo_radius(
                self.field.as_str(),
                GeoRadius {
                    center: Some(GeoPoint {
                        lat: center.lat.into(),
                        lon: center.lon.into(),
                    }),
                    radius: radius as f32,
                },
            )));
        }

        if let Some(geo_polygon) = self.geo_polygon.clone() {
            let exterior = geo_polygon.exterior;
            let interior = geo_polygon.interior;
            let exterior = exterior
                .iter()
                .map(|point| GeoPoint {
                    lat: point.lat.into(),
                    lon: point.lon.into(),
                })
                .collect();

            let interior = interior
                .map(|interior| {
                    interior
                        .iter()
                        .map(|points| {
                            let points = points
                                .iter()
                                .map(|point| GeoPoint {
                                    lat: point.lat.into(),
                                    lon: point.lon.into(),
                                })
                                .collect();
                            GeoLineString { points }
                        })
                        .collect()
                })
                .unwrap_or_default();

            return Ok(Some(qdrant::Condition::geo_polygon(
                self.field.as_str(),
                GeoPolygon {
                    exterior: Some(GeoLineString { points: exterior }),
                    interiors: interior,
                },
            )));
        }

        let matches = match self.r#match.clone() {
            Some(matches) => matches,
            // Return nothing, there isn't a
            None => return Ok(None),
        };

        match matches.first().ok_or(ServiceError::BadRequest(
            "Should have at least one value for match".to_string(),
        ))? {
            MatchCondition::Text(_) => match condition_type {
                "must" | "should" => Ok(Some(qdrant::Condition::matches(
                    self.field.as_str(),
                    matches.iter().map(|x| x.to_string()).collect_vec(),
                ))),
                "must_not" => Ok(Some(
                    qdrant::Filter::must(
                        matches
                            .into_iter()
                            .map(|cond| {
                                qdrant::Condition::matches(
                                    self.field.as_str(),
                                    vec![cond.to_string()],
                                )
                            })
                            .collect_vec(),
                    )
                    .into(),
                )),
                _ => Err(ServiceError::BadRequest(
                    "Invalid condition type".to_string(),
                )),
            },
            MatchCondition::Integer(_) | MatchCondition::Float(_) => match condition_type {
                "must" | "should" => Ok(Some(qdrant::Condition::matches(
                    self.field.as_str(),
                    matches
                        .iter()
                        .map(|x: &MatchCondition| x.to_i64())
                        .collect_vec(),
                ))),
                "must_not" => Ok(Some(
                    qdrant::Filter::must(
                        matches
                            .into_iter()
                            .map(|cond| {
                                qdrant::Condition::matches(self.field.as_str(), vec![cond.to_i64()])
                            })
                            .collect_vec(),
                    )
                    .into(),
                )),
                _ => Err(ServiceError::BadRequest(
                    "Invalid condition type".to_string(),
                )),
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SearchQueryEvent {
    pub id: uuid::Uuid,
    pub search_type: String,
    pub query: String,
    pub request_params: String,
    pub latency: f32,
    pub top_score: f32,
    pub results: Vec<SearchResultType>,
    pub dataset_id: uuid::Uuid,
    pub created_at: String,
}

impl Default for SearchQueryEvent {
    fn default() -> Self {
        SearchQueryEvent {
            id: uuid::Uuid::new_v4(),
            search_type: "search".to_string(),
            query: "".to_string(),
            request_params: "".to_string(),
            latency: 0.0,
            top_score: 0.0,
            results: vec![],
            dataset_id: uuid::Uuid::new_v4(),
            created_at: chrono::Utc::now().to_string(),
        }
    }
}

#[derive(Debug, Row, Serialize, Deserialize, ToSchema, Clone)]
pub struct SearchQueryEventClickhouse {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: uuid::Uuid,
    pub search_type: String,
    pub query: String,
    pub request_params: String,
    pub latency: f32,
    pub top_score: f32,
    pub results: Vec<String>,
    #[serde(with = "clickhouse::serde::uuid")]
    pub dataset_id: uuid::Uuid,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum SearchResultType {
    Search(ScoreChunkDTO),
    GroupSearch(GroupScoreChunk),
}

impl SearchQueryEventClickhouse {
    pub async fn from_clickhouse(self, pool: web::Data<Pool>) -> SearchQueryEvent {
        if let Ok(chunk_results) = self
            .results
            .iter()
            .map(|r| serde_json::from_str::<CHSlimResponse>(r))
            .collect::<Result<Vec<CHSlimResponse>, _>>()
        {
            let chunk_ids = chunk_results
                .iter()
                .map(|r| r.id)
                .collect::<Vec<uuid::Uuid>>();

            let chunks = get_metadata_from_ids_query(chunk_ids.clone(), self.dataset_id, pool)
                .await
                .unwrap_or(vec![]);

            let results = chunk_results
                .iter()
                .map(|r| {
                    let default = ChunkMetadata::default();
                    let chunk = chunks.iter().find(|c| c.id == r.id).unwrap_or(&default);
                    SearchResultType::Search(ScoreChunkDTO {
                        score: r.score,
                        highlights: None,
                        metadata: vec![ChunkMetadataTypes::Metadata(chunk.clone().into())],
                    })
                })
                .collect::<Vec<SearchResultType>>();

            SearchQueryEvent {
                id: uuid::Uuid::from_bytes(*self.id.as_bytes()),
                search_type: self.search_type,
                query: self.query,
                request_params: self.request_params,
                latency: self.latency,
                top_score: self.top_score,
                results,
                dataset_id: uuid::Uuid::from_bytes(*self.dataset_id.as_bytes()),
                created_at: self.created_at.to_string(),
            }
        } else if let Ok(group_results) = self
            .results
            .iter()
            .map(|r| serde_json::from_str::<CHSlimResponseGroup>(r))
            .collect::<Result<Vec<CHSlimResponseGroup>, _>>()
        {
            let chunk_ids = group_results
                .iter()
                .flat_map(|groups| {
                    groups
                        .chunks
                        .iter()
                        .map(|r| r.id)
                        .collect::<Vec<uuid::Uuid>>()
                })
                .collect::<Vec<uuid::Uuid>>();

            let chunks = get_metadata_from_ids_query(chunk_ids.clone(), self.dataset_id, pool)
                .await
                .unwrap_or(vec![]);

            let results = group_results
                .iter()
                .map(|group| {
                    let group_chunks = group
                        .chunks
                        .iter()
                        .map(|r| {
                            let default = ChunkMetadata::default();
                            let chunk = chunks.iter().find(|c| c.id == r.id).unwrap_or(&default);
                            ScoreChunkDTO {
                                score: r.score,
                                highlights: None,
                                metadata: vec![ChunkMetadataTypes::Metadata(chunk.clone().into())],
                            }
                        })
                        .collect::<Vec<ScoreChunkDTO>>();

                    SearchResultType::GroupSearch(GroupScoreChunk {
                        group_id: group.group_id,
                        metadata: group_chunks,
                        ..Default::default()
                    })
                })
                .collect::<Vec<SearchResultType>>();

            SearchQueryEvent {
                id: uuid::Uuid::from_bytes(*self.id.as_bytes()),
                search_type: self.search_type,
                query: self.query,
                request_params: self.request_params,
                latency: self.latency,
                top_score: self.top_score,
                results,
                dataset_id: uuid::Uuid::from_bytes(*self.dataset_id.as_bytes()),
                created_at: self.created_at.to_string(),
            }
        } else {
            SearchQueryEvent::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RagQueryEvent {
    pub id: uuid::Uuid,
    pub rag_type: String,
    pub user_message: String,
    pub search_id: uuid::Uuid,
    pub results: Vec<ChunkMetadataStringTagSet>,
    pub dataset_id: uuid::Uuid,
    pub created_at: String,
}

impl RagQueryEventClickhouse {
    pub async fn from_clickhouse(self, pool: web::Data<Pool>) -> RagQueryEvent {
        let chunk_ids = self
            .results
            .into_iter()
            .map(|r| r.parse::<uuid::Uuid>().unwrap_or_default())
            .collect::<Vec<uuid::Uuid>>();

        let chunks = get_metadata_from_ids_query(chunk_ids, self.dataset_id, pool)
            .await
            .unwrap_or(vec![]);

        let chunk_string_tag_sets = chunks
            .into_iter()
            .map(ChunkMetadataStringTagSet::from)
            .collect::<Vec<ChunkMetadataStringTagSet>>();

        RagQueryEvent {
            id: uuid::Uuid::from_bytes(*self.id.as_bytes()),
            rag_type: self.rag_type,
            user_message: self.user_message,
            search_id: uuid::Uuid::from_bytes(*self.search_id.as_bytes()),
            results: chunk_string_tag_sets,
            dataset_id: uuid::Uuid::from_bytes(*self.dataset_id.as_bytes()),
            created_at: self.created_at.to_string(),
        }
    }
}

#[derive(Debug, Row, Serialize, Deserialize, ToSchema)]
pub struct RagQueryEventClickhouse {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: uuid::Uuid,
    pub rag_type: String,
    pub user_message: String,
    #[serde(with = "clickhouse::serde::uuid")]
    pub search_id: uuid::Uuid,
    pub results: Vec<String>,
    pub llm_response: String,
    #[serde(with = "clickhouse::serde::uuid")]
    pub dataset_id: uuid::Uuid,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Row, Serialize, Deserialize, ToSchema)]
pub struct ClusterTopicsClickhouse {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: uuid::Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub dataset_id: uuid::Uuid,
    pub topic: String,
    pub density: i32,
    pub avg_score: f32,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
}

impl From<ClusterTopicsClickhouse> for SearchClusterTopics {
    fn from(cluster_topic: ClusterTopicsClickhouse) -> Self {
        SearchClusterTopics {
            id: uuid::Uuid::from_bytes(*cluster_topic.id.as_bytes()),
            dataset_id: uuid::Uuid::from_bytes(*cluster_topic.dataset_id.as_bytes()),
            topic: cluster_topic.topic,
            density: cluster_topic.density,
            avg_score: cluster_topic.avg_score,
            created_at: cluster_topic.created_at.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Row)]
pub struct RecommendationEventClickhouse {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: uuid::Uuid,
    pub recommendation_type: String,
    pub positive_ids: Vec<String>,
    pub negative_ids: Vec<String>,
    pub positive_tracking_ids: Vec<String>,
    pub negative_tracking_ids: Vec<String>,
    pub request_params: String,
    pub results: Vec<String>,
    pub top_score: f32,
    #[serde(with = "clickhouse::serde::uuid")]
    pub dataset_id: uuid::Uuid,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Default)]
pub struct RecommendationEvent {
    pub id: uuid::Uuid,
    pub recommendation_type: String,
    pub positive_ids: Vec<uuid::Uuid>,
    pub negative_ids: Vec<uuid::Uuid>,
    pub positive_tracking_ids: Vec<String>,
    pub negative_tracking_ids: Vec<String>,
    pub request_params: String,
    pub results: Vec<SearchResultType>,
    pub top_score: f32,
    pub dataset_id: uuid::Uuid,
    pub created_at: String,
}

impl RecommendationEventClickhouse {
    pub async fn from_clickhouse(self, pool: web::Data<Pool>) -> RecommendationEvent {
        if let Ok(chunk_results) = self
            .results
            .iter()
            .map(|r| serde_json::from_str::<CHSlimResponse>(r))
            .collect::<Result<Vec<CHSlimResponse>, _>>()
        {
            let chunk_ids = chunk_results
                .iter()
                .map(|r| r.id)
                .collect::<Vec<uuid::Uuid>>();

            let chunks = get_metadata_from_ids_query(chunk_ids.clone(), self.dataset_id, pool)
                .await
                .unwrap_or(vec![]);

            let results = chunk_results
                .iter()
                .map(|r| {
                    let default = ChunkMetadata::default();
                    let chunk = chunks.iter().find(|c| c.id == r.id).unwrap_or(&default);
                    SearchResultType::Search(ScoreChunkDTO {
                        score: r.score,
                        highlights: None,
                        metadata: vec![ChunkMetadataTypes::Metadata(chunk.clone().into())],
                    })
                })
                .collect::<Vec<SearchResultType>>();

            RecommendationEvent {
                id: uuid::Uuid::from_bytes(*self.id.as_bytes()),
                recommendation_type: self.recommendation_type,
                positive_ids: self
                    .positive_ids
                    .iter()
                    .map(|id| uuid::Uuid::parse_str(id).unwrap())
                    .collect(),
                negative_ids: self
                    .negative_ids
                    .iter()
                    .map(|id| uuid::Uuid::parse_str(id).unwrap())
                    .collect(),

                positive_tracking_ids: self.positive_tracking_ids.clone(),
                negative_tracking_ids: self.negative_tracking_ids.clone(),
                request_params: self.request_params,
                results,
                top_score: self.top_score,
                dataset_id: uuid::Uuid::from_bytes(*self.dataset_id.as_bytes()),
                created_at: self.created_at.to_string(),
            }
        } else if let Ok(group_results) = self
            .results
            .iter()
            .map(|r| serde_json::from_str::<CHSlimResponseGroup>(r))
            .collect::<Result<Vec<CHSlimResponseGroup>, _>>()
        {
            let chunk_ids = group_results
                .iter()
                .flat_map(|groups| {
                    groups
                        .chunks
                        .iter()
                        .map(|r| r.id)
                        .collect::<Vec<uuid::Uuid>>()
                })
                .collect::<Vec<uuid::Uuid>>();

            let chunks = get_metadata_from_ids_query(chunk_ids.clone(), self.dataset_id, pool)
                .await
                .unwrap_or(vec![]);

            let results = group_results
                .iter()
                .map(|group| {
                    let group_chunks = group
                        .chunks
                        .iter()
                        .map(|r| {
                            let default = ChunkMetadata::default();
                            let chunk = chunks.iter().find(|c| c.id == r.id).unwrap_or(&default);
                            ScoreChunkDTO {
                                score: r.score,
                                highlights: None,
                                metadata: vec![ChunkMetadataTypes::Metadata(chunk.clone().into())],
                            }
                        })
                        .collect::<Vec<ScoreChunkDTO>>();

                    SearchResultType::GroupSearch(GroupScoreChunk {
                        group_id: group.group_id,
                        metadata: group_chunks,
                        ..Default::default()
                    })
                })
                .collect::<Vec<SearchResultType>>();

            RecommendationEvent {
                id: uuid::Uuid::from_bytes(*self.id.as_bytes()),
                recommendation_type: self.recommendation_type,
                positive_ids: self
                    .positive_ids
                    .iter()
                    .map(|id| uuid::Uuid::parse_str(id).unwrap())
                    .collect(),
                negative_ids: self
                    .negative_ids
                    .iter()
                    .map(|id| uuid::Uuid::parse_str(id).unwrap())
                    .collect(),

                positive_tracking_ids: self.positive_tracking_ids.clone(),
                negative_tracking_ids: self.negative_tracking_ids.clone(),
                request_params: self.request_params,
                results,
                top_score: self.top_score,
                dataset_id: uuid::Uuid::from_bytes(*self.dataset_id.as_bytes()),
                created_at: self.created_at.to_string(),
            }
        } else {
            RecommendationEvent::default()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SearchClusterTopics {
    pub id: uuid::Uuid,
    pub dataset_id: uuid::Uuid,
    pub topic: String,
    pub density: i32,
    pub avg_score: f32,
    pub created_at: String,
}

#[derive(Debug, Row, Serialize, Deserialize, ToSchema)]
pub struct SearchClusterMembership {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: uuid::Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub search_query: uuid::Uuid,
    #[serde(with = "clickhouse::serde::uuid")]
    pub cluster_topic: uuid::Uuid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Display, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SearchType {
    #[display(fmt = "search")]
    Search,
    #[display(fmt = "autocomplete")]
    Autocomplete,
    #[display(fmt = "search_over_groups")]
    SearchOverGroups,
    #[display(fmt = "search_within_groups")]
    SearchWithinGroups,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Display, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SearchMethod {
    #[serde(rename = "full_text", alias = "fulltext")]
    #[display(fmt = "fulltext")]
    FullText,
    #[display(fmt = "semantic")]
    Semantic,
    #[display(fmt = "hybrid")]
    Hybrid,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct SearchAnalyticsFilter {
    pub date_range: Option<DateRange>,
    pub search_method: Option<SearchMethod>,
    pub search_type: Option<SearchType>,
}

impl SearchAnalyticsFilter {
    pub fn add_to_query(&self, mut query_string: String) -> String {
        if let Some(date_range) = &self.date_range {
            if let Some(gt) = &date_range.gt {
                query_string.push_str(&format!(" AND created_at > '{}'", gt));
            }
            if let Some(lt) = &date_range.lt {
                query_string.push_str(&format!(" AND created_at < '{}'", lt));
            }
            if let Some(gte) = &date_range.gte {
                query_string.push_str(&format!(" AND created_at >= '{}'", gte));
            }
            if let Some(lte) = &date_range.lte {
                query_string.push_str(&format!(" AND created_at <= '{}'", lte));
            }
        }

        if let Some(search_type) = &self.search_type {
            query_string.push_str(&format!(" AND search_type = '{}'", search_type));
        }
        if let Some(search_method) = &self.search_method {
            query_string.push_str(&format!(
                " AND JSONExtractString(request_params, 'search_type') = '{}'",
                search_method
            ));
        }

        query_string
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Display, Clone)]
#[serde(rename_all = "snake_case")]
pub enum RagTypes {
    #[display(fmt = "chosen_chunks")]
    ChosenChunks,
    #[display(fmt = "all_chunks")]
    AllChunks,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct RAGAnalyticsFilter {
    pub date_range: Option<DateRange>,
    pub rag_type: Option<RagTypes>,
}

impl RAGAnalyticsFilter {
    pub fn add_to_query(&self, mut query_string: String) -> String {
        if let Some(date_range) = &self.date_range {
            if let Some(gt) = &date_range.gt {
                query_string.push_str(&format!(" AND created_at > '{}'", gt));
            }
            if let Some(lt) = &date_range.lt {
                query_string.push_str(&format!(" AND created_at < '{}'", lt));
            }
            if let Some(gte) = &date_range.gte {
                query_string.push_str(&format!(" AND created_at >= '{}'", gte));
            }
            if let Some(lte) = &date_range.lte {
                query_string.push_str(&format!(" AND created_at <= '{}'", lte));
            }
        }

        if let Some(rag_type) = &self.rag_type {
            query_string.push_str(&format!(" AND rag_type = '{}'", rag_type));
        }

        query_string
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ClusterAnalyticsFilter {
    pub date_range: Option<DateRange>,
}

impl ClusterAnalyticsFilter {
    pub fn add_to_query(&self, mut query_string: String) -> String {
        if let Some(date_range) = &self.date_range {
            if let Some(gt) = &date_range.gt {
                query_string.push_str(&format!(" AND created_at > '{}'", gt));
            }
            if let Some(lt) = &date_range.lt {
                query_string.push_str(&format!(" AND created_at < '{}'", lt));
            }
            if let Some(gte) = &date_range.gte {
                query_string.push_str(&format!(" AND created_at >= '{}'", gte));
            }
            if let Some(lte) = &date_range.lte {
                query_string.push_str(&format!(" AND created_at <= '{}'", lte));
            }
        }

        query_string
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, Display)]
pub enum RecommendationType {
    #[display(fmt = "chunk")]
    Chunk,
    #[display(fmt = "group")]
    Group,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct RecommendationAnalyticsFilter {
    pub date_range: Option<DateRange>,
    pub recommendation_type: Option<RecommendationType>,
}

impl RecommendationAnalyticsFilter {
    pub fn add_to_query(&self, mut query_string: String) -> String {
        if let Some(date_range) = &self.date_range {
            if let Some(gt) = &date_range.gt {
                query_string.push_str(&format!(" AND created_at > '{}'", gt));
            }
            if let Some(lt) = &date_range.lt {
                query_string.push_str(&format!(" AND created_at < '{}'", lt));
            }
            if let Some(gte) = &date_range.gte {
                query_string.push_str(&format!(" AND created_at >= '{}'", gte));
            }
            if let Some(lte) = &date_range.lte {
                query_string.push_str(&format!(" AND created_at <= '{}'", lte));
            }
        }

        if let Some(recommendation_type) = &self.recommendation_type {
            query_string.push_str(&format!(
                " AND recommendation_type = '{}'",
                recommendation_type
            ));
        }

        query_string
    }
}

#[derive(Debug, Row, ToSchema, Serialize, Deserialize)]
pub struct DatasetAnalytics {
    pub total_queries: i32,
    pub search_rps: f64,
    pub avg_latency: f64,
    pub p99: f64,
    pub p95: f64,
    pub p50: f64,
}

#[derive(Debug, ToSchema, Row, Serialize, Deserialize)]
pub struct HeadQueries {
    pub query: String,
    pub count: i64,
}

#[derive(Debug, Row, Serialize, Deserialize, ToSchema)]
pub struct SearchRPSGraphClickhouse {
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub time_stamp: OffsetDateTime,
    pub average_rps: f64,
}

#[derive(Debug, Row, Serialize, Deserialize, ToSchema)]
pub struct SearchRPSGraph {
    pub time_stamp: String,
    pub average_rps: f64,
}

impl From<SearchRPSGraphClickhouse> for SearchRPSGraph {
    fn from(graph: SearchRPSGraphClickhouse) -> Self {
        SearchRPSGraph {
            time_stamp: graph.time_stamp.to_string(),
            average_rps: graph.average_rps,
        }
    }
}

#[derive(Debug, Row, Serialize, Deserialize, ToSchema)]
pub struct SearchLatencyGraphClickhouse {
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub time_stamp: OffsetDateTime,
    pub average_latency: f64,
}

#[derive(Debug, Row, Serialize, Deserialize, ToSchema)]
pub struct SearchLatencyGraph {
    pub time_stamp: String,
    pub average_latency: f64,
}

impl From<SearchLatencyGraphClickhouse> for SearchLatencyGraph {
    fn from(graph: SearchLatencyGraphClickhouse) -> Self {
        SearchLatencyGraph {
            time_stamp: graph.time_stamp.to_string(),
            average_latency: graph.average_latency,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ChunkData {
    pub chunk_metadata: ChunkMetadata,
    pub content: String,
    pub group_ids: Option<Vec<uuid::Uuid>>,
    pub upsert_by_tracking_id: bool,
    pub boost_phrase: Option<BoostPhrase>,
    pub distance_phrase: Option<DistancePhrase>,
}

#[derive(Debug, ToSchema, Serialize, Deserialize, Row)]
#[schema(example = json!({
    "search_type": "search",
    "search_count": 8,
}))]
pub struct SearchTypeCount {
    pub search_type: String,
    pub search_method: String,
    pub search_count: i64,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone, ToSchema)]
#[diesel(table_name = stripe_invoices)]
pub struct StripeInvoice {
    pub id: uuid::Uuid,
    pub org_id: uuid::Uuid,
    pub total: i32,
    pub created_at: chrono::NaiveDateTime,
    pub status: String,
    pub hosted_invoice_url: String,
}

impl StripeInvoice {
    pub fn from_details(
        org_id: uuid::Uuid,
        total: i64,
        created_at: chrono::NaiveDateTime,
        status: String,
        url: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            org_id,
            total: total as i32,
            created_at,
            status,
            hosted_invoice_url: url,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Display, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Granularity {
    #[display(fmt = "minute")]
    Minute,
    #[display(fmt = "second")]
    Second,
    #[display(fmt = "hour")]
    Hour,
    #[display(fmt = "day")]
    Day,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Display, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    #[display(fmt = "created_at")]
    CreatedAt,
    #[display(fmt = "latency")]
    Latency,
    #[display(fmt = "top_score")]
    TopScore,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Display, Clone)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[display(fmt = "DESC")]
    Desc,
    #[display(fmt = "ASC")]
    Asc,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum SearchAnalytics {
    #[schema(title = "LatencyGraph")]
    LatencyGraph {
        filter: Option<SearchAnalyticsFilter>,
        granularity: Option<Granularity>,
    },
    #[serde(rename = "rps_graph")]
    #[schema(title = "RPSGraph")]
    RPSGraph {
        filter: Option<SearchAnalyticsFilter>,
        granularity: Option<Granularity>,
    },
    #[schema(title = "SearchMetrics")]
    SearchMetrics {
        filter: Option<SearchAnalyticsFilter>,
    },
    #[schema(title = "HeadQueries")]
    HeadQueries {
        filter: Option<SearchAnalyticsFilter>,
        page: Option<u32>,
    },
    #[schema(title = "LowConfidenceQueries")]
    LowConfidenceQueries {
        filter: Option<SearchAnalyticsFilter>,
        page: Option<u32>,
        threshold: Option<f32>,
    },
    #[schema(title = "NoResultQueries")]
    NoResultQueries {
        filter: Option<SearchAnalyticsFilter>,
        page: Option<u32>,
    },
    #[schema(title = "SearchQueries")]
    SearchQueries {
        filter: Option<SearchAnalyticsFilter>,
        page: Option<u32>,
        sort_by: Option<SortBy>,
        sort_order: Option<SortOrder>,
    },
    #[schema(title = "CountQueries")]
    CountQueries {
        filter: Option<SearchAnalyticsFilter>,
    },
    #[schema(title = "QueryDetails")]
    QueryDetails { search_id: uuid::Uuid },
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum RAGAnalytics {
    #[schema(title = "RAGQueries")]
    #[serde(rename = "rag_queries")]
    RAGQueries {
        filter: Option<RAGAnalyticsFilter>,
        page: Option<u32>,
        sort_by: Option<SortBy>,
        sort_order: Option<SortOrder>,
    },
    #[schema(title = "RAGUsage")]
    #[serde(rename = "rag_usage")]
    RAGUsage { filter: Option<RAGAnalyticsFilter> },
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum RecommendationAnalytics {
    #[schema(title = "LowConfidenceRecommendations")]
    LowConfidenceRecommendations {
        filter: Option<RecommendationAnalyticsFilter>,
        page: Option<u32>,
        threshold: Option<f32>,
    },
    #[schema(title = "RecommendationQueries")]
    RecommendationQueries {
        filter: Option<RecommendationAnalyticsFilter>,
        page: Option<u32>,
        sort_by: Option<SortBy>,
        sort_order: Option<SortOrder>,
    },
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum ClusterAnalytics {
    #[schema(title = "ClusterTopics")]
    ClusterTopics {
        filter: Option<ClusterAnalyticsFilter>,
    },
    #[schema(title = "ClusterQueries")]
    ClusterQueries {
        cluster_id: uuid::Uuid,
        page: Option<u32>,
    },
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Row)]
pub struct RAGUsageResponse {
    pub total_queries: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum SearchAnalyticsResponse {
    #[schema(title = "LatencyGraph")]
    LatencyGraph(LatencyGraphResponse),
    #[schema(title = "RPSGraph")]
    RPSGraph(RPSGraphResponse),
    #[schema(title = "SearchMetrics")]
    SearchMetrics(DatasetAnalytics),
    #[schema(title = "HeadQueries")]
    HeadQueries(HeadQueryResponse),
    #[schema(title = "LowConfidenceQueries")]
    LowConfidenceQueries(SearchQueryResponse),
    #[schema(title = "NoResultQueries")]
    NoResultQueries(SearchQueryResponse),
    #[schema(title = "SearchQueries")]
    SearchQueries(SearchQueryResponse),
    #[schema(title = "CountQueries")]
    CountQueries(QueryCountResponse),
    #[schema(title = "QueryDetails")]
    QueryDetails(SearchQueryEvent),
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum RAGAnalyticsResponse {
    #[schema(title = "RAGQueries")]
    RAGQueries(RagQueryResponse),
    #[schema(title = "RAGUsage")]
    RAGUsage(RAGUsageResponse),
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum ClusterAnalyticsResponse {
    #[schema(title = "ClusterTopics")]
    ClusterTopics(SearchClusterResponse),
    #[schema(title = "ClusterQueries")]
    ClusterQueries(SearchQueryResponse),
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
pub enum RecommendationAnalyticsResponse {
    #[schema(title = "LowConfidenceRecommendations")]
    LowConfidenceRecommendations(RecommendationsEventResponse),
    #[schema(title = "RecommendationQueries")]
    RecommendationQueries(RecommendationsEventResponse),
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Display, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationStrategy {
    AverageVector,
    BestScore,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Display, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecommendType {
    Semantic,
    FullText,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Display, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EventTypeRequest {
    #[display(fmt = "file_uploaded")]
    FileUploaded,
    #[display(fmt = "file_upload_failed")]
    FileUploadFailed,
    #[display(fmt = "chunks_uploaded")]
    ChunksUploaded,
    #[display(fmt = "chunk_action_failed")]
    ChunkActionFailed,
    #[display(fmt = "chunk_updated")]
    ChunkUpdated,
    #[display(fmt = "bulk_chunks_deleted")]
    BulkChunksDeleted,
    #[display(fmt = "dataset_delete_failed")]
    DatasetDeleteFailed,
    #[display(fmt = "qdrant_index_failed")]
    QdrantUploadFailed,
    #[display(fmt = "bulk_chunk_upload_failed")]
    BulkChunkUploadFailed,
    #[display(fmt = "group_chunks_updated")]
    GroupChunksUpdated,
    #[display(fmt = "group_chunks_action_failed")]
    GroupChunksActionFailed,
}
