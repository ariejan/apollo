//! [Discogs](https://discogs.com/) API integration.
//!
//! This module provides a client for the [Discogs](https://discogs.com/) API,
//! which is a comprehensive database of music releases, artists, and labels.
//!
//! # Authentication
//!
//! The Discogs API requires authentication via a personal access token.
//! You can create one at <https://www.discogs.com/settings/developers>.
//!
//! # Rate Limiting
//!
//! The API allows 60 requests per minute for authenticated users.
//! The client automatically enforces rate limiting to stay within these limits.
//!
//! # Example
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
//!
//! // Get full release details
//! if let Some(first) = results.first() {
//!     let release = client.get_release(first.id).await?;
//!     println!("Tracks: {}", release.tracklist.len());
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Barcode Search
//!
//! ```no_run
//! use apollo_sources::discogs::DiscogsClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = DiscogsClient::new("MyApp", "1.0", "your-token")?;
//!
//! // Search by barcode (useful for CD/vinyl identification)
//! let results = client.search_by_barcode("0602537729067").await?;
//! for result in results {
//!     println!("{}", result.title);
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod types;

pub use client::DiscogsClient;
pub use types::{
    Artist, Community, Format, Label, Master, Pagination, Rating, Release, SearchResponse,
    SearchResult, Track,
};
