use crate::{
    errors::ServiceError,
    models::{FileTaskClickhouse, FileTaskStatus},
};

pub async fn insert_task(
    task: FileTaskClickhouse,
    clickhouse_client: &clickhouse::Client,
) -> Result<(), ServiceError> {
    let mut task_inserter = clickhouse_client.insert("file_tasks").map_err(|e| {
        log::error!("Error inserting recommendations: {:?}", e);
        ServiceError::InternalServerError(format!("Error inserting task: {:?}", e))
    })?;

    task_inserter.write(&task).await.map_err(|e| {
        log::error!("Error inserting recommendations: {:?}", e);
        ServiceError::InternalServerError(format!("Error inserting task: {:?}", e))
    })?;

    task_inserter.end().await.map_err(|e| {
        log::error!("Error inserting recommendations: {:?}", e);
        ServiceError::InternalServerError(format!("Error inserting task: {:?}", e))
    })?;

    Ok(())
}

pub async fn update_task_status(
    task_id: uuid::Uuid,
    status: FileTaskStatus,
    clickhouse_client: &clickhouse::Client,
) -> Result<(), ServiceError> {
    let query = format!(
        "ALTER TABLE file_tasks UPDATE status = '{status}' WHERE id = '{task_id}'",
        status = status,
        task_id = task_id
    );

    clickhouse_client
        .query(&query)
        .execute()
        .await
        .map_err(|err| {
            log::error!("Failed to update task status {:?}", err);
            ServiceError::BadRequest("Failed to update task status".to_string())
        })?;

    Ok(())
}

pub async fn get_task(
    task_id: uuid::Uuid,
    clickhouse_client: &clickhouse::Client,
) -> Result<FileTaskClickhouse, ServiceError> {
    let query = format!(
        "SELECT * FROM file_tasks WHERE id = '{task_id}'",
        task_id = task_id
    );

    let task = clickhouse_client
        .query(&query)
        .fetch_one()
        .await
        .map_err(|err| {
            log::error!("Failed to get task {:?}", err);
            ServiceError::BadRequest("Failed to get task".to_string())
        })?;

    Ok(task)
}
