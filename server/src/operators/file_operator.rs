use super::event_operator::create_event_query;
use super::group_operator::{create_group_from_file_query, create_group_query};
use crate::data::models::{ChunkMetadata, Dataset, DatasetAndOrgWithSubAndPlan, EventType};
use crate::handlers::auth_handler::AdminOnly;
use crate::operators::chunk_operator::delete_chunk_metadata_query;
use crate::{data::models::ChunkGroup, handlers::chunk_handler::ReturnCreatedChunk};
use crate::{
    data::models::Event, diesel::Connection, get_env, handlers::chunk_handler::convert_html,
};
use crate::{
    data::models::FileDTO,
    diesel::{ExpressionMethods, QueryDsl},
    errors::ServiceError,
};
use crate::{
    data::models::{File, Pool},
    errors::DefaultError,
    handlers::{
        auth_handler::LoggedUser,
        chunk_handler::{create_chunk, CreateChunkData},
        file_handler::UploadFileResult,
    },
};
use actix_web::{body::MessageBody, web};

use diesel::dsl::sql;
use diesel::sql_types::Integer;
use diesel::{RunQueryDsl, SelectableHelper};
use s3::{creds::Credentials, Bucket, Region};
use std::{path::PathBuf, process::Command};

pub fn get_aws_bucket() -> Result<Bucket, DefaultError> {
    let s3_access_key = get_env!("S3_ACCESS_KEY", "S3_ACCESS_KEY should be set").into();
    let s3_secret_key = get_env!("S3_SECRET_KEY", "S3_SECRET_KEY should be set").into();
    let s3_endpoint = get_env!("S3_ENDPOINT", "S3_ENDPOINT should be set").into();
    let s3_bucket_name = get_env!("S3_BUCKET", "S3_BUCKET should be set");
    let aws_region_name = std::env::var("AWS_REGION").unwrap_or("".to_string());

    let aws_region = Region::Custom {
        region: aws_region_name,
        endpoint: s3_endpoint,
    };

    let aws_credentials = Credentials {
        access_key: Some(s3_access_key),
        secret_key: Some(s3_secret_key),
        security_token: None,
        session_token: None,
        expiration: None,
    };

    let aws_bucket = Bucket::new(s3_bucket_name, aws_region, aws_credentials)
        .map_err(|_| DefaultError {
            message: "Could not create bucket",
        })?
        .with_path_style();

    Ok(aws_bucket)
}

#[allow(clippy::too_many_arguments)]
pub fn create_file_query(
    file_id: uuid::Uuid,
    file_name: &str,
    file_size: i64,
    tag_set: Option<String>,
    metadata: Option<serde_json::Value>,
    link: Option<String>,
    time_stamp: Option<String>,
    dataset_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<File, DefaultError> {
    use crate::data::schema::files::dsl as files_columns;

    let mut conn = pool.get().map_err(|_| DefaultError {
        message: "Could not get database connection",
    })?;

    let new_file = File::from_details(
        Some(file_id),
        file_name,
        file_size,
        tag_set,
        metadata,
        link,
        time_stamp,
        dataset_id,
    );

    let created_file: File = diesel::insert_into(files_columns::files)
        .values(&new_file)
        .get_result(&mut conn)
        .map_err(|_| DefaultError {
            message: "Could not create file, try again",
        })?;

    Ok(created_file)
}

#[allow(clippy::too_many_arguments)]
pub async fn convert_doc_to_html_query(
    file_name: String,
    file_data: Vec<u8>,
    tag_set: Option<String>,
    description: Option<String>,
    link: Option<String>,
    metadata: Option<serde_json::Value>,
    create_chunks: Option<bool>,
    time_stamp: Option<String>,
    user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    redis_client: web::Data<redis::Client>,
) -> Result<UploadFileResult, DefaultError> {
    let file_id = uuid::Uuid::new_v4();
    let file_id_query_clone = file_id;
    let file_name1 = file_name.clone();
    let file_data1 = file_data.clone();
    let tag_set1 = tag_set.clone();
    let dataset_org_plan_sub1 = dataset_org_plan_sub.clone();

    tokio::spawn(async move {
        let new_id = uuid::Uuid::new_v4();
        let uuid_file_name = format!("{}-{}", new_id, file_name.replace('/', ""));
        let glob_string = format!("./tmp/{}*", new_id);

        let temp_html_file_path_buf = std::path::PathBuf::from(&format!(
            "./tmp/{}.html",
            uuid_file_name
                .rsplit_once('.')
                .map(|x| x.0)
                .unwrap_or(&new_id.to_string())
        ));
        let tika_url = std::env::var("TIKA_URL")
            .expect("TIKA_URL must be set")
            .to_string();

        let tika_client = reqwest::Client::new();
        let tika_response = tika_client
            .put(&format!("{}/tika", tika_url))
            .header("Accept", "text/html")
            .body(file_data.clone())
            .send()
            .await
            .map_err(|err| {
                log::error!("Could not send file to tika {:?}", err);
                DefaultError {
                    message: "Could not send file to tika",
                }
            })?;

        let tika_response_bytes = tika_response
            .bytes()
            .await
            .map_err(|err| {
                log::error!("Could not get tika response bytes {:?}", err);
                DefaultError {
                    message: "Could not get tika response bytes",
                }
            })?
            .to_vec();

        std::fs::write(&temp_html_file_path_buf, tika_response_bytes.clone()).map_err(|err| {
            log::error!("Could not write tika response to disk {:?}", err);
            log::error!("Temp file directory {:?}", temp_html_file_path_buf);
            DefaultError {
                message: "Could not write tika response to disk",
            }
        })?;

        // get file metadata from tika
        let tika_metadata_response = tika_client
            .put(&format!("{}/meta", tika_url))
            .header("Accept", "application/json")
            .body(file_data.clone())
            .send()
            .await
            .map_err(|err| {
                log::error!("Could not send file to tika {:?}", err);
                DefaultError {
                    message: "Could not send file to tika",
                }
            })?;

        let mut tika_metadata_response_json: serde_json::Value =
            tika_metadata_response.json().await.map_err(|err| {
                log::error!("Could not get tika metadata response json {:?}", err);
                DefaultError {
                    message: "Could not get tika metadata response json",
                }
            })?;

        if let Some(metadata) = metadata {
            for (key, value) in metadata.as_object().unwrap() {
                tika_metadata_response_json[key] = value.clone();
            }
        }

        let file_size = match file_data.len().try_into() {
            Ok(file_size) => file_size,
            Err(_) => {
                return Err(DefaultError {
                    message: "Could not convert file size to i64",
                })
            }
        };

        let created_file = create_file_query(
            file_id_query_clone,
            &file_name,
            file_size,
            tag_set.clone(),
            Some(tika_metadata_response_json.clone()),
            link.clone(),
            time_stamp.clone(),
            dataset_org_plan_sub1.dataset.id,
            pool.clone(),
        )?;

        let bucket = get_aws_bucket()?;
        bucket
            .put_object(created_file.id.to_string(), file_data.as_slice())
            .await
            .map_err(|e| {
                log::error!("Could not upload file to S3 {:?}", e);
                DefaultError {
                    message: "Could not upload file to S3",
                }
            })?;

        if create_chunks.is_some_and(|create_chunks_bool| !create_chunks_bool) {
            return Ok(());
        }

        let resp = create_chunks_with_handler(
            tag_set,
            file_name,
            created_file.id,
            description,
            Some(tika_metadata_response_json.clone()),
            time_stamp,
            link.clone(),
            user,
            temp_html_file_path_buf,
            glob_string,
            dataset_org_plan_sub1,
            pool,
            redis_client,
        )
        .await;

        if resp.is_err() {
            log::error!("Create chunks with handler failed {:?}", resp);
        }

        Ok(())
    });

    Ok(UploadFileResult {
        file_metadata: File::from_details(
            Some(file_id),
            &file_name1,
            file_data1.len().try_into().unwrap(),
            tag_set1,
            None,
            None,
            None,
            dataset_org_plan_sub.dataset.id,
        ),
    })
}

#[allow(clippy::too_many_arguments)]
pub async fn create_chunks_with_handler(
    tag_set: Option<String>,
    file_name: String,
    created_file_id: uuid::Uuid,
    description: Option<String>,
    metadata: Option<serde_json::Value>,
    time_stamp: Option<String>,
    link: Option<String>,
    user: LoggedUser,
    temp_html_file_path_buf: PathBuf,
    glob_string: String,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    pool: web::Data<Pool>,
    redis_client: web::Data<redis::Client>,
) -> Result<(), DefaultError> {
    let parser_command =
        std::env::var("PARSER_COMMAND").unwrap_or("./server-python/chunker.py".to_string());
    let delete_html_file = || -> Result<(), DefaultError> {
        let files = glob::glob(glob_string.as_str()).expect("Failed to read glob pattern");

        for file in files.flatten() {
            std::fs::remove_file(file).map_err(|_| DefaultError {
                message: "Could not delete temp file",
            })?;
        }

        Ok(())
    };

    let file_path_str = match temp_html_file_path_buf.to_str() {
        Some(file_path_str) => file_path_str,
        None => {
            delete_html_file()?;
            log::error!("HANDLER Could not convert file path to string");
            return Err(DefaultError {
                message: "Could not convert file path to string",
            });
        }
    };

    let parsed_chunks_command_output = Command::new(parser_command).arg(file_path_str).output();

    delete_html_file()?;

    let raw_parsed_chunks = match parsed_chunks_command_output {
        Ok(parsed_chunks_command_output) => parsed_chunks_command_output.stdout,
        Err(_) => {
            log::error!("HANDLER Could not parse chunks");
            return Err(DefaultError {
                message: "Could not parse chunks",
            });
        }
    };

    // raw_parsed_chunks can be serialized to a vector of Strings
    let chunk_htmls: Vec<String> = match serde_json::from_slice(&raw_parsed_chunks) {
        Ok(chunk_htmls) => chunk_htmls,
        Err(err) => {
            log::error!("HANDLER Could not deserialize chunk_htmls {:?}", err);
            return Err(DefaultError {
                message: "Could not deserialize chunk_htmls",
            });
        }
    };

    let mut chunk_ids: Vec<uuid::Uuid> = [].to_vec();

    let split_tag_set: Option<Vec<String>> =
        tag_set.map(|tag_set| tag_set.split(',').map(|x| x.to_string()).collect());

    let name = format!("Group for file {}", file_name);
    let converted_description = convert_html(&description.unwrap_or("".to_string()))?;

    let chunk_group = ChunkGroup::from_details(
        name.clone(),
        converted_description,
        dataset_org_plan_sub.dataset.id,
    );

    let chunk_group = create_group_query(chunk_group, pool.clone()).map_err(|e| {
        log::error!("Could not create group {:?}", e);
        DefaultError {
            message: "Could not create group",
        }
    })?;

    let group_id = chunk_group.id;

    create_group_from_file_query(group_id, created_file_id, pool.clone()).map_err(|e| {
        log::error!("Could not create group from file {:?}", e);
        e
    })?;

    for chunk_html in chunk_htmls {
        let create_chunk_data = CreateChunkData {
            chunk_html: Some(chunk_html.clone()),
            link: link.clone(),
            tag_set: split_tag_set.clone(),
            file_id: Some(created_file_id),
            metadata: metadata.clone(),
            group_ids: Some(vec![group_id]),
            tracking_id: None,
            time_stamp: time_stamp.clone(),
            chunk_vector: None,
            weight: None,
        };
        let web_json_create_chunk_data = web::Json(create_chunk_data);

        match create_chunk(
            web_json_create_chunk_data,
            pool.clone(),
            AdminOnly(user.clone()),
            dataset_org_plan_sub.clone(),
            redis_client.clone(),
        )
        .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    let chunk_metadata: ReturnCreatedChunk = serde_json::from_slice(
                        response.into_body().try_into_bytes().unwrap().as_ref(),
                    )
                    .map_err(|_err| DefaultError {
                        message: "Error creating chunk metadata's for file",
                    })?;
                    chunk_ids.push(chunk_metadata.chunk_metadata.id);
                }
            }
            Err(error) => {
                log::error!("Error creating chunk: {:?}", error.to_string());
            }
        }
    }

    create_event_query(
        Event::from_details(
            dataset_org_plan_sub.dataset.id,
            EventType::FileUploaded {
                group_id,
                file_name: name,
            },
        ),
        pool,
    )
    .map_err(|_| DefaultError {
        message: "Thread error creating notification",
    })?;

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
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let file_metadata: File = files_columns::files
        .filter(files_columns::id.eq(file_uuid))
        .filter(files_columns::dataset_id.eq(dataset_id))
        .get_result(&mut conn)
        .map_err(|_| ServiceError::NotFound)?;

    let bucket = get_aws_bucket().map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;
    let s3_url = bucket
        .presign_get(file_metadata.id.to_string(), 300, None)
        .map_err(|_| ServiceError::BadRequest("Could not get presigned url".to_string()))?;

    let file_dto: FileDTO = file_metadata.into();
    let file_dto: FileDTO = FileDTO { s3_url, ..file_dto };

    Ok(file_dto)
}

pub async fn get_dataset_file_query(
    dataset_id: uuid::Uuid,
    page: u64,
    pool: web::Data<Pool>,
) -> Result<Vec<(File, i32)>, actix_web::Error> {
    use crate::data::schema::files::dsl as files_columns;

    let mut conn = pool
        .clone()
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let file_metadata: Vec<(File, i32)> = files_columns::files
        .filter(files_columns::dataset_id.eq(dataset_id))
        .select((File::as_select(), sql::<Integer>("count(*) OVER()")))
        .limit(10)
        .offset(((page - 1) * 10).try_into().unwrap_or(0))
        .load(&mut conn)
        .map_err(|_| ServiceError::NotFound)?;

    Ok(file_metadata)
}

pub async fn delete_file_query(
    file_uuid: uuid::Uuid,
    dataset: Dataset,
    delete_chunks: Option<bool>,
    pool: web::Data<Pool>,
) -> Result<(), actix_web::Error> {
    use crate::data::schema::chunk_collisions::dsl as chunk_collisions_columns;
    use crate::data::schema::chunk_files::dsl as chunk_files_columns;
    use crate::data::schema::chunk_metadata::dsl as chunk_metadata_columns;
    use crate::data::schema::files::dsl as files_columns;

    let mut conn = pool
        .get()
        .map_err(|_| ServiceError::BadRequest("Could not get database connection".to_string()))?;

    let mut chunk_ids = vec![];
    let mut collisions = vec![];
    let delete_chunks = delete_chunks.unwrap_or(false);

    if delete_chunks {
        let chunks = chunk_metadata_columns::chunk_metadata
            .inner_join(chunk_files_columns::chunk_files)
            .filter(chunk_files_columns::file_id.eq(file_uuid))
            .select(ChunkMetadata::as_select())
            .load::<ChunkMetadata>(&mut conn)
            .map_err(|_| ServiceError::NotFound)?;

        chunk_ids = chunks
            .iter()
            .filter_map(|chunk| {
                if chunk.qdrant_point_id.is_some() {
                    Some(chunk.id)
                } else {
                    None
                }
            })
            .collect();

        collisions = chunks
            .iter()
            .filter_map(|chunk| {
                if chunk.qdrant_point_id.is_none() {
                    Some(chunk.id)
                } else {
                    None
                }
            })
            .collect();
    }

    let file_metadata: File = files_columns::files
        .filter(files_columns::id.eq(file_uuid))
        .filter(files_columns::dataset_id.eq(dataset.id))
        .get_result(&mut conn)
        .map_err(|_| ServiceError::NotFound)?;

    let bucket = get_aws_bucket().map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;
    bucket
        .delete_object(file_metadata.id.to_string())
        .await
        .map_err(|_| ServiceError::BadRequest("Could not delete file from S3".to_string()))?;

    let transaction_result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
        diesel::delete(
            chunk_files_columns::chunk_files.filter(chunk_files_columns::file_id.eq(file_uuid)),
        )
        .execute(conn)?;

        diesel::delete(
            files_columns::files
                .filter(files_columns::id.eq(file_uuid))
                .filter(files_columns::dataset_id.eq(dataset.clone().id)),
        )
        .execute(conn)?;

        if delete_chunks {
            diesel::delete(
                chunk_files_columns::chunk_files
                    .filter(chunk_files_columns::chunk_id.eq_any(collisions.clone())),
            )
            .execute(conn)?;

            diesel::delete(
                chunk_collisions_columns::chunk_collisions
                    .filter(chunk_collisions_columns::chunk_id.eq_any(collisions.clone())),
            )
            .execute(conn)?;
            // there cannot be collisions for a collision, just delete the chunk_metadata without issue
            diesel::delete(
                chunk_metadata_columns::chunk_metadata
                    .filter(chunk_metadata_columns::id.eq_any(collisions.clone()))
                    .filter(chunk_metadata_columns::dataset_id.eq(dataset.id)),
            )
            .execute(conn)?;
        }

        Ok(())
    });

    if delete_chunks {
        for chunk_id in chunk_ids {
            delete_chunk_metadata_query(chunk_id, dataset.clone(), pool.clone())
                .await
                .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;
        }
    }

    match transaction_result {
        Ok(_) => (),
        Err(e) => {
            log::error!("Error deleting file with transaction {:?}", e);
            return Err(ServiceError::BadRequest("Could not delete file".to_string()).into());
        }
    }

    Ok(())
}
