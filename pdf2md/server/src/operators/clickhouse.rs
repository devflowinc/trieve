use crate::{
    errors::ServiceError,
    models::{ChunkClickhouse, ChunkingTask, FileTaskClickhouse, FileTaskStatus, RedisPool},
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

pub async fn insert_page(
    task: ChunkingTask,
    page: ChunkClickhouse,
    clickhouse_client: &clickhouse::Client,
    redis_pool: &RedisPool,
) -> Result<(), ServiceError> {
    let mut page_inserter = clickhouse_client.insert("file_chunks").map_err(|e| {
        log::error!("Error getting page_inserter: {:?}", e);
        ServiceError::InternalServerError(format!("Error getting page_inserter: {:?}", e))
    })?;

    page_inserter.write(&page).await.map_err(|e| {
        log::error!("Error inserting page: {:?}", e);
        ServiceError::InternalServerError(format!("Error inserting page: {:?}", e))
    })?;

    page_inserter.end().await.map_err(|e| {
        log::error!("Error terminating connection: {:?}", e);
        ServiceError::InternalServerError(format!("Error inserting task: {:?}", e))
    })?;

    let mut redis_conn = redis_pool.get().await.map_err(|e| {
        log::error!("Failed to get redis connection: {:?}", e);
        ServiceError::InternalServerError("Failed to get redis connection".to_string())
    })?;

    let total_pages_processed = redis::cmd("incr")
        .arg(format!("{}:count", task.id))
        .query_async::<u32>(&mut *redis_conn)
        .await
        .map_err(|e| {
            log::error!("Failed to push task to chunks_to_process: {:?}", e);
            ServiceError::InternalServerError(
                "Failed to push task to chunks_to_process".to_string(),
            )
        })?;

    let prev_task = get_task(task.id, clickhouse_client).await?;

    log::info!(
        "total_pages: {} pages processed: {}",
        total_pages_processed,
        prev_task.pages
    );

    if total_pages_processed >= prev_task.pages {
        update_task_status(task.id, FileTaskStatus::Completed, clickhouse_client).await?;
    } else {
        update_task_status(
            task.id,
            FileTaskStatus::ProcessingFile(total_pages_processed),
            clickhouse_client,
        )
        .await?;
    }

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
                    pages_processed = {pages}
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
) -> Result<Vec<ChunkClickhouse>, ServiceError> {
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

        return Ok(pages);
    }

    Ok(vec![])
}
