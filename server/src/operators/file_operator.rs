use std::collections::HashMap;

use super::chunk_operator::{create_chunk_metadata, get_row_count_for_organization_id_query};
use super::event_operator::create_event_query;
use super::group_operator::{create_groups_from_file_query, create_groups_query};
use super::parse_operator::{build_chunking_regex, coarse_doc_chunker, convert_html_to_text};
use crate::data::models::ChunkGroup;
use crate::data::models::{Dataset, DatasetAndOrgWithSubAndPlan, DatasetConfiguration, EventType};
use crate::data::models::{FileDTO, FileGroup};
use crate::handlers::chunk_handler::ChunkReqPayload;
use crate::handlers::file_handler::UploadFileReqPayload;
use crate::{data::models::Event, get_env};
use crate::{
    data::models::{File, Pool},
    errors::ServiceError,
};
use actix_web::web;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::BigInt;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::{AsyncConnection, RunQueryDsl};
use itertools::Itertools;
use redis::aio::MultiplexedConnection;
use regex::Regex;
use s3::{creds::Credentials, Bucket, Region};
use ureq::json;

#[tracing::instrument]
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
            sentry::capture_message(
                &format!("Could not create or get bucket {:?}", e),
                sentry::Level::Error,
            );
            log::error!("Could not create or get bucket {:?}", e);
            ServiceError::BadRequest("Could not create or get bucket".to_string())
        })?
        .with_path_style();

    Ok(aws_bucket)
}

#[tracing::instrument(skip(pool))]
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
        .map_err(|_| ServiceError::BadRequest("Could not create file, try again".to_string()))?;

    Ok(created_file)
}

#[allow(clippy::too_many_arguments)]
#[tracing::instrument(skip(pool, redis_conn, clickhouse_client))]
pub async fn create_file_chunks(
    created_file_id: uuid::Uuid,
    upload_file_data: UploadFileReqPayload,
    html_content: String,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    clickhouse_client: web::Data<clickhouse::Client>,
    mut redis_conn: MultiplexedConnection,
) -> Result<(), ServiceError> {
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

    let mut chunks: Vec<ChunkReqPayload> = [].to_vec();

    let name = format!("Group for file {}", upload_file_data.file_name);

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

    let file_group = FileGroup::from_details(created_file_id, group_id);

    create_groups_from_file_query(vec![file_group], pool.clone())
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

    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);

    let chunk_segments = chunks
        .chunks(120)
        .map(|chunk_segment| chunk_segment.to_vec())
        .collect::<Vec<Vec<ChunkReqPayload>>>();

    let mut serialized_messages: Vec<String> = vec![];

    for chunk_segment in chunk_segments {
        let (ingestion_message, _) = create_chunk_metadata(
            chunk_segment,
            dataset_org_plan_sub.dataset.id,
            dataset_config.clone(),
            pool.clone(),
        )
        .await?;

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
            .query_async(&mut redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    }

    create_event_query(
        Event::from_details(
            dataset_org_plan_sub.dataset.id,
            EventType::FileUploaded {
                file_id: created_file_id,
                file_name: name,
            },
        ),
        clickhouse_client.clone(),
    )
    .await
    .map_err(|_| ServiceError::BadRequest("Thread error creating notification".to_string()))?;

    Ok(())
}

#[tracing::instrument(skip(pool))]
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

#[tracing::instrument(skip(pool))]
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

#[tracing::instrument(skip(pool))]
pub async fn delete_file_query(
    file_uuid: uuid::Uuid,
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

    let transaction_result = conn
        .transaction::<_, diesel::result::Error, _>(|conn| {
            async {
                diesel::delete(
                    files_columns::files
                        .filter(files_columns::id.eq(file_uuid))
                        .filter(files_columns::dataset_id.eq(dataset.clone().id)),
                )
                .execute(conn)
                .await?;

                Ok(())
            }
            .scope_boxed()
        })
        .await;

    match transaction_result {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error deleting file with transaction {:?}", e);
            return Err(ServiceError::BadRequest("Could not delete file".to_string()).into());
        }
    }

    Ok(())
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct DocumentChunk {
    pub r#type: String,
    pub bbox: Vec<f64>,
    pub properties: serde_json::Value,
    pub text_representation: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
pub struct ArynResponse {
    pub elements: Vec<DocumentChunk>,
    pub status: Vec<String>,
}

pub async fn get_chunks_from_pdf(
    file_data: Vec<u8>,
    aryn_token: String,
) -> Result<ArynResponse, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let form = reqwest::multipart::Form::new()
        .part(
            "pdf",
            reqwest::multipart::Part::bytes(file_data).file_name("file.pdf"),
        )
        .text(
            "options",
            "{\"extract_table_structure\": true, \"use_ocr\": true}",
        );

    let pdf_chunked = client
        .post("https://api.aryn.cloud/v1/document/partition")
        .bearer_auth(aryn_token)
        .multipart(form)
        .send()
        .await?
        .json::<ArynResponse>()
        .await?;

    Ok(pdf_chunked)
}

pub async fn create_groups_and_chunks_from_chunks(
    document_chunks: Vec<DocumentChunk>,
    created_file_id: uuid::Uuid,
    upload_file_data: UploadFileReqPayload,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    clickhouse_client: web::Data<clickhouse::Client>,
    mut redis_conn: MultiplexedConnection,
) -> Result<(), ServiceError> {
    let mut groups_and_chunks: Vec<(ChunkGroup, Vec<ChunkReqPayload>)> = Vec::new();
    let mut current_group: Option<ChunkGroup> = None;
    let mut current_payloads: Vec<ChunkReqPayload> = Vec::new();
    let dataset_id = dataset_org_plan_sub.dataset.id;
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration);

    for chunk in document_chunks.iter() {
        if chunk.r#type == "Section-header" {
            if let Some(ref group) = current_group {
                groups_and_chunks.push((group.clone(), current_payloads));
                current_payloads = Vec::new();
            }
            if let Some(ref text_representation) = chunk.text_representation.clone() {
                current_group = Some(ChunkGroup::from_details(
                    Some(text_representation.clone()),
                    Some(text_representation.clone()),
                    dataset_id,
                    None,
                    None,
                    None,
                ));
                current_payloads.push(ChunkReqPayload {
                    chunk_html: Some(text_representation.clone().replace('\n', "")),
                    semantic_content: None,
                    link: None,
                    tag_set: None,
                    metadata: Some(json!({
                        "bbox": chunk.bbox,
                        "properties": chunk.properties,
                        "type": chunk.r#type
                    })),
                    group_ids: Some(vec![current_group.clone().unwrap().id]),
                    group_tracking_ids: None,
                    location: None,
                    tracking_id: None,
                    upsert_by_tracking_id: None,
                    time_stamp: None,
                    weight: None,
                    split_avg: None,
                    convert_html_to_text: None,
                    image_urls: None,
                    num_value: None,
                    fulltext_boost: None,
                    semantic_boost: None,
                });
            }
        } else if chunk.r#type == "Text" {
            if let Some(ref group) = current_group {
                if let Some(text_representation) = chunk.text_representation.clone() {
                    current_payloads.push(ChunkReqPayload {
                        chunk_html: Some(text_representation.clone().replace('\n', "")),
                        semantic_content: None,
                        link: None,
                        tag_set: None,
                        metadata: Some(json!({
                            "bbox": chunk.bbox,
                            "properties": chunk.properties,
                            "type": chunk.r#type
                        })),
                        group_ids: Some(vec![group.clone().id]),
                        group_tracking_ids: None,
                        location: None,
                        tracking_id: None,
                        upsert_by_tracking_id: None,
                        time_stamp: None,
                        weight: None,
                        split_avg: None,
                        convert_html_to_text: None,
                        image_urls: None,
                        num_value: None,
                        fulltext_boost: None,
                        semantic_boost: None,
                    });
                }
            }
        }
    }

    if let Some(group) = current_group {
        groups_and_chunks.push((group, current_payloads));
    }

    let groups = groups_and_chunks
        .iter()
        .map(|(group, _)| group.clone())
        .dedup_by(|a, b| a.id == b.id)
        .collect::<Vec<ChunkGroup>>();

    create_groups_query(groups, true, pool.clone())
        .await
        .map_err(|e| {
            log::error!("Could not create group {:?}", e);
            ServiceError::BadRequest("Could not create group".to_string())
        })?;

    let file_groups = groups_and_chunks
        .iter()
        .map(|(group, _)| FileGroup::from_details(created_file_id, group.id))
        .collect::<Vec<FileGroup>>();

    create_groups_from_file_query(file_groups, pool.clone())
        .await
        .map_err(|e| {
            log::error!("Could not create group from file {:?}", e);
            e
        })?;

    let chunk_count = get_row_count_for_organization_id_query(
        dataset_org_plan_sub.organization.organization.id,
        pool.clone(),
    )
    .await?;

    let chunk_len = groups_and_chunks
        .iter()
        .map(|(_, chunks)| chunks.len())
        .sum::<usize>();

    if chunk_count + chunk_len
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

    let segmented_chunks = groups_and_chunks
        .into_iter()
        .flat_map(|(_, chunks)| chunks)
        .collect::<Vec<ChunkReqPayload>>()
        .chunks(120)
        .map(|chunk_segment| chunk_segment.to_vec())
        .collect::<Vec<Vec<ChunkReqPayload>>>();

    for payloads in segmented_chunks {
        let (ingestion_message, _) =
            create_chunk_metadata(payloads, dataset_id, dataset_config.clone(), pool.clone())
                .await?;

        let serialized_message: String =
            serde_json::to_string(&ingestion_message).map_err(|_| {
                ServiceError::BadRequest("Failed to Serialize BulkUploadMessage".to_string())
            })?;

        if serialized_message.is_empty() {
            continue;
        }

        redis::cmd("lpush")
            .arg("ingestion")
            .arg(&serialized_message)
            .query_async(&mut redis_conn)
            .await
            .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    }

    create_event_query(
        Event::from_details(
            dataset_org_plan_sub.dataset.id,
            EventType::FileUploaded {
                file_id: created_file_id,
                file_name: upload_file_data.file_name.clone(),
            },
        ),
        clickhouse_client.clone(),
    )
    .await
    .map_err(|_| ServiceError::BadRequest("Thread error creating notification".to_string()))?;

    Ok(())
}
