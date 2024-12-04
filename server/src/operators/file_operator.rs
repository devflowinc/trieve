use super::chunk_operator::{create_chunk_metadata, get_row_count_for_organization_id_query};
use super::clickhouse_operator::{ClickHouseEvent, EventQueue};
use super::group_operator::{create_group_from_file_query, create_groups_query};
use super::parse_operator::{build_chunking_regex, coarse_doc_chunker, convert_html_to_text};
use crate::data::models::ChunkGroup;
use crate::data::models::FileDTO;
use crate::data::models::{Dataset, DatasetAndOrgWithSubAndPlan, DatasetConfiguration, EventType};
use crate::handlers::chunk_handler::ChunkReqPayload;
use crate::handlers::file_handler::UploadFileReqPayload;
use crate::operators::group_operator::delete_group_by_file_id_query;
use crate::{data::models::WorkerEvent, get_env};
use crate::{
    data::models::{File, Pool},
    errors::ServiceError,
};
use actix_web::web;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::BigInt;
use diesel_async::RunQueryDsl;
use redis::aio::MultiplexedConnection;
use regex::Regex;
use s3::{creds::Credentials, Bucket, Region};
use std::collections::HashMap;

pub fn get_aws_bucket() -> Result<Bucket, ServiceError> {
    let aws_region_name = std::env::var("AWS_REGION").unwrap_or("".to_string());
    let s3_endpoint = get_env!("S3_ENDPOINT", "S3_ENDPOINT should be set").into();
    let s3_bucket_name = get_env!("S3_BUCKET", "S3_BUCKET should be set");

    let aws_region = Region::Custom {
        region: aws_region_name,
        endpoint: s3_endpoint,
    };

    let aws_credentials = if let Ok(creds) = Credentials::from_instance_metadata() {
        creds
    } else {
        let s3_access_key = get_env!("S3_ACCESS_KEY", "S3_ACCESS_KEY should be set").into();
        let s3_secret_key = get_env!("S3_SECRET_KEY", "S3_SECRET_KEY should be set").into();
        Credentials {
            access_key: Some(s3_access_key),
            secret_key: Some(s3_secret_key),
            security_token: None,
            session_token: None,
            expiration: None,
        }
    };

    let aws_bucket = Bucket::new(s3_bucket_name, aws_region, aws_credentials)
        .map_err(|e| {
            log::error!("Could not create or get bucket {:?}", e);
            ServiceError::BadRequest("Could not create or get bucket".to_string())
        })?
        .with_path_style();

    Ok(aws_bucket)
}

pub async fn create_file_query(
    file_id: uuid::Uuid,
    file_size: i64,
    upload_file_data: UploadFileReqPayload,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<File, ServiceError> {
    use crate::data::schema::files::dsl as files_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let new_file = File::from_details(
        Some(file_id),
        &upload_file_data.file_name,
        file_size,
        upload_file_data
            .tag_set
            .map(|tag_set| tag_set.into_iter().map(Some).collect()),
        upload_file_data.metadata,
        upload_file_data.link,
        upload_file_data.time_stamp,
        dataset_id,
    );

    let created_file: File = diesel::insert_into(files_columns::files)
        .values(&new_file)
        .get_result(&mut conn)
        .await
        .map_err(|err| ServiceError::BadRequest(format!("Could not create file {:?}", err)))?;

    Ok(created_file)
}

pub fn preprocess_file_to_chunks(
    html_content: String,
    upload_file_data: UploadFileReqPayload,
) -> Result<Vec<String>, ServiceError> {
    let file_text = convert_html_to_text(&html_content);

    let split_regex: Option<Regex> = upload_file_data
        .split_delimiters
        .map(|delimiters| {
            build_chunking_regex(delimiters).map_err(|e| {
                log::error!("Could not parse chunking delimiters {:?}", e);
                ServiceError::BadRequest("Could not parse chunking delimiters".to_string())
            })
        })
        .transpose()?;

    let rebalance_chunks = upload_file_data.rebalance_chunks.unwrap_or(true);
    let target_splits_per_chunk = upload_file_data.target_splits_per_chunk.unwrap_or(20);

    let chunk_htmls = coarse_doc_chunker(
        file_text,
        split_regex,
        rebalance_chunks,
        target_splits_per_chunk,
    );

    Ok(chunk_htmls)
}

#[allow(clippy::too_many_arguments)]
pub async fn create_file_chunks(
    created_file_id: uuid::Uuid,
    upload_file_data: UploadFileReqPayload,
    chunk_htmls: Vec<String>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    event_queue: web::Data<EventQueue>,
    mut redis_conn: MultiplexedConnection,
) -> Result<(), ServiceError> {
    let mut chunks: Vec<ChunkReqPayload> = [].to_vec();

    let name = upload_file_data.file_name.clone();

    let chunk_group = ChunkGroup::from_details(
        Some(name.clone()),
        upload_file_data.description.clone(),
        dataset_org_plan_sub.dataset.id,
        upload_file_data.group_tracking_id.clone(),
        None,
        upload_file_data
            .tag_set
            .clone()
            .map(|tag_set| tag_set.into_iter().map(Some).collect()),
    );

    let chunk_group_option = create_groups_query(vec![chunk_group], true, pool.clone())
        .await
        .map_err(|e| {
            log::error!("Could not create group {:?}", e);
            ServiceError::BadRequest("Could not create group".to_string())
        })?
        .pop();

    let chunk_group = match chunk_group_option {
        Some(group) => group,
        None => {
            return Err(ServiceError::BadRequest(
                "Could not create group from file".to_string(),
            ));
        }
    };

    let group_id = chunk_group.id;

    create_group_from_file_query(group_id, created_file_id, pool.clone())
        .await
        .map_err(|e| {
            log::error!("Could not create group from file {:?}", e);
            e
        })?;

    for (i, chunk_html) in chunk_htmls.iter().enumerate() {
        let create_chunk_data = ChunkReqPayload {
            chunk_html: Some(chunk_html.clone()),
            semantic_content: None,
            link: upload_file_data.link.clone(),
            tag_set: upload_file_data.tag_set.clone(),
            metadata: upload_file_data.metadata.clone(),
            group_ids: Some(vec![group_id]),
            group_tracking_ids: None,
            location: None,
            tracking_id: upload_file_data
                .group_tracking_id
                .clone()
                .map(|tracking_id| format!("{}|{}", tracking_id, i)),
            upsert_by_tracking_id: None,
            time_stamp: upload_file_data.time_stamp.clone(),
            weight: None,
            split_avg: None,
            convert_html_to_text: None,
            image_urls: None,
            num_value: None,
            fulltext_boost: None,
            semantic_boost: None,
        };
        chunks.push(create_chunk_data);
    }

    let chunk_count = get_row_count_for_organization_id_query(
        dataset_org_plan_sub.organization.organization.id,
        pool.clone(),
    )
    .await?;

    if chunk_count + chunks.len()
        > dataset_org_plan_sub
            .organization
            .plan
            .unwrap_or_default()
            .chunk_count as usize
    {
        return Err(ServiceError::BadRequest(
            "Chunk count exceeds plan limit".to_string(),
        ));
    }

    let chunk_segments = chunks
        .chunks(120)
        .map(|chunk_segment| chunk_segment.to_vec())
        .collect::<Vec<Vec<ChunkReqPayload>>>();

    let mut serialized_messages: Vec<String> = vec![];

    for chunk_segment in chunk_segments {
        let (ingestion_message, _) =
            create_chunk_metadata(chunk_segment, dataset_org_plan_sub.dataset.id).await?;

        let serialized_message: String =
            serde_json::to_string(&ingestion_message).map_err(|_| {
                ServiceError::BadRequest("Failed to Serialize BulkUploadMessage".to_string())
            })?;

        if serialized_message.is_empty() {
            continue;
        }

        serialized_messages.push(serialized_message);
    }

    for serialized_message in serialized_messages {
        redis::cmd("lpush")
            .arg("ingestion")
            .arg(&serialized_message)
            .query_async::<redis::aio::MultiplexedConnection, ()>(&mut redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    }

    event_queue
        .send(ClickHouseEvent::WorkerEvent(
            WorkerEvent::from_details(
                dataset_org_plan_sub.dataset.id,
                EventType::FileUploaded {
                    file_id: created_file_id,
                    file_name: name,
                },
            )
            .into(),
        ))
        .await;

    Ok(())
}

pub async fn get_file_query(
    file_uuid: uuid::Uuid,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<FileDTO, actix_web::Error> {
    use crate::data::schema::files::dsl as files_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let file: File = files_columns::files
        .filter(files_columns::id.eq(file_uuid))
        .filter(files_columns::dataset_id.eq(dataset_id))
        .get_result(&mut conn)
        .await
        .map_err(|e| {
            log::error!("File with specified id not found {:?}", e);

            ServiceError::NotFound("File with specified id not found".to_string())
        })?;

    let mut custom_queries = HashMap::new();
    custom_queries.insert(
        "response-content-disposition".into(),
        format!("attachment; filename=\"{}\"", file.file_name),
    );

    let bucket = get_aws_bucket()?;
    let s3_url = bucket
        .presign_get(file.id.to_string(), 6000, Some(custom_queries))
        .await
        .map_err(|e| {
            log::error!("Could not get presigned url {:?}", e);

            ServiceError::NotFound("Could not get presigned url".to_string())
        })?;

    let file_dto: FileDTO = file.into();
    let file_dto: FileDTO = FileDTO { s3_url, ..file_dto };

    Ok(file_dto)
}

pub async fn get_dataset_file_query(
    dataset_id: uuid::Uuid,
    page: u64,
    pool: web::Data<Pool>,
) -> Result<Vec<(File, i64, Option<uuid::Uuid>)>, actix_web::Error> {
    use crate::data::schema::files::dsl as files_columns;
    use crate::data::schema::groups_from_files::dsl as groups_from_files_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let file_metadata: Vec<(File, i64, Option<uuid::Uuid>)> = files_columns::files
        .left_join(
            groups_from_files_columns::groups_from_files
                .on(groups_from_files_columns::file_id.eq(files_columns::id)),
        )
        .filter(files_columns::dataset_id.eq(dataset_id))
        .select((
            File::as_select(),
            sql::<BigInt>("count(*) OVER()"),
            groups_from_files_columns::group_id.nullable(),
        ))
        .limit(10)
        .offset(((page - 1) * 10).try_into().unwrap_or(0))
        .load(&mut conn)
        .await
        .map_err(|_| ServiceError::NotFound("No dataset found".to_string()))?;

    Ok(file_metadata)
}

pub async fn delete_file_query(
    file_uuid: uuid::Uuid,
    delete_chunks: Option<bool>,
    dataset: Dataset,
    pool: web::Data<Pool>,
    dataset_config: DatasetConfiguration,
) -> Result<(), actix_web::Error> {
    use crate::data::schema::files::dsl as files_columns;

    let mut conn = pool
        .get()
        .await
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let file_metadata: File = files_columns::files
        .filter(files_columns::id.eq(file_uuid))
        .filter(files_columns::dataset_id.eq(dataset.id))
        .get_result(&mut conn)
        .await
        .map_err(|_| ServiceError::NotFound("File with specified id not found".to_string()))?;

    let bucket = get_aws_bucket()?;
    bucket
        .delete_object(file_metadata.id.to_string())
        .await
        .map_err(|_| ServiceError::BadRequest("Could not delete file from S3".to_string()))?;

    diesel::delete(
        files_columns::files
            .filter(files_columns::id.eq(file_uuid))
            .filter(files_columns::dataset_id.eq(dataset.clone().id)),
    )
    .execute(&mut conn)
    .await
    .map_err(|e| {
        log::error!("Error deleting file {:?}", e);
        ServiceError::BadRequest("Could not delete file".to_string())
    })?;

    if delete_chunks.is_some_and(|delete_chunks| delete_chunks) {
        delete_group_by_file_id_query(
            file_uuid,
            dataset,
            chrono::Utc::now().naive_utc(),
            Some(true),
            pool.clone(),
            dataset_config,
        )
        .await?;
    }

    Ok(())
}
