#![allow(clippy::extra_unused_lifetimes)]

use super::schema::*;
use chrono::{DateTime, NaiveDateTime};
use dateparser::DateTimeUtc;
use diesel::{expression::ValidGrouping, r2d2::ConnectionManager, PgConnection};
use openai_dive::v1::resources::chat::{ChatMessage, ChatMessageContent, Role};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

// type alias to use in multiple places
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone, ToSchema)]
#[diesel(table_name = users)]
pub struct User {
    pub id: uuid::Uuid,
    pub email: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub username: Option<String>,
    pub website: Option<String>,
    pub visible_email: bool,
    pub name: Option<String>,
}

impl User {
    pub fn from_details<S: Into<String>>(email: S, name: Option<S>) -> Self {
        User {
            id: uuid::Uuid::new_v4(),
            email: email.into(),
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            username: None,
            website: None,
            visible_email: true,
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
            username: None,
            website: None,
            visible_email: true,
            name: name.map(|n| n.into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, ValidGrouping, Clone, ToSchema)]
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
    pub weight: f64,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone, ToSchema)]
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
pub struct ChunkMetadataWithFileData {
    pub id: uuid::Uuid,
    pub content: String,
    pub chunk_html: Option<String>,
    pub link: Option<String>,
    pub qdrant_point_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub tag_set: Option<String>,
    pub file_id: Option<uuid::Uuid>,
    pub file_name: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub tracking_id: Option<String>,
    pub time_stamp: Option<NaiveDateTime>,
    pub weight: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct SlimUser {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub email: String,
    pub username: Option<String>,
    pub website: Option<String>,
    pub visible_email: bool,
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
            username: user.username,
            website: user.website,
            visible_email: user.visible_email,
            user_orgs,
            orgs,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UserDTO {
    pub id: uuid::Uuid,
    pub email: Option<String>,
    pub username: Option<String>,
    pub website: Option<String>,
    pub visible_email: bool,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(
    Debug, Default, Serialize, Deserialize, Selectable, Queryable, Insertable, Clone, ToSchema,
)]
#[diesel(table_name = chunk_group)]
pub struct ChunkGroup {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub dataset_id: uuid::Uuid,
}

impl ChunkGroup {
    pub fn from_details(name: String, description: String, dataset_id: uuid::Uuid) -> Self {
        ChunkGroup {
            id: uuid::Uuid::new_v4(),
            name,
            description,
            dataset_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct SlimGroup {
    pub id: uuid::Uuid,
    pub name: String,
    pub dataset_id: uuid::Uuid,
    pub of_current_dataset: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Queryable, ToSchema)]
pub struct ChunkGroupAndFile {
    pub id: uuid::Uuid,
    pub dataset_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub file_id: Option<uuid::Uuid>,
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
        }
    }
}

#[derive(
    Debug, Default, Serialize, Deserialize, Selectable, Queryable, Insertable, Clone, ToSchema,
)]
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

#[derive(Debug, Default, Serialize, Deserialize, Queryable, Insertable, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UserDTOWithChunks {
    pub id: uuid::Uuid,
    pub email: Option<String>,
    pub username: Option<String>,
    pub website: Option<String>,
    pub visible_email: bool,
    pub created_at: chrono::NaiveDateTime,
    pub total_chunks_created: i64,
    pub chunks: Vec<ChunkMetadataWithFileData>,
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
    pub score: Option<f64>,
    pub count: i64,
    pub weight: f64,
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
            score: None,
            metadata: chunk.metadata,
            tracking_id: chunk.tracking_id,
            time_stamp: chunk.time_stamp,
            count: 0,
            weight: chunk.weight,
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
            score: None,
            tracking_id: chunk.tracking_id.clone(),
            time_stamp: chunk.time_stamp,
            metadata: chunk.metadata.clone(),
            count: 0,
            weight: chunk.weight,
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
            score: None,
            metadata: chunk.metadata,
            tracking_id: chunk.tracking_id,
            time_stamp: chunk.time_stamp,
            count: chunk.count,
            weight: chunk.weight,
        }
    }
}

#[derive(
    Debug, Default, Serialize, Deserialize, Selectable, Queryable, Insertable, Clone, ToSchema,
)]
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

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
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

#[derive(Debug, Default, Serialize, Deserialize, Queryable)]
pub struct ChunkFileWithName {
    pub chunk_id: uuid::Uuid,
    pub file_id: uuid::Uuid,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Selectable, ToSchema)]
#[diesel(table_name = events)]
pub struct Event {
    pub id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub dataset_id: uuid::Uuid,
    pub event_type: String,
    pub event_data: serde_json::Value,
}

#[derive(Debug, Deserialize, Clone, ToSchema)]
pub enum EventType {
    FileUploaded {
        group_id: uuid::Uuid,
        file_name: String,
    },
    CardUploaded {
        chunk_id: uuid::Uuid,
    },
    CardUploadFailed {
        chunk_id: uuid::Uuid,
        error: String,
    },
}

impl EventType {
    pub fn as_str(&self) -> String {
        match self {
            EventType::FileUploaded { .. } => "file_uploaded".to_string(),
            EventType::CardUploaded { .. } => "card_uploaded".to_string(),
            EventType::CardUploadFailed { .. } => "card_upload_failed".to_string(),
        }
    }
}

impl From<EventType> for serde_json::Value {
    fn from(val: EventType) -> Self {
        match val {
            EventType::FileUploaded {
                group_id,
                file_name,
            } => {
                json!({"group_id": group_id, "file_name": file_name})
            }
            EventType::CardUploaded { chunk_id } => json!({"chunk_id": chunk_id}),
            EventType::CardUploadFailed { chunk_id, error } => {
                json!({"chunk_id": chunk_id, "error": error})
            }
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
#[diesel(table_name = datasets)]
pub struct Dataset {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub organization_id: uuid::Uuid,
    pub server_configuration: serde_json::Value,
    pub client_configuration: serde_json::Value,
}

impl Dataset {
    pub fn from_details(
        name: String,
        organization_id: uuid::Uuid,
        server_configuration: serde_json::Value,
        client_configuration: serde_json::Value,
    ) -> Self {
        Dataset {
            id: uuid::Uuid::new_v4(),
            name,
            organization_id,
            server_configuration,
            client_configuration,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Clone, ToSchema)]
pub struct DatasetDTO {
    pub id: uuid::Uuid,
    pub name: String,
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
            organization_id: dataset.organization_id,
            client_configuration: dataset.client_configuration,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone, ToSchema)]
#[diesel(table_name = dataset_usage_counts)]
pub struct DatasetUsageCount {
    pub id: uuid::Uuid,
    pub dataset_id: uuid::Uuid,
    pub chunk_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
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
#[allow(non_snake_case)]
pub struct ServerDatasetConfiguration {
    pub DOCUMENT_UPLOAD_FEATURE: Option<bool>,
    pub DOCUMENT_DOWNLOAD_FEATURE: Option<bool>,
    pub LLM_BASE_URL: Option<String>,
    pub EMBEDDING_BASE_URL: Option<String>,
    pub RAG_PROMPT: Option<String>,
    pub N_RETRIEVALS_TO_INCLUDE: Option<usize>,
    pub DUPLICATE_DISTANCE_THRESHOLD: Option<f32>,
    pub COLLISIONS_ENABLED: Option<bool>,
    pub EMBEDDING_SIZE: Option<usize>,
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
                .as_bool(),
            DOCUMENT_DOWNLOAD_FEATURE: configuration
                .get("DOCUMENT_DOWNLOAD_FEATURE")
                .unwrap_or(&json!(true))
                .as_bool(),
            LLM_BASE_URL: configuration
                .get("LLM_BASE_URL")
                .unwrap_or(&json!("https://openrouter.ai/api/v1".to_string()))
                .as_str()
                .map(|s| s.to_string()),
            EMBEDDING_BASE_URL: configuration
                .get("EMBEDDING_BASE_URL")
                .unwrap_or(&json!("https://api.openai.com/v1".to_string()))
                .as_str()
                .map(|s| s.to_string()),
            RAG_PROMPT: configuration
                .get("RAG_PROMPT")
                .unwrap_or(&json!("Write a 1-2 sentence semantic search query along the lines of a hypothetical response to: \n\n".to_string()))
                .as_str()
                .map(|s| s.to_string()),
            N_RETRIEVALS_TO_INCLUDE: configuration
                .get("N_RETRIEVALS_TO_INCLUDE")
                .unwrap_or(&json!(3))
                .as_u64()
                .map(|u| u as usize),
            DUPLICATE_DISTANCE_THRESHOLD: configuration
                .get("DUPLICATE_DISTANCE_THRESHOLD")
                .unwrap_or(&json!(1.1))
                .as_f64()
                .map(|f| f as f32),
            EMBEDDING_SIZE: configuration
                .get("EMBEDDING_SIZE")
                .unwrap_or(&json!(1536))
                .as_u64()
                .map(|u| u as usize),

            COLLISIONS_ENABLED: configuration
                .get("COLLISIONS_ENABLED")
                .unwrap_or(&json!(false))
                .as_bool(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
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
                .unwrap_or(&json!(3))
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
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
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
        Organization {
            id: org_plan_sub.id,
            name: org_plan_sub.name,
            created_at: org_plan_sub.created_at,
            updated_at: org_plan_sub.updated_at,
            registerable: org_plan_sub.registerable,
        }
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
pub struct OrganizationWithSubAndPlan {
    pub id: uuid::Uuid,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub registerable: Option<bool>,
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
            id: organization.id,
            name: organization.name,
            registerable: organization.registerable,
            created_at: organization.created_at,
            updated_at: organization.updated_at,
            plan,
            subscription,
        }
    }

    pub fn with_defaults(&self) -> Self {
        OrganizationWithSubAndPlan {
            id: self.id,
            name: self.name.clone(),
            registerable: self.registerable,
            created_at: self.created_at,
            updated_at: self.updated_at,
            plan: Some(self.plan.clone().unwrap_or_default()),
            subscription: self.subscription.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, ToSchema, Ord, PartialOrd)]
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
#[diesel(table_name = organization_usage_counts)]
pub struct OrganizationUsageCount {
    pub id: uuid::Uuid,
    pub org_id: uuid::Uuid,
    pub dataset_count: i32,
    pub user_count: i32,
    pub file_storage: i64,
    pub message_count: i32,
}

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
#[diesel(table_name = user_api_key)]
pub struct UserApiKey {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub api_key_hash: String,
    pub name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub role: i32,
}

impl UserApiKey {
    pub fn from_details(
        user_id: uuid::Uuid,
        api_key_hash: String,
        name: String,
        role: ApiKeyRole,
    ) -> Self {
        UserApiKey {
            id: uuid::Uuid::new_v4(),
            user_id,
            api_key_hash,
            name,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            role: role.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
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
