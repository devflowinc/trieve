use actix_web::web;
use serde::{Deserialize, Serialize};

use crate::{
    data::models::{ChunkMetadata, DatasetConfiguration, Pool},
    errors::ServiceError,
};

use super::chunk_operator::scroll_chunks_from_pg;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EtlChunk {
    pub id: String,
    pub chunk_html: Option<String>,
    pub tag_set: Option<Vec<Option<String>>>,
    pub image_urls: Option<Vec<Option<String>>>,
    pub num_value: Option<f64>,
}

impl From<ChunkMetadata> for EtlChunk {
    fn from(chunk_metadata: ChunkMetadata) -> Self {
        EtlChunk {
            id: chunk_metadata.id.to_string(),
            chunk_html: chunk_metadata.chunk_html,
            tag_set: chunk_metadata.tag_set,
            image_urls: chunk_metadata.image_urls,
            num_value: chunk_metadata.num_value,
        }
    }
}

pub async fn get_all_chunks_for_dataset_id(
    dataset_id: uuid::Uuid,
    dataset_config: DatasetConfiguration,
    pool: web::Data<Pool>,
) -> Result<Vec<String>, ServiceError> {
    let mut offset: Option<uuid::Uuid> = None;
    let mut first_iteration = true;

    // HACK set QDRANT_ONLY to true to get the payload from QDRANT
    let mut temp_dataset_config = dataset_config.clone();
    temp_dataset_config.QDRANT_ONLY = true;

    let mut chunks: Vec<String> = vec![];

    while offset.is_some() || first_iteration {
        //TODO: This has to use PG
        let (chunk_metadatas, offset_id) =
            scroll_chunks_from_pg(pool.clone(), dataset_id, 200, offset).await?;

        let string_chunks = chunk_metadatas
            .iter()
            .map(|x: &ChunkMetadata| {
                let result: EtlChunk = x.clone().into();
                serde_json::to_string(&result)
            })
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| {
                log::error!("Failed to serialize chunk metadata {:?}", e);
                ServiceError::InternalServerError("Failed to serialize chunk metadata".to_string())
            })?;

        chunks.extend(string_chunks);

        offset = offset_id;
        first_iteration = false;
    }
    Ok(chunks)
}
