use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::{
    pooled_connection::{AsyncDieselConnectionManager, ManagerConfig},
    RunQueryDsl,
};
use qdrant_client::{
    qdrant::{PointStruct, UpsertPointsBuilder},
    Qdrant,
};
use trieve_server::{
    data::{models::Organization, schema::organizations::dsl as organization_columns},
    errors::ServiceError,
    establish_connection, get_env,
    operators::qdrant_operator::scroll_qdrant_collection_ids_custom_url,
};

#[allow(clippy::print_stdout)]
#[tokio::main]
async fn main() -> Result<(), ServiceError> {
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

    let origin_qdrant_url =
        std::env::var("ORIGIN_QDRANT_URL").expect("ORIGIN_QDRANT_URL is not set");
    let new_qdrant_url = std::env::var("NEW_QDRANT_URL").expect("NEW_QDRANT_URL is not set");
    let qdrant_api_key = std::env::var("QDRANT_API_KEY").expect("QDRANT_API_KEY is not set");
    let collection_to_clone =
        std::env::var("COLLECTION_TO_CLONE").expect("COLLECTION_TO_CLONE is not set");
    let timeout = std::env::var("QDRANT_TIMEOUT_SEC")
        .unwrap_or("60".to_string())
        .parse::<u64>()
        .unwrap_or(60);

    let sleep_wait_ms = std::env::var("TIMEOUT_MS")
        .unwrap_or("200".to_string())
        .parse::<u64>()
        .unwrap_or(200);

    let excluded_org_ids = std::env::var("EXCLUDED_ORG_IDS")
        .unwrap_or("".to_string())
        .split(',')
        .filter_map(|x| match x.parse::<uuid::Uuid>() {
            Ok(id) => Some(id),
            Err(_) => None,
        })
        .collect::<Vec<uuid::Uuid>>();
    let mut org_id_offset = Some(
        std::env::var("ORG_OFFSET_ID")
            .map(|x| x.parse::<uuid::Uuid>())
            .unwrap_or(Ok(uuid::Uuid::nil()))
            .unwrap_or(uuid::Uuid::nil()),
    );

    while let Some(cur_org_id_offset) = org_id_offset {
        let mut conn = pool
            .get()
            .await
            .expect("Failed to get connection from pool");
        let org: Organization = organization_columns::organizations
            .filter(organization_columns::id.gt(cur_org_id_offset))
            .filter(organization_columns::id.ne_all(excluded_org_ids.clone()))
            .select(Organization::as_select())
            .first::<Organization>(&mut conn)
            .await
            .map_err(|err| {
                ServiceError::BadRequest(format!("Failed to fetch organizations {:?}", err))
            })?;

        let mut qdrant_offset = Some(uuid::Uuid::nil().to_string());
        while let Some(cur_qdrant_offset) = qdrant_offset {
            let original_qdrant_connection = Qdrant::from_url(&origin_qdrant_url)
                .api_key(qdrant_api_key.clone())
                .timeout(std::time::Duration::from_secs(timeout))
                .build()
                .map_err(|err| {
                    ServiceError::BadRequest(format!("Failed to connect to new Qdrant {:?}", err))
                })?;
            let new_qdrant_connection = Qdrant::from_url(&new_qdrant_url)
                .api_key(qdrant_api_key.clone())
                .timeout(std::time::Duration::from_secs(timeout))
                .build()
                .map_err(|err| {
                    ServiceError::BadRequest(format!("Failed to connect to new Qdrant {:?}", err))
                })?;

            log::info!(
                "Fetching points from original collection starting at: {}",
                cur_qdrant_offset
            );

            let (origin_qdrant_points, new_offset) = scroll_qdrant_collection_ids_custom_url(
                collection_to_clone.clone(),
                Some(cur_qdrant_offset.to_string()),
                Some(500),
                original_qdrant_connection,
            )
            .await
            .map_err(|err| {
                log::error!("Failed fetching points from qdrant {:?}", err);
                ServiceError::BadRequest(format!("Failed fetching points from qdrant {:?}", err))
            })?;

            let point_structs_to_upsert = origin_qdrant_points
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
                "Upserting {} points into new Qdrant collection starting at: {}",
                point_structs_to_upsert.len(),
                cur_qdrant_offset
            );

            new_qdrant_connection
                .upsert_points(UpsertPointsBuilder::new(
                    collection_to_clone.clone(),
                    point_structs_to_upsert,
                ))
                .await
                .map_err(|err| {
                    log::error!("Failed inserting chunks to qdrant {:?}", err);
                    ServiceError::BadRequest(format!("Failed inserting chunks to qdrant {:?}", err))
                })?;

            log::info!(
                "Setting offset to: {}",
                new_offset.clone().unwrap_or_default()
            );

            tokio::time::sleep(std::time::Duration::from_millis(sleep_wait_ms)).await;

            qdrant_offset = new_offset;
        }
    }

    Ok(())
}
