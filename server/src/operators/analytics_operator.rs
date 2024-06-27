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
                        "INSERT INTO trieve.search_queries (id, search_type, query, request_params, query_vector, latency, top_score, results, dataset_id, created_at) VALUES (?, ?, ?, ?, embed_p(?), ?,?, ?, ?, now())",
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
                        ServiceError::InternalServerError("Error writing to ClickHouse".to_string())
                    })?;
        }
    }

    Ok(())
}
