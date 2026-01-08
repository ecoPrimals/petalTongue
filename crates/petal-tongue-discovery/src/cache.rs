//! Provider caching layer
//!
//! Implements intelligent caching to reduce API calls and improve performance.
//!
//! NOTE: This module is currently complete but unused. It will be integrated
//! when performance optimization becomes a priority.

#![allow(dead_code)] // Entire module is reserved for future use

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Cached entry with expiration
#[derive(Debug, Clone)]
#[allow(dead_code)] // Reserved for future use when caching is enabled
struct CachedEntry<T> {
    data: T,
    expires_at: Instant,
}

impl<T> CachedEntry<T> {
    #[allow(dead_code)] // Reserved for future use when caching is enabled
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now() + ttl,
        }
    }

    #[allow(dead_code)] // Reserved for future use when caching is enabled
    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

/// Cache key for different data types
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
#[allow(dead_code)] // Reserved for future use when caching is enabled
pub(crate) enum CacheKey {
    Primals,
    Topology,
    Health,
}

/// Provider cache with TTL support
///
/// Uses LRU eviction when cache is full.
#[derive(Debug)]
#[allow(dead_code)] // Reserved for future use when caching is enabled
pub struct ProviderCache<T> {
    cache: Arc<Mutex<LruCache<CacheKey, CachedEntry<T>>>>,
    primals_ttl: Duration,
    topology_ttl: Duration,
    health_ttl: Duration,
    // Statistics
    hits: Arc<Mutex<u64>>,
    misses: Arc<Mutex<u64>>,
}

impl<T: Clone> ProviderCache<T> {
    /// Create a new cache with specified capacity
    #[allow(dead_code)] // Reserved for future use when caching is enabled
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap(),
            ))),
            primals_ttl: Duration::from_secs(30),
            topology_ttl: Duration::from_secs(60),
            health_ttl: Duration::from_secs(10),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }

    /// Create cache with custom TTLs
    pub fn with_ttls(
        capacity: usize,
        primals_ttl: Duration,
        topology_ttl: Duration,
        health_ttl: Duration,
    ) -> Self {
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(
                NonZeroUsize::new(capacity).unwrap(),
            ))),
            primals_ttl,
            topology_ttl,
            health_ttl,
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        }
    }

    /// Get primals from cache
    pub fn get_primals(&self) -> Option<T> {
        self.get(CacheKey::Primals)
    }

    /// Put primals in cache
    pub fn put_primals(&self, data: T) {
        self.put(CacheKey::Primals, data, self.primals_ttl);
    }

    /// Get topology from cache
    pub fn get_topology(&self) -> Option<T> {
        self.get(CacheKey::Topology)
    }

    /// Put topology in cache
    pub fn put_topology(&self, data: T) {
        self.put(CacheKey::Topology, data, self.topology_ttl);
    }

    /// Get health from cache
    pub fn get_health(&self) -> Option<T> {
        self.get(CacheKey::Health)
    }

    /// Put health in cache
    pub fn put_health(&self, data: T) {
        self.put(CacheKey::Health, data, self.health_ttl);
    }

    /// Get from cache (generic)
    fn get(&self, key: CacheKey) -> Option<T> {
        let mut cache = self.cache.lock().unwrap();

        if let Some(entry) = cache.get(&key) {
            if entry.is_expired() {
                // Entry expired, remove it
                cache.pop(&key);
                *self.misses.lock().unwrap() += 1;
                tracing::debug!("Cache MISS (expired): {:?}", key);
                None
            } else {
                // Cache hit!
                *self.hits.lock().unwrap() += 1;
                tracing::debug!("Cache HIT: {:?}", key);
                Some(entry.data.clone())
            }
        } else {
            // Cache miss
            *self.misses.lock().unwrap() += 1;
            tracing::debug!("Cache MISS (not found): {:?}", key);
            None
        }
    }

    /// Put into cache (generic)
    fn put(&self, key: CacheKey, data: T, ttl: Duration) {
        let mut cache = self.cache.lock().unwrap();
        cache.put(key, CachedEntry::new(data, ttl));
    }

    /// Invalidate all cache entries
    pub fn invalidate_all(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        tracing::info!("Cache invalidated (all entries cleared)");
    }

    /// Invalidate specific key
    pub(crate) fn invalidate(&self, key: CacheKey) {
        let mut cache = self.cache.lock().unwrap();
        cache.pop(&key);
        tracing::debug!("Cache invalidated: {:?}", key);
    }

    /// Invalidate primals
    pub fn invalidate_primals(&self) {
        self.invalidate(CacheKey::Primals);
    }

    /// Invalidate topology
    pub fn invalidate_topology(&self) {
        self.invalidate(CacheKey::Topology);
    }

    /// Invalidate health
    pub fn invalidate_health(&self) {
        self.invalidate(CacheKey::Health);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = *self.hits.lock().unwrap();
        let misses = *self.misses.lock().unwrap();
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        CacheStats {
            hits,
            misses,
            total,
            hit_rate,
        }
    }

    /// Reset statistics
    pub fn reset_stats(&self) {
        *self.hits.lock().unwrap() = 0;
        *self.misses.lock().unwrap() = 0;
        tracing::debug!("Cache statistics reset");
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub total: u64,
    pub hit_rate: f64,
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Cache Stats: {} hits, {} misses, {:.1}% hit rate (total: {})",
            self.hits, self.misses, self.hit_rate, self.total
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_creation() {
        let cache: ProviderCache<Vec<String>> = ProviderCache::new(10);
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    fn test_cache_put_get() {
        let cache = ProviderCache::new(10);
        let data = vec!["test".to_string()];

        cache.put_primals(data.clone());
        let retrieved = cache.get_primals();

        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), data);
    }

    #[test]
    fn test_cache_miss() {
        let cache: ProviderCache<Vec<String>> = ProviderCache::new(10);
        let retrieved = cache.get_primals();
        assert!(retrieved.is_none());

        let stats = cache.stats();
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_hit() {
        let cache = ProviderCache::new(10);
        cache.put_primals(vec!["test".to_string()]);

        let _ = cache.get_primals();
        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
    }

    #[test]
    fn test_cache_expiration() {
        let cache = ProviderCache::with_ttls(
            10,
            Duration::from_millis(50), // Very short TTL for testing
            Duration::from_secs(60),
            Duration::from_secs(10),
        );

        cache.put_primals(vec!["test".to_string()]);

        // Should hit immediately
        assert!(cache.get_primals().is_some());

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(100));

        // Should miss after expiration
        assert!(cache.get_primals().is_none());
    }

    #[test]
    fn test_cache_invalidation() {
        let cache = ProviderCache::new(10);
        cache.put_primals(vec!["test".to_string()]);

        assert!(cache.get_primals().is_some());

        cache.invalidate_primals();

        assert!(cache.get_primals().is_none());
    }

    #[test]
    fn test_cache_statistics() {
        let cache = ProviderCache::new(10);
        cache.put_primals(vec!["test".to_string()]);

        let _ = cache.get_primals(); // Hit
        let _ = cache.get_topology(); // Miss
        let _ = cache.get_primals(); // Hit

        let stats = cache.stats();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total, 3);
        assert!((stats.hit_rate - 66.666).abs() < 0.1);
    }

    #[test]
    fn test_cache_reset_stats() {
        let cache = ProviderCache::new(10);
        cache.put_primals(vec!["test".to_string()]);

        let _ = cache.get_primals();
        assert_eq!(cache.stats().hits, 1);

        cache.reset_stats();
        assert_eq!(cache.stats().hits, 0);
    }

    #[test]
    fn test_multiple_key_types() {
        let cache = ProviderCache::new(10);

        cache.put_primals(vec!["primals".to_string()]);
        cache.put_topology(vec!["topology".to_string()]);
        cache.put_health(vec!["health".to_string()]);

        assert_eq!(cache.get_primals().unwrap()[0], "primals");
        assert_eq!(cache.get_topology().unwrap()[0], "topology");
        assert_eq!(cache.get_health().unwrap()[0], "health");
    }

    #[test]
    fn test_lru_eviction() {
        let cache = ProviderCache::new(2); // Small capacity

        cache.put(
            CacheKey::Primals,
            vec!["1".to_string()],
            Duration::from_secs(60),
        );
        cache.put(
            CacheKey::Topology,
            vec!["2".to_string()],
            Duration::from_secs(60),
        );
        cache.put(
            CacheKey::Health,
            vec!["3".to_string()],
            Duration::from_secs(60),
        );

        // Primals should be evicted (LRU)
        assert!(cache.get_primals().is_none());
        assert!(cache.get_topology().is_some());
        assert!(cache.get_health().is_some());
    }
}
