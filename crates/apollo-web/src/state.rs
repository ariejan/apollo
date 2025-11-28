//! Application state for the web server.

use apollo_db::SqliteLibrary;

/// Shared application state.
pub struct AppState {
    /// Database connection.
    pub db: SqliteLibrary,
}

impl AppState {
    /// Create a new application state.
    #[must_use]
    pub const fn new(db: SqliteLibrary) -> Self {
        Self { db }
    }
}
