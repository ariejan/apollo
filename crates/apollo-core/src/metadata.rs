//! Metadata types for tracks, albums, and artists.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

/// Unique identifier for a track.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackId(pub Uuid);

impl TrackId {
    /// Create a new random track ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TrackId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for TrackId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Unique identifier for an album.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AlbumId(pub Uuid);

impl AlbumId {
    /// Create a new random album ID.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for AlbumId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for AlbumId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Audio format/codec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Mp3,
    Flac,
    Ogg,
    Opus,
    Aac,
    Wav,
    Aiff,
    Unknown,
}

impl std::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mp3 => write!(f, "MP3"),
            Self::Flac => write!(f, "FLAC"),
            Self::Ogg => write!(f, "OGG"),
            Self::Opus => write!(f, "Opus"),
            Self::Aac => write!(f, "AAC"),
            Self::Wav => write!(f, "WAV"),
            Self::Aiff => write!(f, "AIFF"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Represents a single audio track in the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    /// Unique identifier.
    pub id: TrackId,
    /// Path to the audio file.
    pub path: PathBuf,
    /// Track title.
    pub title: String,
    /// Primary artist name.
    pub artist: String,
    /// Album artist (may differ from track artist).
    pub album_artist: Option<String>,
    /// Album this track belongs to.
    pub album_id: Option<AlbumId>,
    /// Album title (denormalized for convenience).
    pub album_title: Option<String>,
    /// Track number within the album.
    pub track_number: Option<u32>,
    /// Total tracks on the album.
    pub track_total: Option<u32>,
    /// Disc number for multi-disc albums.
    pub disc_number: Option<u32>,
    /// Total discs in the album.
    pub disc_total: Option<u32>,
    /// Release year.
    pub year: Option<i32>,
    /// Genre tags.
    pub genres: Vec<String>,
    /// Track duration.
    #[serde(with = "duration_serde")]
    pub duration: Duration,
    /// Bitrate in kbps (if applicable).
    pub bitrate: Option<u32>,
    /// Sample rate in Hz.
    pub sample_rate: Option<u32>,
    /// Number of audio channels.
    pub channels: Option<u8>,
    /// Audio format.
    pub format: AudioFormat,
    /// [MusicBrainz](https://musicbrainz.org/) recording ID.
    pub musicbrainz_id: Option<String>,
    /// [AcoustID](https://acoustid.org/) fingerprint identifier.
    pub acoustid: Option<String>,
    /// When the track was added to the library.
    pub added_at: DateTime<Utc>,
    /// When the track metadata was last modified.
    pub modified_at: DateTime<Utc>,
    /// SHA-256 hash of the file contents.
    pub file_hash: String,
}

impl Track {
    /// Create a new track with minimal required fields.
    #[must_use]
    pub fn new(path: PathBuf, title: String, artist: String, duration: Duration) -> Self {
        let now = Utc::now();
        Self {
            id: TrackId::new(),
            path,
            title,
            artist,
            album_artist: None,
            album_id: None,
            album_title: None,
            track_number: None,
            track_total: None,
            disc_number: None,
            disc_total: None,
            year: None,
            genres: Vec::new(),
            duration,
            bitrate: None,
            sample_rate: None,
            channels: None,
            format: AudioFormat::Unknown,
            musicbrainz_id: None,
            acoustid: None,
            added_at: now,
            modified_at: now,
            file_hash: String::new(),
        }
    }
}

/// Represents an album in the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Album {
    /// Unique identifier.
    pub id: AlbumId,
    /// Album title.
    pub title: String,
    /// Album artist.
    pub artist: String,
    /// Release year.
    pub year: Option<i32>,
    /// Genre tags.
    pub genres: Vec<String>,
    /// Number of tracks.
    pub track_count: u32,
    /// Number of discs.
    pub disc_count: u32,
    /// [MusicBrainz](https://musicbrainz.org/) release ID.
    pub musicbrainz_id: Option<String>,
    /// When the album was added to the library.
    pub added_at: DateTime<Utc>,
    /// When the album was last modified.
    pub modified_at: DateTime<Utc>,
}

impl Album {
    /// Create a new album with minimal required fields.
    #[must_use]
    pub fn new(title: String, artist: String) -> Self {
        let now = Utc::now();
        Self {
            id: AlbumId::new(),
            title,
            artist,
            year: None,
            genres: Vec::new(),
            track_count: 0,
            disc_count: 1,
            musicbrainz_id: None,
            added_at: now,
            modified_at: now,
        }
    }
}

/// Represents an artist in the library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artist {
    /// Artist name (primary identifier).
    pub name: String,
    /// Sort name (e.g., "Beatles, The").
    pub sort_name: Option<String>,
    /// [MusicBrainz](https://musicbrainz.org/) artist ID.
    pub musicbrainz_id: Option<String>,
}

impl Artist {
    /// Create a new artist.
    #[must_use]
    pub const fn new(name: String) -> Self {
        Self {
            name,
            sort_name: None,
            musicbrainz_id: None,
        }
    }
}

/// Custom serde module for Duration.
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        duration.as_millis().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis = u64::deserialize(deserializer)?;
        Ok(Duration::from_millis(millis))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn track_creation() {
        let track = Track::new(
            PathBuf::from("/music/test.mp3"),
            "Test Song".to_string(),
            "Test Artist".to_string(),
            Duration::from_secs(180),
        );

        assert_eq!(track.title, "Test Song");
        assert_eq!(track.artist, "Test Artist");
        assert_eq!(track.duration, Duration::from_secs(180));
    }

    #[test]
    fn track_serialization() {
        let track = Track::new(
            PathBuf::from("/music/test.mp3"),
            "Test Song".to_string(),
            "Test Artist".to_string(),
            Duration::from_secs(180),
        );

        let json = serde_json::to_string(&track).unwrap();
        let deserialized: Track = serde_json::from_str(&json).unwrap();

        assert_eq!(track.title, deserialized.title);
        assert_eq!(track.artist, deserialized.artist);
        assert_eq!(track.duration, deserialized.duration);
    }

    /// Strategy for generating valid audio formats.
    fn audio_format_strategy() -> impl Strategy<Value = AudioFormat> {
        prop_oneof![
            Just(AudioFormat::Mp3),
            Just(AudioFormat::Flac),
            Just(AudioFormat::Ogg),
            Just(AudioFormat::Opus),
            Just(AudioFormat::Aac),
            Just(AudioFormat::Wav),
            Just(AudioFormat::Aiff),
            Just(AudioFormat::Unknown),
        ]
    }

    /// Strategy for generating valid file paths (non-empty, valid characters).
    fn path_strategy() -> impl Strategy<Value = PathBuf> {
        "[a-zA-Z0-9_/.-]{1,100}".prop_map(PathBuf::from)
    }

    /// Strategy for generating non-empty strings (for titles, artists, etc.).
    fn non_empty_string() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9 _-]{1,50}"
    }

    /// Strategy for generating duration in reasonable range (0 to 24 hours).
    fn duration_strategy() -> impl Strategy<Value = Duration> {
        (0u64..86_400_000u64).prop_map(Duration::from_millis)
    }

    proptest! {
        /// Test that Track serialization roundtrips correctly for any valid track.
        #[test]
        fn track_serialization_roundtrip(
            path in path_strategy(),
            title in non_empty_string(),
            artist in non_empty_string(),
            duration in duration_strategy(),
        ) {
            let track = Track::new(path, title, artist, duration);

            let json = serde_json::to_string(&track).expect("serialization should succeed");
            let deserialized: Track = serde_json::from_str(&json).expect("deserialization should succeed");

            prop_assert_eq!(&track.title, &deserialized.title);
            prop_assert_eq!(&track.artist, &deserialized.artist);
            prop_assert_eq!(track.duration, deserialized.duration);
            prop_assert_eq!(track.path, deserialized.path);
        }

        /// Test that Album serialization roundtrips correctly for any valid album.
        #[test]
        fn album_serialization_roundtrip(
            title in non_empty_string(),
            artist in non_empty_string(),
            year in proptest::option::of(1900i32..2100i32),
            track_count in 0u32..1000u32,
            disc_count in 1u32..20u32,
        ) {
            let mut album = Album::new(title, artist);
            album.year = year;
            album.track_count = track_count;
            album.disc_count = disc_count;

            let json = serde_json::to_string(&album).expect("serialization should succeed");
            let deserialized: Album = serde_json::from_str(&json).expect("deserialization should succeed");

            prop_assert_eq!(&album.title, &deserialized.title);
            prop_assert_eq!(&album.artist, &deserialized.artist);
            prop_assert_eq!(album.year, deserialized.year);
            prop_assert_eq!(album.track_count, deserialized.track_count);
            prop_assert_eq!(album.disc_count, deserialized.disc_count);
        }

        /// Test that Artist serialization roundtrips correctly for any valid artist.
        #[test]
        fn artist_serialization_roundtrip(
            name in non_empty_string(),
            sort_name in proptest::option::of(non_empty_string()),
            musicbrainz_id in proptest::option::of("[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}"),
        ) {
            let mut artist = Artist::new(name);
            artist.sort_name = sort_name;
            artist.musicbrainz_id = musicbrainz_id;

            let json = serde_json::to_string(&artist).expect("serialization should succeed");
            let deserialized: Artist = serde_json::from_str(&json).expect("deserialization should succeed");

            prop_assert_eq!(&artist.name, &deserialized.name);
            prop_assert_eq!(&artist.sort_name, &deserialized.sort_name);
            prop_assert_eq!(&artist.musicbrainz_id, &deserialized.musicbrainz_id);
        }

        /// Test that AudioFormat display is always valid (non-empty, consistent).
        #[test]
        fn audio_format_display_is_valid(format in audio_format_strategy()) {
            let display = format.to_string();
            prop_assert!(!display.is_empty(), "display should not be empty");
        }

        /// Test that TrackId generates unique IDs.
        #[test]
        fn track_id_uniqueness(_ in 0..100u32) {
            let id1 = TrackId::new();
            let id2 = TrackId::new();
            prop_assert_ne!(id1, id2, "generated IDs should be unique");
        }

        /// Test that AlbumId generates unique IDs.
        #[test]
        fn album_id_uniqueness(_ in 0..100u32) {
            let id1 = AlbumId::new();
            let id2 = AlbumId::new();
            prop_assert_ne!(id1, id2, "generated IDs should be unique");
        }

        /// Test that TrackId Display output is valid UUID format.
        #[test]
        fn track_id_display_is_uuid(_ in 0..100u32) {
            let id = TrackId::new();
            let display = id.to_string();
            // UUID format: 8-4-4-4-12 hex characters
            prop_assert!(display.len() == 36, "UUID should be 36 characters");
            prop_assert!(display.chars().filter(|c| *c == '-').count() == 4, "UUID should have 4 dashes");
        }

        /// Test that AlbumId Display output is valid UUID format.
        #[test]
        fn album_id_display_is_uuid(_ in 0..100u32) {
            let id = AlbumId::new();
            let display = id.to_string();
            prop_assert!(display.len() == 36, "UUID should be 36 characters");
            prop_assert!(display.chars().filter(|c| *c == '-').count() == 4, "UUID should have 4 dashes");
        }

        /// Test duration serialization roundtrip preserves millisecond precision.
        #[test]
        fn duration_serialization_preserves_millis(millis in 0u64..u64::MAX / 1000) {
            let duration = Duration::from_millis(millis);

            // Create a track with this duration
            let track = Track::new(
                PathBuf::from("/test.mp3"),
                "Test".to_string(),
                "Artist".to_string(),
                duration,
            );

            let json = serde_json::to_string(&track).expect("serialization should succeed");
            let deserialized: Track = serde_json::from_str(&json).expect("deserialization should succeed");

            prop_assert_eq!(track.duration.as_millis(), deserialized.duration.as_millis());
        }
    }
}
