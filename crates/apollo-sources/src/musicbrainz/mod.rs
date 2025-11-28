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

mod client;
mod types;

pub use client::MusicBrainzClient;
pub use types::{
    Artist, ArtistCredit, Medium, Recording, RecordingSearchResponse, Release, ReleaseGroup,
    ReleaseSearchResponse, Track,
};
