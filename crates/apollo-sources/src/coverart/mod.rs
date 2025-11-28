//! Cover art fetching from multiple sources.
//!
//! This module provides functionality to fetch album cover art from:
//! - [Cover Art Archive](https://coverartarchive.org/) (linked to [MusicBrainz](https://musicbrainz.org/))
//! - [Discogs](https://discogs.com/) (via search result URLs)
//!
//! # Cover Art Archive Example
//!
//! ```no_run
//! use apollo_sources::coverart::{CoverArtClient, ImageSize};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CoverArtClient::new("MyApp", "1.0")?;
//!
//! // Get all cover art for a release
//! let images = client.get_release_art("76df3287-6cda-33eb-8e9a-044b5e15ffdd").await?;
//! for img in &images {
//!     println!("{:?}: {}", img.cover_type, img.url);
//! }
//!
//! // Get just the front cover
//! if let Ok(front) = client.get_front_cover("76df3287-6cda-33eb-8e9a-044b5e15ffdd", ImageSize::Large).await {
//!     println!("Front: {}", front.url);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Direct URLs
//!
//! ```no_run
//! use apollo_sources::coverart::{CoverArtClient, ImageSize};
//!
//! // Get URLs without making API calls
//! let front_url = CoverArtClient::front_cover_url("76df3287-6cda-33eb-8e9a-044b5e15ffdd", ImageSize::Large);
//! println!("Front cover URL: {front_url}");
//! ```
//!
//! # Downloading Images
//!
//! ```no_run
//! use apollo_sources::coverart::{CoverArtClient, ImageSize};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = CoverArtClient::new("MyApp", "1.0")?;
//!
//! // Download to bytes
//! let url = CoverArtClient::front_cover_url("76df3287-6cda-33eb-8e9a-044b5e15ffdd", ImageSize::Large);
//! let bytes = client.download_image(&url).await?;
//! println!("Downloaded {} bytes", bytes.len());
//!
//! // Download to file
//! client.download_image_to_file(&url, "cover.jpg").await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Discogs Integration
//!
//! ```no_run
//! use apollo_sources::coverart::CoverArtClient;
//! use apollo_sources::discogs::DiscogsClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let discogs = DiscogsClient::new("MyApp", "1.0", "token")?;
//! let results = discogs.search_releases("Abbey Road", Some("Beatles"), 1).await?;
//!
//! if let Some(result) = results.first() {
//!     // Extract cover images from Discogs result
//!     let images = CoverArtClient::from_discogs_urls(
//!         result.thumb.as_deref(),
//!         result.cover_image.as_deref()
//!     );
//!     for img in &images {
//!         println!("Discogs cover: {}", img.url);
//!     }
//! }
//! # Ok(())
//! # }
//! ```

mod client;
mod types;

pub use client::CoverArtClient;
pub use types::{
    CoverArtArchiveImage, CoverArtArchiveResponse, CoverImage, CoverType, ImageSize, Thumbnails,
};
