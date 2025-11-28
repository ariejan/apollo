//! [MusicBrainz](https://musicbrainz.org/) API response types.

use serde::{Deserialize, Serialize};
use std::fmt::Write;

/// A recording from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    /// The MBID of the recording.
    pub id: String,
    /// The title of the recording.
    pub title: String,
    /// The length of the recording in milliseconds.
    #[serde(default)]
    pub length: Option<u64>,
    /// Disambiguation comment.
    #[serde(default)]
    pub disambiguation: Option<String>,
    /// Artist credits for this recording.
    #[serde(default, rename = "artist-credit")]
    pub artist_credit: Vec<ArtistCredit>,
    /// Releases containing this recording.
    #[serde(default)]
    pub releases: Vec<Release>,
    /// ISRCs associated with this recording.
    #[serde(default)]
    pub isrcs: Vec<String>,
    /// Score from search results (0-100).
    #[serde(default)]
    pub score: Option<u8>,
}

impl Recording {
    /// Get the formatted artist name.
    #[must_use]
    pub fn artist_name(&self) -> String {
        self.artist_credit
            .iter()
            .fold(String::new(), |mut acc, ac| {
                let name = ac.name.as_deref().unwrap_or(&ac.artist.name);
                let join = ac.joinphrase.as_deref().unwrap_or("");
                let _ = write!(acc, "{name}{join}");
                acc
            })
    }
}

/// A release (album/single/EP) from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// The MBID of the release.
    pub id: String,
    /// The title of the release.
    pub title: String,
    /// The release status (official, bootleg, etc.).
    #[serde(default)]
    pub status: Option<String>,
    /// The release date (YYYY, YYYY-MM, or YYYY-MM-DD).
    #[serde(default)]
    pub date: Option<String>,
    /// Country code where this release was made.
    #[serde(default)]
    pub country: Option<String>,
    /// Disambiguation comment.
    #[serde(default)]
    pub disambiguation: Option<String>,
    /// Track count for this release.
    #[serde(default, rename = "track-count")]
    pub track_count: Option<u32>,
    /// Artist credits for this release.
    #[serde(default, rename = "artist-credit")]
    pub artist_credit: Vec<ArtistCredit>,
    /// The release group this belongs to.
    #[serde(default, rename = "release-group")]
    pub release_group: Option<ReleaseGroup>,
    /// Media (discs/sides) on this release.
    #[serde(default)]
    pub media: Vec<Medium>,
    /// Score from search results (0-100).
    #[serde(default)]
    pub score: Option<u8>,
}

impl Release {
    /// Get the formatted artist name.
    #[must_use]
    pub fn artist_name(&self) -> String {
        self.artist_credit
            .iter()
            .fold(String::new(), |mut acc, ac| {
                let name = ac.name.as_deref().unwrap_or(&ac.artist.name);
                let join = ac.joinphrase.as_deref().unwrap_or("");
                let _ = write!(acc, "{name}{join}");
                acc
            })
    }

    /// Get the year from the release date.
    #[must_use]
    pub fn year(&self) -> Option<i32> {
        self.date
            .as_ref()
            .and_then(|d| d.split('-').next())
            .and_then(|y| y.parse().ok())
    }
}

/// A release group (album, EP, single, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseGroup {
    /// The MBID of the release group.
    pub id: String,
    /// The title of the release group.
    #[serde(default)]
    pub title: Option<String>,
    /// Primary type (Album, Single, EP, etc.).
    #[serde(default, rename = "primary-type")]
    pub primary_type: Option<String>,
    /// Secondary types (Compilation, Live, etc.).
    #[serde(default, rename = "secondary-types")]
    pub secondary_types: Vec<String>,
}

/// A medium (disc/side) on a release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Medium {
    /// Position of this medium in the release.
    #[serde(default)]
    pub position: Option<u32>,
    /// Format (CD, Vinyl, Digital Media, etc.).
    #[serde(default)]
    pub format: Option<String>,
    /// Tracks on this medium.
    #[serde(default)]
    pub tracks: Vec<Track>,
    /// Total track count.
    #[serde(default, rename = "track-count")]
    pub track_count: Option<u32>,
}

/// A track on a medium.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// The MBID of this track.
    pub id: String,
    /// Position on the medium.
    #[serde(default)]
    pub position: Option<u32>,
    /// Track number as displayed.
    #[serde(default)]
    pub number: Option<String>,
    /// Title of the track (may differ from recording title).
    #[serde(default)]
    pub title: Option<String>,
    /// Length in milliseconds.
    #[serde(default)]
    pub length: Option<u64>,
    /// The recording this track represents.
    #[serde(default)]
    pub recording: Option<Recording>,
}

/// Artist credit entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtistCredit {
    /// The artist.
    pub artist: Artist,
    /// Credit name (if different from artist name).
    #[serde(default)]
    pub name: Option<String>,
    /// Join phrase to next artist (e.g., " & ", " feat. ").
    #[serde(default)]
    pub joinphrase: Option<String>,
}

/// An artist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    /// The MBID of the artist.
    pub id: String,
    /// The name of the artist.
    pub name: String,
    /// Sort name (e.g., "Beatles, The").
    #[serde(default, rename = "sort-name")]
    pub sort_name: Option<String>,
    /// Type (Person, Group, etc.).
    #[serde(default, rename = "type")]
    pub artist_type: Option<String>,
    /// Disambiguation comment.
    #[serde(default)]
    pub disambiguation: Option<String>,
}

/// Search response for recordings.
#[derive(Debug, Deserialize)]
pub struct RecordingSearchResponse {
    /// The recordings found.
    pub recordings: Vec<Recording>,
    /// Total count of results.
    #[serde(default)]
    pub count: u32,
    /// Offset in results.
    #[serde(default)]
    pub offset: u32,
}

/// Search response for releases.
#[derive(Debug, Deserialize)]
pub struct ReleaseSearchResponse {
    /// The releases found.
    pub releases: Vec<Release>,
    /// Total count of results.
    #[serde(default)]
    pub count: u32,
    /// Offset in results.
    #[serde(default)]
    pub offset: u32,
}
