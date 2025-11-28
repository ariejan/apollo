//! [SQLite](https://sqlite.org/) database schema and operations.

// Allow integer casts that are safe in practice for music library data:
// - Track/album counts won't exceed i32::MAX
// - Duration in millis won't exceed i64::MAX for reasonable audio files
// - Channel counts, track numbers, etc. fit in their target types
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_lossless
)]

use crate::error::{DbError, DbResult};
use apollo_core::metadata::{Album, AlbumId, AudioFormat, Track, TrackId};
use apollo_core::playlist::{Playlist, PlaylistId, PlaylistKind, PlaylistLimit, PlaylistSort};
use chrono::{DateTime, Utc};
use sqlx::Row;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info};
use uuid::Uuid;

/// SQLite-based library storage.
pub struct SqliteLibrary {
    pool: SqlitePool,
}

impl SqliteLibrary {
    /// Create a new [SQLite](https://sqlite.org/) library connection.
    ///
    /// # Errors
    ///
    /// Returns an error if the database connection fails or migrations fail.
    pub async fn new(database_url: &str) -> DbResult<Self> {
        info!("Connecting to database: {database_url}");

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        let library = Self { pool };
        library.run_migrations().await?;

        Ok(library)
    }

    /// Create an in-memory database (useful for testing).
    ///
    /// # Errors
    ///
    /// Returns an error if the database connection fails or migrations fail.
    pub async fn in_memory() -> DbResult<Self> {
        Self::new("sqlite::memory:").await
    }

    /// Run database migrations.
    async fn run_migrations(&self) -> DbResult<()> {
        debug!("Running database migrations");

        // Run the initial schema migration
        sqlx::query(include_str!("../migrations/0001_initial_schema.sql"))
            .execute(&self.pool)
            .await?;

        // Run the playlists migration
        sqlx::query(include_str!("../migrations/0002_playlists.sql"))
            .execute(&self.pool)
            .await?;

        info!("Database migrations completed");
        Ok(())
    }

    /// Get a track by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn get_track(&self, id: &TrackId) -> DbResult<Option<Track>> {
        let id_str = id.0.to_string();

        let row = sqlx::query(
            r"SELECT id, path, title, artist, album_artist, album_id, album_title,
                     track_number, track_total, disc_number, disc_total, year,
                     genres, duration_ms, bitrate, sample_rate, channels, format,
                     musicbrainz_id, acoustid, added_at, modified_at, file_hash
              FROM tracks WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| row_to_track(&r)).transpose()
    }

    /// Get an album by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn get_album(&self, id: &AlbumId) -> DbResult<Option<Album>> {
        let id_str = id.0.to_string();

        let row = sqlx::query(
            r"SELECT id, title, artist, year, genres, track_count, disc_count,
                     musicbrainz_id, added_at, modified_at
              FROM albums WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| row_to_album(&r)).transpose()
    }

    /// Get all tracks in an album.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn get_album_tracks(&self, album_id: &AlbumId) -> DbResult<Vec<Track>> {
        let id_str = album_id.0.to_string();

        let rows = sqlx::query(
            r"SELECT id, path, title, artist, album_artist, album_id, album_title,
                     track_number, track_total, disc_number, disc_total, year,
                     genres, duration_ms, bitrate, sample_rate, channels, format,
                     musicbrainz_id, acoustid, added_at, modified_at, file_hash
              FROM tracks WHERE album_id = ?
              ORDER BY disc_number, track_number",
        )
        .bind(&id_str)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(row_to_track).collect()
    }

    /// Add a track to the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn add_track(&self, track: &Track) -> DbResult<TrackId> {
        let id_str = track.id.0.to_string();
        let path_str = track.path.to_string_lossy().to_string();
        let album_id_str = track.album_id.as_ref().map(|id| id.0.to_string());
        let genres_json = serde_json::to_string(&track.genres)
            .map_err(|e| DbError::Serialization(e.to_string()))?;
        let duration_ms = track.duration.as_millis() as i64;
        let format_str = format!("{:?}", track.format).to_lowercase();
        let added_at_str = track.added_at.to_rfc3339();
        let modified_at_str = track.modified_at.to_rfc3339();

        sqlx::query(
            r"INSERT INTO tracks (id, path, title, artist, album_artist, album_id, album_title,
                                  track_number, track_total, disc_number, disc_total, year,
                                  genres, duration_ms, bitrate, sample_rate, channels, format,
                                  musicbrainz_id, acoustid, added_at, modified_at, file_hash)
              VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(&path_str)
        .bind(&track.title)
        .bind(&track.artist)
        .bind(&track.album_artist)
        .bind(&album_id_str)
        .bind(&track.album_title)
        .bind(track.track_number.map(|n| n as i32))
        .bind(track.track_total.map(|n| n as i32))
        .bind(track.disc_number.map(|n| n as i32))
        .bind(track.disc_total.map(|n| n as i32))
        .bind(track.year)
        .bind(&genres_json)
        .bind(duration_ms)
        .bind(track.bitrate.map(|n| n as i32))
        .bind(track.sample_rate.map(|n| n as i32))
        .bind(track.channels.map(|n| n as i32))
        .bind(&format_str)
        .bind(&track.musicbrainz_id)
        .bind(&track.acoustid)
        .bind(&added_at_str)
        .bind(&modified_at_str)
        .bind(&track.file_hash)
        .execute(&self.pool)
        .await?;

        Ok(track.id.clone())
    }

    /// Update an existing track.
    ///
    /// # Errors
    ///
    /// Returns an error if the track doesn't exist or the database operation fails.
    pub async fn update_track(&self, track: &Track) -> DbResult<()> {
        let id_str = track.id.0.to_string();
        let path_str = track.path.to_string_lossy().to_string();
        let album_id_str = track.album_id.as_ref().map(|id| id.0.to_string());
        let genres_json = serde_json::to_string(&track.genres)
            .map_err(|e| DbError::Serialization(e.to_string()))?;
        let duration_ms = track.duration.as_millis() as i64;
        let format_str = format!("{:?}", track.format).to_lowercase();
        let modified_at_str = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r"UPDATE tracks SET
                path = ?, title = ?, artist = ?, album_artist = ?, album_id = ?,
                album_title = ?, track_number = ?, track_total = ?, disc_number = ?,
                disc_total = ?, year = ?, genres = ?, duration_ms = ?, bitrate = ?,
                sample_rate = ?, channels = ?, format = ?, musicbrainz_id = ?,
                acoustid = ?, modified_at = ?, file_hash = ?
              WHERE id = ?",
        )
        .bind(&path_str)
        .bind(&track.title)
        .bind(&track.artist)
        .bind(&track.album_artist)
        .bind(&album_id_str)
        .bind(&track.album_title)
        .bind(track.track_number.map(|n| n as i32))
        .bind(track.track_total.map(|n| n as i32))
        .bind(track.disc_number.map(|n| n as i32))
        .bind(track.disc_total.map(|n| n as i32))
        .bind(track.year)
        .bind(&genres_json)
        .bind(duration_ms)
        .bind(track.bitrate.map(|n| n as i32))
        .bind(track.sample_rate.map(|n| n as i32))
        .bind(track.channels.map(|n| n as i32))
        .bind(&format_str)
        .bind(&track.musicbrainz_id)
        .bind(&track.acoustid)
        .bind(&modified_at_str)
        .bind(&track.file_hash)
        .bind(&id_str)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("track {id_str}")));
        }

        Ok(())
    }

    /// Remove a track from the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the track doesn't exist or the database operation fails.
    pub async fn remove_track(&self, id: &TrackId) -> DbResult<()> {
        let id_str = id.0.to_string();

        let result = sqlx::query("DELETE FROM tracks WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("track {id_str}")));
        }

        Ok(())
    }

    /// Add an album to the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn add_album(&self, album: &Album) -> DbResult<AlbumId> {
        let id_str = album.id.0.to_string();
        let genres_json = serde_json::to_string(&album.genres)
            .map_err(|e| DbError::Serialization(e.to_string()))?;
        let added_at_str = album.added_at.to_rfc3339();
        let modified_at_str = album.modified_at.to_rfc3339();

        sqlx::query(
            r"INSERT INTO albums (id, title, artist, year, genres, track_count, disc_count,
                                  musicbrainz_id, added_at, modified_at)
              VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(&album.title)
        .bind(&album.artist)
        .bind(album.year)
        .bind(&genres_json)
        .bind(album.track_count as i32)
        .bind(album.disc_count as i32)
        .bind(&album.musicbrainz_id)
        .bind(&added_at_str)
        .bind(&modified_at_str)
        .execute(&self.pool)
        .await?;

        Ok(album.id.clone())
    }

    /// Update an existing album.
    ///
    /// # Errors
    ///
    /// Returns an error if the album doesn't exist or the database operation fails.
    pub async fn update_album(&self, album: &Album) -> DbResult<()> {
        let id_str = album.id.0.to_string();
        let genres_json = serde_json::to_string(&album.genres)
            .map_err(|e| DbError::Serialization(e.to_string()))?;
        let modified_at_str = Utc::now().to_rfc3339();

        let result = sqlx::query(
            r"UPDATE albums SET
                title = ?, artist = ?, year = ?, genres = ?, track_count = ?,
                disc_count = ?, musicbrainz_id = ?, modified_at = ?
              WHERE id = ?",
        )
        .bind(&album.title)
        .bind(&album.artist)
        .bind(album.year)
        .bind(&genres_json)
        .bind(album.track_count as i32)
        .bind(album.disc_count as i32)
        .bind(&album.musicbrainz_id)
        .bind(&modified_at_str)
        .bind(&id_str)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("album {id_str}")));
        }

        Ok(())
    }

    /// Remove an album from the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the album doesn't exist or the database operation fails.
    pub async fn remove_album(&self, id: &AlbumId) -> DbResult<()> {
        let id_str = id.0.to_string();

        let result = sqlx::query("DELETE FROM albums WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("album {id_str}")));
        }

        Ok(())
    }

    /// Search tracks using full-text search.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn search_tracks(&self, query: &str) -> DbResult<Vec<Track>> {
        let rows = sqlx::query(
            r"SELECT t.id, t.path, t.title, t.artist, t.album_artist, t.album_id, t.album_title,
                     t.track_number, t.track_total, t.disc_number, t.disc_total, t.year,
                     t.genres, t.duration_ms, t.bitrate, t.sample_rate, t.channels, t.format,
                     t.musicbrainz_id, t.acoustid, t.added_at, t.modified_at, t.file_hash
              FROM tracks t
              JOIN tracks_fts fts ON t.rowid = fts.rowid
              WHERE tracks_fts MATCH ?
              ORDER BY rank",
        )
        .bind(query)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(row_to_track).collect()
    }

    /// List all tracks in the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn list_tracks(&self, limit: u32, offset: u32) -> DbResult<Vec<Track>> {
        let rows = sqlx::query(
            r"SELECT id, path, title, artist, album_artist, album_id, album_title,
                     track_number, track_total, disc_number, disc_total, year,
                     genres, duration_ms, bitrate, sample_rate, channels, format,
                     musicbrainz_id, acoustid, added_at, modified_at, file_hash
              FROM tracks
              ORDER BY artist, album_title, disc_number, track_number
              LIMIT ? OFFSET ?",
        )
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(row_to_track).collect()
    }

    /// List all albums in the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn list_albums(&self, limit: u32, offset: u32) -> DbResult<Vec<Album>> {
        let rows = sqlx::query(
            r"SELECT id, title, artist, year, genres, track_count, disc_count,
                     musicbrainz_id, added_at, modified_at
              FROM albums
              ORDER BY artist, year, title
              LIMIT ? OFFSET ?",
        )
        .bind(limit as i32)
        .bind(offset as i32)
        .fetch_all(&self.pool)
        .await?;

        rows.iter().map(row_to_album).collect()
    }

    /// Count total tracks in the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn count_tracks(&self) -> DbResult<u64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM tracks")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get::<i64, _>("count") as u64)
    }

    /// Count total albums in the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn count_albums(&self) -> DbResult<u64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM albums")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get::<i64, _>("count") as u64)
    }

    /// Find tracks with duplicate file hashes (exact byte-for-byte duplicates).
    ///
    /// Returns groups of tracks that have the same file hash.
    /// Each group contains 2 or more tracks that are exact duplicates.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn find_exact_duplicates(&self) -> DbResult<Vec<Vec<Track>>> {
        // First, find all file hashes that appear more than once
        let hash_rows = sqlx::query(
            r"SELECT file_hash, COUNT(*) as count
              FROM tracks
              WHERE file_hash != ''
              GROUP BY file_hash
              HAVING count > 1
              ORDER BY count DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut duplicate_groups = Vec::new();

        for hash_row in hash_rows {
            let hash: String = hash_row.get("file_hash");

            // Get all tracks with this hash
            let track_rows = sqlx::query(
                r"SELECT id, path, title, artist, album_artist, album_id, album_title,
                         track_number, track_total, disc_number, disc_total, year,
                         genres, duration_ms, bitrate, sample_rate, channels, format,
                         musicbrainz_id, acoustid, added_at, modified_at, file_hash
                  FROM tracks WHERE file_hash = ?
                  ORDER BY added_at ASC",
            )
            .bind(&hash)
            .fetch_all(&self.pool)
            .await?;

            let tracks: Vec<Track> = track_rows
                .iter()
                .map(row_to_track)
                .collect::<DbResult<_>>()?;
            duplicate_groups.push(tracks);
        }

        Ok(duplicate_groups)
    }

    /// Find tracks that are likely duplicates based on metadata similarity.
    ///
    /// Matches tracks with the same title, artist, and similar duration (within tolerance).
    /// Returns groups of potentially duplicate tracks.
    ///
    /// # Arguments
    ///
    /// * `duration_tolerance_ms` - Maximum duration difference in milliseconds to consider similar
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn find_similar_duplicates(
        &self,
        duration_tolerance_ms: i64,
    ) -> DbResult<Vec<Vec<Track>>> {
        // Find tracks with matching title and artist
        let rows = sqlx::query(
            r"SELECT t1.id, t1.path, t1.title, t1.artist, t1.album_artist, t1.album_id, t1.album_title,
                     t1.track_number, t1.track_total, t1.disc_number, t1.disc_total, t1.year,
                     t1.genres, t1.duration_ms, t1.bitrate, t1.sample_rate, t1.channels, t1.format,
                     t1.musicbrainz_id, t1.acoustid, t1.added_at, t1.modified_at, t1.file_hash
              FROM tracks t1
              JOIN tracks t2 ON t1.title = t2.title
                            AND t1.artist = t2.artist
                            AND t1.id != t2.id
                            AND ABS(t1.duration_ms - t2.duration_ms) <= ?
              GROUP BY t1.id
              ORDER BY t1.artist, t1.title, t1.added_at",
        )
        .bind(duration_tolerance_ms)
        .fetch_all(&self.pool)
        .await?;

        // Group tracks by title+artist
        let mut groups: std::collections::HashMap<String, Vec<Track>> =
            std::collections::HashMap::new();

        for row in &rows {
            let track = row_to_track(row)?;
            let key = format!(
                "{}||{}",
                track.artist.to_lowercase(),
                track.title.to_lowercase()
            );
            groups.entry(key).or_default().push(track);
        }

        // Only return groups with multiple tracks
        Ok(groups.into_values().filter(|g| g.len() > 1).collect())
    }

    /// Check if a track with the given file hash already exists.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn track_exists_by_hash(&self, file_hash: &str) -> DbResult<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM tracks WHERE file_hash = ?")
            .bind(file_hash)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get::<i64, _>("count") > 0)
    }

    /// Get a track by its file hash.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn get_track_by_hash(&self, file_hash: &str) -> DbResult<Option<Track>> {
        let row = sqlx::query(
            r"SELECT id, path, title, artist, album_artist, album_id, album_title,
                     track_number, track_total, disc_number, disc_total, year,
                     genres, duration_ms, bitrate, sample_rate, channels, format,
                     musicbrainz_id, acoustid, added_at, modified_at, file_hash
              FROM tracks WHERE file_hash = ?
              LIMIT 1",
        )
        .bind(file_hash)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| row_to_track(&r)).transpose()
    }

    /// Get a track by its file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn get_track_by_path(&self, path: &std::path::Path) -> DbResult<Option<Track>> {
        let path_str = path.to_string_lossy().to_string();

        let row = sqlx::query(
            r"SELECT id, path, title, artist, album_artist, album_id, album_title,
                     track_number, track_total, disc_number, disc_total, year,
                     genres, duration_ms, bitrate, sample_rate, channels, format,
                     musicbrainz_id, acoustid, added_at, modified_at, file_hash
              FROM tracks WHERE path = ?",
        )
        .bind(&path_str)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|r| row_to_track(&r)).transpose()
    }

    // ========================================================================
    // Playlist operations
    // ========================================================================

    /// Get a playlist by its ID.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn get_playlist(&self, id: &PlaylistId) -> DbResult<Option<Playlist>> {
        let id_str = id.0.to_string();

        let row = sqlx::query(
            r"SELECT id, name, description, kind, query, sort, max_tracks, max_duration_secs,
                     created_at, modified_at
              FROM playlists WHERE id = ?",
        )
        .bind(&id_str)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => {
                let mut playlist = row_to_playlist(&r)?;

                // Load track IDs for static playlists
                if playlist.kind == PlaylistKind::Static {
                    playlist.track_ids = self.get_playlist_track_ids(&playlist.id).await?;
                }

                Ok(Some(playlist))
            }
            None => Ok(None),
        }
    }

    /// Get the track IDs for a playlist.
    async fn get_playlist_track_ids(&self, playlist_id: &PlaylistId) -> DbResult<Vec<TrackId>> {
        let id_str = playlist_id.0.to_string();

        let rows = sqlx::query(
            r"SELECT track_id FROM playlist_tracks
              WHERE playlist_id = ?
              ORDER BY position",
        )
        .bind(&id_str)
        .fetch_all(&self.pool)
        .await?;

        let mut track_ids = Vec::with_capacity(rows.len());
        for row in rows {
            let track_id_str: String = row.get("track_id");
            let track_id =
                Uuid::parse_str(&track_id_str).map_err(|e| DbError::InvalidData(e.to_string()))?;
            track_ids.push(TrackId(track_id));
        }

        Ok(track_ids)
    }

    /// Add a playlist to the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn add_playlist(&self, playlist: &Playlist) -> DbResult<PlaylistId> {
        let id_str = playlist.id.0.to_string();
        let kind_str = format!("{}", playlist.kind);
        let query_json = playlist
            .query
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| DbError::Serialization(e.to_string()))?;
        let sort_str = format!("{:?}", playlist.sort).to_lowercase();
        let created_at_str = playlist.created_at.to_rfc3339();
        let modified_at_str = playlist.modified_at.to_rfc3339();

        let max_tracks = playlist.limit.as_ref().and_then(|l| l.max_tracks);
        let max_duration_secs = playlist
            .limit
            .as_ref()
            .and_then(|l| l.max_duration_secs)
            .map(|d| d as i64);

        sqlx::query(
            r"INSERT INTO playlists (id, name, description, kind, query, sort, max_tracks,
                                     max_duration_secs, created_at, modified_at)
              VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id_str)
        .bind(&playlist.name)
        .bind(&playlist.description)
        .bind(&kind_str)
        .bind(&query_json)
        .bind(&sort_str)
        .bind(max_tracks.map(|n| n as i32))
        .bind(max_duration_secs)
        .bind(&created_at_str)
        .bind(&modified_at_str)
        .execute(&self.pool)
        .await?;

        // Add track IDs for static playlists
        if playlist.kind == PlaylistKind::Static {
            self.set_playlist_tracks(&playlist.id, &playlist.track_ids)
                .await?;
        }

        Ok(playlist.id.clone())
    }

    /// Set the tracks for a static playlist.
    async fn set_playlist_tracks(
        &self,
        playlist_id: &PlaylistId,
        track_ids: &[TrackId],
    ) -> DbResult<()> {
        let playlist_id_str = playlist_id.0.to_string();
        let now = Utc::now().to_rfc3339();

        // Delete existing tracks
        sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ?")
            .bind(&playlist_id_str)
            .execute(&self.pool)
            .await?;

        // Insert new tracks
        for (position, track_id) in track_ids.iter().enumerate() {
            let track_id_str = track_id.0.to_string();
            sqlx::query(
                r"INSERT INTO playlist_tracks (playlist_id, track_id, position, added_at)
                  VALUES (?, ?, ?, ?)",
            )
            .bind(&playlist_id_str)
            .bind(&track_id_str)
            .bind(position as i32)
            .bind(&now)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    /// Update an existing playlist.
    ///
    /// # Errors
    ///
    /// Returns an error if the playlist doesn't exist or the database operation fails.
    pub async fn update_playlist(&self, playlist: &Playlist) -> DbResult<()> {
        let id_str = playlist.id.0.to_string();
        let kind_str = format!("{}", playlist.kind);
        let query_json = playlist
            .query
            .as_ref()
            .map(serde_json::to_string)
            .transpose()
            .map_err(|e| DbError::Serialization(e.to_string()))?;
        let sort_str = format!("{:?}", playlist.sort).to_lowercase();
        let modified_at_str = Utc::now().to_rfc3339();

        let max_tracks = playlist.limit.as_ref().and_then(|l| l.max_tracks);
        let max_duration_secs = playlist
            .limit
            .as_ref()
            .and_then(|l| l.max_duration_secs)
            .map(|d| d as i64);

        let result = sqlx::query(
            r"UPDATE playlists SET
                name = ?, description = ?, kind = ?, query = ?, sort = ?,
                max_tracks = ?, max_duration_secs = ?, modified_at = ?
              WHERE id = ?",
        )
        .bind(&playlist.name)
        .bind(&playlist.description)
        .bind(&kind_str)
        .bind(&query_json)
        .bind(&sort_str)
        .bind(max_tracks.map(|n| n as i32))
        .bind(max_duration_secs)
        .bind(&modified_at_str)
        .bind(&id_str)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("playlist {id_str}")));
        }

        // Update track IDs for static playlists
        if playlist.kind == PlaylistKind::Static {
            self.set_playlist_tracks(&playlist.id, &playlist.track_ids)
                .await?;
        }

        Ok(())
    }

    /// Remove a playlist from the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the playlist doesn't exist or the database operation fails.
    pub async fn remove_playlist(&self, id: &PlaylistId) -> DbResult<()> {
        let id_str = id.0.to_string();

        // The playlist_tracks entries are deleted automatically via ON DELETE CASCADE
        let result = sqlx::query("DELETE FROM playlists WHERE id = ?")
            .bind(&id_str)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DbError::NotFound(format!("playlist {id_str}")));
        }

        Ok(())
    }

    /// List all playlists in the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn list_playlists(&self) -> DbResult<Vec<Playlist>> {
        let rows = sqlx::query(
            r"SELECT id, name, description, kind, query, sort, max_tracks, max_duration_secs,
                     created_at, modified_at
              FROM playlists
              ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        let mut playlists = Vec::with_capacity(rows.len());
        for row in &rows {
            let mut playlist = row_to_playlist(row)?;

            // Load track IDs for static playlists
            if playlist.kind == PlaylistKind::Static {
                playlist.track_ids = self.get_playlist_track_ids(&playlist.id).await?;
            }

            playlists.push(playlist);
        }

        Ok(playlists)
    }

    /// Count total playlists in the library.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn count_playlists(&self) -> DbResult<u64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM playlists")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get::<i64, _>("count") as u64)
    }

    /// Add a track to a static playlist.
    ///
    /// # Errors
    ///
    /// Returns an error if the playlist doesn't exist or the database operation fails.
    pub async fn add_track_to_playlist(
        &self,
        playlist_id: &PlaylistId,
        track_id: &TrackId,
    ) -> DbResult<()> {
        let playlist_id_str = playlist_id.0.to_string();
        let track_id_str = track_id.0.to_string();
        let now = Utc::now().to_rfc3339();

        // Get the next position
        let row = sqlx::query(
            "SELECT COALESCE(MAX(position), -1) + 1 as next_pos FROM playlist_tracks WHERE playlist_id = ?",
        )
        .bind(&playlist_id_str)
        .fetch_one(&self.pool)
        .await?;

        let next_pos: i32 = row.get("next_pos");

        sqlx::query(
            r"INSERT OR REPLACE INTO playlist_tracks (playlist_id, track_id, position, added_at)
              VALUES (?, ?, ?, ?)",
        )
        .bind(&playlist_id_str)
        .bind(&track_id_str)
        .bind(next_pos)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        // Update playlist modified_at
        let modified_at = Utc::now().to_rfc3339();
        sqlx::query("UPDATE playlists SET modified_at = ? WHERE id = ?")
            .bind(&modified_at)
            .bind(&playlist_id_str)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Remove a track from a static playlist.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn remove_track_from_playlist(
        &self,
        playlist_id: &PlaylistId,
        track_id: &TrackId,
    ) -> DbResult<()> {
        let playlist_id_str = playlist_id.0.to_string();
        let track_id_str = track_id.0.to_string();

        sqlx::query("DELETE FROM playlist_tracks WHERE playlist_id = ? AND track_id = ?")
            .bind(&playlist_id_str)
            .bind(&track_id_str)
            .execute(&self.pool)
            .await?;

        // Update playlist modified_at
        let modified_at = Utc::now().to_rfc3339();
        sqlx::query("UPDATE playlists SET modified_at = ? WHERE id = ?")
            .bind(&modified_at)
            .bind(&playlist_id_str)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get all tracks in a playlist.
    ///
    /// For static playlists, returns the stored tracks in order.
    /// For smart playlists, evaluates the query and returns matching tracks.
    ///
    /// # Errors
    ///
    /// Returns an error if the database operation fails.
    pub async fn get_playlist_tracks(&self, playlist_id: &PlaylistId) -> DbResult<Vec<Track>> {
        let id_str = playlist_id.0.to_string();

        // First, get the playlist to check its type
        let playlist = self
            .get_playlist(playlist_id)
            .await?
            .ok_or_else(|| DbError::NotFound(format!("playlist {id_str}")))?;

        match playlist.kind {
            PlaylistKind::Static => {
                // Get tracks in playlist order
                let rows = sqlx::query(
                    r"SELECT t.id, t.path, t.title, t.artist, t.album_artist, t.album_id, t.album_title,
                             t.track_number, t.track_total, t.disc_number, t.disc_total, t.year,
                             t.genres, t.duration_ms, t.bitrate, t.sample_rate, t.channels, t.format,
                             t.musicbrainz_id, t.acoustid, t.added_at, t.modified_at, t.file_hash
                      FROM tracks t
                      JOIN playlist_tracks pt ON t.id = pt.track_id
                      WHERE pt.playlist_id = ?
                      ORDER BY pt.position",
                )
                .bind(&id_str)
                .fetch_all(&self.pool)
                .await?;

                rows.iter().map(row_to_track).collect()
            }
            PlaylistKind::Smart => {
                // Evaluate the query
                self.evaluate_smart_playlist(&playlist).await
            }
        }
    }

    /// Evaluate a smart playlist query and return matching tracks.
    async fn evaluate_smart_playlist(&self, playlist: &Playlist) -> DbResult<Vec<Track>> {
        let query = playlist
            .query
            .as_ref()
            .ok_or_else(|| DbError::InvalidData("Smart playlist has no query".to_string()))?;

        // Build the SQL WHERE clause from the query
        let (where_clause, bindings) = query_to_sql(query);

        // Build the ORDER BY clause
        let order_by = match playlist.sort {
            PlaylistSort::Artist => "artist, album_title, disc_number, track_number",
            PlaylistSort::Album => "album_title, disc_number, track_number",
            PlaylistSort::Title => "title",
            PlaylistSort::AddedDesc => "added_at DESC",
            PlaylistSort::AddedAsc => "added_at ASC",
            PlaylistSort::YearDesc => "year DESC, album_title, disc_number, track_number",
            PlaylistSort::YearAsc => "year ASC, album_title, disc_number, track_number",
            PlaylistSort::Random => "RANDOM()",
        };

        // Build LIMIT clause
        let limit_clause = playlist
            .limit
            .as_ref()
            .and_then(|l| l.max_tracks)
            .map(|n| format!("LIMIT {n}"))
            .unwrap_or_default();

        let sql = format!(
            r"SELECT id, path, title, artist, album_artist, album_id, album_title,
                     track_number, track_total, disc_number, disc_total, year,
                     genres, duration_ms, bitrate, sample_rate, channels, format,
                     musicbrainz_id, acoustid, added_at, modified_at, file_hash
              FROM tracks
              WHERE {where_clause}
              ORDER BY {order_by}
              {limit_clause}"
        );

        // Build the query with bindings
        let mut query = sqlx::query(&sql);
        for binding in bindings {
            query = query.bind(binding);
        }

        let rows = query.fetch_all(&self.pool).await?;

        let mut tracks: Vec<Track> = rows.iter().map(row_to_track).collect::<DbResult<_>>()?;

        // Apply max_duration_secs limit if set
        if let Some(limit) = &playlist.limit
            && let Some(max_secs) = limit.max_duration_secs
        {
            let max_ms = max_secs * 1000;
            let mut total_ms = 0u64;
            tracks.retain(|track| {
                let track_ms = track.duration.as_millis() as u64;
                if total_ms + track_ms <= max_ms {
                    total_ms += track_ms;
                    true
                } else {
                    false
                }
            });
        }

        Ok(tracks)
    }
}

/// Convert a Query to a SQL WHERE clause.
fn query_to_sql(query: &apollo_core::query::Query) -> (String, Vec<String>) {
    use apollo_core::query::{Field, Query};

    match query {
        Query::All => ("1 = 1".to_string(), vec![]),
        Query::Text(text) => {
            let pattern = format!("%{text}%");
            (
                "(title LIKE ? OR artist LIKE ? OR album_title LIKE ?)".to_string(),
                vec![pattern.clone(), pattern.clone(), pattern],
            )
        }
        Query::Field { field, value } => {
            let column = match field {
                Field::Artist => "artist",
                Field::AlbumArtist => "album_artist",
                Field::Album => "album_title",
                Field::Title => "title",
                Field::Year => "year",
                Field::Genre => "genres",
                Field::Path => "path",
            };

            if *field == Field::Genre {
                // Genres are stored as JSON array
                let pattern = format!("%\"{value}\"%");
                (format!("{column} LIKE ?"), vec![pattern])
            } else if *field == Field::Path {
                // Path uses prefix matching
                let pattern = format!("{value}%");
                (format!("{column} LIKE ?"), vec![pattern])
            } else if *field == Field::Year {
                // Year uses exact match
                (format!("{column} = ?"), vec![value.clone()])
            } else {
                // Other fields use LIKE for partial matching
                let pattern = format!("%{value}%");
                (format!("{column} LIKE ?"), vec![pattern])
            }
        }
        Query::YearRange { start, end } => (
            "year BETWEEN ? AND ?".to_string(),
            vec![start.to_string(), end.to_string()],
        ),
        Query::And(queries) => {
            let mut clauses = Vec::new();
            let mut all_bindings = Vec::new();
            for q in queries {
                let (clause, bindings) = query_to_sql(q);
                clauses.push(format!("({clause})"));
                all_bindings.extend(bindings);
            }
            (clauses.join(" AND "), all_bindings)
        }
        Query::Or(queries) => {
            let mut clauses = Vec::new();
            let mut all_bindings = Vec::new();
            for q in queries {
                let (clause, bindings) = query_to_sql(q);
                clauses.push(format!("({clause})"));
                all_bindings.extend(bindings);
            }
            (clauses.join(" OR "), all_bindings)
        }
        Query::Not(inner) => {
            let (clause, bindings) = query_to_sql(inner);
            (format!("NOT ({clause})"), bindings)
        }
    }
}

/// Convert a database row to a Playlist.
fn row_to_playlist(row: &sqlx::sqlite::SqliteRow) -> DbResult<Playlist> {
    let id_str: String = row.get("id");
    let id = Uuid::parse_str(&id_str).map_err(|e| DbError::InvalidData(e.to_string()))?;

    let kind_str: String = row.get("kind");
    let kind = match kind_str.as_str() {
        "static" => PlaylistKind::Static,
        "smart" => PlaylistKind::Smart,
        _ => {
            return Err(DbError::InvalidData(format!(
                "Unknown playlist kind: {kind_str}"
            )));
        }
    };

    let query_json: Option<String> = row.get("query");
    let query = query_json
        .map(|s| serde_json::from_str(&s))
        .transpose()
        .map_err(|e| DbError::Serialization(e.to_string()))?;

    let sort_str: String = row.get("sort");
    let sort = parse_playlist_sort(&sort_str);

    let max_tracks: Option<i32> = row.get("max_tracks");
    let max_duration_secs: Option<i64> = row.get("max_duration_secs");
    let limit = if max_tracks.is_some() || max_duration_secs.is_some() {
        Some(PlaylistLimit {
            max_tracks: max_tracks.map(|n| n as u32),
            max_duration_secs: max_duration_secs.map(|n| n as u64),
        })
    } else {
        None
    };

    let created_at_str: String = row.get("created_at");
    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map_err(|e| DbError::InvalidData(e.to_string()))?
        .with_timezone(&Utc);

    let modified_at_str: String = row.get("modified_at");
    let modified_at = DateTime::parse_from_rfc3339(&modified_at_str)
        .map_err(|e| DbError::InvalidData(e.to_string()))?
        .with_timezone(&Utc);

    Ok(Playlist {
        id: PlaylistId(id),
        name: row.get("name"),
        description: row.get("description"),
        kind,
        query,
        sort,
        limit,
        track_ids: Vec::new(), // Loaded separately
        created_at,
        modified_at,
    })
}

/// Parse playlist sort from string.
fn parse_playlist_sort(s: &str) -> PlaylistSort {
    match s.to_lowercase().as_str() {
        "album" => PlaylistSort::Album,
        "title" => PlaylistSort::Title,
        "addeddesc" => PlaylistSort::AddedDesc,
        "addedasc" => PlaylistSort::AddedAsc,
        "yeardesc" => PlaylistSort::YearDesc,
        "yearasc" => PlaylistSort::YearAsc,
        "random" => PlaylistSort::Random,
        // Default to Artist for "artist" and any unknown values
        _ => PlaylistSort::Artist,
    }
}

/// Convert a database row to a Track.
fn row_to_track(row: &sqlx::sqlite::SqliteRow) -> DbResult<Track> {
    let id_str: String = row.get("id");
    let id = Uuid::parse_str(&id_str).map_err(|e| DbError::InvalidData(e.to_string()))?;

    let path_str: String = row.get("path");
    let album_id_str: Option<String> = row.get("album_id");
    let album_id = album_id_str
        .map(|s| Uuid::parse_str(&s).map(AlbumId))
        .transpose()
        .map_err(|e| DbError::InvalidData(e.to_string()))?;

    let genres_json: String = row.get("genres");
    let genres: Vec<String> =
        serde_json::from_str(&genres_json).map_err(|e| DbError::Serialization(e.to_string()))?;

    let duration_ms: i64 = row.get("duration_ms");
    let format_str: String = row.get("format");
    let format = parse_audio_format(&format_str);

    let added_at_str: String = row.get("added_at");
    let added_at = DateTime::parse_from_rfc3339(&added_at_str)
        .map_err(|e| DbError::InvalidData(e.to_string()))?
        .with_timezone(&Utc);

    let modified_at_str: String = row.get("modified_at");
    let modified_at = DateTime::parse_from_rfc3339(&modified_at_str)
        .map_err(|e| DbError::InvalidData(e.to_string()))?
        .with_timezone(&Utc);

    Ok(Track {
        id: TrackId(id),
        path: PathBuf::from(path_str),
        title: row.get("title"),
        artist: row.get("artist"),
        album_artist: row.get("album_artist"),
        album_id,
        album_title: row.get("album_title"),
        track_number: row.get::<Option<i32>, _>("track_number").map(|n| n as u32),
        track_total: row.get::<Option<i32>, _>("track_total").map(|n| n as u32),
        disc_number: row.get::<Option<i32>, _>("disc_number").map(|n| n as u32),
        disc_total: row.get::<Option<i32>, _>("disc_total").map(|n| n as u32),
        year: row.get("year"),
        genres,
        duration: Duration::from_millis(duration_ms as u64),
        bitrate: row.get::<Option<i32>, _>("bitrate").map(|n| n as u32),
        sample_rate: row.get::<Option<i32>, _>("sample_rate").map(|n| n as u32),
        channels: row.get::<Option<i32>, _>("channels").map(|n| n as u8),
        format,
        musicbrainz_id: row.get("musicbrainz_id"),
        acoustid: row.get("acoustid"),
        added_at,
        modified_at,
        file_hash: row.get("file_hash"),
    })
}

/// Convert a database row to an Album.
fn row_to_album(row: &sqlx::sqlite::SqliteRow) -> DbResult<Album> {
    let id_str: String = row.get("id");
    let id = Uuid::parse_str(&id_str).map_err(|e| DbError::InvalidData(e.to_string()))?;

    let genres_json: String = row.get("genres");
    let genres: Vec<String> =
        serde_json::from_str(&genres_json).map_err(|e| DbError::Serialization(e.to_string()))?;

    let added_at_str: String = row.get("added_at");
    let added_at = DateTime::parse_from_rfc3339(&added_at_str)
        .map_err(|e| DbError::InvalidData(e.to_string()))?
        .with_timezone(&Utc);

    let modified_at_str: String = row.get("modified_at");
    let modified_at = DateTime::parse_from_rfc3339(&modified_at_str)
        .map_err(|e| DbError::InvalidData(e.to_string()))?
        .with_timezone(&Utc);

    Ok(Album {
        id: AlbumId(id),
        title: row.get("title"),
        artist: row.get("artist"),
        year: row.get("year"),
        genres,
        track_count: row.get::<i32, _>("track_count") as u32,
        disc_count: row.get::<i32, _>("disc_count") as u32,
        musicbrainz_id: row.get("musicbrainz_id"),
        added_at,
        modified_at,
    })
}

/// Parse audio format from string.
fn parse_audio_format(s: &str) -> AudioFormat {
    match s.to_lowercase().as_str() {
        "mp3" => AudioFormat::Mp3,
        "flac" => AudioFormat::Flac,
        "ogg" => AudioFormat::Ogg,
        "opus" => AudioFormat::Opus,
        "aac" => AudioFormat::Aac,
        "wav" => AudioFormat::Wav,
        "aiff" => AudioFormat::Aiff,
        _ => AudioFormat::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_database() {
        let db = SqliteLibrary::in_memory().await.unwrap();

        // Should start empty
        assert_eq!(db.count_tracks().await.unwrap(), 0);
        assert_eq!(db.count_albums().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_track_crud() {
        let db = SqliteLibrary::in_memory().await.unwrap();

        // Create a track
        let track = Track::new(
            PathBuf::from("/music/test.mp3"),
            "Test Song".to_string(),
            "Test Artist".to_string(),
            Duration::from_secs(180),
        );

        // Add the track
        let id = db.add_track(&track).await.unwrap();
        assert_eq!(id, track.id);

        // Retrieve the track
        let retrieved = db.get_track(&id).await.unwrap().unwrap();
        assert_eq!(retrieved.title, "Test Song");
        assert_eq!(retrieved.artist, "Test Artist");
        assert_eq!(retrieved.duration.as_secs(), 180);

        // Update the track
        let mut updated_track = retrieved;
        updated_track.title = "Updated Song".to_string();
        db.update_track(&updated_track).await.unwrap();

        // Verify update
        let retrieved = db.get_track(&id).await.unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Song");

        // Remove the track
        db.remove_track(&id).await.unwrap();

        // Verify removal
        let retrieved = db.get_track(&id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_album_crud() {
        let db = SqliteLibrary::in_memory().await.unwrap();

        // Create an album
        let mut album = Album::new("Test Album".to_string(), "Test Artist".to_string());
        album.year = Some(2023);
        album.track_count = 10;

        // Add the album
        let id = db.add_album(&album).await.unwrap();
        assert_eq!(id, album.id);

        // Retrieve the album
        let retrieved = db.get_album(&id).await.unwrap().unwrap();
        assert_eq!(retrieved.title, "Test Album");
        assert_eq!(retrieved.artist, "Test Artist");
        assert_eq!(retrieved.year, Some(2023));
        assert_eq!(retrieved.track_count, 10);

        // Update the album
        let mut updated_album = retrieved;
        updated_album.title = "Updated Album".to_string();
        db.update_album(&updated_album).await.unwrap();

        // Verify update
        let retrieved = db.get_album(&id).await.unwrap().unwrap();
        assert_eq!(retrieved.title, "Updated Album");

        // Remove the album
        db.remove_album(&id).await.unwrap();

        // Verify removal
        let retrieved = db.get_album(&id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_album_tracks() {
        let db = SqliteLibrary::in_memory().await.unwrap();

        // Create an album
        let album = Album::new("Test Album".to_string(), "Test Artist".to_string());
        db.add_album(&album).await.unwrap();

        // Create tracks for the album
        for i in 1..=3 {
            let mut track = Track::new(
                PathBuf::from(format!("/music/track{i}.mp3")),
                format!("Track {i}"),
                "Test Artist".to_string(),
                Duration::from_secs(180),
            );
            track.album_id = Some(album.id.clone());
            track.album_title = Some("Test Album".to_string());
            track.track_number = Some(i);
            db.add_track(&track).await.unwrap();
        }

        // Get album tracks
        let tracks = db.get_album_tracks(&album.id).await.unwrap();
        assert_eq!(tracks.len(), 3);
        assert_eq!(tracks[0].title, "Track 1");
        assert_eq!(tracks[1].title, "Track 2");
        assert_eq!(tracks[2].title, "Track 3");
    }

    #[tokio::test]
    async fn test_list_tracks_and_albums() {
        let db = SqliteLibrary::in_memory().await.unwrap();

        // Add some tracks
        for i in 1..=5 {
            let track = Track::new(
                PathBuf::from(format!("/music/track{i}.mp3")),
                format!("Track {i}"),
                "Test Artist".to_string(),
                Duration::from_secs(180),
            );
            db.add_track(&track).await.unwrap();
        }

        // Add some albums
        for i in 1..=3 {
            let album = Album::new(format!("Album {i}"), "Test Artist".to_string());
            db.add_album(&album).await.unwrap();
        }

        // List with pagination
        let tracks = db.list_tracks(2, 0).await.unwrap();
        assert_eq!(tracks.len(), 2);

        let tracks = db.list_tracks(10, 3).await.unwrap();
        assert_eq!(tracks.len(), 2); // Only 2 remaining after offset 3

        let albums = db.list_albums(10, 0).await.unwrap();
        assert_eq!(albums.len(), 3);
    }

    #[tokio::test]
    async fn test_static_playlist_crud() {
        let db = SqliteLibrary::in_memory().await.unwrap();

        // Create a static playlist
        let playlist = Playlist::new_static("My Favorites").with_description("My favorite songs");

        // Add the playlist
        let id = db.add_playlist(&playlist).await.unwrap();
        assert_eq!(id, playlist.id);

        // Retrieve the playlist
        let retrieved = db.get_playlist(&id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "My Favorites");
        assert_eq!(retrieved.description, Some("My favorite songs".to_string()));
        assert!(retrieved.is_static());

        // Update the playlist
        let mut updated_playlist = retrieved;
        updated_playlist.name = "Updated Favorites".to_string();
        db.update_playlist(&updated_playlist).await.unwrap();

        // Verify update
        let retrieved = db.get_playlist(&id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated Favorites");

        // Remove the playlist
        db.remove_playlist(&id).await.unwrap();

        // Verify removal
        let retrieved = db.get_playlist(&id).await.unwrap();
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_smart_playlist_crud() {
        let db = SqliteLibrary::in_memory().await.unwrap();

        // Create a smart playlist
        let query = apollo_core::query::Query::parse("artist:Beatles").unwrap();
        let playlist = Playlist::new_smart("Beatles Songs", query)
            .with_sort(PlaylistSort::YearDesc)
            .with_max_tracks(100);

        // Add the playlist
        let id = db.add_playlist(&playlist).await.unwrap();

        // Retrieve the playlist
        let retrieved = db.get_playlist(&id).await.unwrap().unwrap();
        assert_eq!(retrieved.name, "Beatles Songs");
        assert!(retrieved.is_smart());
        assert_eq!(retrieved.sort, PlaylistSort::YearDesc);
        assert!(retrieved.query.is_some());
        assert!(retrieved.limit.is_some());
        assert_eq!(retrieved.limit.unwrap().max_tracks, Some(100));
    }

    #[tokio::test]
    async fn test_playlist_tracks() {
        let db = SqliteLibrary::in_memory().await.unwrap();

        // Create some tracks
        let track1 = Track::new(
            PathBuf::from("/music/track1.mp3"),
            "Track 1".to_string(),
            "Artist".to_string(),
            Duration::from_secs(180),
        );
        let track2 = Track::new(
            PathBuf::from("/music/track2.mp3"),
            "Track 2".to_string(),
            "Artist".to_string(),
            Duration::from_secs(240),
        );
        db.add_track(&track1).await.unwrap();
        db.add_track(&track2).await.unwrap();

        // Create a static playlist
        let playlist = Playlist::new_static("Test Playlist");
        let playlist_id = db.add_playlist(&playlist).await.unwrap();

        // Add tracks to playlist
        db.add_track_to_playlist(&playlist_id, &track1.id)
            .await
            .unwrap();
        db.add_track_to_playlist(&playlist_id, &track2.id)
            .await
            .unwrap();

        // Get playlist tracks
        let tracks = db.get_playlist_tracks(&playlist_id).await.unwrap();
        assert_eq!(tracks.len(), 2);
        assert_eq!(tracks[0].title, "Track 1");
        assert_eq!(tracks[1].title, "Track 2");

        // Remove a track from playlist
        db.remove_track_from_playlist(&playlist_id, &track1.id)
            .await
            .unwrap();
        let tracks = db.get_playlist_tracks(&playlist_id).await.unwrap();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].title, "Track 2");
    }

    #[tokio::test]
    async fn test_smart_playlist_evaluation() {
        let db = SqliteLibrary::in_memory().await.unwrap();

        // Create tracks with different artists
        for i in 1..=3 {
            let mut track = Track::new(
                PathBuf::from(format!("/music/beatles_{i}.mp3")),
                format!("Song {i}"),
                "Beatles".to_string(),
                Duration::from_secs(180),
            );
            track.year = Some(1965 + i as i32);
            db.add_track(&track).await.unwrap();
        }

        for i in 1..=2 {
            let track = Track::new(
                PathBuf::from(format!("/music/stones_{i}.mp3")),
                format!("Stone Song {i}"),
                "Rolling Stones".to_string(),
                Duration::from_secs(200),
            );
            db.add_track(&track).await.unwrap();
        }

        // Create a smart playlist for Beatles
        let query = apollo_core::query::Query::parse("artist:Beatles").unwrap();
        let playlist = Playlist::new_smart("Beatles", query).with_sort(PlaylistSort::YearAsc);
        let playlist_id = db.add_playlist(&playlist).await.unwrap();

        // Evaluate the playlist
        let tracks = db.get_playlist_tracks(&playlist_id).await.unwrap();
        assert_eq!(tracks.len(), 3);
        assert!(tracks.iter().all(|t| t.artist == "Beatles"));
        // Should be sorted by year ascending
        assert!(tracks[0].year <= tracks[1].year);
        assert!(tracks[1].year <= tracks[2].year);
    }

    #[tokio::test]
    async fn test_list_playlists() {
        let db = SqliteLibrary::in_memory().await.unwrap();

        // Create some playlists
        db.add_playlist(&Playlist::new_static("Playlist A"))
            .await
            .unwrap();
        db.add_playlist(&Playlist::new_static("Playlist B"))
            .await
            .unwrap();

        let query = apollo_core::query::Query::All;
        db.add_playlist(&Playlist::new_smart("All Songs", query))
            .await
            .unwrap();

        // List all playlists
        let playlists = db.list_playlists().await.unwrap();
        assert_eq!(playlists.len(), 3);

        // Count playlists
        let count = db.count_playlists().await.unwrap();
        assert_eq!(count, 3);
    }
}
