//! [AcoustID](https://acoustid.org/) API response types.

use serde::{Deserialize, Serialize};

/// Response from the [AcoustID](https://acoustid.org/) lookup API.
#[derive(Debug, Clone, Deserialize)]
pub struct LookupResponse {
    /// Status of the request ("ok" or "error").
    pub status: String,
    /// Results if successful.
    #[serde(default)]
    pub results: Vec<AcoustIdResult>,
    /// Error details if status is "error".
    #[serde(default)]
    pub error: Option<ApiError>,
}

/// Error from the API.
#[derive(Debug, Clone, Deserialize)]
pub struct ApiError {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub message: String,
}

/// A single result from [AcoustID](https://acoustid.org/) lookup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcoustIdResult {
    /// The [AcoustID](https://acoustid.org/) identifier.
    pub id: String,
    /// Match score (0.0 to 1.0).
    pub score: f64,
    /// Recordings associated with this fingerprint.
    #[serde(default)]
    pub recordings: Vec<Recording>,
}

/// A recording from [AcoustID](https://acoustid.org/) (linked to [MusicBrainz](https://musicbrainz.org/)).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    /// The [MusicBrainz](https://musicbrainz.org/) recording ID.
    pub id: String,
    /// Title of the recording.
    #[serde(default)]
    pub title: Option<String>,
    /// Duration in milliseconds.
    #[serde(default)]
    pub duration: Option<u64>,
    /// Artists on this recording.
    #[serde(default)]
    pub artists: Vec<Artist>,
    /// Release groups containing this recording.
    #[serde(default)]
    pub releasegroups: Vec<ReleaseGroup>,
}

impl Recording {
    /// Get the formatted artist name.
    #[must_use]
    pub fn artist_name(&self) -> String {
        self.artists
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// An artist from [AcoustID](https://acoustid.org/).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    /// The [MusicBrainz](https://musicbrainz.org/) artist ID.
    pub id: String,
    /// The artist name.
    pub name: String,
}

/// A release group (album) from [AcoustID](https://acoustid.org/).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseGroup {
    /// The [MusicBrainz](https://musicbrainz.org/) release group ID.
    pub id: String,
    /// The release group title.
    #[serde(default)]
    pub title: Option<String>,
    /// The type (Album, Single, EP, etc.).
    #[serde(default, rename = "type")]
    pub release_type: Option<String>,
    /// Artists on this release group.
    #[serde(default)]
    pub artists: Vec<Artist>,
}
