//! Error types for metadata source operations.

use thiserror::Error;

/// Errors that can occur when fetching from metadata sources.
#[derive(Debug, Error)]
pub enum SourceError {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("API error: {status} - {message}")]
    Api {
        /// HTTP status code.
        status: u16,
        /// Error message from API.
        message: String,
    },

    /// Failed to parse API response.
    #[error("failed to parse response: {0}")]
    Parse(String),

    /// Rate limit exceeded.
    #[error("rate limit exceeded, retry after {retry_after} seconds")]
    RateLimited {
        /// Seconds to wait before retrying.
        retry_after: u64,
    },

    /// Invalid input provided.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// No results found.
    #[error("no results found")]
    NotFound,
}

/// Result type for source operations.
pub type SourceResult<T> = Result<T, SourceError>;
