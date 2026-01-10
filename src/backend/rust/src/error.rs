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
    #[display("Unauthorized")]
    Unauthorized,
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
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> HttpResponse {
        if let AppError::Internal(ref e) = self {
            error!("Internal server error: {}", e);
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
