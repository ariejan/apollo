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
}
