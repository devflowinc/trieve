use crate::{
    errors::ServiceError,
    models::{FileTaskStatus, TaskMessage},
    operators::clickhouse::update_task_status,
};

#[macro_export]
macro_rules! process_task_with_retry {
    ($redis_conn:expr, &$clickhouse_client:expr, $queue_name:expr, $process_fn:expr, $task_type:ty) => {
        let should_terminate = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(SIGTERM, Arc::clone(&should_terminate))
            .expect("Failed to register shutdown hook");

        loop {
            if should_terminate.load(Ordering::Relaxed) {
                log::info!("Shutting down");
                break;
            }

            let task = listen_to_redis::<$task_type>($redis_conn.clone(), $queue_name).await;

            match task {
                Some(task) => {
                    log::info!("Processing task: {:?}", task.task_id);
                    let result = $process_fn(task.clone()).await;

                    if let Err(err) = result {
                        log::error!("Task processing failed: {:?}", err);

                        // Requeue the failed task
                        if let Err(requeue_err) = pdf2md_server::operators::redis::readd_to_queue(
                            task,
                            err,
                            $queue_name,
                            $redis_conn.clone(),
                            &$clickhouse_client,
                        )
                        .await
                        {
                            log::error!("Failed to requeue task: {:?}", requeue_err);
                        } else {
                            log::info!("Successfully requeued failed task");
                        }
                    }
                }
                None => {
                    // Optional: Add delay or other handling for when no task is available
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        }
    };
}

pub async fn listen_to_redis<T: for<'a> serde::Deserialize<'a>>(
    redis_connection: redis::aio::MultiplexedConnection,
    queue_name: &str,
) -> Option<T> {
    let payload_result: Result<Vec<String>, redis::RedisError> = redis::cmd("brpoplpush")
        .arg(queue_name)
        .arg(format!("{}_processing", queue_name))
        .arg(1.0)
        .query_async(&mut redis_connection.clone())
        .await;

    let serialized_message = if let Ok(payload) = payload_result {
        if payload.is_empty() {
            return None;
        }

        payload
            .first()
            .expect("Payload must have a first element")
            .clone()
    } else {
        log::error!("Unable to process {:?}", payload_result);
        return None;
    };

    let worker_message: T =
        serde_json::from_str(&serialized_message).expect("Failed to parse file message");

    Some(worker_message)
}

pub async fn readd_to_queue<T: for<'a> serde::Serialize + TaskMessage>(
    mut payload: T,
    error: ServiceError,
    queue_name: &str,
    mut redis_connection: redis::aio::MultiplexedConnection,
    clickhouse_client: &clickhouse::Client,
) -> Result<(), ServiceError> {
    let old_payload_message = serde_json::to_string(&payload).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input for retry".to_string())
    })?;

    payload.increment_attempt();

    let _ = redis::cmd("LREM")
        .arg(format!("{}_processing", queue_name))
        .arg(1)
        .arg(old_payload_message.clone())
        .query_async::<String>(&mut redis_connection)
        .await;

    if !payload.has_remaining_attempts() {
        log::error!("Message failed 3 times quitting {:?}", error);

        update_task_status(
            payload.get_task_id(),
            FileTaskStatus::Failed,
            clickhouse_client,
        )
        .await?;

        redis::cmd("lpush")
            .arg(format!("{}_failed", queue_name))
            .arg(old_payload_message)
            .query_async::<String>(&mut redis_connection)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

        return Err(ServiceError::InternalServerError(format!(
            "Message failed 3 times {:?}",
            error
        )));
    }

    let new_payload_message = serde_json::to_string(&payload).map_err(|_| {
        ServiceError::InternalServerError("Failed to reserialize input for retry".to_string())
    })?;

    log::error!(
        "Message failed, re-adding {:?} retry: {:?}",
        error,
        payload.get_attempts()
    );

    redis::cmd("lpush")
        .arg(queue_name)
        .arg(&new_payload_message)
        .query_async::<String>(&mut redis_connection)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    Ok(())
}
