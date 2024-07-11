use serde::{Deserialize, Serialize};

use crate::{
    data::models::{
        ChunkMetadataTypes, ChunkMetadataWithScore, RagQueryEventClickhouse,
        RecommendationEventClickhouse, SearchQueryEventClickhouse,
    },
    errors::ServiceError,
    handlers::{
        chunk_handler::SearchChunkQueryResponseBody, group_handler::SearchWithinGroupResults,
    },
};

use super::search_operator::{GroupScoreChunk, SearchOverGroupsResults};

pub enum ClickHouseEvent {
    SearchQueryEvent(SearchQueryEventClickhouse),
    RecommendationEvent(RecommendationEventClickhouse),
    RagQueryEvent(RagQueryEventClickhouse),
}
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CHSlimResponse {
    pub id: uuid::Uuid,
    pub tracking_id: Option<String>,
    pub score: f64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CHSlimResponseGroup {
    pub group_id: uuid::Uuid,
    pub chunks: Vec<CHSlimResponse>,
}

impl SearchChunkQueryResponseBody {
    pub fn into_response_payload(&self) -> Vec<String> {
        self.score_chunks
            .clone()
            .into_iter()
            .map(|score_chunk| {
                let resp = match &score_chunk.metadata[0] {
                    ChunkMetadataTypes::Content(chunk) => CHSlimResponse {
                        id: chunk.id,
                        tracking_id: chunk.tracking_id.clone(),
                        score: score_chunk.score,
                    },
                    ChunkMetadataTypes::ID(chunk) => CHSlimResponse {
                        id: chunk.id,
                        tracking_id: chunk.tracking_id.clone(),
                        score: score_chunk.score,
                    },
                    ChunkMetadataTypes::Metadata(chunk) => CHSlimResponse {
                        id: chunk.id,
                        tracking_id: chunk.tracking_id.clone(),
                        score: score_chunk.score,
                    },
                };
                serde_json::to_string(&resp).unwrap_or("".to_string())
            })
            .collect::<Vec<String>>()
    }
}

impl SearchWithinGroupResults {
    pub fn into_response_payload(&self) -> Vec<String> {
        self.bookmarks
            .clone()
            .into_iter()
            .map(|score_chunk| {
                let resp = match &score_chunk.metadata[0] {
                    ChunkMetadataTypes::Content(chunk) => CHSlimResponse {
                        id: chunk.id,
                        tracking_id: chunk.tracking_id.clone(),
                        score: score_chunk.score,
                    },
                    ChunkMetadataTypes::ID(chunk) => CHSlimResponse {
                        id: chunk.id,
                        tracking_id: chunk.tracking_id.clone(),
                        score: score_chunk.score,
                    },
                    ChunkMetadataTypes::Metadata(chunk) => CHSlimResponse {
                        id: chunk.id,
                        tracking_id: chunk.tracking_id.clone(),
                        score: score_chunk.score,
                    },
                };
                serde_json::to_string(&resp).unwrap_or("".to_string())
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
                let resp = CHSlimResponseGroup {
                    group_id: group_chunk.group_id,
                    chunks: group_chunk
                        .metadata
                        .into_iter()
                        .map(|score_chunk| match &score_chunk.metadata[0] {
                            ChunkMetadataTypes::Content(chunk) => CHSlimResponse {
                                id: chunk.id,
                                tracking_id: chunk.tracking_id.clone(),
                                score: score_chunk.score,
                            },
                            ChunkMetadataTypes::ID(chunk) => CHSlimResponse {
                                id: chunk.id,
                                tracking_id: chunk.tracking_id.clone(),
                                score: score_chunk.score,
                            },
                            ChunkMetadataTypes::Metadata(chunk) => CHSlimResponse {
                                id: chunk.id,
                                tracking_id: chunk.tracking_id.clone(),
                                score: score_chunk.score,
                            },
                        })
                        .collect(),
                };
                serde_json::to_string(&resp).unwrap_or("".to_string())
            })
            .collect::<Vec<String>>()
    }
}

impl ChunkMetadataWithScore {
    pub fn into_response_payload(&self) -> String {
        let resp = CHSlimResponse {
            id: self.id,
            tracking_id: self.tracking_id.clone(),
            score: self.score as f64,
        };
        serde_json::to_string(&resp).unwrap_or("".to_string())
    }
}

impl GroupScoreChunk {
    pub fn into_response_payload(&self) -> String {
        let resp = CHSlimResponseGroup {
            group_id: self.group_id.clone(),
            chunks: self
                .metadata
                .clone()
                .into_iter()
                .map(|score_chunk| match &score_chunk.metadata[0] {
                    ChunkMetadataTypes::Content(chunk) => CHSlimResponse {
                        id: chunk.id,
                        tracking_id: chunk.tracking_id.clone(),
                        score: score_chunk.score,
                    },
                    ChunkMetadataTypes::ID(chunk) => CHSlimResponse {
                        id: chunk.id,
                        tracking_id: chunk.tracking_id.clone(),
                        score: score_chunk.score,
                    },
                    ChunkMetadataTypes::Metadata(chunk) => CHSlimResponse {
                        id: chunk.id,
                        tracking_id: chunk.tracking_id.clone(),
                        score: score_chunk.score,
                    },
                })
                .collect(),
        };
        serde_json::to_string(&resp).unwrap_or("".to_string())
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
