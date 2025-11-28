//! API request handlers.

use crate::{error::ApiError, state::AppState};
use apollo_core::metadata::{Album, AlbumId, Track, TrackId};
use apollo_core::playlist::{Playlist, PlaylistId, PlaylistLimit, PlaylistSort};
use apollo_core::query::Query as ApolloQuery;
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
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
    /// Total number of playlists.
    #[schema(example = 5)]
    pub playlist_count: u64,
}

/// API representation of a playlist.
#[derive(Debug, Serialize, ToSchema)]
pub struct PlaylistResponse {
    /// Unique identifier.
    #[schema(example = "770e8400-e29b-41d4-a716-446655440002")]
    pub id: String,
    /// Playlist name.
    #[schema(example = "My Favorites")]
    pub name: String,
    /// Optional description.
    #[schema(example = "My favorite songs")]
    pub description: Option<String>,
    /// Playlist type (static or smart).
    #[schema(example = "static")]
    pub kind: String,
    /// Query string for smart playlists.
    #[schema(example = "artist:Beatles")]
    pub query: Option<String>,
    /// Sort order.
    #[schema(example = "artist")]
    pub sort: String,
    /// Maximum number of tracks (smart playlists only).
    pub max_tracks: Option<u32>,
    /// Maximum duration in seconds (smart playlists only).
    pub max_duration_secs: Option<u64>,
    /// Number of tracks in the playlist.
    #[schema(example = 25)]
    pub track_count: usize,
    /// When the playlist was created.
    pub created_at: String,
    /// When the playlist was last modified.
    pub modified_at: String,
}

impl PlaylistResponse {
    fn from_playlist(playlist: &Playlist, track_count: usize) -> Self {
        Self {
            id: playlist.id.0.to_string(),
            name: playlist.name.clone(),
            description: playlist.description.clone(),
            kind: format!("{}", playlist.kind),
            query: playlist.query.as_ref().map(|q| format!("{q}")),
            sort: format!("{}", playlist.sort),
            max_tracks: playlist.limit.as_ref().and_then(|l| l.max_tracks),
            max_duration_secs: playlist.limit.as_ref().and_then(|l| l.max_duration_secs),
            track_count,
            created_at: playlist.created_at.to_rfc3339(),
            modified_at: playlist.modified_at.to_rfc3339(),
        }
    }
}

/// Request to create a new playlist.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePlaylistRequest {
    /// Playlist name.
    #[schema(example = "My Favorites")]
    pub name: String,
    /// Optional description.
    #[schema(example = "My favorite songs")]
    pub description: Option<String>,
    /// Query string for smart playlists. If provided, creates a smart playlist.
    #[schema(example = "artist:Beatles")]
    pub query: Option<String>,
    /// Sort order (one of: artist, album, title).
    #[schema(example = "artist")]
    #[serde(default)]
    pub sort: Option<String>,
    /// Maximum number of tracks (smart playlists only).
    pub max_tracks: Option<u32>,
    /// Maximum duration in seconds (smart playlists only).
    pub max_duration_secs: Option<u64>,
}

/// Request to update a playlist.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdatePlaylistRequest {
    /// New playlist name.
    #[schema(example = "Updated Favorites")]
    pub name: Option<String>,
    /// New description.
    #[schema(example = "Updated description")]
    pub description: Option<String>,
    /// New query string (smart playlists only).
    #[schema(example = "artist:Rolling Stones")]
    pub query: Option<String>,
    /// New sort order.
    #[schema(example = "year_desc")]
    pub sort: Option<String>,
    /// New maximum tracks.
    pub max_tracks: Option<u32>,
    /// New maximum duration.
    pub max_duration_secs: Option<u64>,
}

/// Request to add or remove tracks from a playlist.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PlaylistTracksRequest {
    /// Track IDs to add or remove.
    #[schema(example = json!(["550e8400-e29b-41d4-a716-446655440000"]))]
    pub track_ids: Vec<String>,
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
    let playlist_count = state.db.count_playlists().await?;

    Ok(Json(StatsResponse {
        track_count,
        album_count,
        playlist_count,
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

// ========================================================================
// Playlist handlers
// ========================================================================

/// List all playlists.
#[utoipa::path(
    get,
    path = "/api/playlists",
    tag = "Playlists",
    responses(
        (status = 200, description = "List of playlists", body = Vec<PlaylistResponse>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn list_playlists(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PlaylistResponse>>, ApiError> {
    let playlists = state.db.list_playlists().await?;

    let responses: Vec<PlaylistResponse> = playlists
        .iter()
        .map(|p| {
            let track_count = if p.is_static() {
                p.track_ids.len()
            } else {
                0 // Smart playlist track count requires evaluation
            };
            PlaylistResponse::from_playlist(p, track_count)
        })
        .collect();

    Ok(Json(responses))
}

/// Get a single playlist by ID.
#[utoipa::path(
    get,
    path = "/api/playlists/{id}",
    tag = "Playlists",
    params(
        ("id" = String, Path, description = "Playlist UUID", example = "770e8400-e29b-41d4-a716-446655440002")
    ),
    responses(
        (status = 200, description = "Playlist found", body = PlaylistResponse),
        (status = 400, description = "Invalid playlist ID", body = ErrorResponse),
        (status = 404, description = "Playlist not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_playlist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<PlaylistResponse>, ApiError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid playlist ID: {id}")))?;
    let playlist_id = PlaylistId(uuid);

    let playlist = state
        .db
        .get_playlist(&playlist_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Playlist not found: {id}")))?;

    let track_count = if playlist.is_static() {
        playlist.track_ids.len()
    } else {
        state.db.get_playlist_tracks(&playlist_id).await?.len()
    };

    Ok(Json(PlaylistResponse::from_playlist(
        &playlist,
        track_count,
    )))
}

/// Get all tracks in a playlist.
#[utoipa::path(
    get,
    path = "/api/playlists/{id}/tracks",
    tag = "Playlists",
    params(
        ("id" = String, Path, description = "Playlist UUID", example = "770e8400-e29b-41d4-a716-446655440002")
    ),
    responses(
        (status = 200, description = "List of tracks in the playlist", body = Vec<Track>),
        (status = 400, description = "Invalid playlist ID", body = ErrorResponse),
        (status = 404, description = "Playlist not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn get_playlist_tracks(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Vec<Track>>, ApiError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid playlist ID: {id}")))?;
    let playlist_id = PlaylistId(uuid);

    // Verify playlist exists
    state
        .db
        .get_playlist(&playlist_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Playlist not found: {id}")))?;

    let tracks = state.db.get_playlist_tracks(&playlist_id).await?;
    Ok(Json(tracks))
}

/// Create a new playlist.
#[utoipa::path(
    post,
    path = "/api/playlists",
    tag = "Playlists",
    request_body = CreatePlaylistRequest,
    responses(
        (status = 201, description = "Playlist created", body = PlaylistResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn create_playlist(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreatePlaylistRequest>,
) -> Result<(StatusCode, Json<PlaylistResponse>), ApiError> {
    let playlist = if let Some(query_str) = req.query {
        // Parse the query for smart playlist
        let parsed_query = ApolloQuery::parse(&query_str)
            .map_err(|e| ApiError::BadRequest(format!("Invalid query: {e}")))?;

        let mut pl = Playlist::new_smart(&req.name, parsed_query);

        if let Some(desc) = req.description {
            pl = pl.with_description(desc);
        }

        if let Some(sort_str) = req.sort {
            pl = pl.with_sort(parse_sort(&sort_str));
        }

        if req.max_tracks.is_some() || req.max_duration_secs.is_some() {
            pl = pl.with_limit(PlaylistLimit {
                max_tracks: req.max_tracks,
                max_duration_secs: req.max_duration_secs,
            });
        }

        pl
    } else {
        let mut pl = Playlist::new_static(&req.name);

        if let Some(desc) = req.description {
            pl = pl.with_description(desc);
        }

        pl
    };

    state.db.add_playlist(&playlist).await?;

    let response = PlaylistResponse::from_playlist(&playlist, 0);
    Ok((StatusCode::CREATED, Json(response)))
}

/// Update a playlist.
#[utoipa::path(
    patch,
    path = "/api/playlists/{id}",
    tag = "Playlists",
    params(
        ("id" = String, Path, description = "Playlist UUID", example = "770e8400-e29b-41d4-a716-446655440002")
    ),
    request_body = UpdatePlaylistRequest,
    responses(
        (status = 200, description = "Playlist updated", body = PlaylistResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 404, description = "Playlist not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn update_playlist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePlaylistRequest>,
) -> Result<Json<PlaylistResponse>, ApiError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid playlist ID: {id}")))?;
    let playlist_id = PlaylistId(uuid);

    let mut playlist = state
        .db
        .get_playlist(&playlist_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Playlist not found: {id}")))?;

    if let Some(name) = req.name {
        playlist.name = name;
    }

    if let Some(desc) = req.description {
        playlist.description = Some(desc);
    }

    if let Some(query_str) = req.query {
        if playlist.is_smart() {
            let parsed_query = ApolloQuery::parse(&query_str)
                .map_err(|e| ApiError::BadRequest(format!("Invalid query: {e}")))?;
            playlist.query = Some(parsed_query);
        } else {
            return Err(ApiError::BadRequest(
                "Cannot set query on static playlist".to_string(),
            ));
        }
    }

    if let Some(sort_str) = req.sort {
        playlist.sort = parse_sort(&sort_str);
    }

    if req.max_tracks.is_some() || req.max_duration_secs.is_some() {
        let limit = playlist.limit.get_or_insert_with(PlaylistLimit::default);
        if let Some(max) = req.max_tracks {
            limit.max_tracks = Some(max);
        }
        if let Some(max) = req.max_duration_secs {
            limit.max_duration_secs = Some(max);
        }
    }

    state.db.update_playlist(&playlist).await?;

    let track_count = if playlist.is_static() {
        playlist.track_ids.len()
    } else {
        state.db.get_playlist_tracks(&playlist_id).await?.len()
    };

    Ok(Json(PlaylistResponse::from_playlist(
        &playlist,
        track_count,
    )))
}

/// Delete a playlist.
#[utoipa::path(
    delete,
    path = "/api/playlists/{id}",
    tag = "Playlists",
    params(
        ("id" = String, Path, description = "Playlist UUID", example = "770e8400-e29b-41d4-a716-446655440002")
    ),
    responses(
        (status = 204, description = "Playlist deleted"),
        (status = 400, description = "Invalid playlist ID", body = ErrorResponse),
        (status = 404, description = "Playlist not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn delete_playlist(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid playlist ID: {id}")))?;
    let playlist_id = PlaylistId(uuid);

    state.db.remove_playlist(&playlist_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Add tracks to a static playlist.
#[utoipa::path(
    post,
    path = "/api/playlists/{id}/tracks",
    tag = "Playlists",
    params(
        ("id" = String, Path, description = "Playlist UUID", example = "770e8400-e29b-41d4-a716-446655440002")
    ),
    request_body = PlaylistTracksRequest,
    responses(
        (status = 200, description = "Tracks added", body = PlaylistResponse),
        (status = 400, description = "Invalid request or smart playlist", body = ErrorResponse),
        (status = 404, description = "Playlist not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn add_playlist_tracks(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<PlaylistTracksRequest>,
) -> Result<Json<PlaylistResponse>, ApiError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid playlist ID: {id}")))?;
    let playlist_id = PlaylistId(uuid);

    let playlist = state
        .db
        .get_playlist(&playlist_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Playlist not found: {id}")))?;

    if playlist.is_smart() {
        return Err(ApiError::BadRequest(
            "Cannot add tracks to smart playlist".to_string(),
        ));
    }

    for track_id_str in &req.track_ids {
        let track_uuid = Uuid::parse_str(track_id_str)
            .map_err(|_| ApiError::BadRequest(format!("Invalid track ID: {track_id_str}")))?;
        let track_id = TrackId(track_uuid);

        // Verify track exists
        state
            .db
            .get_track(&track_id)
            .await?
            .ok_or_else(|| ApiError::NotFound(format!("Track not found: {track_id_str}")))?;

        state
            .db
            .add_track_to_playlist(&playlist_id, &track_id)
            .await?;
    }

    // Reload playlist to get updated track list
    let updated_playlist = state
        .db
        .get_playlist(&playlist_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Playlist not found: {id}")))?;

    let track_count = updated_playlist.track_ids.len();
    Ok(Json(PlaylistResponse::from_playlist(
        &updated_playlist,
        track_count,
    )))
}

/// Remove tracks from a static playlist.
#[utoipa::path(
    delete,
    path = "/api/playlists/{id}/tracks",
    tag = "Playlists",
    params(
        ("id" = String, Path, description = "Playlist UUID", example = "770e8400-e29b-41d4-a716-446655440002")
    ),
    request_body = PlaylistTracksRequest,
    responses(
        (status = 200, description = "Tracks removed", body = PlaylistResponse),
        (status = 400, description = "Invalid request or smart playlist", body = ErrorResponse),
        (status = 404, description = "Playlist not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn remove_playlist_tracks(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<PlaylistTracksRequest>,
) -> Result<Json<PlaylistResponse>, ApiError> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequest(format!("Invalid playlist ID: {id}")))?;
    let playlist_id = PlaylistId(uuid);

    let playlist = state
        .db
        .get_playlist(&playlist_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Playlist not found: {id}")))?;

    if playlist.is_smart() {
        return Err(ApiError::BadRequest(
            "Cannot remove tracks from smart playlist".to_string(),
        ));
    }

    for track_id_str in &req.track_ids {
        let track_uuid = Uuid::parse_str(track_id_str)
            .map_err(|_| ApiError::BadRequest(format!("Invalid track ID: {track_id_str}")))?;
        let track_id = TrackId(track_uuid);

        state
            .db
            .remove_track_from_playlist(&playlist_id, &track_id)
            .await?;
    }

    // Reload playlist to get updated track list
    let updated_playlist = state
        .db
        .get_playlist(&playlist_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Playlist not found: {id}")))?;

    let track_count = updated_playlist.track_ids.len();
    Ok(Json(PlaylistResponse::from_playlist(
        &updated_playlist,
        track_count,
    )))
}

/// Parse a sort string into a playlist sort order.
fn parse_sort(s: &str) -> PlaylistSort {
    match s.to_lowercase().as_str() {
        "album" => PlaylistSort::Album,
        "title" => PlaylistSort::Title,
        "added_desc" | "addeddesc" => PlaylistSort::AddedDesc,
        "added_asc" | "addedasc" => PlaylistSort::AddedAsc,
        "year_desc" | "yeardesc" => PlaylistSort::YearDesc,
        "year_asc" | "yearasc" => PlaylistSort::YearAsc,
        "random" => PlaylistSort::Random,
        _ => PlaylistSort::Artist,
    }
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
