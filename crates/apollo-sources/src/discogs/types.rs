//! [Discogs](https://discogs.com/) API response types.

use serde::{Deserialize, Serialize};
use std::fmt::Write;

/// A release from the Discogs API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// The Discogs release ID.
    pub id: u64,
    /// The title of the release.
    pub title: String,
    /// The artists on this release.
    #[serde(default)]
    pub artists: Vec<Artist>,
    /// Year of release.
    #[serde(default)]
    pub year: Option<i32>,
    /// Genres associated with this release.
    #[serde(default)]
    pub genres: Vec<String>,
    /// Styles (subgenres) associated with this release.
    #[serde(default)]
    pub styles: Vec<String>,
    /// The tracklist.
    #[serde(default)]
    pub tracklist: Vec<Track>,
    /// Record labels.
    #[serde(default)]
    pub labels: Vec<Label>,
    /// Formats (CD, Vinyl, etc.).
    #[serde(default)]
    pub formats: Vec<Format>,
    /// Country of release.
    #[serde(default)]
    pub country: Option<String>,
    /// Release date.
    #[serde(default)]
    pub released: Option<String>,
    /// Discogs resource URL.
    #[serde(default)]
    pub resource_url: Option<String>,
    /// URI for this release on the website.
    #[serde(default)]
    pub uri: Option<String>,
    /// Master release ID if this is linked to a master.
    #[serde(default)]
    pub master_id: Option<u64>,
    /// Main release flag (for master releases).
    #[serde(default)]
    pub main_release: Option<u64>,
    /// Number of versions if this is a master release.
    #[serde(default)]
    pub versions_count: Option<u32>,
    /// Notes/description.
    #[serde(default)]
    pub notes: Option<String>,
    /// Community rating data.
    #[serde(default)]
    pub community: Option<Community>,
}

impl Release {
    /// Get the formatted artist name.
    #[must_use]
    pub fn artist_name(&self) -> String {
        let mut result = String::new();
        for (i, artist) in self.artists.iter().enumerate() {
            let _ = write!(result, "{}", artist.name);
            // Add join phrase if present, or default separator if not last artist
            if i < self.artists.len() - 1 {
                let join = artist.join.as_deref().unwrap_or(", ");
                let _ = write!(result, "{join}");
            }
        }
        result
    }
}

/// A master release from the Discogs API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Master {
    /// The Discogs master ID.
    pub id: u64,
    /// The title of the master release.
    pub title: String,
    /// The artists on this master release.
    #[serde(default)]
    pub artists: Vec<Artist>,
    /// Year of the original release.
    #[serde(default)]
    pub year: Option<i32>,
    /// Genres associated with this master release.
    #[serde(default)]
    pub genres: Vec<String>,
    /// Styles (subgenres) associated with this master release.
    #[serde(default)]
    pub styles: Vec<String>,
    /// The tracklist.
    #[serde(default)]
    pub tracklist: Vec<Track>,
    /// Discogs resource URL.
    #[serde(default)]
    pub resource_url: Option<String>,
    /// URI for this release on the website.
    #[serde(default)]
    pub uri: Option<String>,
    /// Main release ID.
    #[serde(default)]
    pub main_release: Option<u64>,
    /// Number of versions.
    #[serde(default)]
    pub versions_count: Option<u32>,
}

impl Master {
    /// Get the formatted artist name.
    #[must_use]
    pub fn artist_name(&self) -> String {
        let mut result = String::new();
        for (i, artist) in self.artists.iter().enumerate() {
            let _ = write!(result, "{}", artist.name);
            // Add join phrase if present, or default separator if not last artist
            if i < self.artists.len() - 1 {
                let join = artist.join.as_deref().unwrap_or(", ");
                let _ = write!(result, "{join}");
            }
        }
        result
    }
}

/// An artist from the Discogs API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    /// The Discogs artist ID.
    #[serde(default)]
    pub id: Option<u64>,
    /// The artist name.
    pub name: String,
    /// Artist name variations.
    #[serde(default)]
    pub anv: Option<String>,
    /// Join string to the next artist (e.g., " & ", " feat. ").
    #[serde(default)]
    pub join: Option<String>,
    /// Artist role on the track/release.
    #[serde(default)]
    pub role: Option<String>,
    /// Discogs resource URL.
    #[serde(default)]
    pub resource_url: Option<String>,
    /// Whether this is a track artist.
    #[serde(default)]
    pub tracks: Option<String>,
}

/// A track on a release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Track position (e.g., "A1", "1", "B2").
    pub position: String,
    /// Track title.
    pub title: String,
    /// Duration as a string (e.g., "3:45").
    #[serde(default)]
    pub duration: Option<String>,
    /// Extra artists on this track.
    #[serde(default)]
    pub extraartists: Vec<Artist>,
    /// Track type (track, heading, index).
    #[serde(default, rename = "type_")]
    pub track_type: Option<String>,
}

impl Track {
    /// Parse the duration string to milliseconds.
    ///
    /// Handles formats like "3:45" (MM:SS) and "1:03:45" (H:MM:SS).
    #[must_use]
    pub fn duration_ms(&self) -> Option<u64> {
        let duration = self.duration.as_ref()?;
        let parts: Vec<&str> = duration.split(':').collect();

        match parts.len() {
            2 => {
                // MM:SS
                let minutes: u64 = parts[0].parse().ok()?;
                let seconds: u64 = parts[1].parse().ok()?;
                Some((minutes * 60 + seconds) * 1000)
            }
            3 => {
                // H:MM:SS
                let hours: u64 = parts[0].parse().ok()?;
                let minutes: u64 = parts[1].parse().ok()?;
                let seconds: u64 = parts[2].parse().ok()?;
                Some((hours * 3600 + minutes * 60 + seconds) * 1000)
            }
            _ => None,
        }
    }
}

/// A record label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    /// The Discogs label ID.
    #[serde(default)]
    pub id: Option<u64>,
    /// The label name.
    pub name: String,
    /// Catalog number.
    #[serde(default)]
    pub catno: Option<String>,
    /// Discogs resource URL.
    #[serde(default)]
    pub resource_url: Option<String>,
}

/// A release format (CD, Vinyl, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Format {
    /// Format name (e.g., "CD", "Vinyl").
    pub name: String,
    /// Quantity.
    #[serde(default)]
    pub qty: Option<String>,
    /// Text description.
    #[serde(default)]
    pub text: Option<String>,
    /// Format descriptions (e.g., "Album", "LP", "Reissue").
    #[serde(default)]
    pub descriptions: Vec<String>,
}

/// Community rating data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    /// Number of users who want this release.
    #[serde(default)]
    pub want: u32,
    /// Number of users who have this release.
    #[serde(default)]
    pub have: u32,
    /// Rating information.
    #[serde(default)]
    pub rating: Option<Rating>,
}

/// Rating data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rating {
    /// Average rating (0-5).
    #[serde(default)]
    pub average: f32,
    /// Number of ratings.
    #[serde(default)]
    pub count: u32,
}

/// Search result from the Discogs API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Result ID.
    pub id: u64,
    /// Result type (release, master, artist, label).
    #[serde(rename = "type")]
    pub result_type: String,
    /// Title.
    pub title: String,
    /// Year.
    #[serde(default)]
    pub year: Option<String>,
    /// Cover image URL (small).
    #[serde(default)]
    pub thumb: Option<String>,
    /// Cover image URL (large).
    #[serde(default)]
    pub cover_image: Option<String>,
    /// Discogs resource URL.
    #[serde(default)]
    pub resource_url: Option<String>,
    /// URI on the website.
    #[serde(default)]
    pub uri: Option<String>,
    /// Country of release.
    #[serde(default)]
    pub country: Option<String>,
    /// Format descriptions.
    #[serde(default)]
    pub format: Vec<String>,
    /// Genre.
    #[serde(default)]
    pub genre: Vec<String>,
    /// Style (subgenre).
    #[serde(default)]
    pub style: Vec<String>,
    /// Label names.
    #[serde(default)]
    pub label: Vec<String>,
    /// Catalog numbers.
    #[serde(default)]
    pub catno: Option<String>,
    /// Barcode.
    #[serde(default)]
    pub barcode: Vec<String>,
    /// Master release ID.
    #[serde(default)]
    pub master_id: Option<u64>,
    /// Main release URL (for master searches).
    #[serde(default)]
    pub master_url: Option<String>,
}

/// Search response from the Discogs API.
#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    /// Pagination info.
    pub pagination: Pagination,
    /// Search results.
    pub results: Vec<SearchResult>,
}

/// Pagination information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Current page number.
    pub page: u32,
    /// Results per page.
    pub per_page: u32,
    /// Total number of results.
    #[serde(default)]
    pub items: u32,
    /// Total number of pages.
    #[serde(default)]
    pub pages: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_duration_ms() {
        let track = Track {
            position: "1".to_string(),
            title: "Test".to_string(),
            duration: Some("3:45".to_string()),
            extraartists: vec![],
            track_type: None,
        };
        assert_eq!(track.duration_ms(), Some(225_000));

        let track2 = Track {
            position: "2".to_string(),
            title: "Long".to_string(),
            duration: Some("1:03:45".to_string()),
            extraartists: vec![],
            track_type: None,
        };
        assert_eq!(track2.duration_ms(), Some(3_825_000));

        let track3 = Track {
            position: "3".to_string(),
            title: "None".to_string(),
            duration: None,
            extraartists: vec![],
            track_type: None,
        };
        assert_eq!(track3.duration_ms(), None);
    }

    #[test]
    fn test_release_artist_name() {
        let release = Release {
            id: 1,
            title: "Test Album".to_string(),
            artists: vec![
                Artist {
                    id: Some(1),
                    name: "Artist One".to_string(),
                    anv: None,
                    join: Some(" & ".to_string()),
                    role: None,
                    resource_url: None,
                    tracks: None,
                },
                Artist {
                    id: Some(2),
                    name: "Artist Two".to_string(),
                    anv: None,
                    join: None,
                    role: None,
                    resource_url: None,
                    tracks: None,
                },
            ],
            year: Some(2020),
            genres: vec![],
            styles: vec![],
            tracklist: vec![],
            labels: vec![],
            formats: vec![],
            country: None,
            released: None,
            resource_url: None,
            uri: None,
            master_id: None,
            main_release: None,
            versions_count: None,
            notes: None,
            community: None,
        };

        assert_eq!(release.artist_name(), "Artist One & Artist Two");
    }
}
