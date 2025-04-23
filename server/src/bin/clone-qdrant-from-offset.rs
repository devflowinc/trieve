use diesel::prelude::*;
use diesel_async::pooled_connection::{AsyncDieselConnectionManager, ManagerConfig};
use diesel_async::RunQueryDsl;
use itertools::Itertools;
use qdrant_client::qdrant::point_id::PointIdOptions;
use qdrant_client::qdrant::{GetPoints, PointId, PointStruct, UpsertPointsBuilder};
use std::collections::{HashMap, HashSet};
use trieve_server::data::models::{Dataset, DatasetConfiguration};
use trieve_server::operators::qdrant_operator::{
    get_qdrant_collection_from_dataset_config, get_qdrant_connection,
};
use trieve_server::{errors::ServiceError, establish_connection, get_env};

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
struct CollectionToPointids {
    qdrant_collection_name: String,
    point_ids: HashSet<String>,
}

#[allow(clippy::print_stdout)]
#[tokio::main]
async fn main() -> Result<(), ()> {
    println!("starting backfill job");
    dotenvy::dotenv().ok();
    let database_url = get_env!("DATABASE_URL", "DATABASE_URL is not set");

    env_logger::builder()
        .target(env_logger::Target::Stdout)
        .filter_level(log::LevelFilter::Info)
        .init();

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

    let postgres_fetch_points_count: i64 = std::env::var("POSTGRES_FETCH_POINTS_COUNT")
        .unwrap_or("500".to_string())
        .parse::<i64>()
        .unwrap_or(500);

    let redis_url = get_env!("REDIS_URL", "REDIS_URL is not set");

    let redis_pool = bb8_redis::bb8::Pool::builder()
        .max_size(2)
        .connection_timeout(std::time::Duration::from_secs(2))
        .build(
            bb8_redis::RedisConnectionManager::new(redis_url).expect("Failed to connect to redis"),
        )
        .await
        .expect("Failed to create redis pool");

    let mut redis_conn = redis_pool.get().await.unwrap();

    let start_offset = std::env::var("START_OFFSET")
        .map(|offset_str| uuid::Uuid::parse_str(&offset_str).ok().unwrap())
        .unwrap_or(uuid::Uuid::nil());

    let stop_offset = std::env::var("END_OFFSET")
        .map(|offset_str| uuid::Uuid::parse_str(&offset_str).ok().unwrap())
        .unwrap_or(uuid::Uuid::max());

    let all_less_than = std::env::var("LESS_THAN")
        .map(|offset_str| {
            chrono::NaiveDateTime::parse_from_str(&offset_str, "%Y-%m-%d %H:%M:%S").unwrap()
        })
        .unwrap_or(chrono::NaiveDateTime::MIN);

    let greater_than = std::env::var("GREATER_THAN")
        .map(|offset_str| {
            chrono::NaiveDateTime::parse_from_str(&offset_str, "%Y-%m-%d %H:%M:%S").unwrap()
        })
        .unwrap_or(chrono::NaiveDateTime::MAX);

    let mut offset = Some(start_offset);

    let origin_qdrant_client = get_qdrant_connection(
        Some(
            std::env::var("ORIGIN_QDRANT_URL")
                .expect("ORIGIN_QDRANT_URL should be set")
                .as_str(),
        ),
        None,
    )
    .await
    .expect("Failed to get origin qdrant connection");

    let dest_qdrant_client = get_qdrant_connection(
        Some(
            std::env::var("DEST_QDRANT_URL")
                .expect("DEST_QDRANT_URL should be set")
                .as_str(),
        ),
        None,
    )
    .await
    .expect("Failed to get dest qdrant connection");

    while let Some(cur_offset) = offset {
        use trieve_server::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
        use trieve_server::data::schema::datasets::dsl as datasets_columns;

        let mut conn = pool
            .get()
            .await
            .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))
            .expect("Failed to get database connection");

        println!("Fetching from postgres @ {}", cur_offset);

        // Get postgres_fetch_points_count chunk_metadata
        let qdrant_dataset_id_pairs = chunk_metadata_columns::chunk_metadata
            .select((
                chunk_metadata_columns::id,
                chunk_metadata_columns::qdrant_point_id,
                chunk_metadata_columns::dataset_id,
            ))
            .filter(chunk_metadata_columns::id.ge(cur_offset))
            .filter(chunk_metadata_columns::updated_at.lt(all_less_than))
            .filter(chunk_metadata_columns::updated_at.gt(greater_than))
            .order_by(chunk_metadata_columns::id)
            .limit(postgres_fetch_points_count)
            .load::<(uuid::Uuid, uuid::Uuid, uuid::Uuid)>(&mut conn)
            .await
            .expect("Failed to query chunks");

        let chunk_ids = qdrant_dataset_id_pairs
            .iter()
            .map(|(chunk_id, _, _)| chunk_id)
            .collect_vec();

        offset = chunk_ids.iter().max().copied().copied();
        if qdrant_dataset_id_pairs.len() < (postgres_fetch_points_count as usize) {
            println!("setting offset to None");
            offset = None;
        }
        log::info!(
            "hi {:?} {:?} {:?}",
            chunk_ids.len(),
            postgres_fetch_points_count,
            qdrant_dataset_id_pairs.len()
        );
        if let Some(new_offset) = offset {
            if new_offset > stop_offset {
                offset = None;
            }
        }

        let dataset_id_config_pair = datasets_columns::datasets
            .filter(
                datasets_columns::id.eq_any(
                    &qdrant_dataset_id_pairs
                        .iter()
                        .map(|(_, _, dataset_id)| dataset_id)
                        .unique()
                        .collect_vec(),
                ),
            )
            .load::<Dataset>(&mut conn)
            .await
            .expect("Failed to load dataset settings");

        // Modified version that combines by collection name
        let collected_qdrant_ids: Vec<CollectionToPointids> = qdrant_dataset_id_pairs
            .into_iter()
            .filter_map(|(_, qdrant_point_id, dataset_id)| {
                dataset_id_config_pair
                    .iter()
                    .find(|dataset| dataset.id == dataset_id)
                    .map(|dataset| {
                        let dataset_config =
                            DatasetConfiguration::from_json(dataset.server_configuration.clone());
                        (
                            get_qdrant_collection_from_dataset_config(&dataset_config),
                            qdrant_point_id.to_string(),
                        )
                    })
            })
            .fold(
                HashMap::new(),
                |mut acc: HashMap<String, HashSet<String>>, (collection_name, point_id)| {
                    acc.entry(collection_name).or_default().insert(point_id);
                    acc
                },
            )
            .into_iter()
            .map(|(qdrant_collection_name, point_ids)| CollectionToPointids {
                point_ids,
                qdrant_collection_name,
            })
            .collect();

        // For each chunk, see if it exists in qdrant. use this id to check against prod
        for collection_pairs in collected_qdrant_ids {
            let qdrant_point_ids_response = origin_qdrant_client
                .get_points(GetPoints {
                    collection_name: collection_pairs.qdrant_collection_name.clone(),
                    ids: collection_pairs
                        .point_ids
                        .iter()
                        .map(|point_uuid| PointId::from(point_uuid.as_str()))
                        .collect(),
                    with_payload: Some(true.into()),
                    with_vectors: Some(true.into()),
                    read_consistency: None,
                    shard_key_selector: None,
                    timeout: None,
                })
                .await;

            match qdrant_point_ids_response {
                Ok(resp) => {
                    let migrate_qdrant_points = resp.result;

                    let qdrant_points_existing = migrate_qdrant_points
                        .clone()
                        .iter()
                        .filter_map(|point| {
                            point.id.as_ref().map(|point_id| {
                                match point_id.point_id_options.clone() {
                                    Some(PointIdOptions::Uuid(qdrant_uuid)) => qdrant_uuid,
                                    _ => unreachable!(),
                                }
                            })
                        })
                        .collect::<HashSet<String>>();

                    // Collect each missing point into list
                    let missing_points: Vec<uuid::Uuid> = collection_pairs
                        .point_ids
                        .difference(&qdrant_points_existing)
                        .filter_map(|string_id| uuid::Uuid::parse_str(string_id).ok())
                        .collect_vec();

                    if !missing_points.is_empty() {
                        log::info!("pushing {} missing points", missing_points.len());
                        let _ = redis::cmd("lpush")
                            .arg("missing_qdrant_uuids")
                            .arg(
                                missing_points
                                    .iter()
                                    .map(|id| id.to_string())
                                    .collect_vec()
                                    .as_slice(),
                            )
                            .query_async::<redis::aio::MultiplexedConnection, ()>(&mut *redis_conn)
                            .await
                            .map_err(|e| {
                                log::error!("Redis failed to push missing qdrant uuids: {}", e);
                            });
                    }

                    let point_structs_to_upsert = migrate_qdrant_points
                        .iter()
                        .filter_map(|retrieved_point| {
                            let id = retrieved_point.id.clone();
                            let payload = retrieved_point.payload.clone();
                            let vectors = retrieved_point.vectors.clone();
                            if let (Some(id), payload, Some(vectors)) = (id, payload, vectors) {
                                Some(PointStruct {
                                    id: Some(id),
                                    payload,
                                    vectors: Some(vectors),
                                })
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<PointStruct>>();

                    log::info!(
                        "Upserting {} points into the new Qdrant collection starting at: {}",
                        point_structs_to_upsert.len(),
                        cur_offset
                    );

                    let mut retries = 0;
                    let max_retries = 5;
                    let mut upsert_result;

                    loop {
                        upsert_result = dest_qdrant_client
                            .upsert_points(UpsertPointsBuilder::new(
                                collection_pairs.qdrant_collection_name.clone(),
                                point_structs_to_upsert.clone(),
                            ))
                            .await;

                        match upsert_result {
                            Ok(_) => {
                                break;
                            }
                            Err(e) if retries >= max_retries => {
                                log::error!(
                                    "Failed to upsert points {:?} into new Qdrant collection, error {:?}",
                                    qdrant_points_existing.len(),
                                    e,
                                );
                                let _ = redis::cmd("sadd")
                                    .arg("failed_qdrant_uuids")
                                    .arg(
                                        qdrant_points_existing
                                            .iter()
                                            .map(|id| id.to_string())
                                            .collect_vec()
                                            .as_slice(),
                                    )
                                    .query_async::<redis::aio::MultiplexedConnection, ()>(
                                        &mut *redis_conn,
                                    )
                                    .await
                                    .map_err(|e| {
                                        log::error!(
                                            "Redis failed to add missing qdrant uuids to set: {}",
                                            e
                                        );
                                    });
                                break;
                            }
                            Err(e) => {
                                // sleep exponential backoff
                                let sleep_wait_ms = 1000 * 2u64.pow(retries);
                                log::info!(
                                    "Failed to upsert points {:?}, retrying in {}ms",
                                    e,
                                    sleep_wait_ms
                                );
                                tokio::time::sleep(std::time::Duration::from_millis(sleep_wait_ms))
                                    .await;

                                retries += 1;
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    Ok(())
}
