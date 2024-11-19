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
    pub id: uuid::Uuid,
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
        self.id
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ChunkingTask {
    pub id: uuid::Uuid,
    pub file_name: String,
    pub page_range: (u32, u32),
    pub model_params: ModelParams,
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
        self.id
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, ToSchema)]
pub struct CreateFileTaskResponse {
    pub id: uuid::Uuid,
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
    /// The name of the llm model to use for the task. If not provided, the default model will be used. We support all models from (OpenRouter)[https://openrouter.ai/models]
    pub llm_model: Option<String>,
    /// The API key to use for the llm being used.
    pub llm_api_key: Option<String>,
    /// The System prompt that will be used for the conversion of the file.
    pub system_prompt: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct ModelParams {
    pub llm_model: Option<String>,
    pub llm_api_key: Option<String>,
    pub system_prompt: Option<String>,
}

impl From<UploadFileReqPayload> for ModelParams {
    fn from(payload: UploadFileReqPayload) -> Self {
        Self {
            llm_model: payload.llm_model,
            llm_api_key: payload.llm_api_key,
            system_prompt: payload.system_prompt,
        }
    }
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
    pub file_url: Option<String>,
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
            file_url: None,
            total_document_pages: task.pages,
            pages_processed: task.pages_processed,
            status: task.status,
            created_at: task.created_at.to_string(),
            pagination_token: None,
            pages: None,
        }
    }
    pub fn new_with_pages(
        task: FileTaskClickhouse,
        pages: Vec<ChunkClickhouse>,
        file_url: String,
    ) -> Self {
        Self {
            id: task.id.clone(),
            file_name: task.file_name.clone(),
            file_url: Some(file_url),
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
