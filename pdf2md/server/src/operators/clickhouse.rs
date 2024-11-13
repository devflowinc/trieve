use crate::{
    errors::ServiceError,
    models::{ChunkClickhouse, FileTaskClickhouse, FileTaskStatus, GetTaskResponse},
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
    let query = match status {
        FileTaskStatus::ProcessingFile(pages) => {
            format!(
                "ALTER TABLE file_tasks UPDATE 
                    status = '{status}', 
                    pages = {pages}
                WHERE id = '{task_id}'",
                status = status,
                pages = pages,
                task_id = task_id
            )
        }
        FileTaskStatus::ChunkingFile(pages) => {
            format!(
                "ALTER TABLE file_tasks UPDATE
                    status = '{status}', 
                    pages_processed = pages_processed + {pages}
                WHERE id = '{task_id}'",
                status = status,
                task_id = task_id,
                pages = pages
            )
        }
        _ => {
            format!(
                "ALTER TABLE file_tasks UPDATE status = '{status}' WHERE id = '{task_id}'",
                status = status,
                task_id = task_id
            )
        }
    };

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
    let task: FileTaskClickhouse = clickhouse_client
        .query("SELECT ?fields FROM file_tasks WHERE id = ?")
        .bind(task_id)
        .fetch_one()
        .await
        .map_err(|err| {
            log::error!("Failed to get task {:?}", err);
            ServiceError::BadRequest("Failed to get task".to_string())
        })?;

    Ok(task)
}

pub async fn get_task_pages(
    task: FileTaskClickhouse,
    limit: Option<u32>,
    offset_id: Option<uuid::Uuid>,
    clickhouse_client: &clickhouse::Client,
) -> Result<GetTaskResponse, ServiceError> {
    if FileTaskStatus::from(task.status.clone()) == FileTaskStatus::Completed || task.pages > 0 {
        let limit = limit.unwrap_or(20);

        log::info!("offset id {:?}", offset_id);

        let pages: Vec<ChunkClickhouse> = clickhouse_client
            .query(
                "SELECT ?fields FROM file_chunks WHERE task_id = ? AND id > ? ORDER BY id LIMIT ?",
            )
            .bind(task.id.clone())
            .bind(offset_id.unwrap_or(uuid::Uuid::nil()))
            .bind(limit)
            .fetch_all()
            .await
            .map_err(|err| {
                log::error!("Failed to get pages {:?}", err);
                ServiceError::BadRequest("Failed to get pages".to_string())
            })?;

        return Ok(GetTaskResponse::new_with_pages(task, pages));
    }

    Ok(GetTaskResponse::new(task))
}

