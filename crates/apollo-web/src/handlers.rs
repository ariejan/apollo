//! API request handlers.

use crate::{error::ApiError, state::AppState};
use apollo_core::metadata::{AlbumId, TrackId};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Default page size for list operations.
const DEFAULT_LIMIT: u32 = 50;
/// Maximum page size for list operations.
const MAX_LIMIT: u32 = 500;

/// Pagination query parameters.
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    /// Maximum number of items to return.
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Number of items to skip.
    #[serde(default)]
    pub offset: u32,
}

const fn default_limit() -> u32 {
    DEFAULT_LIMIT
}

/// Search query parameters.
#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// Search query string.
    pub q: String,
}

/// Paginated response wrapper.
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    /// Items in this page.
    pub items: Vec<T>,
    /// Total number of items.
    pub total: u64,
    /// Current limit.
    pub limit: u32,
    /// Current offset.
    pub offset: u32,
}

/// Library statistics response.
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    /// Total number of tracks.
    pub track_count: u64,
    /// Total number of albums.
    pub album_count: u64,
}

/// Health check response.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Health status.
    pub status: String,
    /// Service version.
    pub version: String,
}

/// Health check endpoint.
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Get library statistics.
pub async fn get_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<StatsResponse>, ApiError> {
    let track_count = state.db.count_tracks().await?;
    let album_count = state.db.count_albums().await?;

    Ok(Json(StatsResponse {
        track_count,
        album_count,
    }))
}

/// List all tracks with pagination.
pub async fn list_tracks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<apollo_core::metadata::Track>>, ApiError> {
    let limit = query.limit.min(MAX_LIMIT);
    let tracks = state.db.list_tracks(limit, query.offset).await?;
    let total = state.db.count_tracks().await?;

    Ok(Json(PaginatedResponse {
        items: tracks,
        total,
        limit,
        offset: query.offset,
    }))
}

/// Get a single track by ID.
pub async fn get_track(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<apollo_core::metadata::Track>, ApiError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid track ID: {id}")))?;
    let track_id = TrackId(uuid);

    let track = state
        .db
        .get_track(&track_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Track not found: {id}")))?;

    Ok(Json(track))
}

/// List all albums with pagination.
pub async fn list_albums(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<apollo_core::metadata::Album>>, ApiError> {
    let limit = query.limit.min(MAX_LIMIT);
    let albums = state.db.list_albums(limit, query.offset).await?;
    let total = state.db.count_albums().await?;

    Ok(Json(PaginatedResponse {
        items: albums,
        total,
        limit,
        offset: query.offset,
    }))
}

/// Get a single album by ID.
pub async fn get_album(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<apollo_core::metadata::Album>, ApiError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid album ID: {id}")))?;
    let album_id = AlbumId(uuid);

    let album = state
        .db
        .get_album(&album_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Album not found: {id}")))?;

    Ok(Json(album))
}

/// Get all tracks in an album.
pub async fn get_album_tracks(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<apollo_core::metadata::Track>>, ApiError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid album ID: {id}")))?;
    let album_id = AlbumId(uuid);

    // Verify album exists
    state
        .db
        .get_album(&album_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Album not found: {id}")))?;

    let tracks = state.db.get_album_tracks(&album_id).await?;
    Ok(Json(tracks))
}

/// Search tracks by query.
pub async fn search_tracks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<apollo_core::metadata::Track>>, ApiError> {
    if query.q.is_empty() {
        return Err(ApiError::BadRequest(
            "Search query cannot be empty".to_string(),
        ));
    }

    // Convert to FTS5 prefix search for simple queries
    let fts_query = if query.q.contains(':') || query.q.contains('"') || query.q.contains('*') {
        // Use as-is if it looks like FTS5 syntax
        query.q.clone()
    } else {
        // Add prefix matching for each word
        query
            .q
            .split_whitespace()
            .map(|word| format!("{word}*"))
            .collect::<Vec<_>>()
            .join(" ")
    };

    let tracks = state.db.search_tracks(&fts_query).await?;
    Ok(Json(tracks))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_pagination() {
        let query: PaginationQuery = serde_json::from_str("{}").unwrap();
        assert_eq!(query.limit, DEFAULT_LIMIT);
        assert_eq!(query.offset, 0);
    }

    #[test]
    fn test_custom_pagination() {
        let query: PaginationQuery =
            serde_json::from_str(r#"{"limit": 100, "offset": 50}"#).unwrap();
        assert_eq!(query.limit, 100);
        assert_eq!(query.offset, 50);
    }
}
