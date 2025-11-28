//! Error types for audio file operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during audio file operations.
#[derive(Debug, Error)]
pub enum AudioError {
    /// Failed to read the audio file.
    #[error("failed to read audio file '{path}': {source}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: lofty::error::LoftyError,
    },

    /// Failed to write to the audio file.
    #[error("failed to write audio file '{path}': {source}")]
    WriteError {
        path: PathBuf,
        #[source]
        source: lofty::error::LoftyError,
    },

    /// The file format is not supported.
    #[error("unsupported audio format for file '{0}'")]
    UnsupportedFormat(PathBuf),

    /// File not found.
    #[error("audio file not found: '{0}'")]
    FileNotFound(PathBuf),

    /// IO error during file operations.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// No audio tags found in the file.
    #[error("no tags found in audio file '{0}'")]
    NoTags(PathBuf),

    /// Directory scan was cancelled.
    #[error("directory scan cancelled")]
    ScanCancelled,
}

impl AudioError {
    /// Create a read error with context.
    pub fn read(path: impl Into<PathBuf>, source: lofty::error::LoftyError) -> Self {
        Self::ReadError {
            path: path.into(),
            source,
        }
    }

    /// Create a write error with context.
    pub fn write(path: impl Into<PathBuf>, source: lofty::error::LoftyError) -> Self {
        Self::WriteError {
            path: path.into(),
            source,
        }
    }
}
