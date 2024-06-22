use clickhouse::Row;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{
    data::models::ChunkMetadataTypes,
    errors::ServiceError,
    handlers::{
        chunk_handler::SearchChunkQueryResponseBody, group_handler::SearchWithinGroupResults,
    },
};

use super::search_operator::SearchOverGroupsResults;

pub enum ClickHouseEvent {
    SearchQueryEvent(SearchQueryEvent),
    //TODO: Recommended Chunks
    //TODO: Recommeneded Groups
    //TODO: RAG over selected Chunks
    //TODO: RAG over all Chunks
}

#[derive(Debug, Row, Serialize, Deserialize)]
pub struct SearchQueryEvent {
    #[serde(with = "clickhouse::serde::uuid")]
    pub id: uuid::Uuid,
    pub search_type: String,
    pub query: String,
    pub request_params: String,
    pub latency: f32,
    pub query_vector: Vec<f32>,
    pub results: Vec<String>,
    #[serde(with = "clickhouse::serde::uuid")]
    pub dataset_id: uuid::Uuid,
    #[serde(with = "clickhouse::serde::time::datetime")]
    pub created_at: OffsetDateTime,
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

pub fn create_full_vector(pairs: Vec<(u32, f32)>) -> Vec<f32> {
    // Step 1: Determine the size of the full vector based on the largest index
    let max_index = pairs.iter().map(|&(index, _)| index).max().unwrap_or(0) as usize;
    let size = max_index + 1; // +1 to account for zero-based indexing

    // Step 2: Initialize a vector with zeros
    let mut full_vector = vec![0.0; size];

    // Step 3: Populate the vector with the given index-value pairs
    for &(index, value) in &pairs {
        full_vector[index as usize] = value;
    }

    full_vector
}

pub async fn send_to_clickhouse(
    event: ClickHouseEvent,
    clickhouse_client: &clickhouse::Client,
) -> Result<(), ServiceError> {
    match event {
        ClickHouseEvent::SearchQueryEvent(event) => {
            let mut inserter =
                clickhouse_client
                    .insert("trieve.search_queries")
                    .map_err(|err| {
                        log::error!("Error creating ClickHouse inserter: {:?}", err);
                        ServiceError::InternalServerError(
                            "Error creating ClickHouse inserter".to_string(),
                        )
                    })?;

            inserter.write(&event).await.map_err(|err| {
                log::error!("Error writing to ClickHouse: {:?}", err);
                ServiceError::InternalServerError("Error writing to ClickHouse".to_string())
            })?;
            inserter.end().await.map_err(|err| {
                log::error!("Error ending ClickHouse inserter: {:?}", err);
                ServiceError::InternalServerError("Error ending ClickHouse inserter".to_string())
            })?;
        }
    }

    Ok(())
}
