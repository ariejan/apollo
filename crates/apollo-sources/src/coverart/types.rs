//! Cover art types.

use serde::{Deserialize, Serialize};

/// Image type/size.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageSize {
    /// Small thumbnail (usually 250px).
    Small,
    /// Medium size (usually 500px).
    Medium,
    /// Large/full size.
    #[default]
    Large,
    /// Original/maximum resolution.
    Original,
}

/// Type of cover art.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoverType {
    /// Front cover art.
    #[default]
    Front,
    /// Back cover art.
    Back,
    /// Medium (CD, vinyl label, etc.).
    Medium,
    /// Booklet pages.
    Booklet,
    /// Other/miscellaneous.
    Other,
}

/// Information about a cover image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverImage {
    /// URL to fetch the image from.
    pub url: String,
    /// Type of cover art.
    pub cover_type: CoverType,
    /// Size/resolution hint.
    pub size: ImageSize,
    /// Whether this is the primary/front cover.
    pub is_front: bool,
    /// Source of the image (e.g., "coverartarchive", "discogs").
    pub source: String,
    /// Optional description or comment.
    #[serde(default)]
    pub comment: Option<String>,
}

impl CoverImage {
    /// Create a new cover image reference.
    #[must_use]
    pub fn new(url: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            cover_type: CoverType::Front,
            size: ImageSize::Large,
            is_front: true,
            source: source.into(),
            comment: None,
        }
    }

    /// Set the cover type.
    #[must_use]
    pub fn with_type(mut self, cover_type: CoverType) -> Self {
        self.cover_type = cover_type;
        self.is_front = cover_type == CoverType::Front;
        self
    }

    /// Set the size.
    #[must_use]
    pub const fn with_size(mut self, size: ImageSize) -> Self {
        self.size = size;
        self
    }

    /// Set whether this is the front cover.
    #[must_use]
    pub const fn with_is_front(mut self, is_front: bool) -> Self {
        self.is_front = is_front;
        self
    }

    /// Set the comment.
    #[must_use]
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }
}

/// Cover Art Archive response for release images.
#[derive(Debug, Deserialize)]
pub struct CoverArtArchiveResponse {
    /// List of images for the release.
    pub images: Vec<CoverArtArchiveImage>,
    /// URL of the release on Cover Art Archive.
    #[serde(default)]
    pub release: Option<String>,
}

/// An image from Cover Art Archive.
#[derive(Debug, Deserialize)]
pub struct CoverArtArchiveImage {
    /// Types of this image (e.g., "Front", "Back").
    #[serde(default)]
    pub types: Vec<String>,
    /// Whether this is the front cover.
    #[serde(default)]
    pub front: bool,
    /// Whether this is the back cover.
    #[serde(default)]
    pub back: bool,
    /// Whether this is approved.
    #[serde(default)]
    pub approved: bool,
    /// Image ID.
    #[serde(default)]
    pub id: u64,
    /// Edit ID.
    #[serde(default)]
    pub edit: Option<u64>,
    /// Comment on the image.
    #[serde(default)]
    pub comment: Option<String>,
    /// URL to the full image.
    pub image: String,
    /// Thumbnail URLs.
    #[serde(default)]
    pub thumbnails: Thumbnails,
}

/// Thumbnail URLs from Cover Art Archive.
#[derive(Debug, Default, Deserialize)]
pub struct Thumbnails {
    /// Small thumbnail (250px).
    #[serde(default, rename = "250")]
    pub small: Option<String>,
    /// Large thumbnail (500px).
    #[serde(default, rename = "500")]
    pub medium: Option<String>,
    /// 1200px version.
    #[serde(default, rename = "1200")]
    pub large: Option<String>,
}

impl CoverArtArchiveImage {
    /// Convert to a `CoverImage`.
    #[must_use]
    pub fn to_cover_image(&self, size: ImageSize) -> CoverImage {
        let url = match size {
            ImageSize::Small => self.thumbnails.small.as_ref().unwrap_or(&self.image),
            ImageSize::Medium => self.thumbnails.medium.as_ref().unwrap_or(&self.image),
            ImageSize::Large => self.thumbnails.large.as_ref().unwrap_or(&self.image),
            ImageSize::Original => &self.image,
        };

        let cover_type = if self.front {
            CoverType::Front
        } else if self.back {
            CoverType::Back
        } else if self.types.iter().any(|t| t.eq_ignore_ascii_case("medium")) {
            CoverType::Medium
        } else if self.types.iter().any(|t| t.eq_ignore_ascii_case("booklet")) {
            CoverType::Booklet
        } else {
            CoverType::Other
        };

        CoverImage {
            url: url.clone(),
            cover_type,
            size,
            is_front: self.front,
            source: "coverartarchive".to_string(),
            comment: self.comment.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cover_image_builder() {
        let img = CoverImage::new("https://example.com/image.jpg", "test")
            .with_type(CoverType::Back)
            .with_size(ImageSize::Small)
            .with_comment("Test image");

        assert_eq!(img.url, "https://example.com/image.jpg");
        assert_eq!(img.cover_type, CoverType::Back);
        assert_eq!(img.size, ImageSize::Small);
        assert!(!img.is_front);
        assert_eq!(img.comment, Some("Test image".to_string()));
    }

    #[test]
    fn test_image_size_default() {
        assert_eq!(ImageSize::default(), ImageSize::Large);
    }

    #[test]
    fn test_cover_type_default() {
        assert_eq!(CoverType::default(), CoverType::Front);
    }
}
