use crate::{
    errors::ServiceError,
    get_env,
    models::{CreateJobRequest, Job, JobStatus, OpenAIBatchInput, Schema},
};
use openai_dive::v1::resources::{batch::Batch, shared::FileUpload};
use openai_dive::v1::resources::{
    batch::BatchStatus,
    file::{FilePurpose, UploadFileParameters},
};
use openai_dive::v1::resources::{
    batch::{BatchCompletionWindow, CreateBatchParametersBuilder},
    chat::{
        ChatCompletionParameters, ChatCompletionResponseFormat, ChatMessage, ChatMessageContent,
        JsonSchemaBuilder,
    },
};
use openai_dive::v1::{api::Client, resources::shared::FileUploadBytes};
use serde_json::to_string;
use time::OffsetDateTime;

use super::{
    s3::{download_from_s3, get_aws_bucket, get_signed_url, upload_to_s3},
    schema::get_schema_query,
};

const ETL_SYSTEM_PROMPT: &str = "You are an ETL engineer working on a data pipeline. You have been given a JSONL file containing unstructured data. Your task is to clean and transform the data into a structured format. Please provide the cleaned and transformed data in JSON format following the provided schema.";

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

fn objects_to_jsonl(
    objects: &[serde_json::Value],
    schema: Schema,
    request: &CreateJobRequest,
) -> Result<Vec<u8>, ServiceError> {
    let response_format = ChatCompletionResponseFormat::JsonSchema(
        JsonSchemaBuilder::default()
            .schema(schema.schema.clone())
            .strict(true)
            .name(schema.name.clone())
            .build()
            .map_err(|e| {
                log::error!("Failed to build JSON schema: {:?}", e);
                ServiceError::InternalServerError("Failed to build JSON schema".to_string())
            })?,
    );

    let input_jsonl = objects
        .iter()
        .enumerate()
        .map(|(i, obj)| {
            let messages = vec![
                ChatMessage::System {
                    content: ChatMessageContent::Text(
                        request
                            .system_prompt
                            .clone()
                            .unwrap_or(ETL_SYSTEM_PROMPT.to_string()),
                    ),
                    name: None,
                },
                ChatMessage::User {
                    content: ChatMessageContent::Text(obj.to_string()),
                    name: None,
                },
            ];

            let body = ChatCompletionParameters {
                model: request
                    .model
                    .as_ref()
                    .unwrap_or(&"gpt-4o-mini".to_string())
                    .to_string(),
                messages,
                response_format: Some(response_format.clone()),
                ..Default::default()
            };

            let body_json: serde_json::Value = serde_json::to_value(&body).unwrap();

            let params = OpenAIBatchInput {
                custom_id: format!("input-{}", i),
                method: "POST".to_string(),
                url: "/v1/chat/completions".to_string(),
                body: body_json,
                max_tokens: request.max_tokens,
            };

            to_string(&params).unwrap()
        })
        .collect::<Vec<String>>()
        .join("\n");

    Ok(input_jsonl.as_bytes().to_owned())
}

fn convert_input_to_batch(
    request: &CreateJobRequest,
    input: Vec<u8>,
    schema: Schema,
) -> Result<Vec<u8>, ServiceError> {
    let input_str = String::from_utf8(input).map_err(|err| {
        log::error!("Failed to convert input to string: {:?}", err);
        ServiceError::InternalServerError("Failed to convert input to string".to_string())
    })?;

    let objects: Vec<serde_json::Value> = input_str
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()
        .map_err(|err| {
            log::error!("Failed to parse JSONL: {:?}", err);
            ServiceError::InternalServerError("Failed to parse JSONL".to_string())
        })?;

    let input_jsonl = objects_to_jsonl(&objects, schema, request)?;

    Ok(input_jsonl)
}

async fn update_job(
    job: Job,
    batch: Batch,
    clickhouse_client: &clickhouse::Client,
) -> Result<Job, ServiceError> {
    let mut job = job;

    if <BatchStatus as std::convert::Into<JobStatus>>::into(batch.status.clone()) != job.status {
        job.status = batch.status.into();
        job.updated_at = OffsetDateTime::now_utc();
    }

    if let Some(output_file_id) = batch.output_file_id {
        if job.output_id.is_none() {
            job.output_id = Some(output_file_id.clone());
            job.updated_at = OffsetDateTime::now_utc();
        }
    }

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

    Ok(job)
}

pub async fn create_job_query(
    request: &CreateJobRequest,
    clickhouse_client: &clickhouse::Client,
) -> Result<Job, ServiceError> {
    let bucket = get_aws_bucket()?;
    let input = download_from_s3(&bucket, format!("/inputs/{}.jsonl", request.input_id)).await?;
    let schema = get_schema_query(&request.schema_id, clickhouse_client).await?;

    let input = convert_input_to_batch(request, input, schema)?;

    let parameters = UploadFileParameters {
        file: FileUpload::Bytes(FileUploadBytes::new(input, "input.jsonl")),
        purpose: FilePurpose::Batch,
    };

    let client = get_llm_client();

    let file = client.files().upload(parameters).await.map_err(|err| {
        log::error!("Failed to upload file: {:?}", err);
        ServiceError::InternalServerError("Failed to upload file".to_string())
    })?;

    let parameters = CreateBatchParametersBuilder::default()
        .input_file_id(file.id)
        .endpoint("/v1/chat/completions".to_string())
        .completion_window(BatchCompletionWindow::H24)
        .build()
        .map_err(|err| {
            log::error!("Failed to build batch parameters: {:?}", err);
            ServiceError::InternalServerError("Failed to build batch parameters".to_string())
        })?;

    let result = client.batches().create(parameters).await.map_err(|err| {
        log::error!("Failed to create batch: {:?}", err);
        ServiceError::InternalServerError("Failed to create batch".to_string())
    })?;

    let job = Job {
        id: uuid::Uuid::new_v4().to_string(),
        batch_id: result.id,
        output_id: None,
        status: BatchStatus::InProgress.into(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
        input_id: request.input_id.clone(),
        schema_id: request.schema_id.clone(),
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

pub async fn get_job_query(
    job_id: &str,
    clickhouse_client: &clickhouse::Client,
) -> Result<Job, ServiceError> {
    let client = get_llm_client();

    let job = clickhouse_client
        .query("SELECT ? FROM jobs WHERE id = ?")
        .bind(job_id)
        .fetch_one::<Job>()
        .await
        .map_err(|err| {
            log::error!("Failed to create job query: {:?}", err);
            ServiceError::InternalServerError("Failed to create job query".to_string())
        })?;

    let batch = client
        .batches()
        .retrieve(&job.batch_id)
        .await
        .map_err(|err| {
            log::error!("Failed to retrieve batch: {:?}", err);
            ServiceError::InternalServerError("Failed to retrieve batch".to_string())
        })?;

    let job = update_job(job, batch, clickhouse_client).await?;

    Ok(job)
}

pub async fn cancel_job_query(
    job_id: &str,
    clickhouse_client: &clickhouse::Client,
) -> Result<Job, ServiceError> {
    let client = get_llm_client();

    let job = clickhouse_client
        .query("SELECT ? FROM jobs WHERE id = ?")
        .bind(job_id)
        .fetch_one::<Job>()
        .await
        .map_err(|err| {
            log::error!("Failed to create job query: {:?}", err);
            ServiceError::InternalServerError("Failed to create job query".to_string())
        })?;

    client
        .batches()
        .cancel(&job.batch_id)
        .await
        .map_err(|err| {
            log::error!("Failed to cancel batch: {:?}", err);
            ServiceError::InternalServerError("Failed to cancel batch".to_string())
        })?;

    let job = Job {
        id: job.id,
        input_id: job.input_id,
        schema_id: job.schema_id,
        status: BatchStatus::Cancelled.into(),
        batch_id: job.batch_id,
        output_id: None,
        created_at: job.created_at,
        updated_at: OffsetDateTime::now_utc(),
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

    Ok(job)
}

pub async fn get_job_output_query(
    job_id: &str,
    clickhouse_client: &clickhouse::Client,
) -> Result<String, ServiceError> {
    let client = get_llm_client();
    let bucket = get_aws_bucket()?;

    let job = clickhouse_client
        .query("SELECT ? FROM jobs WHERE id = ?")
        .bind(job_id)
        .fetch_one::<Job>()
        .await
        .map_err(|err| {
            log::error!("Failed to create job query: {:?}", err);
            ServiceError::InternalServerError("Failed to create job query".to_string())
        })?;

    let batch = client
        .batches()
        .retrieve(&job.batch_id)
        .await
        .map_err(|err| {
            log::error!("Failed to retrieve batch: {:?}", err);
            ServiceError::InternalServerError("Failed to retrieve batch".to_string())
        })?;

    let updated_job = update_job(job.clone(), batch, clickhouse_client).await?;

    if job.output_id.is_some() {
        let url = get_signed_url(&bucket, format!("/outputs/{:?}.jsonl", job.output_id)).await?;
        Ok(url)
    } else {
        let file = client
            .files()
            .retrieve_content(&updated_job.output_id.clone().unwrap())
            .await
            .map_err(|err| {
                log::error!("Failed to retrieve file: {:?}", err);
                ServiceError::InternalServerError("Failed to retrieve file".to_string())
            })?;

        upload_to_s3(
            &bucket,
            format!("/outputs/{:?}.jsonl", updated_job.output_id),
            file.as_bytes(),
        )
        .await?;

        let url = get_signed_url(
            &bucket,
            format!("/outputs/{:?}.jsonl", updated_job.output_id),
        )
        .await?;

        Ok(url)
    }
}
