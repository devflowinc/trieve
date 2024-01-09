#![allow(clippy::extra_unused_lifetimes)]

use super::schema::*;
use chrono::{DateTime, NaiveDateTime};
use dateparser::DateTimeUtc;
use diesel::{expression::ValidGrouping, r2d2::ConnectionManager, PgConnection};
use openai_dive::v1::resources::chat::{ChatMessage, Role};
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
    pub api_key_hash: Option<String>,
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
            api_key_hash: None,
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
            api_key_hash: None,
            name: name.map(|n| n.into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, ValidGrouping, Clone, ToSchema)]
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
    pub dataset_id: uuid::Uuid,
}

impl Topic {
    pub fn from_details<S: Into<String>, T: Into<uuid::Uuid>>(
        resolution: S,
        user_id: T,
        normal_chat: Option<bool>,
        dataset_id: uuid::Uuid,
    ) -> Self {
        Topic {
            id: uuid::Uuid::new_v4(),
            user_id: user_id.into(),
            resolution: resolution.into(),
            side: false,
            deleted: false,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            normal_chat: normal_chat.unwrap_or(false),
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
            content: Some(message.content),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
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
            content: Some(message.content),
            tool_calls: None,
            name: message.name,
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
    pub author_id: uuid::Uuid,
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
    pub author_id: uuid::Uuid,
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
    pub fn from_details<S: Into<String>, T: Into<uuid::Uuid>>(
        content: S,
        chunk_html: &Option<String>,
        link: &Option<String>,
        tag_set: &Option<String>,
        author_id: T,
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
            author_id: author_id.into(),
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
        author_id: T,
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
            author_id: author_id.into(),
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
pub struct ChunkCollisions {
    pub id: uuid::Uuid,
    pub chunk_id: uuid::Uuid,
    pub collision_qdrant_id: Option<uuid::Uuid>,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl ChunkCollisions {
    pub fn from_details<T: Into<uuid::Uuid>>(chunk_id: T, collision_id: T) -> Self {
        ChunkCollisions {
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
    pub author: Option<UserDTO>,
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
#[diesel(table_name = chunk_collection)]
pub struct ChunkCollection {
    pub id: uuid::Uuid,
    pub author_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub dataset_id: uuid::Uuid,
}

impl ChunkCollection {
    pub fn from_details(
        author_id: uuid::Uuid,
        name: String,
        description: String,
        dataset_id: uuid::Uuid,
    ) -> Self {
        ChunkCollection {
            id: uuid::Uuid::new_v4(),
            author_id,
            name,
            description,
            dataset_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SlimCollection {
    pub id: uuid::Uuid,
    pub name: String,
    pub author_id: uuid::Uuid,
    pub of_current_user: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, Queryable, ToSchema)]
pub struct ChunkCollectionAndFile {
    pub id: uuid::Uuid,
    pub author_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub file_id: Option<uuid::Uuid>,
}

#[derive(Debug, Default, Serialize, Deserialize, Queryable)]
pub struct ChunkCollectionAndFileWithCount {
    pub id: uuid::Uuid,
    pub author_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub file_id: Option<uuid::Uuid>,
    pub collection_count: Option<i32>,
}

impl From<ChunkCollectionAndFileWithCount> for ChunkCollectionAndFile {
    fn from(collection: ChunkCollectionAndFileWithCount) -> Self {
        ChunkCollectionAndFile {
            id: collection.id,
            author_id: collection.author_id,
            name: collection.name,
            description: collection.description,
            created_at: collection.created_at,
            updated_at: collection.updated_at,
            file_id: collection.file_id,
        }
    }
}

#[derive(
    Debug, Default, Serialize, Deserialize, Selectable, Queryable, Insertable, Clone, ToSchema,
)]
#[diesel(table_name = chunk_collection_bookmarks)]
pub struct ChunkCollectionBookmark {
    pub id: uuid::Uuid,
    pub collection_id: uuid::Uuid,
    pub chunk_metadata_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl ChunkCollectionBookmark {
    pub fn from_details(collection_id: uuid::Uuid, chunk_metadata_id: uuid::Uuid) -> Self {
        ChunkCollectionBookmark {
            id: uuid::Uuid::new_v4(),
            collection_id,
            chunk_metadata_id,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Queryable, Insertable, Clone)]
#[diesel(table_name = collections_from_files)]
pub struct FileCollection {
    pub id: uuid::Uuid,
    pub file_id: uuid::Uuid,
    pub collection_id: uuid::Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl FileCollection {
    pub fn from_details(file_id: uuid::Uuid, collection_id: uuid::Uuid) -> Self {
        FileCollection {
            id: uuid::Uuid::new_v4(),
            file_id,
            collection_id,
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
    pub author_id: uuid::Uuid,
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
            author_id: chunk.author_id,
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
            author_id: chunk.author_id,
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
            author_id: chunk.author_id,
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
    pub user_id: uuid::Uuid,
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
        user_id: uuid::Uuid,
        file_name: &str,
        size: i64,
        tag_set: Option<String>,
        metadata: Option<serde_json::Value>,
        link: Option<String>,
        time_stamp: Option<String>,
        dataset_id: uuid::Uuid,
    ) -> Self {
        File {
            id: uuid::Uuid::new_v4(),
            user_id,
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
    pub user_id: uuid::Uuid,
    pub file_name: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub size: i64,
    pub base64url_content: String,
    pub metadata: Option<serde_json::Value>,
    pub link: Option<String>,
}

impl From<File> for FileDTO {
    fn from(file: File) -> Self {
        FileDTO {
            id: file.id,
            user_id: file.user_id,
            file_name: file.file_name,
            created_at: file.created_at,
            updated_at: file.updated_at,
            size: file.size,
            base64url_content: "".to_string(),
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

#[derive(Debug, Serialize, Deserialize, Clone, Queryable, Insertable, Selectable)]
#[diesel(table_name = file_upload_completed_notifications)]
pub struct FileUploadCompletedNotification {
    pub id: uuid::Uuid,
    pub user_uuid: uuid::Uuid,
    pub collection_uuid: uuid::Uuid,
    pub user_read: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl FileUploadCompletedNotification {
    pub fn from_details(user_uuid: uuid::Uuid, collection_uuid: uuid::Uuid) -> Self {
        FileUploadCompletedNotification {
            id: uuid::Uuid::new_v4(),
            user_uuid,
            collection_uuid,
            user_read: false,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct FileUploadCompletedNotificationWithName {
    pub id: uuid::Uuid,
    pub user_uuid: uuid::Uuid,
    pub collection_uuid: uuid::Uuid,
    pub collection_name: Option<String>,
    pub user_read: bool,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl FileUploadCompletedNotificationWithName {
    pub fn from_file_upload_notification(
        notification: FileUploadCompletedNotification,
        collection_name: String,
    ) -> Self {
        FileUploadCompletedNotificationWithName {
            id: notification.id,
            user_uuid: notification.user_uuid,
            collection_uuid: notification.collection_uuid,
            collection_name: Some(collection_name),
            user_read: notification.user_read,
            created_at: notification.created_at,
            updated_at: notification.updated_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, ValidGrouping)]
#[diesel(table_name = user_collection_counts)]
pub struct UserCollectionCount {
    pub id: uuid::Uuid,
    pub user_id: uuid::Uuid,
    pub collection_count: i32,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, ValidGrouping)]
#[diesel(table_name = user_notification_counts)]
pub struct UserNotificationCount {
    pub id: uuid::Uuid,
    pub user_uuid: uuid::Uuid,
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

#[derive(Debug, Serialize, Deserialize, Queryable, Insertable, Selectable, Clone, ToSchema)]
#[diesel(table_name = dataset_usage_counts)]
pub struct DatasetUsageCount {
    pub id: uuid::Uuid,
    pub dataset_id: uuid::Uuid,
    pub chunk_count: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, ToSchema)]
pub struct DatasetAndUsage {
    pub dataset: Dataset,
    pub dataset_usage: DatasetUsageCount,
}

impl DatasetAndUsage {
    pub fn from_components(dataset: Dataset, dataset_usage: DatasetUsageCount) -> Self {
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
                .unwrap_or(&json!(false))
                .as_bool(),
            DOCUMENT_DOWNLOAD_FEATURE: configuration
                .get("DOCUMENT_DOWNLOAD_FEATURE")
                .unwrap_or(&json!(false))
                .as_bool(),
            LLM_BASE_URL: configuration
                .get("LLM_BASE_URL")
                .unwrap_or(&json!("https://api.openai.com".to_string()))
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
                .unwrap_or(&json!(0.95))
                .as_f64()
                .map(|f| f as f32),
            EMBEDDING_SIZE: configuration
                .get("EMBEDDING_SIZE")
                .unwrap_or(&json!(1536))
                .as_u64()
                .map(|u| u as usize),

        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[allow(non_snake_case)]
pub struct ClientDatasetConfiguration {
    pub PUBLIC_CREATE_EVIDENCE_FEATURE: Option<bool>,
    pub PUBLIC_SEARCH_QUERIES: Option<String>,
    pub PUBLIC_FRONTMATTER_VALS: Option<String>,
    pub PUBLIC_LINES_BEFORE_SHOW_MORE: Option<usize>,
    pub PUBLIC_DATE_RANGE_VALUE: Option<String>,
    pub PUBLIC_FILTER_ITEMS: Option<serde_json::Value>,
    pub PUBLIC_SUGGESTED_QUERIES: Option<String>,
    pub PUBLIC_SHOW_GITHUB_STARS: Option<bool>,
    pub PUBLIC_IMAGE_RANGE_START_KEY: Option<String>,
    pub PUBLIC_IMAGE_RANGE_END_KEY: Option<String>,
    pub PUBLIC_DOCUMENT_UPLOAD_FEATURE: Option<bool>,
}

impl ClientDatasetConfiguration {
    pub fn from_json(configuration: serde_json::Value) -> Self {
        let default_config = json!({});
        let configuration = configuration
            .as_object()
            .unwrap_or(default_config.as_object().unwrap());

        ClientDatasetConfiguration {
            PUBLIC_CREATE_EVIDENCE_FEATURE: configuration
                .get("PUBLIC_CREATE_EVIDENCE_FEATURE")
                .unwrap_or(&json!(false))
                .as_bool(),
            PUBLIC_SEARCH_QUERIES: configuration
                .get("PUBLIC_SEARCH_QUERIES")
                .unwrap_or(&json!(""))
                .as_str()
                .map(|s| s.to_string()),
            PUBLIC_FRONTMATTER_VALS: configuration
                .get("PUBLIC_FRONTMATTER_VALS")
                .unwrap_or(&json!(""))
                .as_str()
                .map(|s| s.to_string()),
            PUBLIC_LINES_BEFORE_SHOW_MORE: configuration
                .get("PUBLIC_LINES_BEFORE_SHOW_MORE")
                .unwrap_or(&json!(3))
                .as_u64()
                .map(|u| u as usize),
            PUBLIC_DATE_RANGE_VALUE: configuration
                .get("PUBLIC_DATE_RANGE_VALUE")
                .unwrap_or(&json!(""))
                .as_str()
                .map(|s| s.to_string()),
            PUBLIC_FILTER_ITEMS: configuration
                .get("PUBLIC_FILTER_ITEMS")
                .unwrap_or(&json!([]))
                .as_array()
                .map(|a| serde_json::Value::Array(a.clone())),
            PUBLIC_SUGGESTED_QUERIES: configuration
                .get("PUBLIC_SUGGESTED_QUERIES")
                .unwrap_or(&json!(""))
                .as_str()
                .map(|s| s.to_string()),
            PUBLIC_SHOW_GITHUB_STARS: configuration
                .get("PUBLIC_SHOW_GITHUB_STARS")
                .unwrap_or(&json!(false))
                .as_bool(),
            PUBLIC_IMAGE_RANGE_START_KEY: configuration
                .get("PUBLIC_IMAGE_RANGE_START_KEY")
                .unwrap_or(&json!(""))
                .as_str()
                .map(|s| s.to_string()),
            PUBLIC_IMAGE_RANGE_END_KEY: configuration
                .get("PUBLIC_IMAGE_RANGE_END_KEY")
                .unwrap_or(&json!(""))
                .as_str()
                .map(|s| s.to_string()),
            PUBLIC_DOCUMENT_UPLOAD_FEATURE: configuration
                .get("PUBLIC_DOCUMENT_UPLOAD_FEATURE")
                .unwrap_or(&json!(false))
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
    pub configuration: serde_json::Value,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub registerable: Option<bool>,
}

impl Organization {
    pub fn from_details(name: String, configuration: serde_json::Value) -> Self {
        Organization {
            id: uuid::Uuid::new_v4(),
            name,
            configuration,
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
            registerable: Some(true),
        }
    }

    pub fn from_org_with_plan_sub(org_plan_sub: OrganizationWithSubAndPlan) -> Self {
        Organization {
            id: org_plan_sub.id,
            name: org_plan_sub.name,
            configuration: org_plan_sub.configuration,
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
    pub dataset_id: uuid::Uuid,
    pub used: bool,
    pub expires_at: chrono::NaiveDateTime,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

// any type that implements Into<String> can be used to create Invitation
impl Invitation {
    pub fn from_details(email: String, dataset_id: uuid::Uuid) -> Self {
        Invitation {
            id: uuid::Uuid::new_v4(),
            email,
            dataset_id,
            used: false,
            expires_at: chrono::Utc::now().naive_local() + chrono::Duration::days(3),
            created_at: chrono::Utc::now().naive_local(),
            updated_at: chrono::Utc::now().naive_local(),
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
    pub file_storage: i32,
    pub user_count: i32,
    pub dataset_count: i32,
    pub message_count: i32,
    pub amount: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub name: String,
}

impl StripePlan {
    pub fn from_details(
        stripe_id: String,
        chunk_count: i32,
        file_storage: i32,
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

    pub fn default() -> Self {
        StripePlan {
            id: uuid::Uuid::default(),
            stripe_id: "".to_string(),
            chunk_count: 1000,
            file_storage: 0,
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
    Debug, Serialize, Deserialize, Selectable, Clone, Queryable, Insertable, ValidGrouping,
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
    pub configuration: serde_json::Value,
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
            configuration: organization.configuration,
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
            configuration: self.configuration.clone(),
            registerable: self.registerable,
            created_at: self.created_at,
            updated_at: self.updated_at,
            plan: Some(self.plan.clone().unwrap_or(StripePlan::default())),
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
    pub file_storage: i32,
    pub message_count: i32,
}
