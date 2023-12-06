use super::{auth_handler::{LoggedUser, RequireAuth}, dataset_handler::SlimDataset};
use crate::{
    data::models::{File, Pool},
    errors::ServiceError,
    operators::{
        file_operator::{
            convert_doc_to_html_query, delete_file_query, get_file_query, get_user_file_query,
            get_user_id_of_file_query, update_file_query,
        },
        tantivy_operator::TantivyIndexMap,
    },
    AppMutexStore,
};
use actix_files::NamedFile;
use actix_web::{http::header::ContentDisposition, web, HttpResponse};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use magick_rust::MagickWand;
use pyo3::{types::PyDict, Python};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::RwLock;
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

pub async fn user_owns_file(
    user_id: uuid::Uuid,
    file_id: uuid::Uuid,
    pool: web::Data<Pool>,
) -> Result<(), actix_web::Error> {
    let author_id = web::block(move || get_user_id_of_file_query(file_id, pool))
        .await?
        .map_err(|err| ServiceError::BadRequest(err.message.into()))?;

    if author_id != user_id {
        return Err(ServiceError::Forbidden.into());
    }
    Ok(())
}
#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UploadFileData {
    pub base64_docx_file: String,
    pub file_name: String,
    pub file_mime_type: String,
    pub private: bool,
    pub tag_set: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub time_stamp: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub create_cards: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UploadFileResult {
    pub file_metadata: File,
}

#[utoipa::path(
    post,
    path = "/file",
    context_path = "/api",
    tag = "file",
    request_body(content = UploadFileData, description = "JSON request payload to upload a file", content_type = "application/json"),
    responses(
        (status = 200, description = "Confirmation that the file is uploading", body = [UploadFileResult]),
        (status = 400, description = "Service error relating to uploading the file", body = [DefaultError]),
    ),
)]
pub async fn upload_file_handler(
    data: web::Json<UploadFileData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
    tantivy_index_map: web::Data<RwLock<TantivyIndexMap>>,
    dataset: SlimDataset,
    app_mutex: web::Data<AppMutexStore>,
) -> Result<HttpResponse, actix_web::Error> {
    let dataset_id = dataset.id;
    let document_upload_feature =
        std::env::var("DOCUMENT_UPLOAD_FEATURE").unwrap_or("off".to_string());

    if document_upload_feature != "on" {
        return Err(
            ServiceError::BadRequest("Document upload feature is disabled".to_string()).into(),
        );
    }

    let upload_file_data = data.into_inner();
    let pool_inner = pool.clone();

    let base64_engine = engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

    let decoded_file_data = base64_engine
        .decode(upload_file_data.base64_docx_file)
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

    let conversion_result = convert_doc_to_html_query(
        upload_file_data.file_name,
        decoded_file_data,
        upload_file_data.tag_set,
        decoded_description_file_data,
        upload_file_data.link,
        upload_file_data.private,
        upload_file_data.metadata,
        upload_file_data.create_cards,
        upload_file_data.time_stamp,
        user,
        tantivy_index_map,
        app_mutex,
        dataset_id,
        pool_inner,
    )
    .await
    .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    Ok(HttpResponse::Ok().json(conversion_result))
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct UpdateFileData {
    pub file_id: uuid::Uuid,
    pub private: bool,
}

#[utoipa::path(
    put,
    path = "/file",
    context_path = "/api",
    tag = "file",
    request_body(content = UpdateFileData, description = "JSON request payload to update a file", content_type = "application/json"),
    responses(
        (status = 204, description = "Confirmation that the file is updated"),
        (status = 400, description = "Service error relating to initially processing the file", body = [DefaultError]),
    ),
)]
pub async fn update_file_handler(
    data: web::Json<UpdateFileData>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let document_upload_feature =
        std::env::var("DOCUMENT_UPLOAD_FEATURE").unwrap_or("off".to_string());

    if document_upload_feature != "on" {
        return Err(
            ServiceError::BadRequest("Document upload feature is disabled".to_string()).into(),
        );
    }

    let pool1 = pool.clone();

    user_owns_file(user.id, data.file_id, pool).await?;

    web::block(move || update_file_query(data.file_id, data.private, pool1))
        .await?
        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    path = "/file/{file_id}",
    context_path = "/api",
    tag = "file",
    responses(
        (status = 200, description = "The file corresponding to the file_id requested", body = [FileDTO]),
        (status = 400, description = "Service error relating to finding the file", body = [DefaultError]),
    ),
    params(
        ("file_id" = uuid::Uuid, description = "The id of the file to fetch"),
    ),
)]
pub async fn get_file_handler(
    file_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: Option<LoggedUser>,
    _required_user: RequireAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let download_enabled = std::env::var("DOCUMENT_DOWNLOAD_FEATURE").unwrap_or("off".to_string());
    if download_enabled != "on" {
        return Err(
            ServiceError::BadRequest("Document download feature is disabled".to_string()).into(),
        );
    }

    let user_id = user.map(|user| user.id);

    let file = get_file_query(file_id.into_inner(), user_id, pool).await?;

    Ok(HttpResponse::Ok().json(file))
}

#[utoipa::path(
    get,
    path = "/user/files/{user_id}",
    context_path = "/api",
    tag = "user",
    responses(
        (status = 200, description = "JSON body representing the files uploaded by the given user", body = [Vec<File>]),
        (status = 400, description = "Service error relating to getting the files uploaded by the given user", body = [DefaultError]),
    ),
    params(
        ("user_id" = uuid::Uuid, description = "The id of the user to fetch files for"),
    ),
)]
pub async fn get_user_files_handler(
    user_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: Option<LoggedUser>,
    _required_user: RequireAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let accessing_user_id = user.map(|u| u.id);
    let user_id = user_id.into_inner();

    let files = get_user_file_query(user_id, accessing_user_id, pool).await?;

    Ok(HttpResponse::Ok().json(files))
}

#[utoipa::path(
    delete,
    path = "/file/{file_id}",
    context_path = "/api",
    tag = "file",
    responses(
        (status = 204, description = "Confirmation that the file has been deleted"),
        (status = 400, description = "Service error relating to finding or deleting the file", body = [DefaultError]),
    ),
    params(
        ("file_id" = uuid::Uuid, description = "The id of the file to delete"),
    ),
)]
pub async fn delete_file_handler(
    file_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    delete_file_query(file_id.into_inner(), user.id, pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    path = "/image/{file_name}",
    context_path = "/api",
    tag = "file",
    responses(
        (status = 200, description = "The raw image file corresponding to the file_name requested such that it can be a src for an img tag"),
        (status = 400, description = "Service error relating to finding the file", body = [DefaultError]),
    ),
    params(
        ("file_name" = string, description = "The name of the image file to return"),
    ),
)]
pub async fn get_image_file(
    file_name: web::Path<String>,
    _user: LoggedUser,
) -> Result<NamedFile, actix_web::Error> {
    let root_dir = "./images";
    let validated_file_name = validate_file_name(file_name.into_inner())?;

    let file_path: PathBuf = format!("{}/{}", root_dir, validated_file_name).into();

    if file_path.exists() {
        return Ok(NamedFile::open(file_path)?);
    }

    Err(ServiceError::BadRequest("Invalid file name, not found".to_string()).into())
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct GetPdfFromRangeData {
    pub file_start: u32,
    pub file_end: u32,
    pub prefix: String,
    pub file_name: String,
    pub ocr: Option<bool>,
}

pub async fn get_pdf_from_range(
    path_data: web::Path<GetPdfFromRangeData>,
    _user: LoggedUser,
) -> Result<NamedFile, actix_web::Error> {
    let root_dir = "./images";

    let validated_prefix = validate_file_name(path_data.prefix.clone())?;

    let mut wand = MagickWand::new();

    for i in path_data.file_start..=path_data.file_end {
        let file_path: PathBuf = format!("{}/{}{}.png", root_dir, validated_prefix, i).into();

        if file_path.exists() {
            wand.read_image(file_path.to_str().expect("Could not convert path to str"))
                .map_err(|e| {
                    ServiceError::BadRequest(format!(
                        "Could not read image to wand: {} - {}",
                        e,
                        file_path.to_str().expect("Could not convert path to str")
                    ))
                })?;
        }
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
}
