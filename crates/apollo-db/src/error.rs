//! Database error types.

use thiserror::Error;

/// Database-specific errors.
#[derive(Debug, Error)]
pub enum DbError {
    /// SQL execution error.
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// Migration error.
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    /// Record not found.
    #[error("record not found: {0}")]
    NotFound(String),

    /// Serialization/deserialization error.
    #[error("serialization error: {0}")]
    Serialization(String),

    /// Invalid data in database.
    #[error("invalid data: {0}")]
    InvalidData(String),
}

/// Result type for database operations.
pub type DbResult<T> = Result<T, DbError>;
