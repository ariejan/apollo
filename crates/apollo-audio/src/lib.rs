//! # Apollo Audio
//!
//! Audio file reading, writing, and metadata extraction.
//!
//! This crate provides functionality to:
//! - Read metadata tags from audio files (MP3, FLAC, OGG, etc.)
//! - Write metadata tags back to audio files
//! - Scan directories for audio files
//! - Compute file hashes for deduplication
//! - Generate audio fingerprints for music identification
//!
//! # Examples
//!
//! ```no_run
//! use apollo_audio::{read_metadata, AudioError};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), AudioError> {
//! let track = read_metadata(Path::new("song.mp3"))?;
//! println!("Title: {}", track.title);
//! println!("Artist: {}", track.artist);
//! # Ok(())
//! # }
//! ```
//!
//! # Fingerprinting
//!
//! ```no_run
//! use apollo_audio::{generate_fingerprint, AudioError};
//! use std::path::Path;
//!
//! # fn main() -> Result<(), AudioError> {
//! let result = generate_fingerprint(Path::new("song.mp3"))?;
//! println!("Fingerprint: {}", result.fingerprint);
//! println!("Duration: {}s", result.duration);
//! # Ok(())
//! # }
//! ```

mod error;
mod fingerprint;
mod hash;
mod reader;
mod scanner;
mod writer;

pub use error::AudioError;
pub use fingerprint::{FingerprintResult, generate_fingerprint};
pub use hash::compute_file_hash;
pub use reader::{AudioProperties, read_metadata};
pub use scanner::{ScanOptions, ScanProgress, scan_directory};
pub use writer::write_metadata;
