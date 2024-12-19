use time::OffsetDateTime;

use crate::{
    errors::ServiceError,
    models::{CreateInputRequest, CreateInputResponse, Input, InputType},
};

use super::s3::get_aws_bucket;
use reqwest::Client;
use serde_json::to_string;

pub async fn create_input_query(
    request: &CreateInputRequest,
    clickhouse_client: &clickhouse::Client,
) -> Result<CreateInputResponse, ServiceError> {
    let input = Input {
        id: uuid::Uuid::new_v4().to_string(),
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };

    let bucket = get_aws_bucket()?;

    let s3_put_url = if let Some(input_type) = &request.input {
        match &input_type {
            InputType::File(file_url) => {
                let client = Client::new();
                let response = client.get(file_url).send().await.map_err(|e| {
                    log::error!("Failed to download file from URL: {:?}", e);
                    ServiceError::BadRequest("Failed to download file from URL".to_string())
                })?;

                let bytes = response.bytes().await.map_err(|e| {
                    log::error!("Failed to read bytes from response: {:?}", e);
                    ServiceError::BadRequest("Failed to read bytes from response".to_string())
                })?;

                bucket
                    .put_object(input.id.to_string(), &bytes)
                    .await
                    .map_err(|e| {
                        log::error!("Failed to upload file to S3: {:?}", e);
                        ServiceError::InternalServerError("Failed to upload file to S3".to_string())
                    })?;
            }
            InputType::UnstructuredObjects(objects) => {
                let jsonl_data: String = objects
                    .iter()
                    .map(|obj| to_string(obj).unwrap_or_else(|_| "".to_string()))
                    .collect::<Vec<String>>()
                    .join("\n");

                bucket
                    .put_object(input.id.to_string(), jsonl_data.as_bytes())
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

pub async fn get_input_query(
    input_id: &str,
) -> Result<String, ServiceError> {
    let bucket = get_aws_bucket()?;

    let s3_url = bucket
        .presign_get(input_id, 86400, None)
        .await
        .map_err(|e| {
            log::error!("Could not get presigned get url: {:?}", e);
            ServiceError::BadRequest("Could not get presigned get url".to_string())
        })?;

    Ok(s3_url)
}
