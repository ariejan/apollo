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
//! - `GET /api/search` - Search tracks by query
//! - `GET /api/stats` - Get library statistics
//! - `GET /swagger-ui` - Interactive API documentation

mod error;
mod handlers;
mod state;

pub use error::ApiError;
pub use handlers::{
    ErrorResponse, HealthResponse, PaginatedAlbumsResponse, PaginatedTracksResponse, StatsResponse,
};
pub use state::AppState;

use apollo_core::metadata::{Album, AlbumId, Artist, AudioFormat, Track, TrackId};
use axum::{Router, routing::get};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
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
        handlers::search_tracks
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
            PaginatedAlbumsResponse
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
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Track endpoints
        .route("/api/tracks", get(handlers::list_tracks))
        .route("/api/tracks/:id", get(handlers::get_track))
        // Album endpoints
        .route("/api/albums", get(handlers::list_albums))
        .route("/api/albums/:id", get(handlers::get_album))
        .route("/api/albums/:id/tracks", get(handlers::get_album_tracks))
        // Search endpoint
        .route("/api/search", get(handlers::search_tracks))
        // Stats endpoint
        .route("/api/stats", get(handlers::get_stats))
        // Health check
        .route("/health", get(handlers::health_check))
        // OpenAPI documentation
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Add shared state
        .with_state(state)
        // Add middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
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
