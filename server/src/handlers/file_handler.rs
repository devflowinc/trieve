use super::{
    auth_handler::{AdminOnly, LoggedUser},
    group_handler::DeleteGroupData,
};
use crate::{
    data::models::{
        ChunkReqPayloadMappings, CsvJsonlWorkerMessage, DatasetAndOrgWithSubAndPlan,
        DatasetConfiguration, File, FileAndGroupId, FileWorkerMessage, Pool, RedisPool,
    },
    errors::ServiceError,
    middleware::auth_middleware::verify_member,
    operators::{
        crawl_operator::{process_crawl_doc, Document},
        file_operator::{
            create_file_query, delete_file_query, get_aws_bucket, get_csvjsonl_aws_bucket,
            get_dataset_file_query, get_file_query,
        },
        organization_operator::{get_file_size_sum_org, hash_function},
    },
};
use actix_web::{web, HttpResponse};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use broccoli_queue::queue::BroccoliQueue;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

pub fn validate_file_name(s: String) -> Result<String, actix_web::Error> {
    let split_s = s.split('/').last();

    if let Some(name) = split_s {
        if name.contains("..") {
            return Err(ServiceError::BadRequest("Invalid file name".to_string()).into());
        }

        return Ok(name.to_string());
    }

    Err(ServiceError::BadRequest("Invalid file name".to_string()).into())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(example = json!({
    "file_name": "example.pdf",
    "base64_file": "<base64_encoded_file>",
    "tag_set": ["tag1", "tag2"],
    "description": "This is an example file",
    "link": "https://example.com",
    "time_stamp": "2021-01-01 00:00:00.000Z",
    "metadata": {
        "key1": "value1",
        "key2": "value2"
    },
    "create_chunks": true,
    "split_delimiters": [",",".","\n"],
    "target_splits_per_chunk": 20,
    "use_pdf2md_ocr": false
}))]
pub struct UploadFileReqPayload {
    /// Base64 encoded file. This is the standard base64url encoding.
    pub base64_file: String,
    /// Name of the file being uploaded, including the extension.
    pub file_name: String,
    /// Tag set is a comma separated list of tags which will be passed down to the chunks made from the file. Tags are used to filter chunks when searching. HNSW indices are created for each tag such that there is no performance loss when filtering on them.
    pub tag_set: Option<Vec<String>>,
    /// Description is an optional convience field so you do not have to remember what the file contains or is about. It will be included on the group resulting from the file which will hold its chunk.
    pub description: Option<String>,
    /// Link to the file. This can also be any string. This can be used to filter when searching for the file's resulting chunks. The link value will not affect embedding creation.
    pub link: Option<String>,
    /// Time stamp should be an ISO 8601 combined date and time without timezone. Time_stamp is used for time window filtering and recency-biasing search results. Will be passed down to the file's chunks.
    pub time_stamp: Option<String>,
    /// Metadata is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata. Will be passed down to the file's chunks.
    pub metadata: Option<serde_json::Value>,
    /// Create chunks is a boolean which determines whether or not to create chunks from the file. If false, you can manually chunk the file and send the chunks to the create_chunk endpoint with the file_id to associate chunks with the file. Meant mostly for advanced users.
    pub create_chunks: Option<bool>,
    /// Rebalance chunks is an optional field which allows you to specify whether or not to rebalance the chunks created from the file. If not specified, the default true is used. If true, Trieve will evenly distribute remainder splits across chunks such that 66 splits with a `target_splits_per_chunk` of 20 will result in 3 chunks with 22 splits each.
    pub rebalance_chunks: Option<bool>,
    /// Split delimiters is an optional field which allows you to specify the delimiters to use when splitting the file before chunking the text. If not specified, the default [.!?\n] are used to split into sentences. However, you may want to use spaces or other delimiters.
    pub split_delimiters: Option<Vec<String>>,
    /// Target splits per chunk. This is an optional field which allows you to specify the number of splits you want per chunk. If not specified, the default 20 is used. However, you may want to use a different number.
    pub target_splits_per_chunk: Option<usize>,
    /// Group tracking id is an optional field which allows you to specify the tracking id of the group that is created from the file. Chunks created will be created with the tracking id of `group_tracking_id|<index of chunk>`
    pub group_tracking_id: Option<String>,
    /// Parameter to use pdf2md_ocr. If true, the file will be converted to markdown using gpt-4o. Default is false.
    pub pdf2md_options: Option<Pdf2MdOptions>,
    /// Split average will automatically split your file into multiple chunks and average all of the resulting vectors into a single output chunk. Default is false. Explicitly enabling this will cause each file to only produce a single chunk.
    pub split_avg: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Pdf2MdOptions {
    /// Parameter to use pdf2md_ocr. If true, the file will be converted to markdown using gpt-4o. Default is false.
    pub use_pdf2md_ocr: bool,
    /// Prompt to use for the gpt-4o model. Default is None.
    pub system_prompt: Option<String>,
    /// Split headings is an optional field which allows you to specify whether or not to split headings into separate chunks. Default is false.
    pub split_headings: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UploadFileResponseBody {
    /// File object information. Id, name, tag_set, etc.
    pub file_metadata: File,
}

/// Upload File
///
/// Upload a file to S3 bucket attached to your dataset. You can select between a naive chunking strategy where the text is extracted with Apache Tika and split into segments with a target number of segments per chunk OR you can use a vision LLM to convert the file to markdown and create chunks per page. Auth'ed user must be an admin or owner of the dataset's organization to upload a file.
#[utoipa::path(
    post,
    path = "/file",
    context_path = "/api",
    tag = "File",
    request_body(content = UploadFileReqPayload, description = "JSON request payload to upload a file", content_type = "application/json"),
    responses(
        (status = 200, description = "Confirmation that the file is uploading", body = UploadFileResponseBody),
        (status = 400, description = "Service error relating to uploading the file", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn upload_file_handler(
    data: web::Json<UploadFileReqPayload>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    // Disallow split_avg with pdf2md
    if data.pdf2md_options.is_some() && data.split_avg.unwrap_or(false) {
        return Err(
            ServiceError::BadRequest("split_avg is not supported with pdf2md".to_string()).into(),
        );
    }

    let file_size_sum_pool = pool.clone();
    let file_size_sum = get_file_size_sum_org(
        dataset_org_plan_sub.organization.organization.id,
        file_size_sum_pool,
    )
    .await
    .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    if file_size_sum
        >= dataset_org_plan_sub
            .clone()
            .organization
            .plan
            .unwrap_or_default()
            .file_storage
    {
        return Err(ServiceError::BadRequest("File size limit reached".to_string()).into());
    }

    let upload_file_data = data.into_inner();

    let mut cleaned_base64 = upload_file_data
        .base64_file
        .replace('+', "-")
        .replace('/', "_");
    if cleaned_base64.ends_with('=') {
        cleaned_base64.pop();
    }
    let base64_engine = engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

    let decoded_file_data = base64_engine
        .decode(upload_file_data.base64_file.clone())
        .map_err(|_e| ServiceError::BadRequest("Could not decode base64 file".to_string()))?;

    let file_id = uuid::Uuid::new_v4();

    let bucket = get_aws_bucket()?;

    if upload_file_data.file_name.clone().ends_with(".pdf") {
        bucket
            .put_object_with_content_type(
                file_id.to_string(),
                decoded_file_data.as_slice(),
                "application/pdf",
            )
            .await
            .map_err(|e| {
                log::error!("Could not upload file to S3 {:?}", e);
                ServiceError::BadRequest("Could not upload file to S3".to_string())
            })?;
    } else {
        bucket
            .put_object(file_id.to_string(), decoded_file_data.as_slice())
            .await
            .map_err(|e| {
                log::error!("Could not upload file to S3 {:?}", e);
                ServiceError::BadRequest("Could not upload file to S3".to_string())
            })?;
    }

    let file_size_mb = (decoded_file_data.len() as f64 / 1024.0 / 1024.0).round() as i64;

    create_file_query(
        file_id,
        file_size_mb,
        upload_file_data.clone(),
        dataset_org_plan_sub.dataset.id,
        pool.clone(),
    )
    .await?;

    let message = FileWorkerMessage {
        file_id,
        dataset_id: dataset_org_plan_sub.dataset.id,
        upload_file_data: upload_file_data.clone(),
        attempt_number: 0,
    };

    let serialized_message = serde_json::to_string(&message).map_err(|e| {
        log::error!("Could not serialize message: {:?}", e);
        ServiceError::BadRequest("Could not serialize message".to_string())
    })?;

    redis::cmd("lpush")
        .arg("file_ingestion")
        .arg(&serialized_message)
        .query_async::<_, ()>(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let result = UploadFileResponseBody {
        file_metadata: File::from_details(
            Some(file_id),
            &upload_file_data.file_name,
            decoded_file_data.len().try_into().unwrap_or_default(),
            upload_file_data
                .tag_set
                .map(|t| t.into_iter().map(Some).collect()),
            upload_file_data.metadata.clone(),
            upload_file_data.link.clone(),
            upload_file_data.time_stamp.clone(),
            dataset_org_plan_sub.dataset.id,
        ),
    };

    Ok(HttpResponse::Ok().json(result))
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UploadHtmlPageReqPayload {
    pub data: Document,
    pub metadata: serde_json::Value,
    pub scrape_id: uuid::Uuid,
}

/// Upload HTML Page
///
/// Chunk HTML by headings and queue for indexing into the specified dataset.
#[utoipa::path(
    post,
    path = "/file/html_page",
    context_path = "/api",
    tag = "File",
    request_body(content = UploadHtmlPageReqPayload, description = "JSON request payload to upload a file", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that html is being processed"),
        (status = 400, description = "Service error relating to processing the file", body = ErrorResponseBody),
    ),
)]
pub async fn upload_html_page(
    data: web::Json<UploadHtmlPageReqPayload>,
    broccoli_queue: web::Data<BroccoliQueue>,
) -> Result<HttpResponse, actix_web::Error> {
    let req_payload = data.into_inner();

    let dataset_id = req_payload
        .metadata
        .as_object()
        .ok_or_else(|| {
            ServiceError::BadRequest("metadata field must be a JSON object".to_string())
        })?
        .get("dataset_id")
        .ok_or_else(|| {
            ServiceError::BadRequest("metadata field is required to specify dataset_id".to_string())
        })?
        .as_str()
        .ok_or_else(|| {
            ServiceError::BadRequest("metadata field must have a valid dataset_id".to_string())
        })?
        .parse::<uuid::Uuid>()
        .map_err(|_| {
            log::error!("metadata field must have a valid dataset_id");
            ServiceError::BadRequest("metadata field must have a valid dataset_id".to_string())
        })?;

    let webhook_secret = req_payload
        .metadata
        .as_object()
        .ok_or_else(|| {
            ServiceError::BadRequest("metadata field must be a JSON object".to_string())
        })?
        .get("webhook_secret")
        .ok_or_else(|| {
            ServiceError::BadRequest("metadata field is required to specify dataset_id".to_string())
        })?
        .as_str()
        .ok_or_else(|| {
            ServiceError::BadRequest("metadata field must have a valid dataset_id".to_string())
        })?
        .parse::<String>()
        .map_err(|_| {
            log::error!("metadata field must have a valid dataset_id");
            ServiceError::BadRequest("metadata field must have a valid dataset_id".to_string())
        })?;

    let cur_secret = hash_function(
        std::env::var("STRIPE_WEBHOOK_SECRET")
            .unwrap_or("firecrawl".to_string())
            .as_str(),
    );

    if webhook_secret != cur_secret {
        log::error!("Webhook secret does not match.");
        return Err(ServiceError::BadRequest("Webhook secret does not match.".to_string()).into());
    }

    process_crawl_doc(dataset_id, req_payload.data, broccoli_queue).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Deserialize, Serialize, ToSchema, Default)]
pub struct FileSignedUrlOptions {
    content_type: Option<String>,
}

/// Get File Signed URL
///
/// Get a signed s3 url corresponding to the file_id requested such that you can download the file.
#[utoipa::path(
    get,
    path = "/file/{file_id}",
    context_path = "/api",
    tag = "File",
    responses(
        (status = 200, description = "The file's information and s3_url where the original file can be downloaded", body = FileDTO),
        (status = 400, description = "Service error relating to finding the file", body = ErrorResponseBody),
        (status = 404, description = "File not found", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("file_id" = uuid::Uuid, description = "The id of the file to fetch"),
        ("content_type" = Option<String>, Query, description = "Optional field to override the presigned url's Content-Type header"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_file_handler(
    file_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    options: web::Query<FileSignedUrlOptions>,
) -> Result<HttpResponse, actix_web::Error> {
    let file = get_file_query(
        file_id.into_inner(),
        dataset_org_plan_sub.dataset.id,
        options.into_inner().content_type,
        pool,
    )
    .await?;

    Ok(HttpResponse::Ok().json(file))
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[schema(example = json!({
    "file_name": "example.pdf",
    "tag_set": ["tag1", "tag2"],
    "description": "This is an example file",
    "link": "https://example.com",
    "time_stamp": "2021-01-01 00:00:00.000Z",
    "metadata": {
        "key1": "value1",
        "key2": "value2"
    },
}))]
pub struct CreatePresignedUrlForCsvJsonlReqPayload {
    /// Name of the file being uploaded, including the extension. Will be used to determine CSV or JSONL for processing.
    pub file_name: String,
    /// Tag set is a comma separated list of tags which will be passed down to the chunks made from the file. Each tag will be joined with what's creatd per row of the CSV or JSONL file.
    pub tag_set: Option<Vec<String>>,
    /// Description is an optional convience field so you do not have to remember what the file contains or is about. It will be included on the group resulting from the file which will hold its chunk.
    pub description: Option<String>,
    /// Link to the file. This can also be any string. This can be used to filter when searching for the file's resulting chunks. The link value will not affect embedding creation.
    pub link: Option<String>,
    /// Time stamp should be an ISO 8601 combined date and time without timezone. Time_stamp is used for time window filtering and recency-biasing search results. Will be passed down to the file's chunks.
    pub time_stamp: Option<String>,
    /// Metadata is a JSON object which can be used to filter chunks. This is useful for when you want to filter chunks by arbitrary metadata. Unlike with tag filtering, there is a performance hit for filtering on metadata. Will be passed down to the file's chunks.
    pub metadata: Option<serde_json::Value>,
    /// Group tracking id is an optional field which allows you to specify the tracking id of the group that is created from the file. Chunks created will be created with the tracking id of `group_tracking_id|<index of chunk>`
    pub group_tracking_id: Option<String>,
    /// Specify all of the mappings between columns or fields in a CSV or JSONL file and keys in the ChunkReqPayload. Array fields like tag_set and image_urls can have multiple mappings. Boost phrase can also have multiple mappings which get concatenated. Other fields can only have one mapping and only the last mapping will be used.
    pub mappings: Option<ChunkReqPayloadMappings>,
    /// Upsert by tracking_id. If true, chunks will be upserted by tracking_id. If false, chunks with the same tracking_id as another already existing chunk will be ignored. Defaults to true.
    pub upsert_by_tracking_id: Option<bool>,
    /// Amount to multiplicatevly increase the frequency of the tokens in the boost phrase for each row's chunk by. Applies to fulltext (SPLADE) and keyword (BM25) search.
    pub fulltext_boost_factor: Option<f64>,
    /// Arbitrary float (positive or negative) specifying the multiplicate factor to apply before summing the phrase vector with the chunk_html embedding vector. Applies to semantic (embedding model) search.
    pub semantic_boost_factor: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CreatePresignedUrlForCsvJsonResponseBody {
    /// File object information. Id, name, tag_set, etc.
    pub file_metadata: File,
    /// Signed URL to upload the file to.
    pub presigned_put_url: String,
}

/// Create Presigned CSV/JSONL S3 PUT URL
///
/// This route is useful for uploading very large CSV or JSONL files. Once you have completed the upload, chunks will be automatically created from the file for each line in the CSV or JSONL file. The chunks will be indexed and searchable. Auth'ed user must be an admin or owner of the dataset's organization to upload a file.
#[utoipa::path(
    post,
    path = "/file/csv_or_jsonl",
    context_path = "/api",
    tag = "File",
    request_body(content = CreatePresignedUrlForCsvJsonlReqPayload, description = "JSON request payload to upload a CSV or JSONL file", content_type = "application/json"),
    responses(
        (status = 200, description = "File object information and signed put URL", body = CreatePresignedUrlForCsvJsonResponseBody),
        (status = 400, description = "Service error relating to uploading the file", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn create_presigned_url_for_csv_jsonl(
    data: web::Json<CreatePresignedUrlForCsvJsonlReqPayload>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let create_presigned_put_url_data = data.into_inner();

    let file_id = uuid::Uuid::new_v4();

    let bucket = get_csvjsonl_aws_bucket()?;
    let presigned_put_url = bucket
        .presign_put(file_id.to_string(), 86400, None, None)
        .await
        .map_err(|e| {
            log::error!("Could not get presigned put url: {:?}", e);
            ServiceError::BadRequest("Could not get presigned put url".to_string())
        })?;

    let message = CsvJsonlWorkerMessage {
        file_id,
        dataset_id: dataset_org_plan_sub.dataset.id,
        create_presigned_put_url_data: create_presigned_put_url_data.clone(),
        created_at: chrono::Utc::now().naive_utc(),
        attempt_number: 0,
    };

    let serialized_message = serde_json::to_string(&message).map_err(|e| {
        log::error!("Could not serialize message: {:?}", e);
        ServiceError::BadRequest("Could not serialize message".to_string())
    })?;

    redis::cmd("lpush")
        .arg("csv_jsonl_ingestion")
        .arg(&serialized_message)
        .query_async::<_, ()>(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let result = CreatePresignedUrlForCsvJsonResponseBody {
        file_metadata: File::from_details(
            Some(file_id),
            &create_presigned_put_url_data.file_name,
            0,
            create_presigned_put_url_data
                .tag_set
                .map(|t| t.into_iter().map(Some).collect()),
            create_presigned_put_url_data.metadata.clone(),
            create_presigned_put_url_data.link.clone(),
            create_presigned_put_url_data.time_stamp.clone(),
            dataset_org_plan_sub.dataset.id,
        ),
        presigned_put_url,
    };

    Ok(HttpResponse::Ok().json(result))
}

#[derive(Deserialize, Debug, Serialize, ToSchema)]
pub struct DatasetFileQuery {
    pub dataset_id: uuid::Uuid,
    pub page: u64,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct FileData {
    pub file_and_group_ids: Vec<FileAndGroupId>,
    pub total_pages: i64,
}

/// Get Files for Dataset
///
/// Get all files which belong to a given dataset specified by the dataset_id parameter. 10 files are returned per page.
#[utoipa::path(
    get,
    path = "/dataset/files/{dataset_id}/{page}",
    context_path = "/api",
    tag = "File",
    responses(
        (status = 200, description = "JSON body representing the files in the current dataset", body = FileData),
        (status = 400, description = "Service error relating to getting the files in the current datase", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("dataset_id" = uuid::Uuid, description = "The id of the dataset to fetch files for."),
        ("page" = u64, description = "The page number of files you wish to fetch. Each page contains at most 10 files."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
pub async fn get_dataset_files_handler(
    data: web::Path<DatasetFileQuery>,
    pool: web::Data<Pool>,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    required_user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let data = data.into_inner();
    if dataset_org_plan_sub.dataset.id != data.dataset_id {
        return Err(ServiceError::BadRequest(
            "Dataset header does not match given path".to_string(),
        )
        .into());
    }
    if !verify_member(
        &required_user,
        &dataset_org_plan_sub.organization.organization.id,
    ) {
        return Err(ServiceError::Forbidden.into());
    }

    let files = get_dataset_file_query(data.dataset_id, data.page, pool).await?;

    Ok(HttpResponse::Ok().json(FileData {
        file_and_group_ids: files
            .iter()
            .map(|f| FileAndGroupId {
                file: f.0.clone(),
                group_id: f.2,
            })
            .collect(),
        total_pages: files
            .first()
            .map(|file| (file.1 as f64 / 10.0).ceil() as i64)
            .unwrap_or(1),
    }))
}
/// Delete File
///
/// Delete a file from S3 attached to the server based on its id. This will disassociate chunks from the file, but only delete them all together if you specify delete_chunks to be true. Auth'ed user or api key must have an admin or owner role for the specified dataset's organization.
#[utoipa::path(
    delete,
    path = "/file/{file_id}",
    context_path = "/api",
    tag = "File",
    responses(
        (status = 204, description = "Confirmation that the file has been deleted"),
        (status = 400, description = "Service error relating to finding or deleting the file", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = uuid::Uuid, Header, description = "The dataset id or tracking_id to use for the request. We assume you intend to use an id if the value is a valid uuid."),
        ("file_id" = uuid::Uuid, description = "The id of the file to delete"),
        ("delete_chunks" = bool, Query, description = "Delete the chunks within the group"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
pub async fn delete_file_handler(
    file_id: web::Path<uuid::Uuid>,
    query: web::Query<DeleteGroupData>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let dataset_config =
        DatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration.clone());

    delete_file_query(
        file_id.into_inner(),
        query.delete_chunks,
        dataset_org_plan_sub.dataset,
        pool,
        dataset_config,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GetImageResponse {
    pub signed_url: String,
}

pub async fn get_signed_url(
    file_name: web::Path<String>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let bucket = get_aws_bucket()?;

    let unlimited = std::env::var("UNLIMITED")
        .unwrap_or("false".to_string())
        .parse()
        .unwrap_or(false);
    let s3_path = match unlimited {
        true => "files".to_string(),
        false => dataset_org_plan_sub
            .organization
            .organization
            .id
            .to_string(),
    };

    let signed_url = bucket
        .presign_get(format!("{}/{}", s3_path, file_name.into_inner()), 300, None)
        .await
        .map_err(|e| {
            log::error!("Error getting signed url: {}", e);
            ServiceError::BadRequest(format!("Error getting signed url: {}", e))
        })?;

    Ok(HttpResponse::Ok().json(GetImageResponse { signed_url }))
}
