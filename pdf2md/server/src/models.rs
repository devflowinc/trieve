use derive_more::derive::Display;
use s3::creds::time::OffsetDateTime;
use utoipa::ToSchema;

pub type RedisPool = bb8_redis::bb8::Pool<bb8_redis::RedisConnectionManager>;

pub trait TaskMessage {
    fn increment_attempt(&mut self);
    fn get_attempts(&self) -> u8;
    fn has_remaining_attempts(&self) -> bool {
        self.get_attempts() < 3
    }
    fn get_task_id(&self) -> uuid::Uuid;
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct FileTask {
    pub task_id: uuid::Uuid,
    pub file_name: String,
    pub upload_file_data: UploadFileReqPayload,
    pub attempt_number: u8,
}

impl TaskMessage for FileTask {
    fn increment_attempt(&mut self) {
        self.attempt_number += 1;
    }
    fn get_attempts(&self) -> u8 {
        self.attempt_number
    }
    fn get_task_id(&self) -> uuid::Uuid {
        self.task_id
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ChunkingTask {
    pub task_id: uuid::Uuid,
    pub file_name: String,
    pub page_range: (u32, u32),
    pub attempt_number: u8,
}

impl TaskMessage for ChunkingTask {
    fn increment_attempt(&mut self) {
        self.attempt_number += 1;
    }
    fn get_attempts(&self) -> u8 {
        self.attempt_number
    }
    fn get_task_id(&self) -> uuid::Uuid {
        self.task_id
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, ToSchema)]
pub struct CreateFileTaskResponse {
    pub task_id: uuid::Uuid,
    pub file_name: String,
    pub status: FileTaskStatus,
    pub pos_in_queue: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, ToSchema)]
pub struct UploadFileReqPayload {
    /// Name of the file
    pub file_name: String,
    /// Base64 encoded file. This is the standard base64 encoding.
    pub base64_file: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, clickhouse::Row, Clone)]
pub struct FileTaskClickhouse {
    pub id: String,
    pub file_name: String,
    pub pages: u32,
    pub pages_processed: u32,
    pub status: String,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, clickhouse::Row, Clone)]
pub struct ChunkClickhouse {
    pub id: String,
    pub task_id: String,
    pub content: String,
    pub metadata: String,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ToSchema)]
pub struct Chunk {
    pub id: String,
    pub task_id: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub created_at: String,
}

impl From<ChunkClickhouse> for Chunk {
    fn from(c: ChunkClickhouse) -> Self {
        Self {
            id: c.id,
            task_id: c.task_id,
            content: c.content,
            metadata: serde_json::from_str(&c.metadata).unwrap(),
            created_at: c.created_at.to_string(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct GetTaskRequest {
    pub pagination_token: Option<uuid::Uuid>,
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, ToSchema)]
pub struct GetTaskResponse {
    pub id: String,
    pub file_name: String,
    pub total_document_pages: u32,
    pub pages_processed: u32,
    pub status: String,
    pub created_at: String,
    pub pages: Option<Vec<Chunk>>,
    pub pagination_token: Option<String>,
}

impl GetTaskResponse {
    pub fn new(task: FileTaskClickhouse) -> Self {
        Self {
            id: task.id.clone(),
            file_name: task.file_name.clone(),
            total_document_pages: task.pages,
            pages_processed: task.pages_processed,
            status: task.status,
            created_at: task.created_at.to_string(),
            pagination_token: None,
            pages: None,
        }
    }
    pub fn new_with_pages(task: FileTaskClickhouse, pages: Vec<ChunkClickhouse>) -> Self {
        Self {
            id: task.id.clone(),
            file_name: task.file_name.clone(),
            total_document_pages: task.pages,
            pages_processed: task.pages_processed,
            status: task.status,
            created_at: task.created_at.to_string(),
            pagination_token: pages.last().map(|c| c.id.clone()),
            pages: Some(pages.into_iter().map(Chunk::from).collect()),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Display, Clone, PartialEq, Eq, ToSchema)]
pub enum FileTaskStatus {
    #[display("Created")]
    Created,
    #[display("Processing {_0} pages")]
    ProcessingFile(u32),
    #[display("Processed {_0} pages")]
    ChunkingFile(u32),
    #[display("Completed")]
    Completed,
    #[display("Failed")]
    Failed,
}

impl FileTaskStatus {
    pub fn get_pages_processed(&self) -> Option<u32> {
        match self {
            FileTaskStatus::ChunkingFile(pages) => Some(*pages),
            _ => None,
        }
    }
}

impl From<String> for FileTaskStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Created" => FileTaskStatus::Created,
            "Completed" => FileTaskStatus::Completed,
            "Failed" => FileTaskStatus::Failed,
            _ => {
                // Try to parse processing or pageing status
                if let Some(pages) = s
                    .strip_prefix("Processed ")
                    .and_then(|s| s.strip_suffix(" pages"))
                {
                    if let Ok(pages) = pages.parse::<u32>() {
                        return FileTaskStatus::ChunkingFile(pages);
                    }
                } else if let Some(pages) = s
                    .strip_prefix("Processing ")
                    .and_then(|s| s.strip_suffix(" pages"))
                {
                    if let Ok(pages) = pages.parse::<u32>() {
                        return FileTaskStatus::ProcessingFile(pages);
                    }
                }
                FileTaskStatus::Failed
            }
        }
    }
}
