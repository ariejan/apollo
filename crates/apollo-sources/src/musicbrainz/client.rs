//! [MusicBrainz](https://musicbrainz.org/) API client.

use crate::error::{SourceError, SourceResult};
use crate::musicbrainz::types::{
    Recording, RecordingSearchResponse, Release, ReleaseSearchResponse,
};
use reqwest::Client;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};
use std::fmt::Write;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// API base URL.
const API_BASE: &str = "https://musicbrainz.org/ws/2";

/// Minimum delay between requests (the API requires 1 request/second max).
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(1100);

/// API client with rate limiting.
pub struct MusicBrainzClient {
    client: Client,
    last_request: Mutex<Instant>,
}

impl MusicBrainzClient {
    /// Create a new client.
    ///
    /// # Arguments
    ///
    /// * `app_name` - Name of your application
    /// * `app_version` - Version of your application
    /// * `contact` - Contact email or URL
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(app_name: &str, app_version: &str, contact: &str) -> SourceResult<Self> {
        let user_agent = format!("{app_name}/{app_version} ( {contact} )");

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
            // Initialize to past so first request goes through immediately
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

    /// Make a GET request to the API.
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> SourceResult<T> {
        self.wait_for_rate_limit().await;

        let url = format!("{API_BASE}{path}");
        debug!("GET {url}");

        let response = self.client.get(&url).send().await?;
        let status = response.status();

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
        serde_json::from_str(&body).map_err(|e| SourceError::Parse(e.to_string()))
    }

    /// Search for recordings (songs) by title and artist.
    ///
    /// # Arguments
    ///
    /// * `title` - The track title to search for
    /// * `artist` - The artist name to search for (optional)
    /// * `limit` - Maximum number of results (1-100)
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn search_recordings(
        &self,
        title: &str,
        artist: Option<&str>,
        limit: u32,
    ) -> SourceResult<Vec<Recording>> {
        let mut query = format!("recording:\"{}\"", escape_lucene(title));

        if let Some(artist) = artist {
            write!(query, " AND artist:\"{}\"", escape_lucene(artist)).unwrap();
        }

        let path = format!(
            "/recording?query={}&limit={limit}",
            urlencoding::encode(&query)
        );

        let response: RecordingSearchResponse = self.get(&path).await?;
        Ok(response.recordings)
    }

    /// Search for releases (albums) by title and artist.
    ///
    /// # Arguments
    ///
    /// * `title` - The album title to search for
    /// * `artist` - The artist name to search for (optional)
    /// * `limit` - Maximum number of results (1-100)
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn search_releases(
        &self,
        title: &str,
        artist: Option<&str>,
        limit: u32,
    ) -> SourceResult<Vec<Release>> {
        let mut query = format!("release:\"{}\"", escape_lucene(title));

        if let Some(artist) = artist {
            write!(query, " AND artist:\"{}\"", escape_lucene(artist)).unwrap();
        }

        let path = format!(
            "/release?query={}&limit={limit}",
            urlencoding::encode(&query)
        );

        let response: ReleaseSearchResponse = self.get(&path).await?;
        Ok(response.releases)
    }

    /// Look up a recording by its MBID.
    ///
    /// # Arguments
    ///
    /// * `mbid` - The MBID of the recording
    /// * `include` - Optional list of related entities to include (e.g., "releases", "artists")
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the recording is not found.
    pub async fn lookup_recording(&self, mbid: &str, include: &[&str]) -> SourceResult<Recording> {
        let inc = if include.is_empty() {
            String::new()
        } else {
            format!("&inc={}", include.join("+"))
        };

        let path = format!("/recording/{mbid}?fmt=json{inc}");
        self.get(&path).await
    }

    /// Look up a release by its MBID.
    ///
    /// # Arguments
    ///
    /// * `mbid` - The MBID of the release
    /// * `include` - Optional list of related entities to include (e.g., "recordings", "artists")
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the release is not found.
    pub async fn lookup_release(&self, mbid: &str, include: &[&str]) -> SourceResult<Release> {
        let inc = if include.is_empty() {
            String::new()
        } else {
            format!("&inc={}", include.join("+"))
        };

        let path = format!("/release/{mbid}?fmt=json{inc}");
        self.get(&path).await
    }

    /// Search for a recording that best matches the given metadata.
    ///
    /// Returns the best match if the score is above the threshold.
    ///
    /// # Arguments
    ///
    /// * `title` - The track title
    /// * `artist` - The artist name
    /// * `album` - The album title (optional)
    /// * `duration_ms` - The track duration in milliseconds (optional)
    /// * `min_score` - Minimum match score (0-100)
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn find_best_recording(
        &self,
        title: &str,
        artist: &str,
        album: Option<&str>,
        duration_ms: Option<u64>,
        min_score: u8,
    ) -> SourceResult<Option<Recording>> {
        let recordings = self.search_recordings(title, Some(artist), 10).await?;

        // Find best match considering score, album, and duration
        let best = recordings.into_iter().find(|r| {
            // Must meet minimum score
            let score = r.score.unwrap_or(0);
            if score < min_score {
                return false;
            }

            // If album is specified, prefer recordings on matching releases
            if let Some(album) = album {
                let album_lower = album.to_lowercase();
                let has_matching_release = r
                    .releases
                    .iter()
                    .any(|rel| rel.title.to_lowercase().contains(&album_lower));
                if !has_matching_release && !r.releases.is_empty() {
                    return false;
                }
            }

            // If duration is specified, check it's within 10 seconds
            if let (Some(expected), Some(actual)) = (duration_ms, r.length)
                && expected.abs_diff(actual) > 10000
            {
                return false;
            }

            true
        });

        Ok(best)
    }
}

/// Escape special Lucene query characters.
fn escape_lucene(s: &str) -> String {
    let special = [
        '+', '-', '&', '|', '!', '(', ')', '{', '}', '[', ']', '^', '"', '~', '*', '?', ':', '\\',
        '/',
    ];
    let mut result = String::with_capacity(s.len() * 2);
    for c in s.chars() {
        if special.contains(&c) {
            result.push('\\');
        }
        result.push(c);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_lucene() {
        assert_eq!(escape_lucene("simple"), "simple");
        assert_eq!(escape_lucene("Hello: World"), "Hello\\: World");
        assert_eq!(escape_lucene("test (1)"), "test \\(1\\)");
        assert_eq!(escape_lucene("a+b-c"), "a\\+b\\-c");
    }
}
