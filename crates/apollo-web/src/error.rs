//! API error types and responses.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// API error type.
#[derive(Debug)]
pub enum ApiError {
    /// Resource not found.
    NotFound(String),
    /// Invalid request.
    BadRequest(String),
    /// Internal server error.
    Internal(String),
    /// Database error.
    Database(apollo_db::DbError),
}

/// Error response body.
#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, "not_found", msg),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, "bad_request", msg),
            Self::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error", msg),
            Self::Database(err) => {
                tracing::error!("Database error: {err}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "database_error",
                    "An internal database error occurred".to_string(),
                )
            }
        };

        let body = ErrorResponse {
            error: error_type.to_string(),
            message,
        };

        (status, Json(body)).into_response()
    }
}

impl From<apollo_db::DbError> for ApiError {
    fn from(err: apollo_db::DbError) -> Self {
        match &err {
            apollo_db::DbError::NotFound(resource) => Self::NotFound(resource.clone()),
            _ => Self::Database(err),
        }
    }
}
