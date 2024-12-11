use std::time::Duration;
use time::Instant;
use tokio::sync::mpsc;

use crate::{
    data::models::{
        RagQueryEventClickhouse, RecommendationEventClickhouse, SearchQueryEventClickhouse,
        WorkerEventClickhouse,
    },
    errors::ServiceError,
};

#[derive(Debug, Clone)]
pub enum ClickHouseEvent {
    SearchQueryEvent(SearchQueryEventClickhouse),
    RecommendationEvent(RecommendationEventClickhouse),
    RagQueryEvent(RagQueryEventClickhouse),
    WorkerEvent(WorkerEventClickhouse),
}

pub fn get_latency_from_header(header: String) -> f32 {
    header
        .split(", ")
        .filter_map(|s| {
            // Find the "dur=" substring and parse the following value as f32
            s.split(";dur=")
                .nth(1)
                .and_then(|dur| dur.parse::<f32>().ok())
        })
        .sum()
}

pub async fn send_to_clickhouse(
    events: Vec<ClickHouseEvent>,
    clickhouse_client: &clickhouse::Client,
) -> Result<(), ServiceError> {
    if std::env::var("USE_ANALYTICS").unwrap_or("false".to_string()) != "true" {
        return Ok(());
    }

    let mut search_queries_inserter = String::from("INSERT INTO search_queries (id, search_type, query, request_params, query_vector, latency, top_score, results, dataset_id, created_at) VALUES");

    let mut rag_queries_inserter = clickhouse_client.insert("rag_queries").map_err(|e| {
        log::error!("Error inserting rag queries: {:?}", e);
        ServiceError::InternalServerError(format!("Error inserting rag queries: {:?}", e))
    })?;

    let mut recommendations_inserter =
        clickhouse_client.insert("recommendations").map_err(|e| {
            log::error!("Error inserting recommendations: {:?}", e);
            ServiceError::InternalServerError(format!("Error inserting recommendations: {:?}", e))
        })?;

    let mut worker_events_inserter = clickhouse_client.insert("dataset_events").map_err(|e| {
        log::error!("Error inserting recommendations: {:?}", e);
        ServiceError::InternalServerError(format!("Error inserting recommendations: {:?}", e))
    })?;

    for event in events {
        match event {
            ClickHouseEvent::SearchQueryEvent(mut event) => {
                event.results.iter_mut().for_each(|result| {
                    *result = result
                        .replace('\'', "''")
                        .replace('?', "|q")
                        .replace('\n', "")
                });

                search_queries_inserter.push_str(&format!(
                    " ('{}', '{}', '{}', '{}', embed_p('{}'), '{}', '{}', ['{}'], '{}', now()),",
                    event.id,
                    event.search_type,
                    event.query.replace('?', "|q"),
                    event.request_params.replace('?', "|q"),
                    event.query.replace('?', "|q"),
                    event.latency,
                    event.top_score,
                    event.results.join("','"),
                    event.dataset_id
                ));

                if search_queries_inserter.len() > 13000 {
                    clickhouse_client
                        .query(&search_queries_inserter[..search_queries_inserter.len() - 1])
                        .execute()
                        .await
                        .map_err(|err| {
                            log::error!("Error writing to ClickHouse search_queries: {:?}", err);
                            ServiceError::InternalServerError(
                                "Error writing to ClickHouse search_queries".to_string(),
                            )
                        })?;
                    search_queries_inserter = String::from("INSERT INTO search_queries (id, search_type, query, request_params, query_vector, latency, top_score, results, dataset_id, created_at) VALUES");
                }
            }
            ClickHouseEvent::RecommendationEvent(event) => {
                recommendations_inserter.write(&event).await.map_err(|e| {
                    log::error!("Error writing recommendation event: {:?}", e);
                    ServiceError::InternalServerError(format!(
                        "Error writing recommendation event: {:?}",
                        e
                    ))
                })?;
            }
            ClickHouseEvent::RagQueryEvent(event) => {
                rag_queries_inserter.write(&event).await.map_err(|e| {
                    log::error!("Error writing rag query event: {:?}", e);
                    ServiceError::InternalServerError(format!(
                        "Error writing rag query event: {:?}",
                        e
                    ))
                })?;
            }
            ClickHouseEvent::WorkerEvent(event) => {
                worker_events_inserter.write(&event).await.map_err(|e| {
                    log::error!("Error writing worker event: {:?}", e);
                    ServiceError::InternalServerError(format!(
                        "Error writing worker event: {:?}",
                        e
                    ))
                })?;
            }
        }
    }

    if search_queries_inserter != *"INSERT INTO search_queries (id, search_type, query, request_params, query_vector, latency, top_score, results, dataset_id, created_at) VALUES" {
        clickhouse_client
            .query(&search_queries_inserter[..search_queries_inserter.len() - 1])
            .execute()
            .await
            .map_err(|err| {
                log::error!("Error writing to ClickHouse search_queries: {:?}", err);
                ServiceError::InternalServerError("Error writing to ClickHouse search_queries".to_string())
            })?;
    }

    rag_queries_inserter.end().await.map_err(|e| {
        log::error!("Error ending rag queries inserter: {:?}", e);
        ServiceError::InternalServerError(format!("Error ending rag queries inserter: {:?}", e))
    })?;
    recommendations_inserter.end().await.map_err(|e| {
        log::error!("Error ending recommendations inserter: {:?}", e);
        ServiceError::InternalServerError(format!("Error ending recommendations inserter: {:?}", e))
    })?;
    worker_events_inserter.end().await.map_err(|e| {
        log::error!("Error ending worker events inserter: {:?}", e);
        ServiceError::InternalServerError(format!("Error ending worker events inserter: {:?}", e))
    })?;

    Ok(())
}

#[derive(Default, Clone)]
pub struct EventQueue {
    sender: Option<mpsc::Sender<ClickHouseEvent>>,
    clickhouse_client: clickhouse::Client,
}

impl EventQueue {
    pub fn new(clickhouse_client: clickhouse::Client) -> Self {
        Self {
            sender: None,
            clickhouse_client,
        }
    }

    pub fn start_service(&mut self) {
        let clickhouse_client = self.clickhouse_client.clone();
        let (sender, mut reciever) = mpsc::channel(1000);
        self.sender = Some(sender);

        tokio::spawn(async move {
            let mut events = Vec::new();
            let mut timer = Instant::now();
            loop {
                tokio::select! {
                    Some(event) = reciever.recv() => {
                        events.push(event);
                        if Instant::now().0.duration_since(timer.0).as_secs() > 10 || events.len() > 1000 {
                            if let Err(e) = send_to_clickhouse(events.clone(), &clickhouse_client).await {
                                log::error!("Error sending events to clickhouse: {:?}", e);
                            }
                            events.clear();
                            timer = Instant::now();
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_secs(10)) => {
                        if !events.is_empty() {
                            if let Err(e) = send_to_clickhouse(events.clone(), &clickhouse_client).await {
                                log::error!("Error sending events to clickhouse: {:?}", e);
                            }
                            events.clear();
                            timer = Instant::now();
                        }
                    }
                }
            }
        });
    }

    pub async fn send(&self, event: ClickHouseEvent) {
        if let Some(sender) = &self.sender {
            let _ = sender.send(event).await.map_err(|e| {
                log::error!("Error sending event to clickhouse: {:?}", e);
            });
        }
    }
}
