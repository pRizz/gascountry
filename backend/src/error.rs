use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;

use crate::db::DbError;

/// Application error type that can be converted into an HTTP response
#[derive(Debug)]
pub enum AppError {
    /// Internal server error (500)
    Internal(String),
    /// Resource not found (404)
    NotFound(String),
    /// Bad request (400)
    BadRequest(String),
    /// Conflict error (409) - e.g., constraint violations
    Conflict(String),
    /// Unprocessable entity (422) - e.g., parse errors
    UnprocessableEntity {
        message: String,
        field: Option<String>,
        value: Option<String>,
    },
}

/// Error response body
#[derive(Serialize)]
struct ErrorResponse {
    error: ErrorBody,
}

/// Error body with structured information
#[derive(Serialize)]
struct ErrorBody {
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match &self {
            AppError::Internal(msg) => {
                // Log unexpected internal errors
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    msg.clone(),
                    None,
                )
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone(), None),
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone(), None)
            }
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone(), None),
            AppError::UnprocessableEntity {
                message,
                field,
                value,
            } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "PARSE_ERROR",
                message.clone(),
                Some(json!({
                    "field": field,
                    "value": value,
                })),
            ),
        };

        let body = Json(ErrorResponse {
            error: ErrorBody {
                code: code.to_string(),
                message,
                details,
            },
        });

        (status, body).into_response()
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::UnprocessableEntity { message, .. } => {
                write!(f, "Unprocessable entity: {}", message)
            }
        }
    }
}

impl std::error::Error for AppError {}

impl From<DbError> for AppError {
    fn from(err: DbError) -> Self {
        match err {
            DbError::NotFound => AppError::NotFound("Resource not found".to_string()),
            DbError::ParseError {
                message,
                value,
                field,
            } => AppError::UnprocessableEntity {
                message,
                field: Some(field),
                value: Some(value),
            },
            DbError::ConstraintViolation(msg) => AppError::Conflict(msg),
            other => AppError::Internal(other.to_string()),
        }
    }
}

/// Result type alias using AppError
pub type AppResult<T> = Result<T, AppError>;
