//! API request handlers.

use crate::{error::ApiError, state::AppState};
use apollo_core::metadata::{Album, AlbumId, Track, TrackId};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

/// Default page size for list operations.
const DEFAULT_LIMIT: u32 = 50;
/// Maximum page size for list operations.
const MAX_LIMIT: u32 = 500;

/// Pagination query parameters.
#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationQuery {
    /// Maximum number of items to return (default: 50, max: 500).
    #[serde(default = "default_limit")]
    #[param(default = 50, minimum = 1, maximum = 500)]
    pub limit: u32,
    /// Number of items to skip.
    #[serde(default)]
    #[param(default = 0, minimum = 0)]
    pub offset: u32,
}

const fn default_limit() -> u32 {
    DEFAULT_LIMIT
}

/// Search query parameters.
#[derive(Debug, Deserialize, IntoParams)]
pub struct SearchQuery {
    /// Search query string. Supports simple text or FTS5 syntax.
    #[param(example = "bohemian rhapsody")]
    pub q: String,
}

/// Paginated response wrapper for tracks.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedTracksResponse {
    /// Items in this page.
    pub items: Vec<Track>,
    /// Total number of items.
    #[schema(example = 100)]
    pub total: u64,
    /// Current limit.
    #[schema(example = 50)]
    pub limit: u32,
    /// Current offset.
    #[schema(example = 0)]
    pub offset: u32,
}

/// Paginated response wrapper for albums.
#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedAlbumsResponse {
    /// Items in this page.
    pub items: Vec<Album>,
    /// Total number of items.
    #[schema(example = 25)]
    pub total: u64,
    /// Current limit.
    #[schema(example = 50)]
    pub limit: u32,
    /// Current offset.
    #[schema(example = 0)]
    pub offset: u32,
}

/// Library statistics response.
#[derive(Debug, Serialize, ToSchema)]
pub struct StatsResponse {
    /// Total number of tracks.
    #[schema(example = 1234)]
    pub track_count: u64,
    /// Total number of albums.
    #[schema(example = 87)]
    pub album_count: u64,
}

/// Health check response.
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Health status.
    #[schema(example = "healthy")]
    pub status: String,
    /// Service version.
    #[schema(example = "0.1.0")]
    pub version: String,
}

/// Error response.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Error type.
    #[schema(example = "not_found")]
    pub error: String,
    /// Error message.
    #[schema(example = "Track not found: 550e8400-e29b-41d4-a716-446655440000")]
    pub message: String,
}

/// Health check endpoint.
#[utoipa::path(
    get,
    path = "/health",
    tag = "System",
    responses(
        (status = 200, description = "Service is healthy", body = HealthResponse)
    )
)]
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Get library statistics.
#[utoipa::path(
    get,
    path = "/api/stats",
    tag = "Library",
    responses(
        (status = 200, description = "Library statistics", body = StatsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
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
#[utoipa::path(
    get,
    path = "/api/tracks",
    tag = "Tracks",
    params(PaginationQuery),
    responses(
        (status = 200, description = "List of tracks", body = PaginatedTracksResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn list_tracks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedTracksResponse>, ApiError> {
    let limit = query.limit.min(MAX_LIMIT);
    let tracks = state.db.list_tracks(limit, query.offset).await?;
    let total = state.db.count_tracks().await?;

    Ok(Json(PaginatedTracksResponse {
        items: tracks,
        total,
        limit,
        offset: query.offset,
    }))
}

/// Get a single track by ID.
#[utoipa::path(
    get,
    path = "/api/tracks/{id}",
    tag = "Tracks",
    params(
        ("id" = String, Path, description = "Track UUID", example = "550e8400-e29b-41d4-a716-446655440000")
    ),
    responses(
        (status = 200, description = "Track found", body = Track),
        (status = 400, description = "Invalid track ID", body = ErrorResponse),
        (status = 404, description = "Track not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_track(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Track>, ApiError> {
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
#[utoipa::path(
    get,
    path = "/api/albums",
    tag = "Albums",
    params(PaginationQuery),
    responses(
        (status = 200, description = "List of albums", body = PaginatedAlbumsResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn list_albums(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedAlbumsResponse>, ApiError> {
    let limit = query.limit.min(MAX_LIMIT);
    let albums = state.db.list_albums(limit, query.offset).await?;
    let total = state.db.count_albums().await?;

    Ok(Json(PaginatedAlbumsResponse {
        items: albums,
        total,
        limit,
        offset: query.offset,
    }))
}

/// Get a single album by ID.
#[utoipa::path(
    get,
    path = "/api/albums/{id}",
    tag = "Albums",
    params(
        ("id" = String, Path, description = "Album UUID", example = "660e8400-e29b-41d4-a716-446655440001")
    ),
    responses(
        (status = 200, description = "Album found", body = Album),
        (status = 400, description = "Invalid album ID", body = ErrorResponse),
        (status = 404, description = "Album not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_album(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Album>, ApiError> {
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
#[utoipa::path(
    get,
    path = "/api/albums/{id}/tracks",
    tag = "Albums",
    params(
        ("id" = String, Path, description = "Album UUID", example = "660e8400-e29b-41d4-a716-446655440001")
    ),
    responses(
        (status = 200, description = "List of tracks in the album", body = Vec<Track>),
        (status = 400, description = "Invalid album ID", body = ErrorResponse),
        (status = 404, description = "Album not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_album_tracks(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Track>>, ApiError> {
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
#[utoipa::path(
    get,
    path = "/api/search",
    tag = "Search",
    params(SearchQuery),
    responses(
        (status = 200, description = "Search results", body = Vec<Track>),
        (status = 400, description = "Empty search query", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn search_tracks(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<Track>>, ApiError> {
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
