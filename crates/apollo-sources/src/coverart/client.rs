//! Cover art fetching client.

use crate::coverart::types::{
    CoverArtArchiveResponse, CoverImage, CoverType, ImageSize,
};
use crate::error::{SourceError, SourceResult};
use reqwest::Client;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// Cover Art Archive API base URL.
const CAA_API_BASE: &str = "https://coverartarchive.org";

/// Minimum delay between requests.
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(1100);

/// Client for fetching album cover art from various sources.
///
/// Supports:
/// - [Cover Art Archive](https://coverartarchive.org/) (linked to [MusicBrainz](https://musicbrainz.org/))
/// - Direct URL downloads
///
/// # Example
///
/// ```no_run
/// use apollo_sources::coverart::{CoverArtClient, ImageSize};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = CoverArtClient::new("MyApp", "1.0")?;
///
/// // Fetch cover art for a MusicBrainz release
/// let images = client.get_release_art("release-mbid-here").await?;
/// for img in images {
///     println!("{}: {}", img.source, img.url);
/// }
///
/// // Fetch just the front cover
/// let front = client.get_front_cover("release-mbid-here", ImageSize::Large).await?;
/// println!("Front cover: {}", front.url);
/// # Ok(())
/// # }
/// ```
pub struct CoverArtClient {
    client: Client,
    last_request: Mutex<Instant>,
}

impl CoverArtClient {
    /// Create a new cover art client.
    ///
    /// # Arguments
    ///
    /// * `app_name` - Name of your application
    /// * `app_version` - Version of your application
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(app_name: &str, app_version: &str) -> SourceResult<Self> {
        let user_agent = format!("{app_name}/{app_version}");

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&user_agent)
                .map_err(|e| SourceError::InvalidInput(e.to_string()))?,
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            last_request: Mutex::new(
                Instant::now()
                    .checked_sub(MIN_REQUEST_INTERVAL)
                    .unwrap_or_else(Instant::now),
            ),
        })
    }

    /// Wait for rate limiting before making a request.
    async fn wait_for_rate_limit(&self) {
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();

        if elapsed < MIN_REQUEST_INTERVAL {
            let wait = MIN_REQUEST_INTERVAL - elapsed;
            debug!("Rate limiting: waiting {:?}", wait);
            tokio::time::sleep(wait).await;
        }

        *last = Instant::now();
    }

    /// Get all cover art for a [MusicBrainz](https://musicbrainz.org/) release.
    ///
    /// # Arguments
    ///
    /// * `release_mbid` - The [MusicBrainz](https://musicbrainz.org/) release ID
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or no art is found.
    pub async fn get_release_art(&self, release_mbid: &str) -> SourceResult<Vec<CoverImage>> {
        self.wait_for_rate_limit().await;

        let url = format!("{CAA_API_BASE}/release/{release_mbid}");
        debug!("GET {url}");

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(SourceError::NotFound);
        }

        if status == reqwest::StatusCode::SERVICE_UNAVAILABLE
            || status == reqwest::StatusCode::TOO_MANY_REQUESTS
        {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(60);

            warn!("Rate limited, retry after {retry_after} seconds");
            return Err(SourceError::RateLimited { retry_after });
        }

        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(SourceError::Api {
                status: status.as_u16(),
                message,
            });
        }

        let body = response.text().await?;
        let caa_response: CoverArtArchiveResponse =
            serde_json::from_str(&body).map_err(|e| SourceError::Parse(e.to_string()))?;

        // Convert to CoverImage list
        let images = caa_response
            .images
            .iter()
            .map(|img| img.to_cover_image(ImageSize::Large))
            .collect();

        Ok(images)
    }

    /// Get all cover art for a [MusicBrainz](https://musicbrainz.org/) release group.
    ///
    /// Release groups aggregate different editions of the same album.
    ///
    /// # Arguments
    ///
    /// * `release_group_mbid` - The [MusicBrainz](https://musicbrainz.org/) release group ID
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or no art is found.
    pub async fn get_release_group_art(
        &self,
        release_group_mbid: &str,
    ) -> SourceResult<Vec<CoverImage>> {
        self.wait_for_rate_limit().await;

        let url = format!("{CAA_API_BASE}/release-group/{release_group_mbid}");
        debug!("GET {url}");

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(SourceError::NotFound);
        }

        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(SourceError::Api {
                status: status.as_u16(),
                message,
            });
        }

        let body = response.text().await?;
        let caa_response: CoverArtArchiveResponse =
            serde_json::from_str(&body).map_err(|e| SourceError::Parse(e.to_string()))?;

        let images = caa_response
            .images
            .iter()
            .map(|img| img.to_cover_image(ImageSize::Large))
            .collect();

        Ok(images)
    }

    /// Get the front cover for a release.
    ///
    /// # Arguments
    ///
    /// * `release_mbid` - The [MusicBrainz](https://musicbrainz.org/) release ID
    /// * `size` - The desired image size
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or no front cover is found.
    pub async fn get_front_cover(
        &self,
        release_mbid: &str,
        size: ImageSize,
    ) -> SourceResult<CoverImage> {
        let images = self.get_release_art(release_mbid).await?;

        images
            .into_iter()
            .find(|img| img.is_front)
            .map(|mut img| {
                img.size = size;
                img
            })
            .ok_or(SourceError::NotFound)
    }

    /// Get cover art by type.
    ///
    /// # Arguments
    ///
    /// * `release_mbid` - The [MusicBrainz](https://musicbrainz.org/) release ID
    /// * `cover_type` - The type of cover to find
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or no matching cover is found.
    pub async fn get_cover_by_type(
        &self,
        release_mbid: &str,
        cover_type: CoverType,
    ) -> SourceResult<CoverImage> {
        let images = self.get_release_art(release_mbid).await?;

        images
            .into_iter()
            .find(|img| img.cover_type == cover_type)
            .ok_or(SourceError::NotFound)
    }

    /// Download an image from a URL to bytes.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download from
    ///
    /// # Errors
    ///
    /// Returns an error if the download fails.
    pub async fn download_image(&self, url: &str) -> SourceResult<Vec<u8>> {
        self.wait_for_rate_limit().await;

        debug!("Downloading image from {url}");

        let response = self.client.get(url).send().await?;
        let status = response.status();

        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(SourceError::Api {
                status: status.as_u16(),
                message,
            });
        }

        let bytes = response.bytes().await?;
        Ok(bytes.to_vec())
    }

    /// Download an image and save it to a file.
    ///
    /// # Arguments
    ///
    /// * `url` - The URL to download from
    /// * `path` - The path to save the file to
    ///
    /// # Errors
    ///
    /// Returns an error if the download or file write fails.
    pub async fn download_image_to_file(
        &self,
        url: &str,
        path: impl AsRef<Path>,
    ) -> SourceResult<()> {
        let bytes = self.download_image(url).await?;

        tokio::fs::write(path, bytes)
            .await
            .map_err(|e| SourceError::InvalidInput(format!("Failed to write file: {e}")))?;

        Ok(())
    }

    /// Get the direct URL for the front cover image.
    ///
    /// This returns a URL that redirects to the image, useful when you
    /// just need a URL without fetching the full image list.
    ///
    /// # Arguments
    ///
    /// * `release_mbid` - The [MusicBrainz](https://musicbrainz.org/) release ID
    /// * `size` - The desired image size (Small=250, Large=500, Original=full)
    #[must_use]
    pub fn front_cover_url(release_mbid: &str, size: ImageSize) -> String {
        let size_suffix = match size {
            ImageSize::Small => "-250",
            ImageSize::Medium | ImageSize::Large => "-500",
            ImageSize::Original => "",
        };
        format!("{CAA_API_BASE}/release/{release_mbid}/front{size_suffix}")
    }

    /// Get the direct URL for the back cover image.
    ///
    /// # Arguments
    ///
    /// * `release_mbid` - The [MusicBrainz](https://musicbrainz.org/) release ID
    /// * `size` - The desired image size
    #[must_use]
    pub fn back_cover_url(release_mbid: &str, size: ImageSize) -> String {
        let size_suffix = match size {
            ImageSize::Small => "-250",
            ImageSize::Medium | ImageSize::Large => "-500",
            ImageSize::Original => "",
        };
        format!("{CAA_API_BASE}/release/{release_mbid}/back{size_suffix}")
    }

    /// Create cover images from Discogs release data.
    ///
    /// This extracts cover image URLs from Discogs search results.
    ///
    /// # Arguments
    ///
    /// * `thumb_url` - The thumbnail URL (small)
    /// * `cover_url` - The full cover image URL (if available)
    #[must_use]
    pub fn from_discogs_urls(thumb_url: Option<&str>, cover_url: Option<&str>) -> Vec<CoverImage> {
        let mut images = Vec::new();

        if let Some(url) = cover_url {
            images.push(
                CoverImage::new(url, "discogs")
                    .with_type(CoverType::Front)
                    .with_size(ImageSize::Large),
            );
        }

        if let Some(url) = thumb_url {
            images.push(
                CoverImage::new(url, "discogs")
                    .with_type(CoverType::Front)
                    .with_size(ImageSize::Small),
            );
        }

        images
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let result = CoverArtClient::new("TestApp", "1.0");
        assert!(result.is_ok());
    }

    #[test]
    fn test_front_cover_url() {
        let url = CoverArtClient::front_cover_url("test-mbid", ImageSize::Small);
        assert!(url.contains("/front-250"));

        let url = CoverArtClient::front_cover_url("test-mbid", ImageSize::Large);
        assert!(url.contains("/front-500"));

        let url = CoverArtClient::front_cover_url("test-mbid", ImageSize::Original);
        assert!(url.ends_with("/front"));
    }

    #[test]
    fn test_back_cover_url() {
        let url = CoverArtClient::back_cover_url("test-mbid", ImageSize::Small);
        assert!(url.contains("/back-250"));
    }

    #[test]
    fn test_from_discogs_urls() {
        let images = CoverArtClient::from_discogs_urls(Some("thumb.jpg"), Some("cover.jpg"));
        assert_eq!(images.len(), 2);
        assert_eq!(images[0].url, "cover.jpg");
        assert_eq!(images[0].size, ImageSize::Large);
        assert_eq!(images[1].url, "thumb.jpg");
        assert_eq!(images[1].size, ImageSize::Small);
    }
}
