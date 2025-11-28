//! Directory scanning for audio files.

use crate::error::AudioError;
use crate::hash::compute_file_hash;
use crate::reader::read_metadata;
use apollo_core::Track;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, info, trace, warn};
use walkdir::WalkDir;

/// Supported audio file extensions.
const AUDIO_EXTENSIONS: &[&str] = &[
    "mp3", "flac", "ogg", "opus", "m4a", "aac", "wav", "aiff", "aif", "wv", "mpc",
];

/// Options for directory scanning.
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Whether to recurse into subdirectories.
    pub recursive: bool,
    /// Whether to compute file hashes.
    pub compute_hashes: bool,
    /// Whether to follow symbolic links.
    pub follow_symlinks: bool,
    /// Maximum depth to recurse (None for unlimited).
    pub max_depth: Option<usize>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            recursive: true,
            compute_hashes: true,
            follow_symlinks: false,
            max_depth: None,
        }
    }
}

/// Progress information during a scan.
#[derive(Debug, Clone)]
pub struct ScanProgress {
    /// Total files found so far.
    pub files_found: usize,
    /// Files successfully processed.
    pub files_processed: usize,
    /// Files that failed to process.
    pub files_failed: usize,
    /// Current file being processed.
    pub current_file: Option<PathBuf>,
}

impl ScanProgress {
    const fn new() -> Self {
        Self {
            files_found: 0,
            files_processed: 0,
            files_failed: 0,
            current_file: None,
        }
    }
}

/// Result of scanning a directory.
#[derive(Debug)]
pub struct ScanResult {
    /// Successfully read tracks.
    pub tracks: Vec<Track>,
    /// Files that failed to read (path, error message).
    pub errors: Vec<(PathBuf, String)>,
    /// Total files found.
    pub total_files: usize,
}

/// Scan a directory for audio files.
///
/// # Arguments
///
/// * `path` - The directory to scan
/// * `options` - Scanning options
/// * `cancel` - Optional cancellation flag (reference to allow sharing)
/// * `progress_callback` - Optional callback for progress updates
///
/// # Errors
///
/// Returns an error if the directory cannot be read.
pub fn scan_directory(
    path: &Path,
    options: &ScanOptions,
    cancel: Option<&Arc<AtomicBool>>,
    mut progress_callback: Option<impl FnMut(&ScanProgress)>,
) -> Result<ScanResult, AudioError> {
    info!("Scanning directory: {}", path.display());
    debug!("Scan options: {:?}", options);

    let mut tracks = Vec::new();
    let mut errors = Vec::new();
    let mut progress = ScanProgress::new();

    // Build the walker
    let mut walker = WalkDir::new(path).follow_links(options.follow_symlinks);

    if !options.recursive {
        walker = walker.max_depth(1);
    } else if let Some(depth) = options.max_depth {
        walker = walker.max_depth(depth);
    }

    // Collect audio files first
    let audio_files: Vec<PathBuf> = walker
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| is_audio_file(entry.path()))
        .map(|entry| entry.path().to_path_buf())
        .collect();

    progress.files_found = audio_files.len();
    info!("Found {} audio files", audio_files.len());

    // Process each file
    for file_path in audio_files {
        // Check for cancellation
        if let Some(cancel_flag) = cancel
            && cancel_flag.load(Ordering::Relaxed)
        {
            info!("Scan cancelled");
            return Err(AudioError::ScanCancelled);
        }

        progress.current_file = Some(file_path.clone());

        // Report progress
        if let Some(ref mut callback) = progress_callback {
            callback(&progress);
        }

        trace!("Processing: {}", file_path.display());

        match read_metadata(&file_path) {
            Ok(mut track) => {
                // Compute hash if requested
                if options.compute_hashes {
                    match compute_file_hash(&file_path) {
                        Ok(hash) => track.file_hash = hash,
                        Err(e) => {
                            warn!("Failed to compute hash for {}: {}", file_path.display(), e);
                        }
                    }
                }
                tracks.push(track);
                progress.files_processed += 1;
            }
            Err(e) => {
                warn!("Failed to read {}: {}", file_path.display(), e);
                errors.push((file_path, e.to_string()));
                progress.files_failed += 1;
            }
        }
    }

    info!(
        "Scan complete: {} processed, {} failed",
        progress.files_processed, progress.files_failed
    );

    Ok(ScanResult {
        tracks,
        errors,
        total_files: progress.files_found,
    })
}

/// Check if a file is an audio file based on its extension.
fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| AUDIO_EXTENSIONS.contains(&ext.to_lowercase().as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_audio_file() {
        assert!(is_audio_file(Path::new("song.mp3")));
        assert!(is_audio_file(Path::new("song.MP3")));
        assert!(is_audio_file(Path::new("song.flac")));
        assert!(is_audio_file(Path::new("song.ogg")));
        assert!(is_audio_file(Path::new("/path/to/song.m4a")));
        assert!(!is_audio_file(Path::new("document.pdf")));
        assert!(!is_audio_file(Path::new("image.jpg")));
        assert!(!is_audio_file(Path::new("noextension")));
    }

    #[test]
    fn test_scan_options_default() {
        let options = ScanOptions::default();
        assert!(options.recursive);
        assert!(options.compute_hashes);
        assert!(!options.follow_symlinks);
        assert!(options.max_depth.is_none());
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let options = ScanOptions::default();

        let result = scan_directory(temp_dir.path(), &options, None, None::<fn(&ScanProgress)>);

        let result = result.unwrap();
        assert!(result.tracks.is_empty());
        assert!(result.errors.is_empty());
        assert_eq!(result.total_files, 0);
    }
}
