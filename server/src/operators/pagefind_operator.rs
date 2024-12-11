use std::path::PathBuf;

use actix_web::web;
use hashbrown::HashMap;
use pagefind::{Fossicker, SearchState};

use crate::{
    data::models::{Dataset, DatasetConfiguration, Pool, QdrantChunkMetadata},
    errors::ServiceError,
    operators::file_operator::get_pagefind_aws_bucket,
};

use super::{qdrant_operator::scroll_dataset_points, search_operator::assemble_qdrant_filter};

pub fn create_pagefind_index() -> SearchState {
    let config = pagefind::PagefindInboundConfig {
        source: "source".into(),
        site: "site".into(),
        bundle_dir: None,
        output_subdir: None,
        output_path: None,
        root_selector: "root_selector".into(),
        exclude_selectors: vec![],
        glob: "**/*.{html}".into(),
        force_language: None,
        serve: false,
        verbose: false,
        logfile: None,
        keep_index_url: false,
        service: false,
    };
    let opts = pagefind::SearchOptions::load(config).expect("Config is always valid");

    SearchState::new(opts)
}

pub async fn add_record(
    index: &mut SearchState,
    url: String,
    content: String,
    language: String,
    meta: Option<HashMap<String, String>>,
    filters: Option<HashMap<String, Vec<String>>>,
    sort: Option<HashMap<String, String>>,
) -> Result<pagefind::FossickedData, ()> {
    let data = pagefind::fossick::parser::DomParserResult {
        digest: content,
        filters: filters.unwrap_or_default(),
        sort: sort.unwrap_or_default(),
        meta: meta.unwrap_or_default(),
        anchor_content: HashMap::new(),
        has_custom_body: false,
        force_inclusion: true,
        has_html_element: true,
        has_old_bundle_reference: false,
        language: index.options.force_language.clone().unwrap_or(language),
    };
    let file = Fossicker::new_with_data(url, data);
    index.fossick_one(file).await
}

pub async fn get_files(index: &mut SearchState) -> Vec<pagefind::SyntheticFile> {
    index.build_indexes().await;
    index.get_files().await
}

pub async fn build_index_for_dataset_id(
    dataset: Dataset,
    pool: web::Data<Pool>,
) -> Result<(), ServiceError> {
    let mut search_index = create_pagefind_index();

    let filter = assemble_qdrant_filter(None, None, None, dataset.id, pool.clone()).await?;

    let mut offset: Option<uuid::Uuid> = None;
    let mut first_iteration = true;

    let mut dataset_config = DatasetConfiguration::from_json(dataset.server_configuration);

    // HACK set QDRANT_ONLY to true to get the payload from QDRANT
    dataset_config.QDRANT_ONLY = true;

    while offset.is_some() || first_iteration {
        let (search_results, offset_id) =
            scroll_dataset_points(100, offset, None, dataset_config.clone(), filter.clone())
                .await?;

        for result in search_results.iter() {
            let payload: QdrantChunkMetadata = result.clone().into();

            let _ = add_record(
                &mut search_index,
                payload.link.unwrap_or_default().to_string(),
                payload.chunk_html.unwrap_or_default().to_string(),
                "en".to_string(),
                payload.metadata.unwrap_or_default().as_object().map(|m| {
                    m.iter()
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .collect()
                }),
                None,
                None,
            )
            .await;
        }

        offset = offset_id;
        first_iteration = false;
    }

    search_index.build_indexes().await;

    for file in search_index.get_files().await {
        let bucket = get_pagefind_aws_bucket()?;

        // WARNING This s3 bucket cannot be default public. put ACL's on this somehow in case
        // the user does not want their data to be public.
        let mut filename = PathBuf::from("/pagefind");
        filename.push(dataset.id.to_string());
        filename.push(file.filename.clone());

        bucket
            .put_object(
                filename.to_string_lossy().to_string(),
                file.contents.as_ref(),
            )
            .await
            .map_err(|e| {
                log::error!("Could not upload file to S3 {:?}", e);
                ServiceError::BadRequest("Could not upload file to S3".to_string())
            })?;
    }

    Ok(())
}
