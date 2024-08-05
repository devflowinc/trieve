use crate::{
    data::models::{
        RagQueryEventClickhouse, RecommendationEventClickhouse, SearchQueryEventClickhouse,
    },
    errors::ServiceError,
};

pub enum ClickHouseEvent {
    SearchQueryEvent(SearchQueryEventClickhouse),
    RecommendationEvent(RecommendationEventClickhouse),
    RagQueryEvent(RagQueryEventClickhouse),
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
    event: ClickHouseEvent,
    clickhouse_client: &clickhouse::Client,
) -> Result<(), ServiceError> {
    if std::env::var("USE_ANALYTICS").unwrap_or("false".to_string()) != "true" {
        return Ok(());
    }

    match event {
        ClickHouseEvent::SearchQueryEvent(event) => {
            clickhouse_client
                    .query(
                        "INSERT INTO default.search_queries (id, search_type, query, request_params, query_vector, latency, top_score, results, dataset_id, created_at) VALUES (?, ?, ?, ?, embed_p(?), ?,?, ?, ?, now())",
                    )
                    .bind(event.id)
                    .bind(&event.search_type)
                    .bind(&event.query)
                    .bind(&event.request_params)
                    .bind(&event.query)
                    .bind(event.latency)
                    .bind(event.top_score)
                    .bind(&event.results)
                    .bind(event.dataset_id)
                    .execute()
                    .await
                    .map_err(|err| {
                        log::error!("Error writing to ClickHouse: {:?}", err);
                        sentry::capture_message(&format!("Error writing to ClickHouse: {:?}", err), sentry::Level::Error);
                        ServiceError::InternalServerError("Error writing to ClickHouse".to_string())
                    })?;
        }

        ClickHouseEvent::RagQueryEvent(event) => {
            clickhouse_client
                    .query(
                        "INSERT INTO default.rag_queries (id, rag_type, user_message, search_id, results, llm_response, dataset_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, now())",
                    )
                    .bind(event.id)
                    .bind(&event.rag_type)
                    .bind(&event.user_message)
                    .bind(event.search_id)
                    .bind(&event.results)
                    .bind(&event.llm_response)
                    .bind(event.dataset_id)
                    .execute()
                    .await
                    .map_err(|err| {
                        log::error!("Error writing to ClickHouse: {:?}", err);
                        sentry::capture_message(&format!("Error writing to ClickHouse: {:?}", err), sentry::Level::Error);
                        ServiceError::InternalServerError("Error writing to ClickHouse".to_string())
                    })?;
        }
        ClickHouseEvent::RecommendationEvent(event) => {
            clickhouse_client
                    .query(
                        "INSERT INTO default.recommendations (id, recommendation_type, positive_ids, negative_ids, results, top_score, dataset_id, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, now())",
                    )
                    .bind(event.id)
                    .bind(&event.recommendation_type)
                    .bind(&event.positive_ids)
                    .bind(&event.negative_ids)
                    .bind(&event.results)
                    .bind(event.top_score)
                    .bind(event.dataset_id)
                    .execute()
                    .await
                    .map_err(|err| {
                        log::error!("Error writing to ClickHouse: {:?}", err);
                        sentry::capture_message(&format!("Error writing to ClickHouse: {:?}", err), sentry::Level::Error);
                        ServiceError::InternalServerError("Error writing to ClickHouse".to_string())
                    })?;
        }
    }

    Ok(())
}
