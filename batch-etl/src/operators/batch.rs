use clickhouse::Client;
use openai_dive::v1::resources::batch::Batch;
use time::OffsetDateTime;

use crate::{errors::ServiceError, models::ClickhouseBatch};

use super::{
    job::get_llm_client,
    s3::{get_aws_bucket, get_signed_url, upload_to_s3},
};

pub async fn get_pending_batches(
    clickhouse_client: &Client,
) -> Result<Vec<(ClickhouseBatch, String, Option<String>)>, ServiceError> {
    let response = clickhouse_client
        .query("SELECT ?fields, j.id, j.webhook_url FROM batches b FINAL JOIN jobs j ON b.job_id = j.id WHERE b.output_id = ''")
        .fetch_all::<(ClickhouseBatch, String, String)>()
        .await
        .map_err(|e| ServiceError::InternalServerError(e.to_string()))?;

    let response = response
        .into_iter()
        .map(|(batch, id, webhook_url)| {
            if webhook_url.is_empty() {
                (batch, id, None)
            } else {
                (batch, id, Some(webhook_url))
            }
        })
        .collect();

    Ok(response)
}

pub async fn update_batch(
    clickhouse_batch: ClickhouseBatch,
    batch: Batch,
    clickhouse_client: &clickhouse::Client,
) -> Result<ClickhouseBatch, ServiceError> {
    let mut clickhouse_batch = clickhouse_batch;

    if format!("{:?}", batch.status) != clickhouse_batch.status {
        clickhouse_batch.status = format!("{:?}", batch.status);
        clickhouse_batch.updated_at = OffsetDateTime::now_utc();
    }

    if let Some(output_file_id) = batch.output_file_id {
        if clickhouse_batch.output_id.is_empty() {
            clickhouse_batch.output_id = output_file_id.clone();
            clickhouse_batch.updated_at = OffsetDateTime::now_utc();
        }
    }

    let mut inserter = clickhouse_client.insert("batches").map_err(|err| {
        log::error!("Failed to insert batch: {:?}", err);
        ServiceError::InternalServerError("Failed to insert batch".to_string())
    })?;

    inserter.write(&clickhouse_batch).await.map_err(|err| {
        log::error!("Failed to write batch: {:?}", err);
        ServiceError::InternalServerError("Failed to write batch".to_string())
    })?;

    inserter.end().await.map_err(|err| {
        log::error!("Failed to end batch insert: {:?}", err);
        ServiceError::InternalServerError("Failed to end batch insert".to_string())
    })?;

    clickhouse_client
        .query("OPTIMIZE TABLE batches FINAL")
        .execute()
        .await
        .map_err(|err| {
            log::error!("Failed to optimize table: {:?}", err);
            ServiceError::InternalServerError("Failed to optimize table".to_string())
        })?;

    Ok(clickhouse_batch)
}

pub async fn get_batch_output(
    clickhouse_client: &Client,
    clickhouse_batch: ClickhouseBatch,
) -> Result<Option<String>, ServiceError> {
    let client = get_llm_client();
    let bucket = get_aws_bucket()?;

    let batch = client
        .batches()
        .retrieve(&clickhouse_batch.batch_id)
        .await
        .map_err(|err| {
            log::error!("Failed to retrieve batch: {:?}", err);
            ServiceError::InternalServerError("Failed to retrieve batch".to_string())
        })?;

    let updated_clickhouse_batch =
        update_batch(clickhouse_batch.clone(), batch, clickhouse_client).await?;

    if !clickhouse_batch.output_id.is_empty() {
        let url = get_signed_url(
            &bucket,
            format!("/outputs/{}.jsonl", clickhouse_batch.output_id),
        )
        .await?;

        Ok(Some(url))
    } else if !updated_clickhouse_batch.output_id.is_empty() {
        let file = client
            .files()
            .retrieve_content(&updated_clickhouse_batch.output_id.clone())
            .await
            .map_err(|err| {
                log::error!("Failed to retrieve file: {:?}", err);
                ServiceError::InternalServerError("Failed to retrieve file".to_string())
            })?;

        upload_to_s3(
            &bucket,
            format!("/outputs/{:?}.jsonl", updated_clickhouse_batch.output_id),
            file.as_bytes(),
        )
        .await?;

        let url = get_signed_url(
            &bucket,
            format!("/outputs/{:?}.jsonl", updated_clickhouse_batch.output_id),
        )
        .await?;

        Ok(Some(url))
    } else {
        Ok(None)
    }
}
