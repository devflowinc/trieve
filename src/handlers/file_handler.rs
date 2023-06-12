use crate::{
    data::models::Pool,
    errors::{DefaultError, ServiceError},
    operators::file_operator::{convert_docx_to_html_query, CoreCard},
};
use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use futures_util::TryStreamExt;
use serde::{Deserialize, Serialize};

use super::auth_handler::LoggedUser;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadFileResult {
    pub created_cards: Vec<CoreCard>,
    pub rejected_cards: Vec<CoreCard>,
}

pub async fn upload_file_handler(
    mut payload: Multipart,
    pool: web::Data<Pool>,
    user: LoggedUser,
) -> Result<HttpResponse, actix_web::Error> {
    let pool_inner = pool.clone();
    let mut file_data = Vec::new();
    let mut file_field: actix_multipart::Field = match payload.try_next().await? {
        Some(field) => {
            if field.content_disposition().get_name() == Some("docx_file") {
                field
            } else {
                return Ok(HttpResponse::BadRequest().json(DefaultError {
                    message: "Must include only docx_file key in form data",
                }));
            }
        }
        None => {
            return Ok(HttpResponse::BadRequest().json(DefaultError {
                message: "Must include only docx_file key in form data",
            }))
        }
    };

    let content_disposition = file_field.content_disposition();
    let file_name = match content_disposition.get_filename() {
        Some(name) => {
            match name.rsplit_once('.') {
                Some((_, extension)) => {
                    if extension == "docx" {
                        "docx"
                    } else {
                        return Ok(HttpResponse::BadRequest().json(DefaultError {
                            message: "Must upload a docx file",
                        }));
                    }
                }
                None => {
                    return Ok(HttpResponse::BadRequest().json(DefaultError {
                        message: "Must upload a docx file",
                    }));
                }
            };
            name.to_owned()
        }
        None => {
            return Ok(HttpResponse::BadRequest().json(DefaultError {
                message: "Must upload a docx file",
            }));
        }
    };

    while let Some(chunk) = file_field.try_next().await? {
        file_data.extend_from_slice(&chunk);
    }

    let conversion_result = convert_docx_to_html_query(file_name, file_data, user, pool_inner)
        .await
        .map_err(|e| ServiceError::BadRequest(e.message.to_string()))?;

    Ok(HttpResponse::Ok().json(conversion_result))
}
