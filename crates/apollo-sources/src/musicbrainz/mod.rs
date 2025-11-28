//! [MusicBrainz](https://musicbrainz.org/) API integration.
//!
//! This module provides a client for the API,
//! which is a community-maintained open music encyclopedia.
//!
//! # Rate Limiting
//!
//! The API requires that applications make no more than one request per second.
//! The client automatically enforces this limit.
//!
//! # Caching
//!
//! Use [`CachedMusicBrainzClient`] for automatic response caching, which reduces
//! API calls and improves performance. The cache can optionally be persisted to disk.
//!
//! # Example
//!
//! ```no_run
//! use apollo_sources::musicbrainz::MusicBrainzClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = MusicBrainzClient::new("MyApp", "1.0", "contact@example.com")?;
//!
//! // Search for recordings
//! let recordings = client.search_recordings("Yesterday", Some("Beatles"), 5).await?;
//! for recording in recordings {
//!     println!("{} - {}", recording.artist_name(), recording.title);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Cached Example
//!
//! ```no_run
//! use apollo_sources::musicbrainz::CachedMusicBrainzClient;
//! use apollo_sources::cache::CacheConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CachedMusicBrainzClient::with_defaults("MyApp", "1.0", "contact@example.com")?;
//!
//! // First call hits the API
//! let recordings = client.search_recordings("Yesterday", Some("Beatles"), 5).await?;
//!
//! // Second call uses cache (no API request)
//! let recordings = client.search_recordings("Yesterday", Some("Beatles"), 5).await?;
//! # Ok(())
//! # }
//! ```

mod cached;
mod client;
mod types;

pub use cached::{CacheStats, CachedMusicBrainzClient};
pub use client::MusicBrainzClient;
pub use types::{
    Artist, ArtistCredit, Medium, Recording, RecordingSearchResponse, Release, ReleaseGroup,
    ReleaseSearchResponse, Track,
};
