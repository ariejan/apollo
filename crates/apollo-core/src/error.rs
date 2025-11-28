//! Error types for Apollo core operations.

use thiserror::Error;

/// Core error type for Apollo operations.
#[derive(Debug, Error)]
pub enum Error {
    /// Track was not found in the library.
    #[error("track not found: {0}")]
    TrackNotFound(String),

    /// Album was not found in the library.
    #[error("album not found: {0}")]
    AlbumNotFound(String),

    /// Invalid query syntax.
    #[error("invalid query: {0}")]
    InvalidQuery(String),

    /// Validation error for metadata.
    #[error("validation error: {0}")]
    Validation(String),

    /// Configuration error.
    #[error("configuration error: {message}")]
    Config {
        /// Error message describing what went wrong.
        message: String,
    },
}

/// Result type alias using the core Error type.
pub type Result<T> = std::result::Result<T, Error>;
