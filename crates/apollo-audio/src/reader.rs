//! Audio metadata reading functionality.

use crate::error::AudioError;
use apollo_core::{AudioFormat, Track, TrackId};
use chrono::Utc;
use lofty::file::{AudioFile, FileType, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::ItemKey;
use std::path::Path;
use std::time::Duration;
use tracing::{debug, trace};

/// Audio properties extracted from a file.
#[derive(Debug, Clone)]
pub struct AudioProperties {
    /// Duration of the audio.
    pub duration: Duration,
    /// Bitrate in kbps (if available).
    pub bitrate: Option<u32>,
    /// Sample rate in Hz.
    pub sample_rate: Option<u32>,
    /// Number of audio channels.
    pub channels: Option<u8>,
}

/// Read metadata from an audio file and return a Track.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The file format is not supported
/// - No tags are found in the file
pub fn read_metadata(path: &Path) -> Result<Track, AudioError> {
    debug!("Reading metadata from: {}", path.display());

    // Open and probe the file
    let tagged_file = Probe::open(path)
        .map_err(|e| AudioError::read(path, e))?
        .guess_file_type()
        .map_err(AudioError::Io)?
        .read()
        .map_err(|e| AudioError::read(path, e))?;

    // Get the primary tag, or fall back to the first available tag
    let tag = tagged_file
        .primary_tag()
        .or_else(|| tagged_file.first_tag())
        .ok_or_else(|| AudioError::NoTags(path.to_path_buf()))?;

    // Get audio properties
    let properties = tagged_file.properties();

    // Determine format
    let format = file_type_to_audio_format(tagged_file.file_type());

    // Extract basic metadata
    let title = tag.get_string(&ItemKey::TrackTitle).map_or_else(
        || {
            // Fall back to filename without extension
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string()
        },
        String::from,
    );

    let artist = tag
        .get_string(&ItemKey::TrackArtist)
        .map_or_else(|| "Unknown Artist".to_string(), String::from);

    let album_artist = tag.get_string(&ItemKey::AlbumArtist).map(String::from);

    let album_title = tag.get_string(&ItemKey::AlbumTitle).map(String::from);

    let track_number = tag.get_string(&ItemKey::TrackNumber).and_then(parse_number);

    let track_total = tag.get_string(&ItemKey::TrackTotal).and_then(parse_number);

    let disc_number = tag.get_string(&ItemKey::DiscNumber).and_then(parse_number);

    let disc_total = tag.get_string(&ItemKey::DiscTotal).and_then(parse_number);

    let year = tag.get_string(&ItemKey::Year).and_then(parse_year);

    // Parse genres (may be a single string or multiple values)
    let genres = extract_genres(tag);

    // MusicBrainz IDs
    let musicbrainz_id = tag
        .get_string(&ItemKey::MusicBrainzRecordingId)
        .map(String::from);

    // AcoustID is stored under a custom key
    let acoustid = tag
        .get_string(&ItemKey::Unknown("ACOUSTID_ID".to_string()))
        .map(String::from);

    // Build the track
    let now = Utc::now();
    let track = Track {
        id: TrackId::new(),
        path: path.to_path_buf(),
        title,
        artist,
        album_artist,
        album_id: None, // Will be linked later during import
        album_title,
        track_number,
        track_total,
        disc_number,
        disc_total,
        year,
        genres,
        duration: properties.duration(),
        bitrate: properties.audio_bitrate(),
        sample_rate: properties.sample_rate(),
        channels: properties.channels(),
        format,
        musicbrainz_id,
        acoustid,
        added_at: now,
        modified_at: now,
        file_hash: String::new(), // Will be computed separately if needed
    };

    trace!(
        "Read track: '{}' by '{}' ({:?})",
        track.title, track.artist, format
    );

    Ok(track)
}

/// Convert lofty's `FileType` to our `AudioFormat`.
const fn file_type_to_audio_format(file_type: FileType) -> AudioFormat {
    match file_type {
        FileType::Mpeg => AudioFormat::Mp3,
        FileType::Flac => AudioFormat::Flac,
        FileType::Opus => AudioFormat::Opus,
        FileType::Vorbis => AudioFormat::Ogg,
        FileType::Aac => AudioFormat::Aac,
        FileType::Wav => AudioFormat::Wav,
        FileType::Aiff => AudioFormat::Aiff,
        _ => AudioFormat::Unknown,
    }
}

/// Parse a number from a string, handling "1/10" format.
fn parse_number(s: &str) -> Option<u32> {
    // Handle "1/10" format (track number / total)
    let num_part = s.split('/').next()?;
    num_part.trim().parse().ok()
}

/// Parse a year from various formats.
fn parse_year(s: &str) -> Option<i32> {
    // Try full date format first (YYYY-MM-DD)
    if s.len() >= 4 {
        s[..4].parse().ok()
    } else {
        s.parse().ok()
    }
}

/// Extract genres from tags, handling different formats.
fn extract_genres(tag: &lofty::tag::Tag) -> Vec<String> {
    let mut genres = Vec::new();

    // Get all genre items
    for item in tag.items() {
        if item.key() == &ItemKey::Genre
            && let Some(text) = item.value().text()
        {
            // Handle comma-separated genres
            for genre in text.split([',', ';', '/']) {
                let genre = genre.trim();
                if !genre.is_empty() && !genres.contains(&genre.to_string()) {
                    genres.push(genre.to_string());
                }
            }
        }
    }

    // If no genres found, try the standard accessor
    if genres.is_empty()
        && let Some(genre) = tag.get_string(&ItemKey::Genre)
    {
        for g in genre.split([',', ';', '/']) {
            let g = g.trim();
            if !g.is_empty() {
                genres.push(g.to_string());
            }
        }
    }

    genres
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        assert_eq!(parse_number("5"), Some(5));
        assert_eq!(parse_number("1/10"), Some(1));
        assert_eq!(parse_number("  3  "), Some(3));
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("abc"), None);
    }

    #[test]
    fn test_parse_year() {
        assert_eq!(parse_year("2023"), Some(2023));
        assert_eq!(parse_year("2023-05-15"), Some(2023));
        assert_eq!(parse_year("1985"), Some(1985));
        assert_eq!(parse_year(""), None);
    }

    #[test]
    fn test_file_type_to_audio_format() {
        assert_eq!(file_type_to_audio_format(FileType::Mpeg), AudioFormat::Mp3);
        assert_eq!(file_type_to_audio_format(FileType::Flac), AudioFormat::Flac);
        assert_eq!(
            file_type_to_audio_format(FileType::Vorbis),
            AudioFormat::Ogg
        );
        assert_eq!(file_type_to_audio_format(FileType::Opus), AudioFormat::Opus);
    }
}
