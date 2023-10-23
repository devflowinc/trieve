use std::path::PathBuf;

use crate::{
    data::models::{File, Pool},
    errors::ServiceError,
    operators::file_operator::{
        convert_doc_to_html_query, delete_file_query, get_file_query, get_user_file_query,
        get_user_id_of_file_query, update_file_query,
    },
};
use actix_files::NamedFile;
use actix_web::{web, HttpResponse};
use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use serde::{Deserialize, Serialize};

use super::auth_handler::{LoggedUser, RequireAuth};
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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadFileData {
    pub base64_docx_file: String,
    pub file_name: String,
    pub file_mime_type: String,
    pub private: bool,
    pub tag_set: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadFileResult {
    pub file_metadata: File,
}

pub async fn upload_file_handler(
    data: web::Json<UploadFileData>,
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

    let private = upload_file_data.private;

    let file_mime = upload_file_data.file_mime_type;

    let conversion_result = convert_doc_to_html_query(
        upload_file_data.file_name,
        decoded_file_data,
        file_mime,
        upload_file_data.tag_set,
        decoded_description_file_data,
        private,
        user,
        pool_inner,
    )
    .await
    .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    Ok(HttpResponse::Ok().json(conversion_result))
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateFileData {
    pub file_id: uuid::Uuid,
    pub private: bool,
}

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

pub async fn delete_file_handler(
    file_id: web::Path<uuid::Uuid>,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    delete_file_query(file_id.into_inner(), user.id, pool).await?;

    Ok(HttpResponse::NoContent().finish())
}

pub async fn get_image_file(
    file_name: web::Path<String>,
    _user: LoggedUser,
) -> Result<NamedFile, actix_web::Error> {
    let root_dir = "./images";
    // split the path by slashes and make surer there are no .. in the path
    if let Some(name) = file_name.to_string().split('/').last() {
        if name.contains("..") {
            return Err(ServiceError::BadRequest("Invalid file name".to_string()).into());
        }

        let file_path: PathBuf = format!("{}/{}", root_dir, name).into();

        if file_path.exists() {
            return Ok(NamedFile::open(file_path)?);
        }
    }

    Err(ServiceError::BadRequest("Invalid file name, not found".to_string()).into())
}
