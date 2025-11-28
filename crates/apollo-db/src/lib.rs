//! # Apollo Database
//!
//! SQLite-based storage for the Apollo music library.
//!
//! This crate provides a persistent storage backend implementing the
//! [`Library`](apollo_core::library::Library) trait from apollo-core.

mod error;
mod schema;

pub use error::{DbError, DbResult};
pub use schema::SqliteLibrary;

/// Re-export sqlx for convenience.
pub use sqlx;
