// Clippy lint exception for utoipa derive macro
#![allow(clippy::needless_for_each)]

//! # Apollo Web
//!
//! REST API and web interface for Apollo.
//!
//! This crate provides an HTTP API for accessing the Apollo music library.
//!
//! ## Endpoints
//!
//! - `GET /api/tracks` - List all tracks with pagination
//! - `GET /api/tracks/:id` - Get a single track by ID
//! - `GET /api/albums` - List all albums with pagination
//! - `GET /api/albums/:id` - Get a single album by ID
//! - `GET /api/albums/:id/tracks` - Get all tracks in an album
//! - `GET /api/playlists` - List all playlists
//! - `GET /api/playlists/:id` - Get a single playlist by ID
//! - `GET /api/playlists/:id/tracks` - Get all tracks in a playlist
//! - `POST /api/playlists` - Create a new playlist
//! - `PATCH /api/playlists/:id` - Update a playlist
//! - `DELETE /api/playlists/:id` - Delete a playlist
//! - `POST /api/playlists/:id/tracks` - Add tracks to a playlist
//! - `DELETE /api/playlists/:id/tracks` - Remove tracks from a playlist
//! - `GET /api/search` - Search tracks by query
//! - `GET /api/stats` - Get library statistics
//! - `POST /api/import` - Import music from a directory
//! - `GET /swagger-ui` - Interactive API documentation

mod error;
mod handlers;
pub mod import;
mod state;

pub use error::ApiError;
pub use handlers::{
    CreatePlaylistRequest, ErrorResponse, HealthResponse, ImportRequest, ImportResponse,
    PaginatedAlbumsResponse, PaginatedTracksResponse, PlaylistResponse, PlaylistTracksRequest,
    StatsResponse, UpdatePlaylistRequest,
};
pub use import::{ImportOptions, ImportProgress, ImportResult, ImportService};
pub use state::AppState;

use apollo_core::metadata::{Album, AlbumId, Artist, AudioFormat, Track, TrackId};
use axum::{
    Router,
    routing::{get, post},
};
use std::path::Path;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// [OpenAPI](https://www.openapis.org/) documentation for the Apollo API.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Apollo API",
        description = "REST API for the Apollo music library manager",
        version = "0.1.0",
        license(name = "MIT OR Apache-2.0"),
        contact(name = "Apollo Team")
    ),
    servers(
        (url = "/", description = "Local server")
    ),
    tags(
        (name = "Tracks", description = "Track management endpoints"),
        (name = "Albums", description = "Album management endpoints"),
        (name = "Playlists", description = "Playlist management endpoints"),
        (name = "Import", description = "Music import endpoints"),
        (name = "Search", description = "Search endpoints"),
        (name = "Library", description = "Library statistics"),
        (name = "System", description = "System health endpoints")
    ),
    paths(
        handlers::health_check,
        handlers::get_stats,
        handlers::list_tracks,
        handlers::get_track,
        handlers::list_albums,
        handlers::get_album,
        handlers::get_album_tracks,
        handlers::search_tracks,
        handlers::list_playlists,
        handlers::get_playlist,
        handlers::get_playlist_tracks,
        handlers::create_playlist,
        handlers::update_playlist,
        handlers::delete_playlist,
        handlers::add_playlist_tracks,
        handlers::remove_playlist_tracks,
        handlers::import_music
    ),
    components(
        schemas(
            Track,
            Album,
            Artist,
            TrackId,
            AlbumId,
            AudioFormat,
            HealthResponse,
            StatsResponse,
            ErrorResponse,
            PaginatedTracksResponse,
            PaginatedAlbumsResponse,
            PlaylistResponse,
            CreatePlaylistRequest,
            UpdatePlaylistRequest,
            PlaylistTracksRequest,
            ImportRequest,
            ImportResponse
        )
    )
)]
pub struct ApiDoc;

/// Create the API router with all endpoints.
///
/// # Arguments
///
/// * `state` - The shared application state containing the database connection
///
/// # Returns
///
/// An Axum router configured with all API endpoints
pub fn create_router(state: Arc<AppState>) -> Router {
    create_router_with_static_files(state, None)
}

/// Create the API router with optional static file serving.
///
/// # Arguments
///
/// * `state` - The shared application state containing the database connection
/// * `static_files_path` - Optional path to directory containing static files (index.html, etc.)
///
/// # Returns
///
/// An Axum router configured with all API endpoints and optional static file serving
pub fn create_router_with_static_files(
    state: Arc<AppState>,
    static_files_path: Option<&Path>,
) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let mut router = Router::new()
        // Track endpoints
        .route("/api/tracks", get(handlers::list_tracks))
        .route("/api/tracks/:id", get(handlers::get_track))
        // Album endpoints
        .route("/api/albums", get(handlers::list_albums))
        .route("/api/albums/:id", get(handlers::get_album))
        .route("/api/albums/:id/tracks", get(handlers::get_album_tracks))
        // Playlist endpoints
        .route(
            "/api/playlists",
            get(handlers::list_playlists).post(handlers::create_playlist),
        )
        .route(
            "/api/playlists/:id",
            get(handlers::get_playlist)
                .patch(handlers::update_playlist)
                .delete(handlers::delete_playlist),
        )
        .route(
            "/api/playlists/:id/tracks",
            get(handlers::get_playlist_tracks)
                .post(handlers::add_playlist_tracks)
                .delete(handlers::remove_playlist_tracks),
        )
        // Search endpoint
        .route("/api/search", get(handlers::search_tracks))
        // Stats endpoint
        .route("/api/stats", get(handlers::get_stats))
        // Import endpoint
        .route("/api/import", post(handlers::import_music))
        // Health check
        .route("/health", get(handlers::health_check))
        // OpenAPI documentation
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Add shared state
        .with_state(state);

    // Serve static files if path is provided (for embedded web UI)
    if let Some(path) = static_files_path {
        let index_file = path.join("index.html");
        router = router
            .fallback_service(ServeDir::new(path).not_found_service(ServeFile::new(index_file)));
    }

    // Add middleware
    router.layer(cors).layer(TraceLayer::new_for_http())
}

#[cfg(test)]
mod tests {
    use super::*;
    use apollo_core::metadata::{Album, Track};
    use apollo_db::SqliteLibrary;
    use axum_test::TestServer;
    use std::path::PathBuf;
    use std::time::Duration;

    async fn create_test_server() -> TestServer {
        let db = SqliteLibrary::in_memory().await.unwrap();
        let state = Arc::new(AppState::new(db));
        let router = create_router(state);
        TestServer::new(router).unwrap()
    }

    async fn create_test_server_with_data() -> TestServer {
        let db = SqliteLibrary::in_memory().await.unwrap();

        // Add some test tracks
        for i in 1..=3 {
            let track = Track::new(
                PathBuf::from(format!("/music/track{i}.mp3")),
                format!("Track {i}"),
                "Test Artist".to_string(),
                Duration::from_secs(180),
            );
            db.add_track(&track).await.unwrap();
        }

        // Add a test album
        let album = Album::new("Test Album".to_string(), "Test Artist".to_string());
        db.add_album(&album).await.unwrap();

        let state = Arc::new(AppState::new(db));
        let router = create_router(state);
        TestServer::new(router).unwrap()
    }

    #[tokio::test]
    async fn test_health_check() {
        let server = create_test_server().await;

        let response = server.get("/health").await;
        response.assert_status_ok();

        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], "healthy");
    }

    #[tokio::test]
    async fn test_stats_empty_library() {
        let server = create_test_server().await;

        let response = server.get("/api/stats").await;
        response.assert_status_ok();

        let body: serde_json::Value = response.json();
        assert_eq!(body["track_count"], 0);
        assert_eq!(body["album_count"], 0);
    }

    #[tokio::test]
    async fn test_stats_with_data() {
        let server = create_test_server_with_data().await;

        let response = server.get("/api/stats").await;
        response.assert_status_ok();

        let body: serde_json::Value = response.json();
        assert_eq!(body["track_count"], 3);
        assert_eq!(body["album_count"], 1);
    }

    #[tokio::test]
    async fn test_list_tracks() {
        let server = create_test_server_with_data().await;

        let response = server.get("/api/tracks").await;
        response.assert_status_ok();

        let body: serde_json::Value = response.json();
        assert_eq!(body["total"], 3);
        assert_eq!(body["items"].as_array().unwrap().len(), 3);
    }

    #[tokio::test]
    async fn test_list_tracks_with_pagination() {
        let server = create_test_server_with_data().await;

        let response = server.get("/api/tracks?limit=2&offset=0").await;
        response.assert_status_ok();

        let body: serde_json::Value = response.json();
        assert_eq!(body["total"], 3);
        assert_eq!(body["items"].as_array().unwrap().len(), 2);
        assert_eq!(body["limit"], 2);
        assert_eq!(body["offset"], 0);
    }

    #[tokio::test]
    async fn test_list_albums() {
        let server = create_test_server_with_data().await;

        let response = server.get("/api/albums").await;
        response.assert_status_ok();

        let body: serde_json::Value = response.json();
        assert_eq!(body["total"], 1);
        assert_eq!(body["items"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_get_track_not_found() {
        let server = create_test_server().await;

        let response = server
            .get("/api/tracks/00000000-0000-0000-0000-000000000000")
            .await;
        response.assert_status_not_found();
    }

    #[tokio::test]
    async fn test_get_track_invalid_id() {
        let server = create_test_server().await;

        let response = server.get("/api/tracks/invalid-id").await;
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let server = create_test_server().await;

        let response = server.get("/api/search?q=").await;
        response.assert_status_bad_request();
    }

    #[tokio::test]
    async fn test_search_tracks() {
        let server = create_test_server_with_data().await;

        let response = server.get("/api/search?q=Track").await;
        response.assert_status_ok();

        let body: serde_json::Value = response.json();
        let items = body.as_array().unwrap();
        assert_eq!(items.len(), 3);
    }
}
