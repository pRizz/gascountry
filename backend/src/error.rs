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
    /// User action required (422) - actionable errors with help steps
    UserActionRequired {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
        help_steps: Vec<String>,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    help_steps: Vec<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details, help_steps) = match &self {
            AppError::Internal(msg) => {
                // Log unexpected internal errors
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    msg.clone(),
                    None,
                    Vec::new(),
                )
            }
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, "NOT_FOUND", msg.clone(), None, Vec::new()),
            AppError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone(), None, Vec::new())
            }
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone(), None, Vec::new()),
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
                Vec::new(),
            ),
            AppError::UserActionRequired {
                code,
                message,
                details,
                help_steps,
            } => (
                StatusCode::UNPROCESSABLE_ENTITY,
                code.as_str(),
                message.clone(),
                details.clone(),
                help_steps.clone(),
            ),
        };

        let body = Json(ErrorResponse {
            error: ErrorBody {
                code: code.to_string(),
                message,
                details,
                help_steps,
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
            AppError::UserActionRequired { code, message, .. } => {
                write!(f, "User action required [{}]: {}", code, message)
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

impl From<crate::git::CloneError> for AppError {
    fn from(err: crate::git::CloneError) -> Self {
        match err {
            crate::git::CloneError::SshAuthFailed { message, help_steps, needs_passphrase } => {
                AppError::UserActionRequired {
                    code: "SSH_AUTH_FAILED".to_string(),
                    message,
                    details: Some(serde_json::json!({ "needs_passphrase": needs_passphrase })),
                    help_steps,
                }
            }
            crate::git::CloneError::HttpsAuthFailed { message, help_steps, is_github } => {
                AppError::UserActionRequired {
                    code: "HTTPS_AUTH_FAILED".to_string(),
                    message,
                    details: Some(serde_json::json!({ "is_github": is_github })),
                    help_steps,
                }
            }
            crate::git::CloneError::NetworkError { message } => {
                AppError::Internal(format!("Network error: {}", message))
            }
            crate::git::CloneError::OperationFailed { message } => {
                AppError::Internal(format!("Clone failed: {}", message))
            }
        }
    }
}

impl From<crate::ralph::RalphError> for AppError {
    fn from(err: crate::ralph::RalphError) -> Self {
        match err {
            crate::ralph::RalphError::RepoBusy(repo_id) => AppError::BadRequest(format!(
                "Repository {} already has a running ralph process",
                repo_id
            )),
            crate::ralph::RalphError::SessionAlreadyRunning(session_id) => AppError::BadRequest(format!(
                "Session {} already has a running process",
                session_id
            )),
            crate::ralph::RalphError::SpawnFailed(msg) => {
                AppError::Internal(format!("Failed to start ralph: {}", msg))
            }
            crate::ralph::RalphError::NotFound { message, help_steps } => {
                AppError::UserActionRequired {
                    code: "RALPH_NOT_FOUND".to_string(),
                    message,
                    details: None,
                    help_steps,
                }
            }
            crate::ralph::RalphError::NotRunning(session_id) => AppError::BadRequest(format!(
                "Session {} has no running process",
                session_id
            )),
        }
    }
}

/// Result type alias using AppError
pub type AppResult<T> = Result<T, AppError>;
