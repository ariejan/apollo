//! Application state for the web server.

use apollo_db::SqliteLibrary;
use std::sync::Arc;

/// Shared application state.
pub struct AppState {
    /// Database connection.
    pub db: Arc<SqliteLibrary>,
}

impl AppState {
    /// Create a new application state.
    #[must_use]
    pub fn new(db: SqliteLibrary) -> Self {
        Self { db: Arc::new(db) }
    }
}
