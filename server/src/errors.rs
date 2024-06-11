use actix_web::{error::ResponseError, HttpResponse};
use derive_more::Display;
use diesel::result::{DatabaseErrorKind, Error as DBError};
use serde::{Deserialize, Serialize};
use std::convert::From;
use utoipa::ToSchema;
use uuid::Error as ParseError;

#[derive(Serialize, Deserialize, Debug, Display, ToSchema)]
#[schema(example = json!({"message": "Bad Request"}))]
pub struct ErrorResponseBody {
    pub message: String,
}

#[derive(Debug, Display, Clone)]
pub enum ServiceError {
    #[display(fmt = "Internal Server Error: {_0}")]
    InternalServerError(String),

    #[display(fmt = "BadRequest: {_0}")]
    BadRequest(String),

    #[display(fmt = "BadRequest: Duplicate Tracking Id Found")]
    DuplicateTrackingId(String),

    #[display(fmt = "Unauthorized")]
    Unauthorized,

    #[display(fmt = "Forbidden")]
    Forbidden,

    #[display(fmt = "Not Found")]
    NotFound(String),

    #[display(fmt = "Json Deserialization Error: {_0}")]
    JsonDeserializeError(String),
}

// impl ResponseError trait allows to convert our errors into http responses with appropriate data
impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        sentry::capture_message(&format!("Error {:?}", self), sentry::Level::Error);
        match self {
            ServiceError::InternalServerError(ref message) => HttpResponse::InternalServerError()
                .json(ErrorResponseBody {
                    message: message.to_string(),
                }),
            ServiceError::BadRequest(ref message) => {
                HttpResponse::BadRequest().json(ErrorResponseBody {
                    message: message.to_string(),
                })
            }
            ServiceError::DuplicateTrackingId(ref id) => {
                HttpResponse::BadRequest().json(ErrorResponseBody {
                    message: format!("Stoped overwriting data, Duplicate Tracking Id {:?}", id),
                })
            }
            ServiceError::Unauthorized => HttpResponse::Unauthorized().json(ErrorResponseBody {
                message: "Unauthorized".to_string(),
            }),
            ServiceError::Forbidden => HttpResponse::Forbidden().json(ErrorResponseBody {
                message: "Forbidden".to_string(),
            }),
            ServiceError::NotFound(ref message) => {
                HttpResponse::NotFound().json(ErrorResponseBody {
                    message: format!("Not Found: {}", message),
                })
            }
            ServiceError::JsonDeserializeError(ref message) => {
                HttpResponse::BadRequest().json(ErrorResponseBody {
                    message: format!("Json Deserialization Error: {}", message),
                })
            }
        }
    }
}

// we can return early in our handlers if UUID provided by the user is not valid
// and provide a custom message
impl From<ParseError> for ServiceError {
    fn from(_: ParseError) -> ServiceError {
        ServiceError::BadRequest("Invalid UUID".into())
    }
}

impl From<DBError> for ServiceError {
    fn from(error: DBError) -> ServiceError {
        // Right now we just care about UniqueViolation from diesel
        // But this would be helpful to easily map errors as our app grows
        match error {
            DBError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message = info.details().unwrap_or_else(|| info.message()).to_string();
                    return ServiceError::BadRequest(message);
                }
                ServiceError::InternalServerError("Unknown DB Error. Please try again later".into())
            }
            _ => ServiceError::InternalServerError(
                "Internal Server Error. Please try again later".into(),
            ),
        }
    }
}
