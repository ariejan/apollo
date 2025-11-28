//! # Apollo Core
//!
//! Core types and business logic for the Apollo music library manager.
//!
//! This crate contains no I/O operations and is designed to be purely functional
//! where possible.

pub mod config;
pub mod error;
pub mod library;
pub mod metadata;
pub mod playlist;
pub mod query;
pub mod template;

pub use config::Config;
pub use error::Error;
pub use metadata::{Album, AlbumId, Artist, AudioFormat, Track, TrackId};
pub use playlist::{Playlist, PlaylistId, PlaylistKind, PlaylistLimit, PlaylistSort};
pub use template::{PathTemplate, TemplateContext};
