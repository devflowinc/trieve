use std::{collections::BTreeMap, path::PathBuf};

use actix_web::web;
use pagefind::{api::PagefindIndex, options::PagefindServiceConfig};

use crate::{
    data::models::{self, DatasetConfiguration, Pool, QdrantChunkMetadata, WorkerEvent},
    errors::ServiceError,
    operators::{clickhouse_operator::ClickHouseEvent, file_operator::get_pagefind_aws_bucket},
};

use super::{qdrant_operator::scroll_dataset_points, search_operator::assemble_qdrant_filter};

#[tracing::instrument(skip_all)]
pub async fn build_index_for_dataset_id(
    dataset_id: uuid::Uuid,
    dataset_config: DatasetConfiguration,
    pool: web::Data<Pool>,
    event_queue: &web::Data<crate::EventQueue>,
) -> Result<(), ServiceError> {
    let options = PagefindServiceConfig::builder()
        .keep_index_url(true)
        .force_language("en".to_string())
        .build();
    let mut search_index = PagefindIndex::new(Some(options)).expect("config is valid");

    let filter = assemble_qdrant_filter(None, None, None, dataset_id, pool.clone()).await?;

    let mut offset: Option<uuid::Uuid> = None;
    let mut first_iteration = true;

    // HACK set QDRANT_ONLY to true to get the payload from QDRANT
    let mut temp_dataset_config = dataset_config.clone();
    temp_dataset_config.QDRANT_ONLY = true;

    while offset.is_some() || first_iteration {
        let (search_results, offset_id) = scroll_dataset_points(
            200,
            offset,
            None,
            temp_dataset_config.clone(),
            filter.clone(),
        )
        .await?;

        for result in search_results.iter() {
            let payload: QdrantChunkMetadata = result.clone().into();

            let mut meta_keys: BTreeMap<String, String> = payload
                .metadata
                .unwrap_or_default()
                .as_object()
                .map(|m| {
                    m.iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect::<BTreeMap<String, String>>()
                })
                .unwrap_or_default();

            let mut sort_keys = BTreeMap::new();

            // filters: Option<HashMap<String, Vec<String>>>,
            let mut filters: BTreeMap<String, Vec<String>> = BTreeMap::new();
            if let Some(tags) = payload.tag_set {
                filters.insert("tag_set".to_string(), tags.clone());

                meta_keys.insert("tag_set".to_string(), tags.join(", "));
            }

            if let Some(time_stamp) = payload.time_stamp {
                filters.insert("time_stamp".to_string(), vec![time_stamp.to_string()]);
                meta_keys.insert("tag_set".to_string(), time_stamp.to_string());
                sort_keys.insert("time_stamp".to_string(), time_stamp.to_string());
            }

            if let Some(group_ids) = payload.group_ids.clone() {
                filters.insert(
                    "group_ids".to_string(),
                    group_ids.iter().map(|i| i.to_string()).collect(),
                );
                meta_keys.insert(
                    "group_ids".to_string(),
                    group_ids.iter().map(|i| i.to_string()).collect(),
                );
            }

            if let Some(image_urls) = payload.image_urls {
                meta_keys.insert("image_urls".to_string(), image_urls.join(", "));
            }

            let _ = search_index
                .add_custom_record(
                    payload.link.unwrap_or_default().to_string(),
                    payload.chunk_html.unwrap_or_default().to_string(),
                    "en".to_string(),
                    Some(meta_keys),
                    Some(filters),
                    Some(sort_keys),
                )
                .await;
        }

        offset = offset_id;
        first_iteration = false;
    }

    search_index
        .build_indexes()
        .await
        .map_err(|e| ServiceError::BadRequest(format!("Could not build pagefind index {:?}", e)))?;

    let files = search_index.get_files().await.map_err(|e| {
        ServiceError::BadRequest(format!("Could not get files from pagefind index {:?}", e))
    })?;
    let total_files = files.len();
    log::info!("Uploading {:?} pagefind indexed files to S3", total_files);

    let futures = files.into_iter().enumerate().map(
        |(i, file)| -> tokio::task::JoinHandle<Result<(), ServiceError>> {
            let mut filename = PathBuf::from("/pagefind");
            filename.push(dataset_id.to_string());
            filename.push(file.filename.clone());

            // WARNING This s3 bucket cannot be default public. put ACL's on this somehow in case
            // the user does not want their data to be public.
            tokio::task::spawn(async move {
                let bucket = get_pagefind_aws_bucket()?;
                bucket
                    .put_object(
                        filename.to_string_lossy().to_string(),
                        &file.contents.clone(),
                    )
                    .await
                    .map_err(|e| {
                        ServiceError::BadRequest(format!("Could not upload file to S3 {:?}", e))
                    })?;

                log::info!("Uploaded file {:?} to S3", i);
                Ok(())
            })
        },
    );

    futures::future::join_all(futures).await;

    event_queue
        .send(ClickHouseEvent::WorkerEvent(
            WorkerEvent::from_details(
                dataset_id,
                None,
                models::EventType::PagefindIndexingFinished { total_files },
            )
            .into(),
        ))
        .await;

    Ok(())
}
