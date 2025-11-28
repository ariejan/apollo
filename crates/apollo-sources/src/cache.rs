//! Response caching for metadata sources.
//!
//! This module provides caching for API responses to reduce network requests
//! and comply with rate limits.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::RwLock;
use tracing::debug;

/// Default TTL for cache entries (1 hour).
const DEFAULT_TTL: Duration = Duration::from_secs(3600);

/// Maximum cache size (number of entries).
const DEFAULT_MAX_SIZE: usize = 10000;

/// A cached value with expiration time.
#[derive(Debug, Clone)]
struct CacheEntry<V> {
    /// The cached value.
    value: V,
    /// When this entry was created.
    created: Instant,
    /// Time-to-live for this entry.
    ttl: Duration,
}

impl<V> CacheEntry<V> {
    /// Check if this entry has expired.
    fn is_expired(&self) -> bool {
        self.created.elapsed() >= self.ttl
    }
}

/// A persistent cache entry for disk storage.
#[derive(Debug, Serialize, Deserialize)]
struct PersistentEntry<V> {
    /// The cached value.
    value: V,
    /// When this entry was created (Unix timestamp).
    created_at: u64,
    /// TTL in seconds.
    ttl_secs: u64,
}

impl<V> PersistentEntry<V> {
    /// Check if this entry has expired based on current time.
    fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();
        now >= self.created_at + self.ttl_secs
    }
}

/// Configuration for the response cache.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Time-to-live for cache entries.
    pub ttl: Duration,
    /// Maximum number of entries to keep in memory.
    pub max_size: usize,
    /// Optional path for persistent cache storage.
    pub persist_path: Option<std::path::PathBuf>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            ttl: DEFAULT_TTL,
            max_size: DEFAULT_MAX_SIZE,
            persist_path: None,
        }
    }
}

impl CacheConfig {
    /// Create a new configuration with default TTL.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the TTL for cache entries.
    #[must_use]
    pub const fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = ttl;
        self
    }

    /// Set the maximum cache size.
    #[must_use]
    pub const fn with_max_size(mut self, max_size: usize) -> Self {
        self.max_size = max_size;
        self
    }

    /// Set the path for persistent cache storage.
    #[must_use]
    pub fn with_persist_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.persist_path = Some(path.into());
        self
    }
}

/// In-memory response cache with optional disk persistence.
#[derive(Debug)]
pub struct ResponseCache<K, V> {
    /// In-memory cache storage.
    entries: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    /// Cache configuration.
    config: CacheConfig,
}

impl<K, V> ResponseCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Create a new cache with the given configuration.
    #[must_use]
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create a new cache with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Get a value from the cache.
    pub async fn get(&self, key: &K) -> Option<V> {
        let entries = self.entries.read().await;
        let result = entries.get(key).and_then(|entry| {
            if entry.is_expired() {
                debug!("Cache entry expired");
                None
            } else {
                debug!("Cache hit");
                Some(entry.value.clone())
            }
        });
        drop(entries);
        result
    }

    /// Insert a value into the cache.
    pub async fn insert(&self, key: K, value: V) {
        let mut entries = self.entries.write().await;

        // Evict expired entries and enforce size limit
        if entries.len() >= self.config.max_size {
            Self::evict_expired(&mut entries);

            // If still over limit, remove oldest entries
            let current_len = entries.len();
            if current_len >= self.config.max_size {
                let evict_count = current_len - self.config.max_size + 1;
                Self::evict_oldest(&mut entries, evict_count);
            }
        }

        entries.insert(
            key,
            CacheEntry {
                value,
                created: Instant::now(),
                ttl: self.config.ttl,
            },
        );
    }

    /// Insert a value with a custom TTL.
    pub async fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        let mut entries = self.entries.write().await;

        entries.insert(
            key,
            CacheEntry {
                value,
                created: Instant::now(),
                ttl,
            },
        );
    }

    /// Remove a value from the cache.
    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().await;
        entries.remove(key).map(|e| e.value)
    }

    /// Clear all entries from the cache.
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
    }

    /// Get the number of entries in the cache (including expired).
    pub async fn len(&self) -> usize {
        let entries = self.entries.read().await;
        entries.len()
    }

    /// Check if the cache is empty.
    pub async fn is_empty(&self) -> bool {
        let entries = self.entries.read().await;
        entries.is_empty()
    }

    /// Remove expired entries from the cache.
    pub async fn cleanup(&self) {
        let mut entries = self.entries.write().await;
        Self::evict_expired(&mut entries);
    }

    /// Evict expired entries.
    fn evict_expired(entries: &mut HashMap<K, CacheEntry<V>>) {
        let before = entries.len();
        entries.retain(|_, e| !e.is_expired());
        let evicted = before - entries.len();
        if evicted > 0 {
            debug!("Evicted {evicted} expired cache entries");
        }
    }

    /// Evict the oldest entries.
    fn evict_oldest(entries: &mut HashMap<K, CacheEntry<V>>, count: usize) {
        // Find and remove the oldest entries
        let mut ages: Vec<(K, Duration)> = entries
            .iter()
            .map(|(k, e)| (k.clone(), e.created.elapsed()))
            .collect();
        ages.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by age descending

        for (key, _) in ages.into_iter().take(count) {
            entries.remove(&key);
        }
        debug!("Evicted {count} oldest cache entries");
    }
}

impl<K, V> ResponseCache<K, V>
where
    K: Eq + Hash + Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
    V: Clone + Send + Sync + Serialize + for<'de> Deserialize<'de> + 'static,
{
    /// Load the cache from disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub async fn load_from_disk(&self) -> Result<(), std::io::Error> {
        let Some(path) = &self.config.persist_path else {
            return Ok(());
        };

        if !path.exists() {
            return Ok(());
        }

        let content = tokio::fs::read_to_string(path).await?;
        let persistent: HashMap<K, PersistentEntry<V>> = serde_json::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut entries = self.entries.write().await;
        for (key, pentry) in persistent {
            if !pentry.is_expired() {
                entries.insert(
                    key,
                    CacheEntry {
                        value: pentry.value,
                        created: Instant::now(), // Use current time since we can't serialize Instant
                        ttl: Duration::from_secs(pentry.ttl_secs),
                    },
                );
            }
        }
        let loaded_count = entries.len();
        drop(entries);

        debug!("Loaded {loaded_count} cache entries from disk");
        Ok(())
    }

    /// Save the cache to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub async fn save_to_disk(&self) -> Result<(), std::io::Error> {
        let Some(path) = &self.config.persist_path else {
            return Ok(());
        };

        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs();

        let persistent: HashMap<K, PersistentEntry<V>> = {
            let entries = self.entries.read().await;
            entries
                .iter()
                .filter(|(_, e)| !e.is_expired())
                .map(|(k, e)| {
                    let remaining_ttl = e.ttl.saturating_sub(e.created.elapsed());
                    (
                        k.clone(),
                        PersistentEntry {
                            value: e.value.clone(),
                            created_at: now,
                            ttl_secs: remaining_ttl.as_secs(),
                        },
                    )
                })
                .collect()
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let content = serde_json::to_string_pretty(&persistent)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        tokio::fs::write(path, content).await?;

        debug!("Saved {} cache entries to disk", persistent.len());
        Ok(())
    }
}

/// A cache key for [MusicBrainz](https://musicbrainz.org/) recording searches.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordingSearchKey {
    /// Track title.
    pub title: String,
    /// Artist name.
    pub artist: Option<String>,
    /// Result limit.
    pub limit: u32,
}

/// A cache key for [MusicBrainz](https://musicbrainz.org/) release searches.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReleaseSearchKey {
    /// Album title.
    pub title: String,
    /// Artist name.
    pub artist: Option<String>,
    /// Result limit.
    pub limit: u32,
}

/// A cache key for [MusicBrainz](https://musicbrainz.org/) lookups by MBID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LookupKey {
    /// The MBID.
    pub mbid: String,
    /// Include relationships (comma-separated).
    pub include: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_insert_and_get() {
        let cache: ResponseCache<String, String> = ResponseCache::with_defaults();

        cache.insert("key1".to_string(), "value1".to_string()).await;

        assert_eq!(
            cache.get(&"key1".to_string()).await,
            Some("value1".to_string())
        );
        assert_eq!(cache.get(&"key2".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let config = CacheConfig::new().with_ttl(Duration::from_millis(50));
        let cache: ResponseCache<String, String> = ResponseCache::new(config);

        cache.insert("key1".to_string(), "value1".to_string()).await;

        // Value should be present immediately
        assert_eq!(
            cache.get(&"key1".to_string()).await,
            Some("value1".to_string())
        );

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Value should be expired
        assert_eq!(cache.get(&"key1".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_custom_ttl() {
        let cache: ResponseCache<String, String> = ResponseCache::with_defaults();

        cache
            .insert_with_ttl(
                "key1".to_string(),
                "value1".to_string(),
                Duration::from_millis(50),
            )
            .await;

        assert_eq!(
            cache.get(&"key1".to_string()).await,
            Some("value1".to_string())
        );

        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(cache.get(&"key1".to_string()).await, None);
    }

    #[tokio::test]
    async fn test_cache_remove() {
        let cache: ResponseCache<String, String> = ResponseCache::with_defaults();

        cache.insert("key1".to_string(), "value1".to_string()).await;
        assert!(cache.get(&"key1".to_string()).await.is_some());

        let removed = cache.remove(&"key1".to_string()).await;
        assert_eq!(removed, Some("value1".to_string()));
        assert!(cache.get(&"key1".to_string()).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache: ResponseCache<String, String> = ResponseCache::with_defaults();

        cache.insert("key1".to_string(), "value1".to_string()).await;
        cache.insert("key2".to_string(), "value2".to_string()).await;

        assert_eq!(cache.len().await, 2);

        cache.clear().await;

        assert!(cache.is_empty().await);
    }

    #[tokio::test]
    async fn test_cache_max_size() {
        let config = CacheConfig::new().with_max_size(2);
        let cache: ResponseCache<String, String> = ResponseCache::new(config);

        cache.insert("key1".to_string(), "value1".to_string()).await;
        cache.insert("key2".to_string(), "value2".to_string()).await;
        cache.insert("key3".to_string(), "value3".to_string()).await;

        // One entry should have been evicted
        assert!(cache.len().await <= 2);
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let config = CacheConfig::new().with_ttl(Duration::from_millis(50));
        let cache: ResponseCache<String, String> = ResponseCache::new(config);

        cache.insert("key1".to_string(), "value1".to_string()).await;
        cache.insert("key2".to_string(), "value2".to_string()).await;

        assert_eq!(cache.len().await, 2);

        tokio::time::sleep(Duration::from_millis(100)).await;

        cache.cleanup().await;

        assert!(cache.is_empty().await);
    }

    #[tokio::test]
    async fn test_recording_search_key() {
        let key1 = RecordingSearchKey {
            title: "Yesterday".to_string(),
            artist: Some("Beatles".to_string()),
            limit: 10,
        };
        let key2 = RecordingSearchKey {
            title: "Yesterday".to_string(),
            artist: Some("Beatles".to_string()),
            limit: 10,
        };
        let key3 = RecordingSearchKey {
            title: "Yesterday".to_string(),
            artist: None,
            limit: 10,
        };

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}
