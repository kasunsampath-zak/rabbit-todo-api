use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error")
            }
            AppError::NotFound(ref msg) => (StatusCode::NOT_FOUND, msg.as_str()),
            AppError::Unauthorized(ref msg) => (StatusCode::UNAUTHORIZED, msg.as_str()),
            AppError::Forbidden(ref msg) => (StatusCode::FORBIDDEN, msg.as_str()),
            AppError::BadRequest(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::InternalError(ref msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str())
            }
            AppError::ValidationError(ref msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
        };

        let body = Json(json!({
            "success": false,
            "error": error_message,
            "data": null
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
