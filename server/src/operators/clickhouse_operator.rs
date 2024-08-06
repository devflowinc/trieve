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

    let mut search_queries_inserter = String::from("INSERT INTO default.search_queries (id, search_type, query, request_params, query_vector, latency, top_score, results, dataset_id, created_at) VALUES");

    let mut rag_queries_inserter =
        clickhouse_client
            .insert("default.rag_queries")
            .map_err(|e| {
                log::error!("Error inserting rag queries: {:?}", e);
                sentry::capture_message("Error inserting rag queries", sentry::Level::Error);
                ServiceError::InternalServerError(format!("Error inserting rag queries: {:?}", e))
            })?;

    let mut recommendations_inserter = clickhouse_client
        .insert("default.recommendations")
        .map_err(|e| {
            log::error!("Error inserting recommendations: {:?}", e);
            sentry::capture_message("Error inserting recommendations", sentry::Level::Error);
            ServiceError::InternalServerError(format!("Error inserting recommendations: {:?}", e))
        })?;

    let mut worker_events_inserter =
        clickhouse_client
            .insert("default.dataset_events")
            .map_err(|e| {
                log::error!("Error inserting recommendations: {:?}", e);
                sentry::capture_message("Error inserting recommendations", sentry::Level::Error);
                ServiceError::InternalServerError(format!(
                    "Error inserting recommendations: {:?}",
                    e
                ))
            })?;

    for event in events {
        match event {
            ClickHouseEvent::SearchQueryEvent(mut event) => {
                event
                    .results
                    .iter_mut()
                    .for_each(|result| *result = result.replace('\'', "''").replace('?', "\\q"));
                search_queries_inserter.push_str(&format!(
                    " ('{}', '{}', '{}', '{}', embed_p('{}'), '{}', '{}', ['{}'], '{}', now()),",
                    event.id,
                    event.search_type,
                    event.query.replace('?', "\\q"),
                    event.request_params.replace('?', "\\q"),
                    event.query.replace('?', "\\q"),
                    event.latency,
                    event.top_score,
                    event.results.join("','"),
                    event.dataset_id
                ));
            }
            ClickHouseEvent::RecommendationEvent(event) => {
                recommendations_inserter.write(&event).await.map_err(|e| {
                    log::error!("Error writing recommendation event: {:?}", e);
                    sentry::capture_message(
                        "Error writing recommendation event",
                        sentry::Level::Error,
                    );
                    ServiceError::InternalServerError(format!(
                        "Error writing recommendation event: {:?}",
                        e
                    ))
                })?;
            }
            ClickHouseEvent::RagQueryEvent(event) => {
                rag_queries_inserter.write(&event).await.map_err(|e| {
                    log::error!("Error writing rag query event: {:?}", e);
                    sentry::capture_message("Error writing rag query event", sentry::Level::Error);
                    ServiceError::InternalServerError(format!(
                        "Error writing rag query event: {:?}",
                        e
                    ))
                })?;
            }
            ClickHouseEvent::WorkerEvent(event) => {
                worker_events_inserter.write(&event).await.map_err(|e| {
                    log::error!("Error writing worker event: {:?}", e);
                    sentry::capture_message("Error writing worker event", sentry::Level::Error);
                    ServiceError::InternalServerError(format!(
                        "Error writing worker event: {:?}",
                        e
                    ))
                })?;
            }
        }
    }

    if search_queries_inserter != *"INSERT INTO default.search_queries (id, search_type, query, request_params, query_vector, latency, top_score, results, dataset_id, created_at) VALUES" {
        clickhouse_client
            .query(&search_queries_inserter[..search_queries_inserter.len() - 1])
            .execute()
            .await
            .map_err(|err| {
                log::error!("Error writing to ClickHouse default.search_queries: {:?}", err);
                sentry::capture_message(
                    &format!("Error writing to ClickHouse default.search_queries: {:?}", err),
                    sentry::Level::Error,
                );
                ServiceError::InternalServerError("Error writing to ClickHouse default.search_queries".to_string())
            })?;
    }

    rag_queries_inserter.end().await.map_err(|e| {
        log::error!("Error ending rag queries inserter: {:?}", e);
        sentry::capture_message("Error ending rag queries inserter", sentry::Level::Error);
        ServiceError::InternalServerError(format!("Error ending rag queries inserter: {:?}", e))
    })?;
    recommendations_inserter.end().await.map_err(|e| {
        log::error!("Error ending recommendations inserter: {:?}", e);
        sentry::capture_message(
            "Error ending recommendations inserter",
            sentry::Level::Error,
        );
        ServiceError::InternalServerError(format!("Error ending recommendations inserter: {:?}", e))
    })?;
    worker_events_inserter.end().await.map_err(|e| {
        log::error!("Error ending worker events inserter: {:?}", e);
        sentry::capture_message("Error ending worker events inserter", sentry::Level::Error);
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
                                sentry::capture_message(
                                    "Error sending events to clickhouse",
                                    sentry::Level::Error,
                                );
                            }
                            events.clear();
                            timer = Instant::now();
                        }
                    }
                    _ = tokio::time::sleep(Duration::from_secs(10)) => {
                        if !events.is_empty() {
                            if let Err(e) = send_to_clickhouse(events.clone(), &clickhouse_client).await {
                                log::error!("Error sending events to clickhouse: {:?}", e);
                                sentry::capture_message(
                                    "Error sending events to clickhouse",
                                    sentry::Level::Error,
                                );
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
        match &self.sender {
            Some(sender) => {
                sender.send(event).await.unwrap();
            }
            None => {
                log::error!("EventQueue sender not initialized");
                sentry::capture_message("EventQueue sender not initialized", sentry::Level::Error);
            }
        }
    }
}
