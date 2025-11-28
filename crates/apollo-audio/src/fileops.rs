//! File operations for organizing music files.
//!
//! This module provides functions to move and copy audio files using path templates.

use std::fs;
use std::path::{Path, PathBuf};

use apollo_core::metadata::Track;
use apollo_core::template::{PathTemplate, TemplateContext};

use crate::AudioError;

/// Result of a file organization operation.
#[derive(Debug, Clone)]
pub struct OrganizeResult {
    /// The original file path.
    pub source: PathBuf,
    /// The destination file path.
    pub destination: PathBuf,
    /// Whether the file was moved (true) or copied (false).
    pub moved: bool,
}

/// Options for organizing files.
#[derive(Debug, Clone)]
pub struct OrganizeOptions {
    /// Move files instead of copying.
    pub move_files: bool,
    /// Overwrite existing files.
    pub overwrite: bool,
    /// Create parent directories as needed.
    pub create_dirs: bool,
}

impl Default for OrganizeOptions {
    fn default() -> Self {
        Self {
            move_files: false,
            overwrite: false,
            create_dirs: true,
        }
    }
}

/// Organize a file by moving or copying it to a path determined by the template.
///
/// # Arguments
///
/// * `source` - Path to the source audio file
/// * `base_dir` - Base directory for the destination
/// * `template` - Path template for generating the destination path
/// * `track` - Track metadata for template variables
/// * `options` - Organization options
///
/// # Errors
///
/// Returns an error if:
/// - The source file doesn't exist
/// - The template rendering fails
/// - The file operation fails
/// - A destination file exists and overwrite is false
pub fn organize_file(
    source: &Path,
    base_dir: &Path,
    template: &PathTemplate,
    track: &Track,
    options: &OrganizeOptions,
) -> Result<OrganizeResult, AudioError> {
    // Check source exists
    if !source.exists() {
        return Err(AudioError::FileNotFound(source.to_path_buf()));
    }

    // Build template context from track
    let ctx = TemplateContext::from_track(track);

    // Render destination path
    let relative_path = template
        .render_with_extension(&ctx)
        .map_err(|e| AudioError::Io(std::io::Error::other(e.to_string())))?;

    let destination = base_dir.join(&relative_path);

    // Check if destination already exists
    if destination.exists() && !options.overwrite {
        return Err(AudioError::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("Destination file already exists: {}", destination.display()),
        )));
    }

    // Create parent directories if needed
    if options.create_dirs
        && let Some(parent) = destination.parent()
    {
        fs::create_dir_all(parent)?;
    }

    // Perform the file operation
    if options.move_files {
        // Try rename first (fast for same filesystem)
        if fs::rename(source, &destination).is_err() {
            // Fall back to copy + delete
            fs::copy(source, &destination)?;
            fs::remove_file(source)?;
        }
    } else {
        fs::copy(source, &destination)?;
    }

    Ok(OrganizeResult {
        source: source.to_path_buf(),
        destination,
        moved: options.move_files,
    })
}

/// Compute the destination path for a file without actually moving/copying it.
///
/// This is useful for previewing what will happen during organization.
///
/// # Errors
///
/// Returns an error if template rendering fails.
pub fn preview_destination(
    base_dir: &Path,
    template: &PathTemplate,
    track: &Track,
) -> Result<PathBuf, AudioError> {
    let ctx = TemplateContext::from_track(track);

    let relative_path = template
        .render_with_extension(&ctx)
        .map_err(|e| AudioError::Io(std::io::Error::other(e.to_string())))?;

    Ok(base_dir.join(&relative_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    fn create_test_track(path: PathBuf) -> Track {
        let mut track = Track::new(
            path,
            "Bohemian Rhapsody".to_string(),
            "Queen".to_string(),
            Duration::from_secs(354),
        );
        track.album_title = Some("A Night at the Opera".to_string());
        track.album_artist = Some("Queen".to_string());
        track.track_number = Some(11);
        track.disc_number = Some(1);
        track.year = Some(1975);
        track.genres = vec!["Rock".to_string()];
        track
    }

    #[test]
    fn test_preview_destination() {
        let template = PathTemplate::parse("$artist/$album/$track - $title").unwrap();
        let track = create_test_track(PathBuf::from("/music/test.mp3"));
        let base_dir = PathBuf::from("/library");

        let dest = preview_destination(&base_dir, &template, &track).unwrap();

        assert_eq!(
            dest,
            PathBuf::from("/library/Queen/A Night at the Opera/11 - Bohemian Rhapsody.mp3")
        );
    }

    #[test]
    fn test_organize_file_copy() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");
        fs::create_dir_all(&source_dir).unwrap();

        // Create a dummy source file
        let source_file = source_dir.join("test.mp3");
        fs::write(&source_file, b"fake mp3 data").unwrap();

        let template = PathTemplate::parse("$artist/$album/$track - $title").unwrap();
        let track = create_test_track(source_file.clone());

        let options = OrganizeOptions {
            move_files: false,
            overwrite: false,
            create_dirs: true,
        };

        let result = organize_file(&source_file, &dest_dir, &template, &track, &options).unwrap();

        // Source should still exist (copy, not move)
        assert!(source_file.exists());
        // Destination should exist
        assert!(result.destination.exists());
        assert!(!result.moved);
    }

    #[test]
    fn test_organize_file_move() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");
        fs::create_dir_all(&source_dir).unwrap();

        // Create a dummy source file
        let source_file = source_dir.join("test.mp3");
        fs::write(&source_file, b"fake mp3 data").unwrap();

        let template = PathTemplate::parse("$artist/$title").unwrap();
        let track = create_test_track(source_file.clone());

        let options = OrganizeOptions {
            move_files: true,
            overwrite: false,
            create_dirs: true,
        };

        let result = organize_file(&source_file, &dest_dir, &template, &track, &options).unwrap();

        // Source should NOT exist (moved)
        assert!(!source_file.exists());
        // Destination should exist
        assert!(result.destination.exists());
        assert!(result.moved);
    }

    #[test]
    fn test_organize_file_no_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&dest_dir.join("Queen")).unwrap();

        // Create source and existing destination
        let source_file = source_dir.join("test.mp3");
        fs::write(&source_file, b"source data").unwrap();
        fs::write(dest_dir.join("Queen/Bohemian Rhapsody.mp3"), b"existing").unwrap();

        let template = PathTemplate::parse("$artist/$title").unwrap();
        let track = create_test_track(source_file.clone());

        let options = OrganizeOptions {
            move_files: false,
            overwrite: false,
            create_dirs: true,
        };

        let result = organize_file(&source_file, &dest_dir, &template, &track, &options);
        assert!(result.is_err());
    }

    #[test]
    fn test_organize_file_with_overwrite() {
        let temp_dir = TempDir::new().unwrap();
        let source_dir = temp_dir.path().join("source");
        let dest_dir = temp_dir.path().join("dest");
        fs::create_dir_all(&source_dir).unwrap();
        fs::create_dir_all(&dest_dir.join("Queen")).unwrap();

        // Create source and existing destination
        let source_file = source_dir.join("test.mp3");
        fs::write(&source_file, b"source data").unwrap();
        let existing = dest_dir.join("Queen/Bohemian Rhapsody.mp3");
        fs::write(&existing, b"existing").unwrap();

        let template = PathTemplate::parse("$artist/$title").unwrap();
        let track = create_test_track(source_file.clone());

        let options = OrganizeOptions {
            move_files: false,
            overwrite: true,
            create_dirs: true,
        };

        let result = organize_file(&source_file, &dest_dir, &template, &track, &options).unwrap();
        assert!(result.destination.exists());

        // Content should be from source
        let content = fs::read(&result.destination).unwrap();
        assert_eq!(content, b"source data");
    }
}
