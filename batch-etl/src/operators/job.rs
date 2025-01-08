use std::collections::HashMap;

use crate::{
    errors::ServiceError,
    get_env,
    models::{
        ClickhouseBatch, CreateJobMessage, CreateJobRequest, GetJobResponse, Job, OpenAIBatchInput,
        Schema,
    },
};
use broccoli_queue::{error::BroccoliError, queue::BroccoliQueue};
use openai_dive::v1::resources::file::{FilePurpose, UploadFileParameters};
use openai_dive::v1::resources::{
    batch::{BatchCompletionWindow, CreateBatchParametersBuilder},
    chat::{
        ChatCompletionParameters, ChatCompletionResponseFormat, ChatMessage, ChatMessageContent,
        JsonSchemaBuilder,
    },
};
use openai_dive::v1::resources::{
    chat::{ChatMessageContentPart, ChatMessageImageContentPart, ImageUrlType},
    shared::FileUpload,
};
use openai_dive::v1::{api::Client, resources::shared::FileUploadBytes};
use serde_json::json;
use time::OffsetDateTime;

use super::{
    batch::{get_batch_output, update_batch},
    s3::{download_from_s3, get_aws_bucket},
    schema::get_schema_query,
};

const ETL_SYSTEM_PROMPT: &str = "You are an ETL engineer working on a data pipeline. You have been given a JSONL file containing unstructured data. Your task is to clean and transform the data into a structured format. Please provide the cleaned and transformed data in JSON format following the provided schema.";

pub fn get_llm_client() -> Client {
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
) -> Result<Vec<Vec<u8>>, BroccoliError> {
    let json_schema: serde_json::Value = serde_json::from_str(&schema.schema).map_err(|err| {
        log::error!("Failed to parse JSON schema: {:?}", err);
        BroccoliError::Job("Failed to parse JSON schema".to_string())
    })?;

    let response_format = ChatCompletionResponseFormat::JsonSchema(
        JsonSchemaBuilder::default()
            .schema(json_schema.clone())
            .strict(true)
            .name(schema.name.clone())
            .build()
            .map_err(|e| {
                log::error!("Failed to build JSON schema: {:?}", e);
                BroccoliError::Job("Failed to build JSON schema".to_string())
            })?,
    );

    let mut buffers = Vec::new();
    let mut buf = Vec::new();
    let mut request_count = 0;

    for (i, obj) in objects.iter().enumerate() {
        let mut messages = vec![
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

        if request.image_options.as_ref().is_some_and(|x| x.use_images) {
            if let Some(serde_json::Value::Array(image_urls)) =
                obj.get(request.image_options.as_ref().unwrap().image_key.as_str())
            {
                let image_messages = image_urls
                    .iter()
                    .take(5)
                    .map(|url| {
                        ChatMessageContentPart::Image(ChatMessageImageContentPart {
                            r#type: "image_url".to_string(),
                            image_url: ImageUrlType {
                                url: url.as_str().unwrap().to_string(),
                                detail: None,
                            },
                        })
                    })
                    .collect::<Vec<ChatMessageContentPart>>();

                let image_message = ChatMessage::User {
                    content: ChatMessageContent::ContentPart(image_messages),
                    name: None,
                };

                messages.push(image_message);
            }
        }

        let body = ChatCompletionParameters {
            model: request
                .model
                .as_ref()
                .unwrap_or(&"gpt-4o-mini".to_string())
                .to_string(),
            messages,
            response_format: Some(response_format.clone()),
            max_tokens: request.max_tokens,
            ..Default::default()
        };

        let body_json: serde_json::Value = serde_json::to_value(&body).unwrap();
        let custom_id = if let Some(custom_id) = request.custom_id.as_ref() {
            if let Some(value) = obj.get(custom_id) {
                value.to_string()
            } else {
                format!("input-{}", i)
            }
        } else {
            format!("input-{}", i)
        };

        let params = OpenAIBatchInput {
            custom_id,
            method: "POST".to_string(),
            url: "/v1/chat/completions".to_string(),
            body: body_json,
        };

        jsonl::write(&mut buf, &params).unwrap();
        request_count += 1;

        if request_count >= 50_000 || buf.len() >= 150 * 1024 * 1024 {
            buffers.push(buf);
            buf = Vec::new();
            request_count = 0;
        }
    }

    if !buf.is_empty() {
        buffers.push(buf);
    }

    Ok(buffers)
}

fn convert_input_to_batch(
    request: &CreateJobRequest,
    input: Vec<u8>,
    schema: Schema,
) -> Result<Vec<Vec<u8>>, BroccoliError> {
    let input_str = String::from_utf8(input).map_err(|err| {
        log::error!("Failed to convert input to string: {:?}", err);
        BroccoliError::Job("Failed to convert input to string".to_string())
    })?;

    let objects: Vec<serde_json::Value> = input_str
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()
        .map_err(|err| {
            log::error!("Failed to parse JSONL: {:?}", err);
            BroccoliError::Job("Failed to parse JSONL".to_string())
        })?;

    let input_jsonl = objects_to_jsonl(&objects, schema, request)?;

    Ok(input_jsonl)
}

pub async fn create_openai_job_query(
    message: CreateJobMessage,
    clickhouse_client: &clickhouse::Client,
) -> Result<(), BroccoliError> {
    let request = message.payload.clone();
    let bucket = get_aws_bucket().map_err(|err| {
        log::error!("Failed to get AWS bucket: {:?}", err);
        BroccoliError::Job("Failed to get AWS bucket".to_string())
    })?;
    let input = download_from_s3(&bucket, format!("/inputs/{}.jsonl", request.input_id))
        .await
        .map_err(|err| {
            log::error!("Failed to download input: {:?}", err);
            BroccoliError::Job("Failed to download input".to_string())
        })?;
    let schema = get_schema_query(&request.schema_id, clickhouse_client)
        .await
        .map_err(|err| {
            log::error!("Failed to get schema: {:?}", err);
            BroccoliError::Job("Failed to get schema".to_string())
        })?;

    let input_jsonls = convert_input_to_batch(&request, input, schema)?;

    for (i, input) in input_jsonls.into_iter().enumerate() {
        let parameters = UploadFileParameters {
            file: FileUpload::Bytes(FileUploadBytes::new(
                input,
                format!("input-{}-{}.jsonl", message.job_id, i),
            )),
            purpose: FilePurpose::Batch,
        };

        let client = get_llm_client();

        let file = client.files().upload(parameters).await.map_err(|err| {
            log::error!("Failed to upload file: {:?}", err);
            BroccoliError::Job(format!("Failed to upload file: {:?}", err))
        })?;

        let parameters = CreateBatchParametersBuilder::default()
            .input_file_id(file.id)
            .endpoint("/v1/chat/completions".to_string())
            .completion_window(BatchCompletionWindow::H24)
            .build()
            .map_err(|err| {
                log::error!("Failed to build batch parameters: {:?}", err);
                BroccoliError::Job(format!("Failed to build batch parameters: {:?}", err))
            })?;

        let result = client.batches().create(parameters).await.map_err(|err| {
            log::error!("Failed to create batch: {:?}", err);
            BroccoliError::Job(format!("Failed to create batch: {:?}", err))
        })?;

        let clickhouse_batch = ClickhouseBatch {
            batch_id: result.id.clone(),
            job_id: message.job_id.clone(),
            output_id: String::new(),
            status: format!("{:?}", result.status),
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        };

        update_batch(clickhouse_batch, result, clickhouse_client)
            .await
            .map_err(|err| {
                log::error!("Failed to update batch: {:?}", err);
                BroccoliError::Job(format!("Failed to update batch: {:?}", err))
            })?;
    }

    Ok(())
}

pub async fn create_job_query(
    request: &CreateJobRequest,
    clickhouse_client: &clickhouse::Client,
    broccoli_queue: &BroccoliQueue,
) -> Result<Job, ServiceError> {
    let job = Job {
        id: request
            .job_id
            .clone()
            .unwrap_or(uuid::Uuid::new_v4().to_string()),
        webhook_url: request.webhook_url.clone().unwrap_or_default(),
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

    let message = CreateJobMessage {
        payload: request.clone(),
        job_id: job.id.clone(),
    };

    broccoli_queue
        .publish("create_job_queue", &message, None)
        .await
        .map_err(|err| {
            log::error!("Failed to publish job message: {:?}", err);
            ServiceError::InternalServerError("Failed to publish job message".to_string())
        })?;

    Ok(job)
}

pub async fn get_job_query(
    job_id: &str,
    clickhouse_client: &clickhouse::Client,
) -> Result<GetJobResponse, ServiceError> {
    let client = get_llm_client();

    let job = clickhouse_client
        .query("SELECT ?fields FROM jobs WHERE id = ?")
        .bind(job_id)
        .fetch_one::<Job>()
        .await
        .map_err(|err| {
            log::error!("Failed to create job query: {:?}", err);
            ServiceError::InternalServerError("Failed to create job query".to_string())
        })?;

    let batches = clickhouse_client
        .query("SELECT ?fields FROM batches WHERE job_id = ?")
        .bind(job_id)
        .fetch_all::<ClickhouseBatch>()
        .await
        .map_err(|err| {
            log::error!("Failed to create job query: {:?}", err);
            ServiceError::InternalServerError("Failed to create job query".to_string())
        })?;

    let mut clickhouse_batches: Vec<ClickhouseBatch> = vec![];

    for clickhouse_batch in batches {
        let batch = client
            .batches()
            .retrieve(&clickhouse_batch.batch_id)
            .await
            .map_err(|err| {
                log::error!("Failed to retrieve batch: {:?}", err);
                ServiceError::InternalServerError("Failed to retrieve batch".to_string())
            })?;

        let clickhouse_batch = update_batch(clickhouse_batch, batch, clickhouse_client).await?;

        clickhouse_batches.push(clickhouse_batch);
    }

    Ok(GetJobResponse {
        job,
        batches: clickhouse_batches,
    })
}

pub async fn cancel_job_query(
    job_id: &str,
    clickhouse_client: &clickhouse::Client,
) -> Result<GetJobResponse, ServiceError> {
    let client = get_llm_client();

    let job = clickhouse_client
        .query("SELECT ?fields FROM jobs WHERE id = ?")
        .bind(job_id)
        .fetch_one::<Job>()
        .await
        .map_err(|err| {
            log::error!("Failed to create job query: {:?}", err);
            ServiceError::InternalServerError("Failed to create job query".to_string())
        })?;

    let clickhouse_batches = clickhouse_client
        .query("SELECT batch_id FROM batches WHERE job_id = ?")
        .bind(job_id)
        .fetch_all::<ClickhouseBatch>()
        .await
        .map_err(|err| {
            log::error!("Failed to create job query: {:?}", err);
            ServiceError::InternalServerError("Failed to create job query".to_string())
        })?;

    for clickhouse_batch in &clickhouse_batches {
        let batch = client
            .batches()
            .cancel(&clickhouse_batch.batch_id)
            .await
            .map_err(|err| {
                log::error!("Failed to cancel batch: {:?}", err);
                ServiceError::InternalServerError("Failed to cancel batch".to_string())
            })?;

        update_batch(clickhouse_batch.clone(), batch, clickhouse_client).await?;
    }

    Ok(GetJobResponse {
        job,
        batches: clickhouse_batches,
    })
}

pub async fn get_job_output_query(
    job_id: &str,
    clickhouse_client: &clickhouse::Client,
) -> Result<HashMap<String, Option<String>>, ServiceError> {
    let clickhouse_batches = clickhouse_client
        .query("SELECT ?fields FROM batches WHERE job_id = ?")
        .bind(job_id)
        .fetch_all::<ClickhouseBatch>()
        .await
        .map_err(|err| {
            log::error!("Failed to create job query: {:?}", err);
            ServiceError::InternalServerError("Failed to create job query".to_string())
        })?;

    let mut output_map = HashMap::new();
    for clickhouse_batch in clickhouse_batches {
        output_map.insert(
            clickhouse_batch.batch_id.clone(),
            get_batch_output(clickhouse_client, clickhouse_batch).await?,
        );
    }

    Ok(output_map)
}

pub async fn send_webhook(url: String, id: String, batch_url: String) -> Result<(), ServiceError> {
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .json(&json!({
            "status": "completed",
            "job_id": id,
            "batch_url": batch_url,
        }))
        .send()
        .await
        .map_err(|err| {
            log::error!("Failed to send webhook: {:?}", err);
            ServiceError::InternalServerError("Failed to send webhook".to_string())
        })?;

    if !response.status().is_success() {
        log::error!("Failed to send webhook: {:?}", response);
        return Err(ServiceError::InternalServerError(
            "Failed to send webhook".to_string(),
        ));
    }

    Ok(())
}
