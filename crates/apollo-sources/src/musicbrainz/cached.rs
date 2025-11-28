//! Cached [MusicBrainz](https://musicbrainz.org/) API client.

use crate::cache::{CacheConfig, LookupKey, RecordingSearchKey, ReleaseSearchKey, ResponseCache};
use crate::error::SourceResult;
use crate::musicbrainz::client::MusicBrainzClient;
use crate::musicbrainz::types::{Recording, Release};
use tracing::debug;

/// A caching wrapper around [`MusicBrainzClient`].
///
/// This client caches API responses to reduce network requests and comply with rate limits.
/// Cache entries expire after the configured TTL.
///
/// # Example
///
/// ```no_run
/// use apollo_sources::musicbrainz::CachedMusicBrainzClient;
/// use apollo_sources::cache::CacheConfig;
/// use std::time::Duration;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = CacheConfig::new()
///     .with_ttl(Duration::from_secs(3600))
///     .with_persist_path("cache/musicbrainz.json");
///
/// let client = CachedMusicBrainzClient::new("MyApp", "1.0", "contact@example.com", config)?;
///
/// // First call hits the API
/// let recordings = client.search_recordings("Yesterday", Some("Beatles"), 5).await?;
///
/// // Second call uses cache
/// let recordings = client.search_recordings("Yesterday", Some("Beatles"), 5).await?;
/// # Ok(())
/// # }
/// ```
pub struct CachedMusicBrainzClient {
    /// The underlying client.
    inner: MusicBrainzClient,
    /// Cache for recording searches.
    recording_search_cache: ResponseCache<RecordingSearchKey, Vec<Recording>>,
    /// Cache for release searches.
    release_search_cache: ResponseCache<ReleaseSearchKey, Vec<Release>>,
    /// Cache for recording lookups.
    recording_lookup_cache: ResponseCache<LookupKey, Recording>,
    /// Cache for release lookups.
    release_lookup_cache: ResponseCache<LookupKey, Release>,
}

impl CachedMusicBrainzClient {
    /// Create a new cached client.
    ///
    /// # Arguments
    ///
    /// * `app_name` - Name of your application
    /// * `app_version` - Version of your application
    /// * `contact` - Contact email or URL
    /// * `cache_config` - Cache configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(
        app_name: &str,
        app_version: &str,
        contact: &str,
        cache_config: CacheConfig,
    ) -> SourceResult<Self> {
        Ok(Self {
            inner: MusicBrainzClient::new(app_name, app_version, contact)?,
            recording_search_cache: ResponseCache::new(cache_config.clone()),
            release_search_cache: ResponseCache::new(cache_config.clone()),
            recording_lookup_cache: ResponseCache::new(cache_config.clone()),
            release_lookup_cache: ResponseCache::new(cache_config),
        })
    }

    /// Create a new cached client with default cache configuration.
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
    pub fn with_defaults(app_name: &str, app_version: &str, contact: &str) -> SourceResult<Self> {
        Self::new(app_name, app_version, contact, CacheConfig::default())
    }

    /// Load all caches from disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache files cannot be read.
    pub async fn load_cache(&self) -> Result<(), std::io::Error> {
        self.recording_search_cache.load_from_disk().await?;
        self.release_search_cache.load_from_disk().await?;
        self.recording_lookup_cache.load_from_disk().await?;
        self.release_lookup_cache.load_from_disk().await?;
        Ok(())
    }

    /// Save all caches to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the cache files cannot be written.
    pub async fn save_cache(&self) -> Result<(), std::io::Error> {
        self.recording_search_cache.save_to_disk().await?;
        self.release_search_cache.save_to_disk().await?;
        self.recording_lookup_cache.save_to_disk().await?;
        self.release_lookup_cache.save_to_disk().await?;
        Ok(())
    }

    /// Clear all caches.
    pub async fn clear_cache(&self) {
        self.recording_search_cache.clear().await;
        self.release_search_cache.clear().await;
        self.recording_lookup_cache.clear().await;
        self.release_lookup_cache.clear().await;
    }

    /// Get cache statistics.
    pub async fn cache_stats(&self) -> CacheStats {
        CacheStats {
            recording_searches: self.recording_search_cache.len().await,
            release_searches: self.release_search_cache.len().await,
            recording_lookups: self.recording_lookup_cache.len().await,
            release_lookups: self.release_lookup_cache.len().await,
        }
    }

    /// Search for recordings (songs) by title and artist.
    ///
    /// Results are cached for the configured TTL.
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
        let key = RecordingSearchKey {
            title: title.to_string(),
            artist: artist.map(ToString::to_string),
            limit,
        };

        // Check cache first
        if let Some(cached) = self.recording_search_cache.get(&key).await {
            debug!("Cache hit for recording search: {title}");
            return Ok(cached);
        }

        // Fetch from API
        debug!("Cache miss for recording search: {title}");
        let results = self.inner.search_recordings(title, artist, limit).await?;

        // Store in cache
        self.recording_search_cache
            .insert(key, results.clone())
            .await;

        Ok(results)
    }

    /// Search for releases (albums) by title and artist.
    ///
    /// Results are cached for the configured TTL.
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
        let key = ReleaseSearchKey {
            title: title.to_string(),
            artist: artist.map(ToString::to_string),
            limit,
        };

        // Check cache first
        if let Some(cached) = self.release_search_cache.get(&key).await {
            debug!("Cache hit for release search: {title}");
            return Ok(cached);
        }

        // Fetch from API
        debug!("Cache miss for release search: {title}");
        let results = self.inner.search_releases(title, artist, limit).await?;

        // Store in cache
        self.release_search_cache.insert(key, results.clone()).await;

        Ok(results)
    }

    /// Look up a recording by its MBID.
    ///
    /// Results are cached for the configured TTL.
    ///
    /// # Arguments
    ///
    /// * `mbid` - The MBID of the recording
    /// * `include` - Optional list of related entities to include
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the recording is not found.
    pub async fn lookup_recording(&self, mbid: &str, include: &[&str]) -> SourceResult<Recording> {
        let key = LookupKey {
            mbid: mbid.to_string(),
            include: include.join(","),
        };

        // Check cache first
        if let Some(cached) = self.recording_lookup_cache.get(&key).await {
            debug!("Cache hit for recording lookup: {mbid}");
            return Ok(cached);
        }

        // Fetch from API
        debug!("Cache miss for recording lookup: {mbid}");
        let result = self.inner.lookup_recording(mbid, include).await?;

        // Store in cache
        self.recording_lookup_cache
            .insert(key, result.clone())
            .await;

        Ok(result)
    }

    /// Look up a release by its MBID.
    ///
    /// Results are cached for the configured TTL.
    ///
    /// # Arguments
    ///
    /// * `mbid` - The MBID of the release
    /// * `include` - Optional list of related entities to include
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails or the release is not found.
    pub async fn lookup_release(&self, mbid: &str, include: &[&str]) -> SourceResult<Release> {
        let key = LookupKey {
            mbid: mbid.to_string(),
            include: include.join(","),
        };

        // Check cache first
        if let Some(cached) = self.release_lookup_cache.get(&key).await {
            debug!("Cache hit for release lookup: {mbid}");
            return Ok(cached);
        }

        // Fetch from API
        debug!("Cache miss for release lookup: {mbid}");
        let result = self.inner.lookup_release(mbid, include).await?;

        // Store in cache
        self.release_lookup_cache.insert(key, result.clone()).await;

        Ok(result)
    }

    /// Search for a recording that best matches the given metadata.
    ///
    /// This uses the underlying client's `find_best_recording` method
    /// but benefits from search caching.
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
        // Use cached search
        let recordings = self.search_recordings(title, Some(artist), 10).await?;

        // Apply the same matching logic as the inner client
        let best = recordings.into_iter().find(|r| {
            let score = r.score.unwrap_or(0);
            if score < min_score {
                return false;
            }

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

/// Statistics about cache usage.
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Number of cached recording search results.
    pub recording_searches: usize,
    /// Number of cached release search results.
    pub release_searches: usize,
    /// Number of cached recording lookups.
    pub recording_lookups: usize,
    /// Number of cached release lookups.
    pub release_lookups: usize,
}

impl CacheStats {
    /// Total number of cached entries.
    #[must_use]
    pub const fn total(&self) -> usize {
        self.recording_searches
            + self.release_searches
            + self.recording_lookups
            + self.release_lookups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_cached_client_creation() {
        let config = CacheConfig::new().with_ttl(Duration::from_secs(60));
        let client = CachedMusicBrainzClient::new("TestApp", "0.1", "test@example.com", config);
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let client = CachedMusicBrainzClient::with_defaults("TestApp", "0.1", "test@example.com")
            .expect("client creation should succeed");

        let stats = client.cache_stats().await;
        assert_eq!(stats.total(), 0);
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let client = CachedMusicBrainzClient::with_defaults("TestApp", "0.1", "test@example.com")
            .expect("client creation should succeed");

        client.clear_cache().await;
        let stats = client.cache_stats().await;
        assert_eq!(stats.total(), 0);
    }
}
