//! # Apollo Sources
//!
//! Integration with external metadata sources like
//! [MusicBrainz](https://musicbrainz.org/) and [Discogs](https://discogs.com/).
//!
//! This crate provides clients for fetching music metadata from online databases.
//! All clients implement rate limiting to comply with API requirements.
//!
//! # Supported Sources
//!
//! - [MusicBrainz](https://musicbrainz.org/): Community-maintained open music encyclopedia
//! - Discogs: (planned)
//!
//! # Caching
//!
//! All clients support response caching to reduce API calls and improve performance.
//! Use [`CachedMusicBrainzClient`](musicbrainz::CachedMusicBrainzClient) for cached access.
//!
//! # Example
//!
//! ```no_run
//! use apollo_sources::musicbrainz::MusicBrainzClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = MusicBrainzClient::new("MyApp", "1.0", "contact@example.com")?;
//!
//! // Search for a recording
//! let recordings = client.search_recordings("Yesterday", Some("Beatles"), 5).await?;
//! println!("Found {} recordings", recordings.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Cached Example
//!
//! ```no_run
//! use apollo_sources::musicbrainz::CachedMusicBrainzClient;
//! use apollo_sources::cache::CacheConfig;
//! use std::time::Duration;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = CacheConfig::new()
//!     .with_ttl(Duration::from_secs(3600))  // 1 hour cache
//!     .with_persist_path("cache/musicbrainz.json");
//!
//! let client = CachedMusicBrainzClient::new("MyApp", "1.0", "contact@example.com", config)?;
//!
//! // Load any existing cache from disk
//! client.load_cache().await.ok();
//!
//! // This will be cached
//! let recordings = client.search_recordings("Yesterday", Some("Beatles"), 5).await?;
//!
//! // Second call uses cache (no API request)
//! let recordings = client.search_recordings("Yesterday", Some("Beatles"), 5).await?;
//!
//! // Save cache to disk for next session
//! client.save_cache().await?;
//! # Ok(())
//! # }
//! ```

pub mod cache;
mod error;
pub mod musicbrainz;

pub use cache::{CacheConfig, ResponseCache};
pub use error::{SourceError, SourceResult};
