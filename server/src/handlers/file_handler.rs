use super::auth_handler::{AdminOnly, LoggedUser};
use crate::{
    data::models::{
        DatasetAndOrgWithSubAndPlan, File, FileAndGroupId, FileWorkerMessage, Pool, RedisPool,
        ServerDatasetConfiguration,
    },
    errors::ServiceError,
    middleware::auth_middleware::verify_member,
    operators::{
        file_operator::{
            delete_file_query, get_aws_bucket, get_dataset_file_query, get_file_query,
        },
        organization_operator::get_file_size_sum_org,
    },
};
use actix_files::NamedFile;
#[cfg(feature = "ocr")]
use actix_web::http::header::ContentDisposition;
use actix_web::{web, HttpResponse};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
#[cfg(feature = "ocr")]
use magick_rust::MagickWand;
#[cfg(feature = "ocr")]
use pyo3::{types::PyDict, Python};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[tracing::instrument]
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
    "file_mime_type": "application/pdf",
    "base64_file": "base64_encoded_file",
    "tag_set": ["tag1", "tag2"],
    "description": "This is an example file",
    "link": "https://example.com",
    "time_stamp": "2021-01-01T00:00:00Z",
    "metadata": {
        "key1": "value1",
        "key2": "value2"
    },
    "create_chunks": true,
    "split_delimiters": [",",".","\n"],
    "target_splits_per_chunk": 20,
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
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UploadFileResult {
    pub file_metadata: File,
}

/// Upload File
///
/// Upload a file to S3 attached to the server. The file will be converted to HTML with tika and chunked algorithmically, images will be OCR'ed with tesseract. The resulting chunks will be indexed and searchable. Optionally, you can only upload the file and manually create chunks associated to the file after. See docs.trieve.ai and/or contact us for more details and tips. Auth'ed user must be an admin or owner of the dataset's organization to upload a file.
#[utoipa::path(
    post,
    path = "/file",
    context_path = "/api",
    tag = "File",
    request_body(content = UploadFileReqPayload, description = "JSON request payload to upload a file", content_type = "application/json"),
    responses(
        (status = 200, description = "Confirmation that the file is uploading", body = UploadFileResult),
        (status = 400, description = "Service error relating to uploading the file", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn upload_file_handler(
    data: web::Json<UploadFileReqPayload>,
    pool: web::Data<Pool>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    redis_pool: web::Data<RedisPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let tx_ctx = sentry::TransactionContext::new("upload_file_handler", "upload_file");
    let transaction = sentry::start_transaction(tx_ctx);
    sentry::configure_scope(|scope| scope.set_span(Some(transaction.clone().into())));

    let mut redis_conn = redis_pool
        .get()
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;

    let get_file_size_span = transaction.start_child("get_file_size_sum", "get_file_size_sum");
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

    get_file_size_span.finish();

    let upload_file_data = data.into_inner();

    let base64_decode_span = transaction.start_child("base64_decode", "base64_decode");
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

    base64_decode_span.finish();

    let file_id = uuid::Uuid::new_v4();

    let bucket_upload_span = transaction.start_child("bucket_upload", "bucket_upload");

    let bucket = get_aws_bucket()?;
    bucket
        .put_object(file_id.to_string(), decoded_file_data.as_slice())
        .await
        .map_err(|e| {
            log::error!("Could not upload file to S3 {:?}", e);
            ServiceError::BadRequest("Could not upload file to S3".to_string())
        })?;

    bucket_upload_span.finish();

    let message = FileWorkerMessage {
        file_id,
        dataset_org_plan_sub: dataset_org_plan_sub.clone(),
        upload_file_data: upload_file_data.clone(),
        attempt_number: 0,
    };

    let serialized_message = serde_json::to_string(&message).map_err(|e| {
        log::error!("Could not serialize message: {:?}", e);
        ServiceError::BadRequest("Could not serialize message".to_string())
    })?;

    let push_to_redis_span = transaction.start_child("push_to_redis", "push_to_redis");
    redis::cmd("lpush")
        .arg("file_ingestion")
        .arg(&serialized_message)
        .query_async(&mut *redis_conn)
        .await
        .map_err(|err| ServiceError::BadRequest(err.to_string()))?;
    push_to_redis_span.finish();

    let result = UploadFileResult {
        file_metadata: File::from_details(
            Some(file_id),
            &upload_file_data.file_name,
            decoded_file_data.len().try_into().unwrap(),
            upload_file_data
                .tag_set
                .map(|t| t.into_iter().map(Some).collect()),
            None,
            None,
            None,
            dataset_org_plan_sub.dataset.id,
        ),
    };

    transaction.finish();

    Ok(HttpResponse::Ok().json(result))
}

/// Get File
///
/// Download a file based on its id.
#[utoipa::path(
    get,
    path = "/file/{file_id}",
    context_path = "/api",
    tag = "File",
    responses(
        (status = 200, description = "The signed s3 url corresponding to the file_id requested", body = FileDTO),
        (status = 400, description = "Service error relating to finding the file", body = ErrorResponseBody),
        (status = 404, description = "File not found", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("file_id" = uuid::Uuid, description = "The id of the file to fetch"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn get_file_handler(
    file_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let file = get_file_query(file_id.into_inner(), dataset_org_plan_sub.dataset.id, pool).await?;

    Ok(HttpResponse::Ok().json(file))
}

#[derive(Deserialize, Debug, Serialize, ToSchema)]
pub struct DatasetFileQuery {
    pub dataset_id: uuid::Uuid,
    pub page: u64,
}
#[derive(Serialize, Deserialize)]
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
        (status = 200, description = "JSON body representing the files in the current dataset", body = Vec<File>),
        (status = 400, description = "Service error relating to getting the files in the current datase", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid::Uuid, description = "The id of the dataset to fetch files for."),
        ("page" = u64, description = "The page number of files you wish to fetch. Each page contains at most 10 files."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
    )
)]
#[tracing::instrument(skip(pool))]
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
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("file_id" = uuid::Uuid, description = "The id of the file to delete"),
    ),
    security(
        ("ApiKey" = ["admin"]),
    )
)]
#[tracing::instrument(skip(pool))]
pub async fn delete_file_handler(
    file_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let server_dataset_config = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    );
    delete_file_query(
        file_id.into_inner(),
        dataset_org_plan_sub.dataset,
        pool,
        server_dataset_config,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GetImageResponse {
    pub signed_url: String,
}

#[tracing::instrument]
pub async fn get_signed_url(
    file_name: web::Path<String>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, ServiceError> {
    let bucket = get_aws_bucket()?;

    let unlimited = std::env::var("UNLIMITED").unwrap_or("false".to_string());
    let s3_path = match unlimited.as_str() {
        "true" => "files".to_string(),
        "false" => dataset_org_plan_sub
            .organization
            .organization
            .id
            .to_string(),
        _ => dataset_org_plan_sub
            .organization
            .organization
            .id
            .to_string(),
    };

    let signed_url = bucket
        .presign_get(format!("{}/{}", s3_path, file_name.into_inner()), 300, None)
        .await
        .map_err(|e| {
            sentry::capture_message(
                &format!("Error getting signed url: {}", e),
                sentry::Level::Error,
            );
            log::error!("Error getting signed url: {}", e);
            ServiceError::BadRequest(format!("Error getting signed url: {}", e))
        })?;

    Ok(HttpResponse::Ok().json(GetImageResponse { signed_url }))
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GetPdfFromRangeData {
    pub organization_id: String,
    pub file_start: u32,
    pub file_end: u32,
    pub prefix: String,
    pub file_name: String,
    pub ocr: Option<bool>,
}

#[allow(unused_variables)]
#[tracing::instrument]
pub async fn get_pdf_from_range(
    path_data: web::Path<GetPdfFromRangeData>,
    _user: LoggedUser,
) -> Result<NamedFile, actix_web::Error> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ocr")] {

    let validated_prefix = validate_file_name(path_data.prefix.clone())?;

    let mut wand = MagickWand::new();
    let bucket = get_aws_bucket()?;

    let unlimited = std::env::var("UNLIMITED").unwrap_or("false".to_string());
    let s3_path = match unlimited.as_str() {
        "true" => "images".to_string(),
        "false" => path_data.organization_id.clone(),
        _ => path_data.organization_id.clone(),
    };

    for i in path_data.file_start..=path_data.file_end {
        let file = bucket
            .get_object(format!("{}/{}{}.png", s3_path, validated_prefix, i).as_str())
            .await
            .map_err(|e| {
                log::error!("Error getting image file: {}", e);
                ServiceError::BadRequest(e.to_string())
            })?;

        wand.read_image_blob(file.as_slice()).map_err(|e| {
            ServiceError::BadRequest(format!("Could not read image to wand: {}", e))
        })?;
    }

    let mut pdf_file_name = path_data.file_name.clone();
    if !pdf_file_name.ends_with(".pdf") {
        pdf_file_name.push_str(".pdf");
    }

    wand.set_filename(pdf_file_name.as_str())
        .map_err(|e| ServiceError::BadRequest(format!("Could not set filename for wand: {}", e)))?;

    let file_path = format!("./tmp/{}-{}", uuid::Uuid::new_v4(), pdf_file_name);

    wand.write_images(file_path.as_str(), true).map_err(|e| {
        ServiceError::BadRequest(format!("Could not write images to pdf with wand: {}", e))
    })?;

    if path_data.ocr.unwrap_or(false) {
        Python::with_gil(|sys| -> Result<(), actix_web::Error> {
            let ocrmypdf = sys.import("ocrmypdf").map_err(|e| {
                ServiceError::BadRequest(format!("Could not import ocrmypdf module: {}", e))
            })?;

            let kwargs = PyDict::new(sys);
            kwargs.set_item("deskew", true).map_err(|e| {
                ServiceError::BadRequest(format!(
                    "Could not set deskew argument for ocrmypdf: {}",
                    e
                ))
            })?;

            ocrmypdf
                .call_method("ocr", (file_path.clone(), file_path.clone()), Some(kwargs))
                .map_err(|e| {
                    ServiceError::BadRequest(format!(
                        "Could not call ocr method for ocrmypdf: {}",
                        e
                    ))
                })?;

            Ok(())
        })?;
    }

    let mut response_file = NamedFile::open(file_path.clone())?;
    let parameters = NamedFile::open(file_path.clone())?
        .content_disposition()
        .parameters
        .clone();

    std::fs::remove_file(file_path)
        .map_err(|e| ServiceError::BadRequest(format!("Could not remove temporary file: {}", e)))?;

    response_file = response_file.set_content_disposition(ContentDisposition {
        disposition: actix_web::http::header::DispositionType::Inline,
        parameters,
    });

    Ok(response_file)
    } else {
       Err(ServiceError::BadRequest("OCR feature not enabled".to_string()).into())
    }
    }
}
