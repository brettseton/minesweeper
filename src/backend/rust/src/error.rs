use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use derive_more::Display;
use serde::Serialize;
use tracing::error;

#[derive(Debug, Display)]
pub enum AppError {
    #[display("Internal Server Error")]
    Internal(String),
    #[display("Not Found: {_0}")]
    NotFound(String),
    #[display("Bad Request: {_0}")]
    BadRequest(String),
    #[display("Service Unavailable: {_0}")]
    ServiceUnavailable(String),
    #[display("Unauthorized")]
    Unauthorized,
    #[display("Forbidden")]
    Forbidden,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::Internal(ref e) => {
                error!("Internal server error: {}", e);
            }
            AppError::Unauthorized => {
                tracing::warn!("Security Event: Unauthorized access attempt");
            }
            AppError::Forbidden => {
                tracing::warn!("Security Event: Forbidden request");
            }
            AppError::BadRequest(ref e) => {
                tracing::warn!("Security Event: Bad request: {}", e);
            }
            AppError::ServiceUnavailable(ref e) => {
                tracing::warn!("Service unavailable: {}", e);
            }
            _ => {}
        }

        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: self.to_string(),
        })
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl From<mongodb::error::Error> for AppError {
    fn from(e: mongodb::error::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<mongodb::bson::ser::Error> for AppError {
    fn from(e: mongodb::bson::ser::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}
