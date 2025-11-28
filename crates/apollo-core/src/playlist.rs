//! Playlist types for organizing tracks.
//!
//! Apollo supports two types of playlists:
//!
//! 1. **Static playlists** - A fixed list of tracks that can be manually curated
//! 2. **Smart playlists** - Dynamic playlists that match tracks based on a query
//!
//! # Smart Playlists
//!
//! Smart playlists use the query language from [`crate::query`] to automatically
//! match tracks. The playlist updates automatically as tracks are added or modified.
//!
//! ## Query Examples
//!
//! - `artist:Beatles` - All Beatles songs
//! - `year:2020..2024` - Songs from 2020-2024
//! - `genre:rock` - All rock songs
//! - `added:30d` - Songs added in the last 30 days

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

use crate::metadata::TrackId;
use crate::query::Query;

/// Unique identifier for a playlist.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlaylistId(pub Uuid);

impl PlaylistId {
    /// Generate a new unique playlist ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PlaylistId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for PlaylistId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The type of playlist.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaylistKind {
    /// A static playlist with a fixed list of tracks.
    #[default]
    Static,
    /// A smart playlist that dynamically matches tracks based on a query.
    Smart,
}

impl fmt::Display for PlaylistKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Static => write!(f, "static"),
            Self::Smart => write!(f, "smart"),
        }
    }
}

/// Options for limiting smart playlist results.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlaylistLimit {
    /// Maximum number of tracks.
    pub max_tracks: Option<u32>,
    /// Maximum total duration in seconds.
    pub max_duration_secs: Option<u64>,
}

/// Sort order for smart playlist results.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlaylistSort {
    /// Sort by artist name, then album, then track number.
    #[default]
    Artist,
    /// Sort by album name, then track number.
    Album,
    /// Sort by track title.
    Title,
    /// Sort by date added (newest first).
    AddedDesc,
    /// Sort by date added (oldest first).
    AddedAsc,
    /// Sort by year (newest first).
    YearDesc,
    /// Sort by year (oldest first).
    YearAsc,
    /// Random order.
    Random,
}

impl fmt::Display for PlaylistSort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Artist => write!(f, "artist"),
            Self::Album => write!(f, "album"),
            Self::Title => write!(f, "title"),
            Self::AddedDesc => write!(f, "added (newest)"),
            Self::AddedAsc => write!(f, "added (oldest)"),
            Self::YearDesc => write!(f, "year (newest)"),
            Self::YearAsc => write!(f, "year (oldest)"),
            Self::Random => write!(f, "random"),
        }
    }
}

/// A playlist of tracks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    /// Unique identifier.
    pub id: PlaylistId,
    /// Playlist name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Playlist type.
    pub kind: PlaylistKind,
    /// Query for smart playlists (None for static playlists).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<Query>,
    /// Sort order for smart playlists.
    pub sort: PlaylistSort,
    /// Limits for smart playlists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PlaylistLimit>,
    /// Track IDs for static playlists.
    pub track_ids: Vec<TrackId>,
    /// When the playlist was created.
    pub created_at: DateTime<Utc>,
    /// When the playlist was last modified.
    pub modified_at: DateTime<Utc>,
}

impl Playlist {
    /// Create a new static playlist.
    #[must_use]
    pub fn new_static(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: PlaylistId::new(),
            name: name.into(),
            description: None,
            kind: PlaylistKind::Static,
            query: None,
            sort: PlaylistSort::default(),
            limit: None,
            track_ids: Vec::new(),
            created_at: now,
            modified_at: now,
        }
    }

    /// Create a new smart playlist with a query.
    #[must_use]
    pub fn new_smart(name: impl Into<String>, query: Query) -> Self {
        let now = Utc::now();
        Self {
            id: PlaylistId::new(),
            name: name.into(),
            description: None,
            kind: PlaylistKind::Smart,
            query: Some(query),
            sort: PlaylistSort::default(),
            limit: None,
            track_ids: Vec::new(),
            created_at: now,
            modified_at: now,
        }
    }

    /// Set the playlist description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the sort order.
    #[must_use]
    pub const fn with_sort(mut self, sort: PlaylistSort) -> Self {
        self.sort = sort;
        self
    }

    /// Set the limits.
    #[must_use]
    pub const fn with_limit(mut self, limit: PlaylistLimit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set maximum tracks limit.
    #[must_use]
    pub fn with_max_tracks(mut self, max_tracks: u32) -> Self {
        let limit = self.limit.get_or_insert_with(PlaylistLimit::default);
        limit.max_tracks = Some(max_tracks);
        self
    }

    /// Set maximum duration limit.
    #[must_use]
    pub fn with_max_duration_secs(mut self, max_duration_secs: u64) -> Self {
        let limit = self.limit.get_or_insert_with(PlaylistLimit::default);
        limit.max_duration_secs = Some(max_duration_secs);
        self
    }

    /// Add a track to a static playlist.
    ///
    /// Does nothing for smart playlists.
    pub fn add_track(&mut self, track_id: TrackId) {
        if self.kind == PlaylistKind::Static {
            self.track_ids.push(track_id);
            self.modified_at = Utc::now();
        }
    }

    /// Remove a track from a static playlist.
    ///
    /// Does nothing for smart playlists.
    pub fn remove_track(&mut self, track_id: &TrackId) {
        if self.kind == PlaylistKind::Static {
            self.track_ids.retain(|id| id != track_id);
            self.modified_at = Utc::now();
        }
    }

    /// Check if this is a smart playlist.
    #[must_use]
    pub fn is_smart(&self) -> bool {
        self.kind == PlaylistKind::Smart
    }

    /// Check if this is a static playlist.
    #[must_use]
    pub fn is_static(&self) -> bool {
        self.kind == PlaylistKind::Static
    }

    /// Get the track count.
    ///
    /// For static playlists, returns the stored track count.
    /// For smart playlists, this may be 0 until the playlist is evaluated.
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Vec::len() is not const-stable
    pub fn track_count(&self) -> usize {
        self.track_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_static_playlist() {
        let playlist = Playlist::new_static("My Favorites");

        assert!(playlist.is_static());
        assert!(!playlist.is_smart());
        assert_eq!(playlist.name, "My Favorites");
        assert!(playlist.track_ids.is_empty());
        assert!(playlist.query.is_none());
    }

    #[test]
    fn test_create_smart_playlist() {
        let query = Query::parse("artist:Beatles").unwrap();
        let playlist = Playlist::new_smart("Beatles Songs", query);

        assert!(playlist.is_smart());
        assert!(!playlist.is_static());
        assert_eq!(playlist.name, "Beatles Songs");
        assert!(playlist.query.is_some());
    }

    #[test]
    fn test_add_track_to_static() {
        let mut playlist = Playlist::new_static("Test");
        let track_id = TrackId::new();

        playlist.add_track(track_id.clone());

        assert_eq!(playlist.track_count(), 1);
        assert_eq!(playlist.track_ids[0], track_id);
    }

    #[test]
    fn test_add_track_to_smart_noop() {
        let query = Query::parse("artist:Test").unwrap();
        let mut playlist = Playlist::new_smart("Test", query);
        let track_id = TrackId::new();

        playlist.add_track(track_id);

        // Should not add track to smart playlist
        assert_eq!(playlist.track_count(), 0);
    }

    #[test]
    fn test_remove_track() {
        let mut playlist = Playlist::new_static("Test");
        let track_id1 = TrackId::new();
        let track_id2 = TrackId::new();

        playlist.add_track(track_id1.clone());
        playlist.add_track(track_id2.clone());
        playlist.remove_track(&track_id1);

        assert_eq!(playlist.track_count(), 1);
        assert_eq!(playlist.track_ids[0], track_id2);
    }

    #[test]
    fn test_builder_methods() {
        let playlist = Playlist::new_static("Test")
            .with_description("A test playlist")
            .with_sort(PlaylistSort::Random)
            .with_max_tracks(100);

        assert_eq!(playlist.description, Some("A test playlist".to_string()));
        assert_eq!(playlist.sort, PlaylistSort::Random);
        assert!(playlist.limit.is_some());
        assert_eq!(playlist.limit.unwrap().max_tracks, Some(100));
    }

    #[test]
    fn test_playlist_id_display() {
        let id = PlaylistId::new();
        let display = format!("{id}");
        // UUID format validation
        assert!(uuid::Uuid::parse_str(&display).is_ok());
    }

    #[test]
    fn test_playlist_kind_display() {
        assert_eq!(format!("{}", PlaylistKind::Static), "static");
        assert_eq!(format!("{}", PlaylistKind::Smart), "smart");
    }

    #[test]
    fn test_playlist_serialization() {
        let playlist = Playlist::new_static("Test");
        let json = serde_json::to_string(&playlist).unwrap();
        let deserialized: Playlist = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.name, playlist.name);
        assert_eq!(deserialized.kind, playlist.kind);
    }
}
