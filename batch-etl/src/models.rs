use clickhouse::Row;
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
    pub schema: String,
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

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
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

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct CreateJobRequest {
    /// id of the input.
    pub input_id: String,
    /// id of the schema.
    pub schema_id: String,
    /// Model to use for the input
    pub model: Option<String>,
    /// Url to call when the job is done
    pub webhook_url: Option<String>,
    /// Image options to use for the input
    pub image_options: Option<ImageOptions>,
    /// Id to use for the job. If none is provided, the system will generate a job id.
    pub job_id: Option<String>,
    /// Which key to use from the input that contains the custom id to use. If none is provided, the system will generate a custom id.
    pub custom_id: Option<String>,
    /// Max tokens to generate
    pub max_tokens: Option<u32>,
    /// Pass your custom prompt to the system. If none is provided, the system will use the default prompt.
    pub system_prompt: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
pub struct ImageOptions {
    /// If true, the API will look for a images_url array in the input and use the images in the array as the input to the model.
    pub use_images: bool,
    /// What key to look for the images in the input
    pub image_key: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Row, Clone)]
pub struct Job {
    /// Unique identifier of the job
    pub id: String,
    /// id of the input.
    pub input_id: String,
    /// id of the schema.
    pub schema_id: String,
    /// Url to call when the job is done
    pub webhook_url: String,
    /// Created at timestamp
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
    /// Updated at timestamp
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Row, Clone)]
pub struct ClickhouseBatch {
    /// Unique identifier of the batch
    pub batch_id: String,
    /// id of the job.
    pub job_id: String,
    /// id of the output.
    pub output_id: String,
    /// Status of the batch
    pub status: String,
    /// Created at timestamp
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
    /// Updated at timestamp
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct OpenAIBatchInput {
    pub custom_id: String,
    pub method: String,
    pub url: String,
    pub body: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateJobMessage {
    pub payload: CreateJobRequest,
    pub job_id: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GetJobResponse {
    /// Overarching Job
    pub job: Job,
    /// Batches of the job
    pub batches: Vec<ClickhouseBatch>,
}
