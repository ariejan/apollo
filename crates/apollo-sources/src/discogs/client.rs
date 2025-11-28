//! [Discogs](https://discogs.com/) API client.

use crate::discogs::types::{Master, Pagination, Release, SearchResponse, SearchResult};
use crate::error::{SourceError, SourceResult};
use reqwest::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use std::fmt::Write;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, warn};

/// Discogs API base URL.
const API_BASE: &str = "https://api.discogs.com";

/// Minimum delay between requests.
/// Discogs allows 60 requests per minute for authenticated users.
/// We use 1.1 seconds to be safe and avoid rate limiting.
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(1100);

/// [Discogs](https://discogs.com/) API client with rate limiting.
///
/// The Discogs API provides access to a comprehensive database of music
/// releases, artists, and labels. This client supports searching and
/// looking up releases by ID.
///
/// # Authentication
///
/// The client requires a personal access token from Discogs. You can create
/// one at <https://www.discogs.com/settings/developers>.
///
/// # Rate Limiting
///
/// The API allows 60 requests per minute for authenticated users. This client
/// automatically enforces rate limiting to stay within these limits.
///
/// # Example
///
/// ```no_run
/// use apollo_sources::discogs::DiscogsClient;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = DiscogsClient::new("MyApp", "1.0", "your-token")?;
///
/// // Search for releases
/// let results = client.search_releases("Abbey Road", Some("Beatles"), 5).await?;
/// for result in results {
///     println!("{} ({})", result.title, result.year.unwrap_or_default());
/// }
/// # Ok(())
/// # }
/// ```
pub struct DiscogsClient {
    client: Client,
    last_request: Mutex<Instant>,
}

impl DiscogsClient {
    /// Create a new Discogs client.
    ///
    /// # Arguments
    ///
    /// * `app_name` - Name of your application
    /// * `app_version` - Version of your application
    /// * `token` - Discogs personal access token
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(app_name: &str, app_version: &str, token: &str) -> SourceResult<Self> {
        let user_agent = format!("{app_name}/{app_version}");

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&user_agent)
                .map_err(|e| SourceError::InvalidInput(e.to_string()))?,
        );
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Discogs token={token}"))
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

    /// Make a GET request to the API.
    async fn get<T: serde::de::DeserializeOwned>(&self, path: &str) -> SourceResult<T> {
        self.wait_for_rate_limit().await;

        let url = format!("{API_BASE}{path}");
        debug!("GET {url}");

        let response = self.client.get(&url).send().await?;
        let status = response.status();

        // Check rate limiting headers
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(60);

            warn!("Rate limited, retry after {retry_after} seconds");
            return Err(SourceError::RateLimited { retry_after });
        }

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
        serde_json::from_str(&body).map_err(|e| SourceError::Parse(e.to_string()))
    }

    /// Search for releases.
    ///
    /// # Arguments
    ///
    /// * `title` - The release title to search for
    /// * `artist` - The artist name to filter by (optional)
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
    ) -> SourceResult<Vec<SearchResult>> {
        let mut query = format!("release_title={}", urlencoding::encode(title));

        if let Some(artist) = artist {
            let _ = write!(query, "&artist={}", urlencoding::encode(artist));
        }

        let _ = write!(query, "&type=release&per_page={limit}");

        let path = format!("/database/search?{query}");
        let response: SearchResponse = self.get(&path).await?;
        Ok(response.results)
    }

    /// Search for master releases.
    ///
    /// Master releases represent the canonical version of an album,
    /// with individual releases (CD, vinyl, reissues) linked to them.
    ///
    /// # Arguments
    ///
    /// * `title` - The release title to search for
    /// * `artist` - The artist name to filter by (optional)
    /// * `limit` - Maximum number of results (1-100)
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn search_masters(
        &self,
        title: &str,
        artist: Option<&str>,
        limit: u32,
    ) -> SourceResult<Vec<SearchResult>> {
        let mut query = format!("release_title={}", urlencoding::encode(title));

        if let Some(artist) = artist {
            let _ = write!(query, "&artist={}", urlencoding::encode(artist));
        }

        let _ = write!(query, "&type=master&per_page={limit}");

        let path = format!("/database/search?{query}");
        let response: SearchResponse = self.get(&path).await?;
        Ok(response.results)
    }

    /// Search using a general query string.
    ///
    /// This searches across all fields and returns mixed results.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query
    /// * `limit` - Maximum number of results (1-100)
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn search(
        &self,
        query: &str,
        limit: u32,
    ) -> SourceResult<(Vec<SearchResult>, Pagination)> {
        let path = format!(
            "/database/search?q={}&per_page={limit}",
            urlencoding::encode(query)
        );
        let response: SearchResponse = self.get(&path).await?;
        Ok((response.results, response.pagination))
    }

    /// Look up a release by its Discogs ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The Discogs release ID
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the release is not found.
    pub async fn get_release(&self, id: u64) -> SourceResult<Release> {
        let path = format!("/releases/{id}");
        self.get(&path).await
    }

    /// Look up a master release by its Discogs ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The Discogs master release ID
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the master is not found.
    pub async fn get_master(&self, id: u64) -> SourceResult<Master> {
        let path = format!("/masters/{id}");
        self.get(&path).await
    }

    /// Search for a release by barcode.
    ///
    /// This is useful for matching physical media by scanning barcodes.
    ///
    /// # Arguments
    ///
    /// * `barcode` - The barcode to search for
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn search_by_barcode(&self, barcode: &str) -> SourceResult<Vec<SearchResult>> {
        let path = format!(
            "/database/search?barcode={}&type=release",
            urlencoding::encode(barcode)
        );
        let response: SearchResponse = self.get(&path).await?;
        Ok(response.results)
    }

    /// Search for a release by catalog number.
    ///
    /// # Arguments
    ///
    /// * `catno` - The catalog number to search for
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn search_by_catalog_number(&self, catno: &str) -> SourceResult<Vec<SearchResult>> {
        let path = format!(
            "/database/search?catno={}&type=release",
            urlencoding::encode(catno)
        );
        let response: SearchResponse = self.get(&path).await?;
        Ok(response.results)
    }

    /// Find the best matching release for the given metadata.
    ///
    /// This searches for releases and returns the best match based on
    /// title and artist similarity.
    ///
    /// # Arguments
    ///
    /// * `title` - The album title
    /// * `artist` - The artist name
    /// * `year` - The release year (optional, for filtering)
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails.
    pub async fn find_best_release(
        &self,
        title: &str,
        artist: &str,
        year: Option<i32>,
    ) -> SourceResult<Option<SearchResult>> {
        let results = self.search_releases(title, Some(artist), 10).await?;

        // Filter by year if specified
        let best = if let Some(expected_year) = year {
            results.into_iter().find(|r| {
                r.year
                    .as_ref()
                    .and_then(|y| y.parse::<i32>().ok())
                    .is_some_and(|y| (y - expected_year).abs() <= 1)
            })
        } else {
            results.into_iter().next()
        };

        Ok(best)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let result = DiscogsClient::new("TestApp", "1.0", "test-token");
        assert!(result.is_ok());
    }
}
