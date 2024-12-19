use time::OffsetDateTime;

use super::input::get_input_as_bytes_query;
use crate::{
    errors::ServiceError,
    get_env,
    models::{CreateJobRequest, Job, JobStatus},
};
use openai_dive::v1::resources::batch::{BatchCompletionWindow, CreateBatchParametersBuilder};
use openai_dive::v1::resources::file::{FilePurpose, UploadFileParameters};
use openai_dive::v1::resources::shared::FileUpload;
use openai_dive::v1::{api::Client, resources::shared::FileUploadBytes};

fn get_llm_client() -> Client {
    let base_url = get_env!("LLM_BASE_URL", "LLM_BASE_URL should be set").into();

    let llm_api_key: String = get_env!(
        "LLM_API_KEY",
        "LLM_API_KEY for openrouter or self-hosted should be set"
    )
    .into();

    Client {
        headers: None,
        project: None,
        api_key: llm_api_key,
        http_client: reqwest::Client::new(),
        base_url,
        organization: None,
    }
}

pub async fn create_job(
    request: &CreateJobRequest,
    clickhouse_client: &clickhouse::Client,
) -> Result<Job, ServiceError> {
    let input = get_input_as_bytes_query(&request.input_id).await?;

    let parameters = UploadFileParameters {
        file: FileUpload::Bytes(FileUploadBytes::new(input, "input.jsonl")),
        purpose: FilePurpose::Batch,
    };

    let client = get_llm_client();

    let file = client.files().upload(parameters).await.unwrap();

    let parameters = CreateBatchParametersBuilder::default()
        .input_file_id(file.id)
        .endpoint("/v1/chat/completions".to_string())
        .completion_window(BatchCompletionWindow::H24)
        .build()
        .unwrap();

    let result = client.batches().create(parameters).await.unwrap();

    let job = Job {
        id: uuid::Uuid::new_v4().to_string(),
        batch_id: result.id,
        output_id: None,
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        input_id: request.input_id.clone(),
        schema_id: request.schema_id.clone(),
        status: JobStatus::Created,
    };

    let mut inserter = clickhouse_client.insert("jobs").map_err(|err| {
        log::error!("Failed to insert job: {:?}", err);
        ServiceError::InternalServerError("Failed to insert job".to_string())
    })?;

    inserter.write(&job).await.map_err(|err| {
        log::error!("Failed to write job: {:?}", err);
        ServiceError::InternalServerError("Failed to write job".to_string())
    })?;

    inserter.end().await.map_err(|err| {
        log::error!("Failed to end job insert: {:?}", err);
        ServiceError::InternalServerError("Failed to end job insert".to_string())
    })?;

    Ok(job.clone())
}
