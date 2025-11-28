//! Library abstraction for track and album management.

use crate::error::Result;
use crate::metadata::{Album, AlbumId, Track, TrackId};

/// Trait for library storage backends.
pub trait Library {
    /// Get a track by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    fn get_track(&self, id: &TrackId) -> Result<Option<Track>>;

    /// Get an album by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    fn get_album(&self, id: &AlbumId) -> Result<Option<Album>>;

    /// Get all tracks in an album.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    fn get_album_tracks(&self, album_id: &AlbumId) -> Result<Vec<Track>>;

    /// Add a track to the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    fn add_track(&mut self, track: Track) -> Result<TrackId>;

    /// Update an existing track.
    ///
    /// # Errors
    ///
    /// Returns an error if the track doesn't exist or the database operation fails.
    fn update_track(&mut self, track: Track) -> Result<()>;

    /// Remove a track from the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the track doesn't exist or the database operation fails.
    fn remove_track(&mut self, id: &TrackId) -> Result<()>;

    /// Add an album to the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    fn add_album(&mut self, album: Album) -> Result<AlbumId>;

    /// Update an existing album.
    ///
    /// # Errors
    ///
    /// Returns an error if the album doesn't exist or the database operation fails.
    fn update_album(&mut self, album: Album) -> Result<()>;

    /// Remove an album from the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the album doesn't exist or the database operation fails.
    fn remove_album(&mut self, id: &AlbumId) -> Result<()>;
}

/// Statistics about the library.
#[derive(Debug, Clone, Default)]
pub struct LibraryStats {
    /// Total number of tracks.
    pub track_count: u64,
    /// Total number of albums.
    pub album_count: u64,
    /// Total number of unique artists.
    pub artist_count: u64,
    /// Total duration of all tracks.
    pub total_duration_secs: u64,
    /// Total file size in bytes.
    pub total_size_bytes: u64,
}
