//! # Apollo Audio
//!
//! Audio file reading, writing, and metadata extraction.
//!
//! This crate provides functionality to:
//! - Read metadata tags from audio files (MP3, FLAC, OGG, etc.)
//! - Write metadata tags back to audio files
//! - Scan directories for audio files
//! - Compute file hashes for deduplication
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

mod error;
mod hash;
mod reader;
mod scanner;
mod writer;

pub use error::AudioError;
pub use hash::compute_file_hash;
pub use reader::{AudioProperties, read_metadata};
pub use scanner::{ScanOptions, ScanProgress, scan_directory};
pub use writer::write_metadata;
