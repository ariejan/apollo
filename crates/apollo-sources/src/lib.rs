//! # Apollo Sources
//!
//! Integration with external metadata sources like
//! [MusicBrainz](https://musicbrainz.org/), [AcoustID](https://acoustid.org/),
//! and [Discogs](https://discogs.com/).
//!
//! This crate provides clients for fetching music metadata from online databases.
//! All clients implement rate limiting to comply with API requirements.
//!
//! # Supported Sources
//!
//! - [MusicBrainz](https://musicbrainz.org/): Community-maintained open music encyclopedia
//! - [AcoustID](https://acoustid.org/): Audio fingerprint identification service
//! - [Discogs](https://discogs.com/): Comprehensive music release database
//! - [Cover Art Archive](https://coverartarchive.org/): Album cover art from [MusicBrainz](https://musicbrainz.org/)
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
//! # Fingerprint Lookup Example
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
//!     println!("Found: {} (score: {:.2})", result.id, result.score);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Discogs Example
//!
//! ```no_run
//! use apollo_sources::discogs::DiscogsClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = DiscogsClient::new("MyApp", "1.0", "your-token")?;
//!
//! // Search for releases
//! let results = client.search_releases("Abbey Road", Some("Beatles"), 5).await?;
//! for result in &results {
//!     println!("{} ({})", result.title, result.year.as_deref().unwrap_or_default());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Cover Art Example
//!
//! ```no_run
//! use apollo_sources::coverart::{CoverArtClient, ImageSize};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CoverArtClient::new("MyApp", "1.0")?;
//!
//! // Get cover art for a MusicBrainz release
//! let images = client.get_release_art("76df3287-6cda-33eb-8e9a-044b5e15ffdd").await?;
//! println!("Found {} images", images.len());
//!
//! // Get just the front cover
//! let front = client.get_front_cover("76df3287-6cda-33eb-8e9a-044b5e15ffdd", ImageSize::Large).await?;
//! println!("Front cover: {}", front.url);
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

pub mod acoustid;
pub mod cache;
pub mod coverart;
pub mod discogs;
mod error;
pub mod musicbrainz;

pub use cache::{CacheConfig, ResponseCache};
pub use error::{SourceError, SourceResult};
