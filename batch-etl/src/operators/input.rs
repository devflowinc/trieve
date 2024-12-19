use time::OffsetDateTime;

use crate::{
    errors::ServiceError,
    models::{CreateInputRequest, CreateInputResponse, Input, InputType},
};

use super::s3::{get_aws_bucket, get_presigned_put_url, upload_to_s3};
use reqwest::Client;

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

                let input_jsonl = response.bytes().await.map_err(|e| {
                    log::error!("Failed to read bytes from response: {:?}", e);
                    ServiceError::BadRequest("Failed to read bytes from response".to_string())
                })?;

                upload_to_s3(&bucket, format!("/inputs/{}.jsonl", input.id), &input_jsonl).await?;
            }
            InputType::UnstructuredObjects(objects) => {
                let input_jsonl = objects
                    .iter()
                    .map(|obj| obj.to_string())
                    .collect::<Vec<String>>()
                    .join("\n");

                upload_to_s3(
                    &bucket,
                    format!("/inputs/{}.jsonl", input.id),
                    input_jsonl.as_bytes(),
                )
                .await?;
            }
        }
        None
    } else {
        Some(get_presigned_put_url(&bucket, format!("/inputs/{}.jsonl", input.id), 86400).await?)
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
