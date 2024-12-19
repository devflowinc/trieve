use actix_web::{
    error::{JsonPayloadError, ResponseError},
    HttpResponse,
};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use std::convert::From;
use utoipa::ToSchema;
use uuid::Error as ParseError;

#[derive(Serialize, Deserialize, Debug, Display, ToSchema)]
#[schema(example = json!({"message": "Bad Request"}))]
pub struct ErrorResponseBody {
    pub message: String,
}

#[derive(Debug, Display, Clone, Eq, PartialEq, ToSchema)]
pub enum ServiceError {
    #[display("Internal Server Error: {_0}")]
    InternalServerError(String),

    #[display("BadRequest: {_0}")]
    BadRequest(String),

    #[display("BadRequest: Duplicate Tracking Id Found")]
    DuplicateTrackingId(String),

    #[display("Unauthorized")]
    Unauthorized,

    #[display("Forbidden")]
    Forbidden,

    #[display("Not Found")]
    NotFound(String),

    #[display("Json Deserialization Error: {_0}")]
    JsonDeserializeError(String),

    #[display("Payload Too Large")]
    PayloadTooLarge(String),
}

// impl ResponseError trait allows to convert our errors into http responses with appropriate data
impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
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
            ServiceError::PayloadTooLarge(ref message) => {
                HttpResponse::PayloadTooLarge().json(ErrorResponseBody {
                    message: message.to_string(),
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

pub fn custom_json_error_handler(
    err: JsonPayloadError,
    _req: &actix_web::HttpRequest,
) -> actix_web::Error {
    let (error_message, solution) = match &err {
                JsonPayloadError::ContentType => (
                    "Content type header error",
                    "Ensure the content type request header of the HTTP request is set as `Content-Type: application/json`."
                ),
                JsonPayloadError::Payload(_) => (
                    "Payload error",
                    "Check that the JSON payload matches the expected structure."
                ),
                JsonPayloadError::Deserialize(deserialize_err) => match deserialize_err.classify() {
                    serde_json::error::Category::Io => (
                        "I/O error while reading JSON",
                        "Verify that the server has sufficient permissions to access the file or data source."
                    ),
                    serde_json::error::Category::Syntax => (
                        "Syntax error in JSON",
                        "Fix syntax errors in the JSON payload to adhere to JSON formatting rules."
                    ),
                    serde_json::error::Category::Data => (
                        "Data error in JSON",
                        "Ensure that the data in the JSON payload is valid and consistent with the expected schema."
                    ),
                    serde_json::error::Category::Eof => (
                        "Unexpected end of JSON input",
                        "Ensure that the JSON payload is complete and not truncated."
                    ),
                },
                _ => (
                    "Other JSON payload error",
                    "Inspect the JSON payload and the server's handling of JSON requests for any issues."
                ),
            };

    let detailed_error_message = format!(
        "*Type* : {} | *Message* : {} | {}",
        error_message, err, solution
    );
    ServiceError::JsonDeserializeError(detailed_error_message).into()
}
