//! # Apollo Core
//!
//! Core types and business logic for the Apollo music library manager.
//!
//! This crate contains no I/O operations and is designed to be purely functional
//! where possible.

pub mod metadata;
pub mod library;
pub mod query;
pub mod error;

pub use error::Error;
pub use metadata::{Track, Album, Artist, AudioFormat};
