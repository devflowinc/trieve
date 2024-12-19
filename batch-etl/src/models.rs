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
    /// id of the input.
    pub id: String,
    /// Input to be used to pass into the job, if none is provided, an S3 url will be returned that you can upload your input to.
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
