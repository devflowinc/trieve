use openai_dive::v1::resources::chat::{
    ChatCompletionParameters, ChatCompletionResponseFormat, ChatMessage, ChatMessageContent,
    JsonSchemaBuilder,
};
use time::OffsetDateTime;

use crate::{
    errors::ServiceError,
    models::{CreateInputRequest, CreateInputResponse, Input, InputType, Schema},
};

use super::{s3::get_aws_bucket, schema::get_schema_query};
use reqwest::Client;
use serde_json::to_string;

const ETL_SYSTEM_PROMPT: &str = "You are an ETL engineer working on a data pipeline. You have been given a JSONL file containing unstructured data. Your task is to clean and transform the data into a structured format. Please provide the cleaned and transformed data in JSON format following the provided schema.";

pub fn objects_to_jsonl(
    objects: &[serde_json::Value],
    schema: Schema,
    request: &CreateInputRequest,
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
        .map(|obj| {
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

            let params = ChatCompletionParameters {
                model: request
                    .model
                    .as_ref()
                    .unwrap_or(&"gpt-4o-mini".to_string())
                    .to_string(),
                messages,
                response_format: Some(response_format.clone()),
                ..Default::default()
            };

            to_string(&params).unwrap()
        })
        .collect::<Vec<String>>()
        .join("\n");

    Ok(input_jsonl.as_bytes().to_owned())
}

pub async fn create_input_query(
    request: &CreateInputRequest,
    clickhouse_client: &clickhouse::Client,
) -> Result<CreateInputResponse, ServiceError> {
    let input = Input {
        id: uuid::Uuid::new_v4().to_string(),
        schema_id: request.schema_id.clone(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };

    let schema = get_schema_query(&input.schema_id, clickhouse_client).await?;

    let bucket = get_aws_bucket()?;

    let s3_put_url = if let Some(input_type) = &request.input {
        match &input_type {
            InputType::File(file_url) => {
                let client = Client::new();
                let response = client.get(file_url).send().await.map_err(|e| {
                    log::error!("Failed to download file from URL: {:?}", e);
                    ServiceError::BadRequest("Failed to download file from URL".to_string())
                })?;

                let unstructured_objects: Vec<serde_json::Value> = response
                    .json::<String>()
                    .await
                    .map_err(|e| {
                        log::error!("Failed to read bytes from response: {:?}", e);
                        ServiceError::BadRequest("Failed to read bytes from response".to_string())
                    })?
                    .split("\n")
                    .map(|line| serde_json::from_str(line).unwrap())
                    .collect();

                let input_jsonl = objects_to_jsonl(&unstructured_objects, schema, request)?;

                bucket
                    .put_object(format!("/inputs/{}.jsonl", input.id), &input_jsonl)
                    .await
                    .map_err(|e| {
                        log::error!("Failed to upload file to S3: {:?}", e);
                        ServiceError::InternalServerError("Failed to upload file to S3".to_string())
                    })?;
            }
            InputType::UnstructuredObjects(objects) => {
                let input_jsonl = objects_to_jsonl(objects, schema, request)?;

                bucket
                    .put_object(format!("/inputs/{}.jsonl", input.id), &input_jsonl)
                    .await
                    .map_err(|e| {
                        log::error!("Failed to upload JSONL file to S3: {:?}", e);
                        ServiceError::InternalServerError(
                            "Failed to upload JSONL file to S3".to_string(),
                        )
                    })?;
            }
        }
        None
    } else {
        Some(
            bucket
                .presign_put(input.id.to_string(), 86400, None, None)
                .await
                .map_err(|e| {
                    log::error!("Could not get presigned put url: {:?}", e);
                    ServiceError::BadRequest("Could not get presigned put url".to_string())
                })?,
        )
    };

    let mut inserter = clickhouse_client.insert("inputs").map_err(|err| {
        log::error!("Failed to insert input: {:?}", err);
        ServiceError::InternalServerError("Failed to insert input".to_string())
    })?;

    inserter.write(&input).await.map_err(|err| {
        log::error!("Failed to write input: {:?}", err);
        ServiceError::InternalServerError("Failed to write input".to_string())
    })?;

    inserter.end().await.map_err(|err| {
        log::error!("Failed to end input insert: {:?}", err);
        ServiceError::InternalServerError("Failed to end input insert".to_string())
    })?;

    Ok(CreateInputResponse {
        input_id: input.id,
        s3_put_url,
    })
}

pub async fn get_input_query(input_id: &str) -> Result<String, ServiceError> {
    let bucket = get_aws_bucket()?;

    let s3_url = bucket
        .presign_get(format!("/inputs/{}.jsonl", input_id), 86400, None)
        .await
        .map_err(|e| {
            log::error!("Could not get presigned get url: {:?}", e);
            ServiceError::BadRequest("Could not get presigned get url".to_string())
        })?;

    Ok(s3_url)
}

pub async fn get_input_as_bytes_query(input_id: &str) -> Result<Vec<u8>, ServiceError> {
    let bucket = get_aws_bucket()?;

    let bytes = bucket
        .get_object(format!("/inputs/{}.jsonl", input_id))
        .await
        .map_err(|e| {
            log::error!("Could not get object from S3: {:?}", e);
            ServiceError::BadRequest("Could not get object from S3".to_string())
        })?;

    Ok(bytes.to_vec())
}
