//! Import service for orchestrating music import with metadata lookup.
//!
//! This module provides a complete import pipeline that:
//! 1. Scans a directory for audio files
//! 2. Reads metadata from files
//! 3. Optionally looks up metadata from `MusicBrainz`
//! 4. Groups tracks into albums
//! 5. Creates album entries in the database
//! 6. Optionally fetches album art
//! 7. Optionally writes tags back to files
//! 8. Imports tracks into the database

use apollo_audio::{ScanOptions, ScanProgress, scan_directory, write_metadata};
use apollo_core::Config;
use apollo_core::metadata::{Album, AlbumId, Track};
use apollo_db::SqliteLibrary;
use apollo_sources::coverart::{CoverArtClient, ImageSize};
use apollo_sources::musicbrainz::MusicBrainzClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

/// Options for controlling the import process.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct ImportOptions {
    /// Directory to import from.
    pub source_path: PathBuf,
    /// Maximum recursion depth for directory scanning.
    pub max_depth: Option<usize>,
    /// Follow symbolic links during scanning.
    pub follow_symlinks: bool,
    /// Look up metadata from `MusicBrainz` for tracks without MBIDs.
    pub auto_tag: bool,
    /// Minimum score for `MusicBrainz` matches (0-100).
    pub min_match_score: u8,
    /// Group tracks into albums and create album entries.
    pub create_albums: bool,
    /// Fetch album art from Cover Art Archive.
    pub fetch_album_art: bool,
    /// Write updated metadata back to files.
    pub write_tags: bool,
    /// Compute file hashes for deduplication.
    pub compute_hashes: bool,
}

impl ImportOptions {
    /// Create options from configuration.
    #[must_use]
    pub const fn from_config(config: &Config) -> Self {
        Self {
            source_path: PathBuf::new(),
            max_depth: None,
            follow_symlinks: false,
            auto_tag: config.musicbrainz.auto_tag,
            min_match_score: 80,
            create_albums: config.import.auto_create_albums,
            fetch_album_art: config.import.copy_album_art,
            write_tags: config.import.write_tags,
            compute_hashes: config.import.compute_hashes,
        }
    }

    /// Set the source path.
    #[must_use]
    pub fn with_source(mut self, path: PathBuf) -> Self {
        self.source_path = path;
        self
    }
}

/// Progress update during import.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ImportProgress {
    /// Scanning directory for files.
    Scanning {
        files_found: usize,
        current_file: Option<String>,
    },
    /// Looking up metadata for a track.
    LookingUp { track_index: usize, total: usize },
    /// Creating albums.
    CreatingAlbums { count: usize },
    /// Fetching album art.
    FetchingArt { album_index: usize, total: usize },
    /// Importing tracks to database.
    Importing {
        imported: usize,
        skipped: usize,
        failed: usize,
        total: usize,
    },
    /// Import complete.
    Complete(ImportResult),
}

/// Result of an import operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImportResult {
    /// Number of tracks found.
    pub tracks_found: usize,
    /// Number of tracks successfully imported.
    pub tracks_imported: usize,
    /// Number of tracks skipped (duplicates).
    pub tracks_skipped: usize,
    /// Number of tracks that failed to import.
    pub tracks_failed: usize,
    /// Number of albums created.
    pub albums_created: usize,
    /// Errors encountered during import.
    pub errors: Vec<String>,
}

/// Service for importing music into the library.
pub struct ImportService {
    db: Arc<SqliteLibrary>,
    mb_client: Option<MusicBrainzClient>,
    art_client: Option<CoverArtClient>,
}

impl ImportService {
    /// Create a new import service.
    ///
    /// # Arguments
    ///
    /// * `db` - Database connection
    /// * `config` - Configuration for API clients
    #[must_use]
    pub fn new(db: Arc<SqliteLibrary>, config: &Config) -> Self {
        let mb_client = if config.musicbrainz.enabled {
            MusicBrainzClient::new(
                &config.musicbrainz.app_name,
                &config.musicbrainz.app_version,
                &config.musicbrainz.contact_email,
            )
            .ok()
        } else {
            None
        };

        let art_client = CoverArtClient::new(
            &config.musicbrainz.app_name,
            &config.musicbrainz.app_version,
        )
        .ok();

        Self {
            db,
            mb_client,
            art_client,
        }
    }

    /// Create a new import service with just a database (no external lookups).
    #[must_use]
    pub const fn new_basic(db: Arc<SqliteLibrary>) -> Self {
        Self {
            db,
            mb_client: None,
            art_client: None,
        }
    }

    /// Import music from a directory.
    ///
    /// # Arguments
    ///
    /// * `options` - Import configuration options
    /// * `progress_tx` - Optional channel for progress updates
    ///
    /// # Errors
    ///
    /// Returns an error if scanning fails.
    #[allow(clippy::too_many_lines)]
    pub async fn import(
        &self,
        options: &ImportOptions,
        progress_tx: Option<mpsc::Sender<ImportProgress>>,
    ) -> Result<ImportResult, crate::error::ApiError> {
        let mut result = ImportResult::default();

        // Step 1: Scan directory
        info!("Scanning directory: {}", options.source_path.display());
        if let Some(ref tx) = progress_tx {
            let _ = tx
                .send(ImportProgress::Scanning {
                    files_found: 0,
                    current_file: None,
                })
                .await;
        }

        let scan_options = ScanOptions {
            recursive: true,
            max_depth: options.max_depth,
            follow_symlinks: options.follow_symlinks,
            compute_hashes: options.compute_hashes,
        };

        let cancel = Arc::new(AtomicBool::new(false));

        let no_callback: Option<fn(&ScanProgress)> = None;
        let scan_result = scan_directory(
            &options.source_path,
            &scan_options,
            Some(&cancel),
            no_callback,
        )
        .map_err(|e| crate::error::ApiError::Internal(e.to_string()))?;

        result.tracks_found = scan_result.tracks.len();

        // Collect errors from scanning
        for (path, error) in &scan_result.errors {
            result.errors.push(format!("{}: {}", path.display(), error));
        }

        if scan_result.tracks.is_empty() {
            return Ok(result);
        }

        // Step 2: Optionally look up metadata from MusicBrainz
        let mut tracks = scan_result.tracks;

        if options.auto_tag
            && let Some(ref mb_client) = self.mb_client
        {
            tracks = self
                .lookup_metadata(
                    mb_client,
                    tracks,
                    options.min_match_score,
                    progress_tx.as_ref(),
                )
                .await;
        }

        // Step 3: Group tracks into albums and create album entries
        let album_map = if options.create_albums {
            let albums = Self::group_into_albums(&tracks);
            if let Some(ref tx) = progress_tx {
                let _ = tx
                    .send(ImportProgress::CreatingAlbums {
                        count: albums.len(),
                    })
                    .await;
            }
            self.create_album_entries(&albums, &mut result).await
        } else {
            HashMap::new()
        };

        // Step 4: Optionally fetch album art
        if options.fetch_album_art
            && let Some(ref art_client) = self.art_client
        {
            self.fetch_album_art(art_client, &album_map, progress_tx.as_ref())
                .await;
        }

        // Step 5: Optionally write tags back to files
        if options.write_tags {
            Self::write_tags_to_files(&tracks, &mut result);
        }

        // Step 6: Import tracks into database
        let total = tracks.len();
        for mut track in tracks {
            if let Some(ref tx) = progress_tx {
                let _ = tx
                    .send(ImportProgress::Importing {
                        imported: result.tracks_imported,
                        skipped: result.tracks_skipped,
                        failed: result.tracks_failed,
                        total,
                    })
                    .await;
            }

            // Link track to album if we created one
            if let Some(album_title) = track.album_title.as_ref() {
                let artist = track.album_artist.as_ref().unwrap_or(&track.artist);
                let key = format!("{}::{}", artist.to_lowercase(), album_title.to_lowercase());
                if let Some(album_id) = album_map.get(&key) {
                    track.album_id = Some(album_id.clone());
                }
            }

            match self.db.add_track(&track).await {
                Ok(_) => {
                    result.tracks_imported += 1;
                    debug!("Imported: {} - {}", track.artist, track.title);
                }
                Err(apollo_db::DbError::Sqlx(ref e))
                    if e.to_string().contains("UNIQUE constraint") =>
                {
                    result.tracks_skipped += 1;
                    debug!("Skipped (duplicate): {} - {}", track.artist, track.title);
                }
                Err(e) => {
                    result.tracks_failed += 1;
                    result.errors.push(format!(
                        "Failed to import {} - {}: {e}",
                        track.artist, track.title
                    ));
                    warn!("Failed to import: {} - {}: {e}", track.artist, track.title);
                }
            }
        }

        if let Some(ref tx) = progress_tx {
            let _ = tx.send(ImportProgress::Complete(result.clone())).await;
        }

        info!(
            "Import complete: {} imported, {} skipped, {} failed, {} albums created",
            result.tracks_imported,
            result.tracks_skipped,
            result.tracks_failed,
            result.albums_created
        );

        Ok(result)
    }

    /// Look up metadata from `MusicBrainz` for tracks.
    async fn lookup_metadata(
        &self,
        client: &MusicBrainzClient,
        mut tracks: Vec<Track>,
        min_score: u8,
        progress_tx: Option<&mpsc::Sender<ImportProgress>>,
    ) -> Vec<Track> {
        let total = tracks.len();

        for (i, track) in tracks.iter_mut().enumerate() {
            if let Some(tx) = progress_tx {
                let _ = tx
                    .send(ImportProgress::LookingUp {
                        track_index: i,
                        total,
                    })
                    .await;
            }

            // Skip if already has a MusicBrainz ID
            if track.musicbrainz_id.is_some() {
                continue;
            }

            // Try to find a match
            let album = track.album_title.as_deref();
            #[allow(clippy::cast_possible_truncation)]
            let duration_ms = track.duration.as_millis() as u64;

            match client
                .find_best_recording(
                    &track.title,
                    &track.artist,
                    album,
                    Some(duration_ms),
                    min_score,
                )
                .await
            {
                Ok(Some(recording)) => {
                    // Update track with MusicBrainz data
                    track.musicbrainz_id = Some(recording.id.clone());

                    // Update title/artist if we got a better match
                    let artist_name = recording.artist_name();
                    if !artist_name.is_empty() {
                        track.artist = artist_name;
                    }
                    track.title.clone_from(&recording.title);

                    // Set album info from first release if available
                    if let Some(release) = recording.releases.first()
                        && track.album_title.is_none()
                    {
                        track.album_title = Some(release.title.clone());
                    }

                    debug!(
                        "MusicBrainz match: {} - {} -> {}",
                        track.artist, track.title, recording.id
                    );
                }
                Ok(None) => {
                    debug!(
                        "No MusicBrainz match for: {} - {}",
                        track.artist, track.title
                    );
                }
                Err(e) => {
                    warn!(
                        "MusicBrainz lookup failed for {} - {}: {e}",
                        track.artist, track.title
                    );
                }
            }
        }

        tracks
    }

    /// Group tracks into albums based on album title and artist.
    fn group_into_albums(tracks: &[Track]) -> HashMap<String, Vec<&Track>> {
        let mut albums: HashMap<String, Vec<&Track>> = HashMap::new();

        for track in tracks {
            if let Some(album_title) = &track.album_title {
                let artist = track
                    .album_artist
                    .as_ref()
                    .unwrap_or(&track.artist)
                    .to_lowercase();
                let key = format!("{}::{}", artist, album_title.to_lowercase());
                albums.entry(key).or_default().push(track);
            }
        }

        albums
    }

    /// Create album entries in the database.
    async fn create_album_entries(
        &self,
        albums: &HashMap<String, Vec<&Track>>,
        result: &mut ImportResult,
    ) -> HashMap<String, AlbumId> {
        let mut album_map = HashMap::new();

        for (key, tracks) in albums {
            if tracks.is_empty() {
                continue;
            }

            // Use first track for album info
            let first_track = tracks[0];
            let album_title = first_track
                .album_title
                .as_ref()
                .expect("grouped by album title");
            let artist = first_track
                .album_artist
                .as_ref()
                .unwrap_or(&first_track.artist)
                .clone();

            // Check if album already exists (by title and artist)
            // For now, we just create a new one
            let mut album = Album::new(album_title.clone(), artist);
            album.track_count = u32::try_from(tracks.len()).unwrap_or(u32::MAX);

            // Set year from first track that has it
            for track in tracks {
                if let Some(year) = track.year {
                    album.year = Some(year);
                    break;
                }
            }

            match self.db.add_album(&album).await {
                Ok(_) => {
                    album_map.insert(key.clone(), album.id);
                    result.albums_created += 1;
                    debug!("Created album: {} - {}", album.artist, album.title);
                }
                Err(e) => {
                    warn!(
                        "Failed to create album {} - {}: {e}",
                        album.artist, album.title
                    );
                    result.errors.push(format!("Failed to create album: {e}"));
                }
            }
        }

        album_map
    }

    /// Fetch album art for albums with `MusicBrainz` IDs.
    async fn fetch_album_art(
        &self,
        client: &CoverArtClient,
        album_map: &HashMap<String, AlbumId>,
        progress_tx: Option<&mpsc::Sender<ImportProgress>>,
    ) {
        let total = album_map.len();

        for (index, album_id) in album_map.values().enumerate() {
            if let Some(tx) = progress_tx {
                let _ = tx
                    .send(ImportProgress::FetchingArt {
                        album_index: index,
                        total,
                    })
                    .await;
            }

            // Get album from database to check for MusicBrainz release ID
            if let Ok(Some(album)) = self.db.get_album(album_id).await
                && let Some(ref mbid) = album.musicbrainz_id
            {
                match client.get_front_cover(mbid, ImageSize::Large).await {
                    Ok(cover) => {
                        debug!(
                            "Found album art for {} - {}: {}",
                            album.artist, album.title, cover.url
                        );
                        // TODO: Save cover art to file or embed in tracks
                    }
                    Err(e) => {
                        debug!("No album art for {} - {}: {e}", album.artist, album.title);
                    }
                }
            }
        }
    }

    /// Write tags back to audio files.
    fn write_tags_to_files(tracks: &[Track], result: &mut ImportResult) {
        for track in tracks {
            if let Err(e) = write_metadata(&track.path, track) {
                warn!("Failed to write tags to {}: {e}", track.path.display());
                result.errors.push(format!(
                    "Failed to write tags to {}: {e}",
                    track.path.display()
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_options_default() {
        let options = ImportOptions::default();
        assert!(!options.auto_tag);
        assert!(!options.create_albums);
        assert!(!options.fetch_album_art);
        assert!(!options.write_tags);
        assert!(!options.compute_hashes);
    }

    #[test]
    fn test_import_result_default() {
        let result = ImportResult::default();
        assert_eq!(result.tracks_found, 0);
        assert_eq!(result.tracks_imported, 0);
        assert_eq!(result.albums_created, 0);
    }
}
