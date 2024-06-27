use crate::{
    data::models::{ChunkMetadataTypes, SearchQueryEventClickhouse},
    errors::ServiceError,
    handlers::{
        chunk_handler::SearchChunkQueryResponseBody, group_handler::SearchWithinGroupResults,
    },
};

use super::search_operator::SearchOverGroupsResults;

pub enum ClickHouseEvent {
    SearchQueryEvent(SearchQueryEventClickhouse),
    //TODO: Recommended Chunks
    //TODO: Recommeneded Groups
    //TODO: RAG over selected Chunks
    //TODO: RAG over all Chunks
}

pub async fn run_clickhouse_migrations(client: &clickhouse::Client) {
    client
        .query(
            "
            CREATE TABLE IF NOT EXISTS dataset_events (
                id UUID,
                created_at DateTime,
                dataset_id UUID,
                event_type String,
                event_data String
            ) ENGINE = MergeTree()
            ORDER BY (dataset_id, created_at, event_type, id)
            PARTITION BY
                (toYYYYMM(created_at),
                dataset_id)
            TTL created_at + INTERVAL 30 DAY;
        ",
        )
        .execute()
        .await
        .unwrap();

    client
        .query(
            "
        CREATE TABLE IF NOT EXISTS search_queries
        (
            id UUID,
            search_type String,
            query String,
            request_params String,
            latency Float32,
            top_score Float32,
            results Array(String),
            query_vector Array(Float32),
            dataset_id UUID,
            created_at DateTime
        ) ENGINE = MergeTree()
        ORDER BY (dataset_id, created_at, top_score, latency, id)
        PARTITION BY
            (toYYYYMM(created_at),
            dataset_id)
        TTL created_at + INTERVAL 30 DAY
        ",
        )
        .execute()
        .await
        .unwrap();

    client
        .query(
            "
        CREATE TABLE IF NOT EXISTS cluster_topics
        (
            id UUID,
            dataset_id UUID,
            topic String,
            density Int32,
            avg_score Float32,
            created_at DateTime
        ) ENGINE = MergeTree()
        ORDER BY (dataset_id, id)
        PARTITION BY
            dataset_id
        ",
        )
        .execute()
        .await
        .unwrap();

    client
        .query(
            "
        CREATE TABLE IF NOT EXISTS search_cluster_memberships
        (
            id UUID,
            search_id UUID,
            cluster_id UUID,
            distance_to_centroid Float32,
        ) ENGINE = MergeTree()
        ORDER BY id
        ",
        )
        .execute()
        .await
        .unwrap();
}

impl SearchChunkQueryResponseBody {
    pub fn into_response_payload(&self) -> Vec<String> {
        self.score_chunks
            .clone()
            .into_iter()
            .map(|score_chunk| match &score_chunk.metadata[0] {
                ChunkMetadataTypes::Content(chunk) => match &chunk.tracking_id {
                    Some(tracking_id) => {
                        format!(
                            "{{\"id\": \"{}\", \"tracking_id\":\"{}\", \"score\": {}}}",
                            chunk.id, tracking_id, score_chunk.score
                        )
                    }
                    None => {
                        format!(
                            "{{\"id\": \"{}\", \"score\": {}}}",
                            chunk.id, score_chunk.score
                        )
                    }
                },
                ChunkMetadataTypes::ID(chunk) => match &chunk.tracking_id {
                    Some(tracking_id) => {
                        format!(
                            "{{\"id\": \"{}\", \"tracking_id\":\"{}\", \"score\": {}}}",
                            chunk.id, tracking_id, score_chunk.score
                        )
                    }
                    None => {
                        format!(
                            "{{\"id\": \"{}\", \"score\": {}}}",
                            chunk.id, score_chunk.score
                        )
                    }
                },
                ChunkMetadataTypes::Metadata(chunk) => match &chunk.tracking_id {
                    Some(tracking_id) => {
                        format!(
                            "{{\"id\": \"{}\", \"tracking_id\":\"{}\", \"score\": {}}}",
                            chunk.id, tracking_id, score_chunk.score
                        )
                    }
                    None => {
                        format!(
                            "{{\"id\": \"{}\", \"score\": {}}}",
                            chunk.id, score_chunk.score
                        )
                    }
                },
            })
            .collect::<Vec<String>>()
    }
}

impl SearchWithinGroupResults {
    pub fn into_response_payload(&self) -> Vec<String> {
        self.bookmarks
            .clone()
            .into_iter()
            .map(|score_chunk| match &score_chunk.metadata[0] {
                ChunkMetadataTypes::Content(chunk) => match &chunk.tracking_id {
                    Some(tracking_id) => {
                        format!(
                            "{{\"id\": \"{}\", \"tracking_id\":\"{}\", \"score\": {}}}",
                            chunk.id, tracking_id, score_chunk.score
                        )
                    }
                    None => {
                        format!(
                            "{{\"id\": \"{}\", \"score\": {}}}",
                            chunk.id, score_chunk.score
                        )
                    }
                },
                ChunkMetadataTypes::ID(chunk) => match &chunk.tracking_id {
                    Some(tracking_id) => {
                        format!(
                            "{{\"id\": \"{}\", \"tracking_id\":\"{}\", \"score\": {}}}",
                            chunk.id, tracking_id, score_chunk.score
                        )
                    }
                    None => {
                        format!(
                            "{{\"id\": \"{}\", \"score\": {}}}",
                            chunk.id, score_chunk.score
                        )
                    }
                },
                ChunkMetadataTypes::Metadata(chunk) => match &chunk.tracking_id {
                    Some(tracking_id) => {
                        format!(
                            "{{\"id\": \"{}\", \"tracking_id\":\"{}\", \"score\": {}}}",
                            chunk.id, tracking_id, score_chunk.score
                        )
                    }
                    None => {
                        format!(
                            "{{\"id\": \"{}\", \"score\": {}}}",
                            chunk.id, score_chunk.score
                        )
                    }
                },
            })
            .collect::<Vec<String>>()
    }
}

impl SearchOverGroupsResults {
    pub fn into_response_payload(&self) -> Vec<String> {
        self.group_chunks
            .clone()
            .into_iter()
            .map(|group_chunk| {
                let mut group_chunk_string = format!(
                    "{{\"group_id\": \"{}\", \"chunks\": [",
                    group_chunk.group_id
                );
                group_chunk_string.push_str(
                    &group_chunk
                        .metadata
                        .clone()
                        .into_iter()
                        .map(|score_chunk| match &score_chunk.metadata[0] {
                            ChunkMetadataTypes::Content(chunk) => match &chunk.tracking_id {
                                Some(tracking_id) => {
                                    format!(
                                        "{{\"id\": \"{}\", \"tracking_id\":\"{}\", \"score\": {}}}",
                                        chunk.id, tracking_id, score_chunk.score
                                    )
                                }
                                None => {
                                    format!(
                                        "{{\"id\": \"{}\", \"score\": {}}}",
                                        chunk.id, score_chunk.score
                                    )
                                }
                            },
                            ChunkMetadataTypes::ID(chunk) => match &chunk.tracking_id {
                                Some(tracking_id) => {
                                    format!(
                                        "{{\"id\": \"{}\", \"tracking_id\":\"{}\", \"score\": {}}}",
                                        chunk.id, tracking_id, score_chunk.score
                                    )
                                }
                                None => {
                                    format!(
                                        "{{\"id\": \"{}\", \"score\": {}}}",
                                        chunk.id, score_chunk.score
                                    )
                                }
                            },
                            ChunkMetadataTypes::Metadata(chunk) => match &chunk.tracking_id {
                                Some(tracking_id) => {
                                    format!(
                                        "{{\"id\": \"{}\", \"tracking_id\":\"{}\", \"score\": {}}}",
                                        chunk.id, tracking_id, score_chunk.score
                                    )
                                }
                                None => {
                                    format!(
                                        "{{\"id\": \"{}\", \"score\": {}}}",
                                        chunk.id, score_chunk.score
                                    )
                                }
                            },
                        })
                        .collect::<Vec<String>>()
                        .join(", "),
                );
                group_chunk_string.push_str("]}");
                group_chunk_string
            })
            .collect::<Vec<String>>()
    }
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
    }

    Ok(())
}
