use derive_more::derive::Display;
use s3::creds::time::OffsetDateTime;

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

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct CreateFileTaskResponse {
    pub task_id: uuid::Uuid,
    pub status: FileTaskStatus,
    pub pos_in_queue: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub struct UploadFileReqPayload {
    /// Base64 encoded file. This is the standard base64 encoding.
    pub base64_file: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, clickhouse::Row, Clone)]
pub struct FileTaskClickhouse {
    pub id: String,
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

#[derive(Debug, serde::Serialize, serde::Deserialize, Display, Clone)]
pub enum FileTaskStatus {
    #[display("Created")]
    Created,
    #[display("Processing File")]
    ProcessingFile,
    #[display("Chunking File")]
    ChunkingFile,
    #[display("Completed")]
    Completed,
    #[display("Failed")]
    Failed,
}

impl FileTaskStatus {
    pub fn from_string(status: &str) -> Self {
        match status {
            "CREATED" => FileTaskStatus::Created,
            "PROCESSING_FILE" => FileTaskStatus::ProcessingFile,
            "CHUNKING_FILE" => FileTaskStatus::ChunkingFile,
            "COMPLETED" => FileTaskStatus::Completed,
            "FAILED" => FileTaskStatus::Failed,
            _ => FileTaskStatus::Failed,
        }
    }
}
