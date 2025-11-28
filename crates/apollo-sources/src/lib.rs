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

mod error;
pub mod musicbrainz;

pub use error::{SourceError, SourceResult};
