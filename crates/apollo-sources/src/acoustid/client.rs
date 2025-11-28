//! [AcoustID](https://acoustid.org/) API client.

use crate::acoustid::types::{AcoustIdResult, LookupResponse};
use crate::error::{SourceError, SourceResult};
use reqwest::Client;
use reqwest::header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// API base URL.
const API_BASE: &str = "https://api.acoustid.org/v2";

/// Minimum delay between requests (3 requests per second max).
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(350);

/// Client for the [AcoustID](https://acoustid.org/) API.
///
/// This client handles fingerprint lookups against the [AcoustID](https://acoustid.org/) database.
/// Results include [MusicBrainz](https://musicbrainz.org/) recording IDs that can be used to fetch
/// detailed metadata.
pub struct AcoustIdClient {
    /// HTTP client.
    client: Client,
    /// API key.
    api_key: String,
    /// Last request time for rate limiting.
    last_request: Mutex<Instant>,
}

impl AcoustIdClient {
    /// Create a new [AcoustID](https://acoustid.org/) client.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your API key (get one at <https://acoustid.org/new-application>)
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(api_key: impl Into<String>) -> SourceResult<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("Apollo/0.1 (https://github.com/yourusername/apollo)"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            api_key: api_key.into(),
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

    /// Look up a fingerprint in the [AcoustID](https://acoustid.org/) database.
    ///
    /// # Arguments
    ///
    /// * `fingerprint` - The audio fingerprint (from Chromaprint)
    /// * `duration` - Audio duration in seconds
    ///
    /// # Returns
    ///
    /// Returns a list of matching results, sorted by score.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn lookup(
        &self,
        fingerprint: &str,
        duration: u32,
    ) -> SourceResult<Vec<AcoustIdResult>> {
        self.lookup_with_meta(fingerprint, duration, &["recordings", "releasegroups"])
            .await
    }

    /// Look up a fingerprint with specific metadata to include.
    ///
    /// # Arguments
    ///
    /// * `fingerprint` - The audio fingerprint (from Chromaprint)
    /// * `duration` - Audio duration in seconds
    /// * `meta` - Metadata to include (e.g., "recordings", "releasegroups", "compress")
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn lookup_with_meta(
        &self,
        fingerprint: &str,
        duration: u32,
        meta: &[&str],
    ) -> SourceResult<Vec<AcoustIdResult>> {
        self.wait_for_rate_limit().await;

        let meta_str = meta.join("+");
        let url = format!(
            "{API_BASE}/lookup?client={}&duration={}&fingerprint={}&meta={}",
            urlencoding::encode(&self.api_key),
            duration,
            urlencoding::encode(fingerprint),
            urlencoding::encode(&meta_str)
        );

        debug!(
            "AcoustID lookup: duration={}s, fingerprint_len={}",
            duration,
            fingerprint.len()
        );

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

            warn!("AcoustID rate limited, retry after {retry_after} seconds");
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
        let lookup: LookupResponse =
            serde_json::from_str(&body).map_err(|e| SourceError::Parse(e.to_string()))?;

        if lookup.status != "ok" {
            if let Some(error) = lookup.error {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                return Err(SourceError::Api {
                    status: error.code as u16,
                    message: error.message,
                });
            }
            return Err(SourceError::Api {
                status: 0,
                message: "Unknown API error".to_string(),
            });
        }

        debug!("AcoustID found {} results", lookup.results.len());
        Ok(lookup.results)
    }

    /// Look up a fingerprint and return the best matching recording.
    ///
    /// # Arguments
    ///
    /// * `fingerprint` - The audio fingerprint
    /// * `duration` - Audio duration in seconds
    /// * `min_score` - Minimum score threshold (0.0 to 1.0)
    ///
    /// # Returns
    ///
    /// Returns the best matching recording if score is above threshold.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn find_best_match(
        &self,
        fingerprint: &str,
        duration: u32,
        min_score: f64,
    ) -> SourceResult<Option<crate::acoustid::types::Recording>> {
        let results = self.lookup(fingerprint, duration).await?;

        // Find the best result above threshold
        for result in results {
            if result.score >= min_score {
                // Return the first recording from the best result
                if let Some(recording) = result.recordings.into_iter().next() {
                    return Ok(Some(recording));
                }
            }
        }

        Ok(None)
    }

    /// Get all [MusicBrainz](https://musicbrainz.org/) recording IDs from a fingerprint lookup.
    ///
    /// # Arguments
    ///
    /// * `fingerprint` - The audio fingerprint
    /// * `duration` - Audio duration in seconds
    /// * `min_score` - Minimum score threshold (0.0 to 1.0)
    ///
    /// # Returns
    ///
    /// Returns a list of [MusicBrainz](https://musicbrainz.org/) recording IDs.
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn get_recording_ids(
        &self,
        fingerprint: &str,
        duration: u32,
        min_score: f64,
    ) -> SourceResult<Vec<String>> {
        let results = self.lookup(fingerprint, duration).await?;

        let mut recording_ids = Vec::new();
        for result in results {
            if result.score >= min_score {
                for recording in result.recordings {
                    if !recording_ids.contains(&recording.id) {
                        recording_ids.push(recording.id);
                    }
                }
            }
        }

        Ok(recording_ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = AcoustIdClient::new("test-api-key");
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_empty_key() {
        let client = AcoustIdClient::new("");
        // Should still create client (empty key will fail at request time)
        assert!(client.is_ok());
    }
}
