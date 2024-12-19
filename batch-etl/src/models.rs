use clickhouse::Row;
use openai_dive::v1::resources::batch::BatchStatus;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateSchemaRequest {
    /// Name of the schema
    pub name: String,
    /// Schema definition in the OpenAI structured outputs format -- https://platform.openai.com/docs/guides/structured-outputs?lang=curl&context=without_parse#how-to-use
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Row)]
pub struct Schema {
    /// Unique identifier of the schema
    pub id: String,
    /// Name of the schema
    pub name: String,
    /// Schema definition in the OpenAI structured outputs format -- https://platform.openai.com/docs/guides/structured-outputs?lang=curl&context=without_parse#how-to-use
    pub schema: serde_json::Value,
    /// Created at timestamp
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
    /// Updated at timestamp
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateInputRequest {
    /// Input to be used to pass into the job, if none is provided, an S3 url will be returned that you can upload your jsonl file to.
    /// The maximum size of the input is 200MB or 50,000 objects
    pub input: Option<InputType>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
/// Create inputs to pass into jobs
pub enum InputType {
    /// Link to the JSONL file of unstructured objects
    File(String),
    /// List of unstructured JSON objects
    /// Make big batches as doing many small batches can be slow to insert
    UnstructuredObjects(Vec<serde_json::Value>),
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Row, Clone)]
pub struct Input {
    /// Unique identifier of the input
    pub id: String,
    /// Created at timestamp
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
    /// Updated at timestamp
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateInputResponse {
    /// id of the input.
    pub input_id: String,
    /// S3 url to upload the input to. This is only returned if no input is provided.
    pub s3_put_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateJobRequest {
    /// id of the input.
    pub input_id: String,
    /// id of the schema.
    pub schema_id: String,
    /// Model to use for the input
    pub model: Option<String>,
    /// Max tokens to generate
    pub max_tokens: Option<u32>,
    /// Pass your custom prompt to the system. If none is provided, the system will use the default prompt.
    pub system_prompt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Row, Clone)]
pub struct Job {
    /// Unique identifier of the job
    pub id: String,
    /// id of the input.
    pub input_id: String,
    /// id of the schema.
    pub schema_id: String,
    /// Status of the job
    pub status: JobStatus,
    /// OpenAI batch job id
    pub batch_id: String,
    /// Output of the job
    pub output_id: Option<String>,
    /// Created at timestamp
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
    /// Updated at timestamp
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    /// The job is being validatated
    Validating,
    /// The job has failed
    Failed,
    /// The job is in progress
    InProgress,
    /// The job is finalizing
    Finalizing,
    /// The job has completed
    Completed,
    /// The job has expired
    Expired,
    /// The job is being cancelled
    Cancelling,
    /// The job has been cancelled
    Cancelled,
}

impl From<BatchStatus> for JobStatus {
    fn from(status: BatchStatus) -> Self {
        match status {
            BatchStatus::Validating => JobStatus::Validating,
            BatchStatus::Failed => JobStatus::Failed,
            BatchStatus::InProgress => JobStatus::InProgress,
            BatchStatus::Finalizing => JobStatus::Finalizing,
            BatchStatus::Completed => JobStatus::Completed,
            BatchStatus::Expired => JobStatus::Expired,
            BatchStatus::Cancelling => JobStatus::Cancelling,
            BatchStatus::Cancelled => JobStatus::Cancelled,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OpenAIBatchInput {
    pub custom_id: String,
    pub method: String,
    pub url: String,
    pub body: serde_json::Value,
    pub max_tokens: Option<u32>,
}
