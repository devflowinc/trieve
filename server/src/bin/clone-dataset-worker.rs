use actix_web::web;
use broccoli_queue::{error::BroccoliError, queue::BroccoliQueue};
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use itertools::Itertools;
use signal_hook::consts::SIGTERM;
use std::error::Error;
use std::io::Read;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use trieve_server::{
    data::models::{Pool, UnifiedId},
    establish_connection, get_env,
    handlers::{
        auth_handler::AdminOnly,
        chunk_handler::{
            create_chunk, ChunkReqPayload, CreateBatchChunkReqPayload, CreateChunkReqPayloadEnum,
            FullTextBoost, SemanticBoost,
        },
        dataset_handler::CloneDatasetMessage,
        file_handler::{upload_file_helper, UploadFileReqPayload},
    },
    operators::{
        chunk_operator::scroll_chunks_with_boosts_and_groups,
        dataset_operator::get_dataset_and_organization_from_dataset_id_query,
        file_operator::get_file_query,
        group_operator::{create_group_from_file_query, create_groups_query},
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();

    let database_url = get_env!("DATABASE_URL", "DATABASE_URL is not set");

    let mut config = ManagerConfig::default();
    config.custom_setup = Box::new(establish_connection);

    let mgr = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new_with_config(
        database_url,
        config,
    );

    let pool = diesel_async::pooled_connection::deadpool::Pool::builder(mgr)
        .max_size(3)
        .build()
        .expect("Failed to create diesel_async pool");

    let web_pool = actix_web::web::Data::new(pool.clone());

    let redis_url = get_env!("REDIS_URL", "REDIS_URL is not set");
    let redis_connections: u32 = std::env::var("REDIS_CONNECTIONS")
        .unwrap_or("2".to_string())
        .parse()
        .unwrap_or(2);

    let should_terminate = Arc::new(AtomicBool::new(false));
    signal_hook::flag::register(SIGTERM, Arc::clone(&should_terminate))
        .expect("Failed to register shutdown hook");

    let broccoli_queue = BroccoliQueue::builder(redis_url)
        .pool_connections(redis_connections.try_into().unwrap())
        .failed_message_retry_strategy(Default::default())
        .build()
        .await
        .expect("Failed to create broccoli queue");

    let web_broccoli_queue = web::Data::new(broccoli_queue.clone());

    log::info!("Starting clone dataset worker");

    broccoli_queue
        .process_messages("clone_dataset", None, None, move |msg| {
            clone_dataset_worker(msg.payload, web_broccoli_queue.clone(), web_pool.clone())
        })
        .await?;

    Ok(())
}

async fn clone_dataset_worker(
    msg: CloneDatasetMessage,
    broccoli_queue: web::Data<BroccoliQueue>,
    web_pool: web::Data<Pool>,
) -> Result<(), BroccoliError> {
    log::info!("Cloning dataset: {:?}", msg.dataset_to_clone);
    let dataset_to_clone = get_dataset_and_organization_from_dataset_id_query(
        UnifiedId::TrieveUuid(msg.dataset_to_clone),
        None,
        web_pool.clone(),
    )
    .await
    .map_err(|e| BroccoliError::Job(e.to_string()))?;

    let new_dataset = get_dataset_and_organization_from_dataset_id_query(
        UnifiedId::TrieveUuid(msg.new_dataset),
        None,
        web_pool.clone(),
    )
    .await
    .map_err(|e| BroccoliError::Job(e.to_string()))?;

    let mut chunk_cursor = Some(uuid::Uuid::nil());
    while let Some(offset_id) = chunk_cursor {
        log::info!("Scrolling chunks from offset: {:?}", offset_id);
        let (chunk_metadatas_with_boosts_and_groups, next_offset_id) =
            scroll_chunks_with_boosts_and_groups(
                web_pool.clone(),
                dataset_to_clone.dataset.id,
                120,
                Some(offset_id),
            )
            .await
            .map_err(|e| BroccoliError::Job(e.to_string()))?;

        let groups_to_create = chunk_metadatas_with_boosts_and_groups
            .iter()
            .flat_map(|(_, _, groups)| groups)
            .flatten()
            .unique_by(|g| g.id)
            .collect::<Vec<_>>();

        let chunk_req_payloads: Vec<ChunkReqPayload> = chunk_metadatas_with_boosts_and_groups
            .iter()
            .map(|(chunk_metadata, boosts, groups)| {
                let group_tracking_ids: Option<Vec<String>> = groups.as_ref().map(|groups| {
                    groups
                        .iter()
                        .map(|g| g.tracking_id.clone().unwrap_or(g.id.to_string()))
                        .collect::<Vec<_>>()
                });

                ChunkReqPayload {
                    // TODO: add semantic and fulltext content
                    // We don't currently store this
                    chunk_html: chunk_metadata.chunk_html.clone(),
                    link: chunk_metadata.link.clone(),
                    tag_set: chunk_metadata
                        .tag_set
                        .as_ref()
                        .map(|ts| ts.iter().map(|t| t.clone().unwrap_or_default()).collect()),
                    num_value: chunk_metadata.num_value,
                    metadata: chunk_metadata.metadata.clone(),
                    tracking_id: chunk_metadata.tracking_id.clone(),
                    upsert_by_tracking_id: Some(true),
                    group_tracking_ids,
                    time_stamp: chunk_metadata.time_stamp.map(|ts| ts.to_string()),
                    location: chunk_metadata.location,
                    image_urls: chunk_metadata
                        .image_urls
                        .as_ref()
                        .map(|urls| urls.iter().map(|u| u.clone().unwrap_or_default()).collect()),
                    weight: Some(chunk_metadata.weight),
                    fulltext_boost: boosts.as_ref().and_then(|b| {
                        b.fulltext_boost_phrase.as_ref().map(|f| FullTextBoost {
                            phrase: f.clone(),
                            boost_factor: b.fulltext_boost_factor.unwrap_or(0.0),
                        })
                    }),
                    semantic_boost: boosts.as_ref().and_then(|b| {
                        b.semantic_boost_phrase.as_ref().map(|s| SemanticBoost {
                            phrase: s.clone(),
                            distance_factor: b.semantic_boost_factor.unwrap_or(0.0) as f32,
                        })
                    }),
                    ..Default::default()
                }
            })
            .collect();

        if !groups_to_create.is_empty() {
            let created_groups = create_groups_query(
                groups_to_create
                    .iter()
                    .map(|g| g.clone_group(new_dataset.dataset.id))
                    .collect(),
                true,
                web_pool.clone(),
            )
            .await
            .map_err(|e| BroccoliError::Job(e.to_string()))?;

            for group in groups_to_create.into_iter().filter(|g| g.file_id.is_some()) {
                // Each groups has a unique file associated with it so we don't need to worry abt
                // creating the same file multiple times

                let file_id = group.file_id.unwrap();
                let file = get_file_query(
                    file_id,
                    3600,
                    dataset_to_clone.dataset.id,
                    None,
                    web_pool.clone(),
                )
                .await
                .map_err(|e| BroccoliError::Job(e.to_string()))?;

                let file_data = ureq::get(file.s3_url.as_str())
                    .call()
                    .map_err(|e| BroccoliError::Job(e.to_string()))?;
                let mut file_data_vec = Vec::new();
                file_data
                    .into_reader()
                    .read_to_end(&mut file_data_vec)
                    .unwrap();

                let new_file_id = upload_file_helper(
                    UploadFileReqPayload {
                        file_name: file.file_name.clone(),
                        metadata: file.metadata,
                        link: file.link,
                        time_stamp: file.time_stamp.map(|ts| ts.to_string()),
                        tag_set: file.tag_set,
                        ..Default::default()
                    },
                    file_data_vec,
                    web_pool.clone(),
                    new_dataset.clone(),
                )
                .await
                .map_err(|e| BroccoliError::Job(e.to_string()))?;

                let new_group = created_groups.iter().find(|g| {
                    g.tracking_id == group.tracking_id
                        || g.tracking_id == Some(group.id.to_string())
                });

                if let Some(new_group) = new_group {
                    create_group_from_file_query(new_group.id, new_file_id, web_pool.clone())
                        .await
                        .map_err(|e| BroccoliError::Job(e.to_string()))?;
                }
            }
        }

        create_chunk(
            web::Json(CreateChunkReqPayloadEnum::Batch(
                CreateBatchChunkReqPayload(chunk_req_payloads),
            )),
            web_pool.clone(),
            AdminOnly::default(),
            broccoli_queue.clone(),
            new_dataset.clone(),
        )
        .await
        .map_err(|e| BroccoliError::Job(e.to_string()))?;

        chunk_cursor = next_offset_id;
    }

    log::info!("Cloned dataset: {:?}", msg.new_dataset);
    Ok(())
}
