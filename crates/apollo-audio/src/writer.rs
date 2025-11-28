//! Audio metadata writing functionality.

use crate::error::AudioError;
use apollo_core::Track;
use lofty::config::WriteOptions;
use lofty::file::{AudioFile, FileType, TaggedFileExt};
use lofty::probe::Probe;
use lofty::tag::{Accessor, ItemKey, Tag, TagType};
use std::path::Path;
use tracing::{debug, trace};

/// Write metadata from a Track back to an audio file.
///
/// This updates the existing tags in the file with values from the Track.
/// Only non-None fields are written.
///
/// # Errors
///
/// Returns an error if:
/// - The file cannot be read
/// - The file format doesn't support writing
/// - Writing fails
///
/// # Panics
///
/// This function will not panic under normal conditions. The internal expect
/// is guarded by logic that ensures a tag exists before access.
pub fn write_metadata(path: &Path, track: &Track) -> Result<(), AudioError> {
    debug!("Writing metadata to: {}", path.display());

    // Open and probe the file
    let mut tagged_file = Probe::open(path)
        .map_err(|e| AudioError::read(path, e))?
        .guess_file_type()
        .map_err(AudioError::Io)?
        .read()
        .map_err(|e| AudioError::read(path, e))?;

    // Ensure there's a tag to write to
    let has_tag = tagged_file.primary_tag().is_some() || tagged_file.first_tag().is_some();

    if !has_tag {
        // Create a new tag of the appropriate type
        let tag_type = get_preferred_tag_type(tagged_file.file_type());
        tagged_file.insert_tag(Tag::new(tag_type));
    }

    // Get a mutable reference to the tag - we use the index to work around borrow checker
    // Get tag type (prefer first available tag)
    let tag_type = tagged_file
        .tags()
        .first()
        .map_or(TagType::Id3v2, lofty::tag::Tag::tag_type);

    let tag = tagged_file
        .tag_mut(tag_type)
        .expect("tag should exist after creation");

    // Set basic fields
    tag.set_title(track.title.clone());
    tag.set_artist(track.artist.clone());

    // Set optional string fields
    if let Some(ref album_artist) = track.album_artist {
        tag.insert_text(ItemKey::AlbumArtist, album_artist.clone());
    }

    if let Some(ref album_title) = track.album_title {
        tag.set_album(album_title.clone());
    }

    // Set track number
    if let Some(num) = track.track_number {
        if let Some(total) = track.track_total {
            tag.insert_text(ItemKey::TrackNumber, format!("{num}/{total}"));
        } else {
            tag.set_track(num);
        }
    }

    // Set disc number
    if let Some(num) = track.disc_number {
        if let Some(total) = track.disc_total {
            tag.insert_text(ItemKey::DiscNumber, format!("{num}/{total}"));
        } else {
            tag.set_disk(num);
        }
    }

    // Set year (convert i32 to u32, skip if negative)
    if let Some(year) = track.year
        && let Ok(year_u32) = u32::try_from(year)
    {
        tag.set_year(year_u32);
    }

    // Set genres
    if !track.genres.is_empty() {
        tag.set_genre(track.genres.join("; "));
    }

    // Set MusicBrainz ID
    if let Some(ref mbid) = track.musicbrainz_id {
        tag.insert_text(ItemKey::MusicBrainzRecordingId, mbid.clone());
    }

    // Set AcoustID (uses custom key)
    if let Some(ref acoustid) = track.acoustid {
        tag.insert_text(
            ItemKey::Unknown("ACOUSTID_ID".to_string()),
            acoustid.clone(),
        );
    }

    trace!("Saving tags to file");

    // Save the file
    tagged_file
        .save_to_path(path, WriteOptions::default())
        .map_err(|e| AudioError::write(path, e))?;

    debug!("Successfully wrote metadata to: {}", path.display());
    Ok(())
}

/// Get the preferred tag type for a file type.
const fn get_preferred_tag_type(file_type: FileType) -> TagType {
    match file_type {
        FileType::Flac | FileType::Opus | FileType::Vorbis => TagType::VorbisComments,
        FileType::Mp4 => TagType::Mp4Ilst,
        FileType::Ape => TagType::Ape,
        // Mpeg, Aiff, Wav, and others default to ID3v2
        _ => TagType::Id3v2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_preferred_tag_type() {
        assert_eq!(get_preferred_tag_type(FileType::Mpeg), TagType::Id3v2);
        assert_eq!(
            get_preferred_tag_type(FileType::Flac),
            TagType::VorbisComments
        );
        assert_eq!(
            get_preferred_tag_type(FileType::Vorbis),
            TagType::VorbisComments
        );
        assert_eq!(get_preferred_tag_type(FileType::Mp4), TagType::Mp4Ilst);
    }
}
