//! [AcoustID](https://acoustid.org/) API integration.
//!
//! This module provides a client for the [AcoustID](https://acoustid.org/) service,
//! which identifies music tracks based on their audio fingerprints.
//!
//! # Example
//!
//! ```no_run
//! use apollo_sources::acoustid::AcoustIdClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = AcoustIdClient::new("your-api-key")?;
//!
//! // Look up a track by its fingerprint
//! let results = client.lookup("fingerprint-string", 180).await?;
//! for result in results {
//!     println!("Found: {} (score: {})", result.id, result.score);
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod types;

pub use client::AcoustIdClient;
pub use types::{AcoustIdResult, Recording as AcoustIdRecording, ReleaseGroup};
