use std::io::Write;

use super::auth_handler::{AdminOnly, LoggedUser};
use crate::{
    data::models::{DatasetAndOrgWithSubAndPlan, File, Pool, ServerDatasetConfiguration},
    errors::ServiceError,
    operators::{
        file_operator::{
            convert_doc_to_html_query, delete_file_query, get_aws_bucket, get_dataset_file_query,
            get_file_query,
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
pub struct UploadFileData {
    /// Base64 encoded file. Convert + to -, / to _, and remove the ending = if present. This is the standard base64url encoding.
    pub base64_file: String,
    /// Name of the file being uploaded, including the extension.
    pub file_name: String,
    /// MIME type of the file being uploaded.
    pub file_mime_type: String,
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
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UploadFileResult {
    pub file_metadata: File,
}

/// upload_file
///
/// Upload a file to S3 attached to the server. The file will be converted to HTML with tika and chunked algorithmically, images will be OCR'ed with tesseract. The resulting chunks will be indexed and searchable. Optionally, you can only upload the file and manually create chunks associated to the file after. See docs.trieve.ai and/or contact us for more details and tips. Auth'ed user must be an admin or owner of the dataset's organization to upload a file.
#[utoipa::path(
    post,
    path = "/file",
    context_path = "/api",
    tag = "file",
    request_body(content = UploadFileData, description = "JSON request payload to upload a file", content_type = "application/json"),
    responses(
        (status = 200, description = "Confirmation that the file is uploading", body = UploadFileResult),
        (status = 400, description = "Service error relating to uploading the file", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
    ),
    security(
        ("ApiKey" = ["admin"]),
        ("Cookie" = ["admin"])
    )
)]
pub async fn upload_file_handler(
    data: web::Json<UploadFileData>,
    pool: web::Data<Pool>,
    user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    redis_client: web::Data<redis::Client>,
) -> Result<HttpResponse, actix_web::Error> {
    let document_upload_feature = ServerDatasetConfiguration::from_json(
        dataset_org_plan_sub.dataset.server_configuration.clone(),
    )
    .DOCUMENT_UPLOAD_FEATURE
    .unwrap_or(false);

    if document_upload_feature {
        return Err(
            ServiceError::BadRequest("Document upload feature is disabled".to_string()).into(),
        );
    }

    let file_size_sum_pool = pool.clone();
    let file_size_sum = web::block(move || {
        get_file_size_sum_org(dataset_org_plan_sub.organization.id, file_size_sum_pool)
    })
    .await?
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
    let pool_inner = pool.clone();

    let base64_engine = engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

    let decoded_file_data = base64_engine
        .decode(upload_file_data.base64_file)
        .map_err(|_e| ServiceError::BadRequest("Could not decode base64 file".to_string()))?;
    let decoded_description_file_data = if upload_file_data.description.is_some() {
        Some(
            String::from_utf8(
                base64_engine
                    .decode(upload_file_data.description.unwrap_or_default())
                    .map_err(|_e| {
                        ServiceError::BadRequest("Could not decode base64 file".to_string())
                    })?,
            )
            .map_err(|_e| ServiceError::BadRequest("Could not decode base64 file".to_string()))?,
        )
    } else {
        None
    };

    let file_tag_set = upload_file_data
        .tag_set
        .clone()
        .map(|tag_set| tag_set.join(","));

    let conversion_result = convert_doc_to_html_query(
        upload_file_data.file_name,
        decoded_file_data,
        file_tag_set,
        decoded_description_file_data,
        upload_file_data.link,
        upload_file_data.metadata,
        upload_file_data.create_chunks,
        upload_file_data.time_stamp,
        user.0,
        dataset_org_plan_sub.clone(),
        pool_inner,
        redis_client,
    )
    .await
    .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    Ok(HttpResponse::Ok().json(conversion_result))
}

/// get_file
///
/// Download a file from S3 attached to the server based on its id. We plan to add support for getting signed S3 URLs to download from S3 directly in a release soon.
#[utoipa::path(
    get,
    path = "/file/{file_id}",
    context_path = "/api",
    tag = "file",
    responses(
        (status = 200, description = "The signed s3 url corresponding to the file_id requested", body = FileDTO),
        (status = 400, description = "Service error relating to finding the file", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("file_id" = uuid::Uuid, description = "The id of the file to fetch"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
pub async fn get_file_handler(
    file_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    _user: LoggedUser,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    let download_enabled =
        ServerDatasetConfiguration::from_json(dataset_org_plan_sub.dataset.server_configuration)
            .DOCUMENT_DOWNLOAD_FEATURE
            .unwrap_or(false);
    if download_enabled {
        return Err(
            ServiceError::BadRequest("Document download feature is disabled".to_string()).into(),
        );
    }

    let file = get_file_query(file_id.into_inner(), dataset_org_plan_sub.dataset.id, pool).await?;

    Ok(HttpResponse::Ok().json(file))
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct DatasetFileQuery {
    pub dataset_id: uuid::Uuid,
    pub page: u64,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct FileData {
    pub files: Vec<File>,
    pub total_pages: i64,
}

/// get_dataset_files
///
/// Get all files which belong to a given dataset specified by the dataset_id parameter.
#[utoipa::path(
    get,
    path = "/user/files/{user_id}",
    context_path = "/api",
    tag = "file",
    responses(
        (status = 200, description = "JSON body representing the files in the current dataset", body = Vec<File>),
        (status = 400, description = "Service error relating to getting the files in the current datase", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("dataset_id" = uuid::Uuid, description = "The id of the dataset to fetch files for."),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
pub async fn get_dataset_files_handler(
    data: web::Path<DatasetFileQuery>,
    pool: web::Data<Pool>,
    _dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
    _required_user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let data = data.into_inner();

    let files = get_dataset_file_query(data.dataset_id, data.page, pool).await?;

    Ok(HttpResponse::Ok().json(FileData {
        files: files.iter().map(|f| f.0.clone()).collect(),
        total_pages: files
            .first()
            .map(|file| (file.1 as f64 / 10.0).ceil() as i64)
            .unwrap_or(1),
    }))
}

/// delete_file
///
/// Delete a file from S3 attached to the server based on its id. This will disassociate chunks from the file, but will not delete the chunks. We plan to add support for deleting chunks in a release soon. Auth'ed user must be an admin or owner of the dataset's organization to upload a file.
#[utoipa::path(
    delete,
    path = "/file/{file_id}",
    context_path = "/api",
    tag = "file",
    responses(
        (status = 204, description = "Confirmation that the file has been deleted"),
        (status = 400, description = "Service error relating to finding or deleting the file", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("file_id" = uuid::Uuid, description = "The id of the file to delete"),
        ("delete_chunks" = Option<bool>, Query, description = "Whether or not to delete the chunks associated with the file"),
    ),
    security(
        ("ApiKey" = ["admin"]),
        ("Cookie" = ["admin"])
    )
)]
pub async fn delete_file_handler(
    file_id: web::Path<uuid::Uuid>,
    delete_chunks: web::Query<Option<bool>>,
    pool: web::Data<Pool>,
    _user: AdminOnly,
    dataset_org_plan_sub: DatasetAndOrgWithSubAndPlan,
) -> Result<HttpResponse, actix_web::Error> {
    delete_file_query(
        file_id.into_inner(),
        dataset_org_plan_sub.dataset,
        delete_chunks.into_inner(),
        pool,
    )
    .await?;

    Ok(HttpResponse::NoContent().finish())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GetImageResponse {
    pub signed_url: String,
}

/// get_image_file
///
/// We strongly recommend not using this endpoint. It is disabled on the managed version and only meant for niche on-prem use cases where an image directory is mounted. Get in touch with us thru information on docs.trieve.ai for more information.
#[utoipa::path(
    get,
    path = "/image/{file_name}",
    context_path = "/api",
    tag = "file",
    responses(
        (status = 200, description = "The raw image file corresponding to the file_name requested such that it can be a src for an img tag"),
        (status = 400, description = "Service error relating to finding the file", body = ErrorResponseBody),
    ),
    params(
        ("TR-Dataset" = String, Header, description = "The dataset id to use for the request"),
        ("file_name" = string, description = "The name of the image file to return"),
    ),
    security(
        ("ApiKey" = ["readonly"]),
        ("Cookie" = ["readonly"])
    )
)]
pub async fn get_image_file(
    file_name: web::Path<String>,
    _user: LoggedUser,
) -> Result<NamedFile, actix_web::Error> {
    let validated_file_name = validate_file_name(file_name.into_inner())?;

    let bucket = get_aws_bucket().map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    let file_data = bucket
        .get_object(format!("images/{}", validated_file_name).as_str())
        .await
        .map_err(|e| {
            log::error!("Error getting image file: {}", e);
            ServiceError::BadRequest(e.to_string())
        })?;

    let mut file = std::fs::File::create(format!("./tmp/{}", validated_file_name))?;
    file.write_all(file_data.as_slice())?;

    let named_file = NamedFile::open(format!("./tmp/{}", validated_file_name))?;

    std::fs::remove_file(format!("./tmp/{}", validated_file_name.clone()))?;

    Ok(named_file)
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GetPdfFromRangeData {
    pub file_start: u32,
    pub file_end: u32,
    pub prefix: String,
    pub file_name: String,
    pub ocr: Option<bool>,
}

#[allow(unused_variables)]
pub async fn get_pdf_from_range(
    path_data: web::Path<GetPdfFromRangeData>,
    _user: LoggedUser,
) -> Result<NamedFile, actix_web::Error> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ocr")] {

    let validated_prefix = validate_file_name(path_data.prefix.clone())?;

    let mut wand = MagickWand::new();
    let bucket = get_aws_bucket().map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    for i in path_data.file_start..=path_data.file_end {
        let file = bucket
            .get_object(format!("images/{}{}.png", validated_prefix, i).as_str())
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
