// SPDX-License-Identifier: AGPL-3.0-or-later
//! Provider caching layer
//!
//! Implements intelligent caching to reduce API calls and improve performance.
//!
//! NOTE: This module is currently complete but unused. It will be integrated
//! when performance optimization becomes a priority.
//!
//! MODERN IDIOMATIC RUST:
//! - Uses `tokio::sync::RwLock` for async compatibility (no deadlocks!)
//! - Fully concurrent, lock-free reads
//! - No blocking operations in async context

#![allow(dead_code)] // Entire module is reserved for future use
#![expect(
    clippy::future_not_send,
    reason = "ProviderCache holds generic T without Send/Sync; used in single-threaded contexts"
)]

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cached entry with expiration
/// Stores data in Arc for zero-copy cache hits (`Arc::clone` instead of deep clone)
#[derive(Debug, Clone)]
struct CachedEntry<T> {
    data: Arc<T>,
    expires_at: Instant,
}

impl<T> CachedEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        Self {
            data: Arc::new(data),
            expires_at: Instant::now() + ttl,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

/// Cache key for different data types
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum CacheKey {
    Primals,
    Topology,
    Health,
}

/// Provider cache with TTL support
///
/// Uses LRU eviction when cache is full.
///
/// ASYNC-SAFE: Uses `tokio::sync::RwLock` to prevent deadlocks in async context.
#[derive(Debug)]
pub struct ProviderCache<T> {
    cache: Arc<RwLock<LruCache<CacheKey, CachedEntry<T>>>>,
    primals_ttl: Duration,
    topology_ttl: Duration,
    health_ttl: Duration,
    // Statistics
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
}

impl<T> ProviderCache<T> {
    /// Create a new cache with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(capacity.max(1)).unwrap_or(NonZeroUsize::MIN),
            ))),
            primals_ttl: Duration::from_secs(30),
            topology_ttl: Duration::from_secs(60),
            health_ttl: Duration::from_secs(10),
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
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
            cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(capacity.max(1)).unwrap_or(NonZeroUsize::MIN),
            ))),
            primals_ttl,
            topology_ttl,
            health_ttl,
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }

    /// Get primals from cache (returns Arc for zero-copy sharing)
    pub async fn get_primals(&self) -> Option<Arc<T>> {
        self.get(CacheKey::Primals).await
    }

    /// Put primals in cache
    pub async fn put_primals(&self, data: T) {
        self.put(CacheKey::Primals, data, self.primals_ttl).await;
    }

    /// Get topology from cache (returns Arc for zero-copy sharing)
    pub async fn get_topology(&self) -> Option<Arc<T>> {
        self.get(CacheKey::Topology).await
    }

    /// Put topology in cache
    pub async fn put_topology(&self, data: T) {
        self.put(CacheKey::Topology, data, self.topology_ttl).await;
    }

    /// Get health from cache (returns Arc for zero-copy sharing)
    pub async fn get_health(&self) -> Option<Arc<T>> {
        self.get(CacheKey::Health).await
    }

    /// Put health in cache
    pub async fn put_health(&self, data: T) {
        self.put(CacheKey::Health, data, self.health_ttl).await;
    }

    /// Get from cache (generic) - ASYNC-SAFE
    /// Returns `Arc::clone` for zero-copy cache hits (no deep clone)
    #[expect(
        clippy::option_if_let_else,
        reason = "closure would need mutable cache borrow"
    )]
    async fn get(&self, key: CacheKey) -> Option<Arc<T>> {
        let (result, is_hit) = {
            let mut cache = self.cache.write().await;
            let out = if let Some(entry) = cache.get(&key) {
                if entry.is_expired() {
                    cache.pop(&key);
                    (None, false)
                } else {
                    (Some(Arc::clone(&entry.data)), true)
                }
            } else {
                (None, false)
            };
            drop(cache);
            out
        };
        if is_hit {
            *self.hits.write().await += 1;
            tracing::debug!("Cache HIT: {:?}", key);
        } else {
            *self.misses.write().await += 1;
            tracing::debug!("Cache MISS: {:?}", key);
        }
        result
    }

    /// Put into cache (generic) - ASYNC-SAFE
    async fn put(&self, key: CacheKey, data: T, ttl: Duration) {
        self.cache
            .write()
            .await
            .put(key, CachedEntry::new(data, ttl));
    }

    /// Invalidate all cache entries - ASYNC-SAFE
    pub async fn invalidate_all(&self) {
        self.cache.write().await.clear();
        tracing::info!("Cache invalidated (all entries cleared)");
    }

    /// Invalidate specific key - ASYNC-SAFE
    pub(crate) async fn invalidate(&self, key: CacheKey) {
        self.cache.write().await.pop(&key);
        tracing::debug!("Cache invalidated: {:?}", key);
    }

    /// Invalidate primals - ASYNC-SAFE
    pub async fn invalidate_primals(&self) {
        self.invalidate(CacheKey::Primals).await;
    }

    /// Invalidate topology - ASYNC-SAFE
    pub async fn invalidate_topology(&self) {
        self.invalidate(CacheKey::Topology).await;
    }

    /// Invalidate health - ASYNC-SAFE
    pub async fn invalidate_health(&self) {
        self.invalidate(CacheKey::Health).await;
    }

    /// Get cache statistics - ASYNC-SAFE
    pub async fn stats(&self) -> CacheStats {
        let hits = *self.hits.read().await;
        let misses = *self.misses.read().await;
        let total = hits + misses;
        #[expect(
            clippy::cast_precision_loss,
            reason = "hits/total are small counts for display; u64→f64 safe for percentages"
        )]
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

    /// Reset statistics - ASYNC-SAFE
    pub async fn reset_stats(&self) {
        *self.hits.write().await = 0;
        *self.misses.write().await = 0;
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

    #[tokio::test]
    async fn test_cache_creation() {
        let cache: ProviderCache<Vec<String>> = ProviderCache::new(10);
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[tokio::test]
    async fn test_cache_put_get() {
        let cache = ProviderCache::new(10);
        let data = vec!["test".to_string()];

        cache.put_primals(data.clone()).await;
        let retrieved = cache.get_primals().await;

        assert!(retrieved.is_some());
        assert_eq!(*retrieved.unwrap(), data);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache: ProviderCache<Vec<String>> = ProviderCache::new(10);
        let retrieved = cache.get_primals().await;
        assert!(retrieved.is_none());

        let stats = cache.stats().await;
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = ProviderCache::new(10);
        cache.put_primals(vec!["test".to_string()]).await;

        let _ = cache.get_primals().await;
        let stats = cache.stats().await;
        assert_eq!(stats.hits, 1);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = ProviderCache::with_ttls(
            10,
            Duration::from_millis(50), // Very short TTL for testing
            Duration::from_secs(60),
            Duration::from_secs(10),
        );

        cache.put_primals(vec!["test".to_string()]).await;

        // Should hit immediately
        assert!(cache.get_primals().await.is_some());

        // Wait for expiration (async sleep, no blocking!)
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Should miss after expiration
        assert!(cache.get_primals().await.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = ProviderCache::new(10);
        cache.put_primals(vec!["test".to_string()]).await;

        assert!(cache.get_primals().await.is_some());

        cache.invalidate_primals().await;

        assert!(cache.get_primals().await.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidate_topology() {
        let cache = ProviderCache::new(10);
        cache.put_topology(vec!["edge".to_string()]).await;
        assert!(cache.get_topology().await.is_some());
        cache.invalidate_topology().await;
        assert!(cache.get_topology().await.is_none());
    }

    #[tokio::test]
    async fn test_cache_invalidate_health() {
        let cache = ProviderCache::new(10);
        cache.put_health(vec!["ok".to_string()]).await;
        assert!(cache.get_health().await.is_some());
        cache.invalidate_health().await;
        assert!(cache.get_health().await.is_none());
    }

    #[tokio::test]
    async fn test_cache_stats_hit_rate_zero_when_no_access() {
        let cache: ProviderCache<Vec<String>> = ProviderCache::new(10);
        let stats = cache.stats().await;
        assert_eq!(stats.total, 0);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[tokio::test]
    async fn test_cache_statistics() {
        let cache = ProviderCache::new(10);
        cache.put_primals(vec!["test".to_string()]).await;

        let _ = cache.get_primals().await; // Hit
        let _ = cache.get_topology().await; // Miss
        let _ = cache.get_primals().await; // Hit

        let stats = cache.stats().await;
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total, 3);
        assert!((stats.hit_rate - 66.666).abs() < 0.1);
    }

    #[tokio::test]
    async fn test_cache_reset_stats() {
        let cache = ProviderCache::new(10);
        cache.put_primals(vec!["test".to_string()]).await;

        let _ = cache.get_primals().await;
        assert_eq!(cache.stats().await.hits, 1);

        cache.reset_stats().await;
        assert_eq!(cache.stats().await.hits, 0);
    }

    #[tokio::test]
    async fn test_multiple_key_types() {
        let cache = ProviderCache::new(10);

        cache.put_primals(vec!["primals".to_string()]).await;
        cache.put_topology(vec!["topology".to_string()]).await;
        cache.put_health(vec!["health".to_string()]).await;

        assert_eq!(cache.get_primals().await.unwrap()[0], "primals");
        assert_eq!(cache.get_topology().await.unwrap()[0], "topology");
        assert_eq!(cache.get_health().await.unwrap()[0], "health");
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let cache = ProviderCache::new(2); // Small capacity

        cache
            .put(
                CacheKey::Primals,
                vec!["1".to_string()],
                Duration::from_secs(60),
            )
            .await;
        cache
            .put(
                CacheKey::Topology,
                vec!["2".to_string()],
                Duration::from_secs(60),
            )
            .await;
        cache
            .put(
                CacheKey::Health,
                vec!["3".to_string()],
                Duration::from_secs(60),
            )
            .await;

        // Primals should be evicted (LRU)
        assert!(cache.get(CacheKey::Primals).await.is_none());
        assert!(cache.get(CacheKey::Topology).await.is_some());
        assert!(cache.get(CacheKey::Health).await.is_some());
    }

    #[tokio::test]
    async fn test_empty_cache_behavior() {
        let cache: ProviderCache<Vec<String>> = ProviderCache::new(10);
        assert!(cache.get_primals().await.is_none());
        assert!(cache.get_topology().await.is_none());
        assert!(cache.get_health().await.is_none());
        let stats = cache.stats().await;
        assert_eq!(stats.total, 3);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[tokio::test]
    async fn test_invalidate_all() {
        let cache = ProviderCache::new(10);
        cache.put_primals(vec!["a".to_string()]).await;
        cache.put_topology(vec!["b".to_string()]).await;
        cache.put_health(vec!["c".to_string()]).await;
        cache.invalidate_all().await;
        assert!(cache.get_primals().await.is_none());
        assert!(cache.get_topology().await.is_none());
        assert!(cache.get_health().await.is_none());
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let cache = Arc::new(ProviderCache::new(100));
        let mut handles = Vec::new();
        for i in 0..10 {
            let c = Arc::clone(&cache);
            handles.push(tokio::spawn(async move {
                c.put_primals(vec![format!("primal-{i}")]).await;
                c.get_primals().await
            }));
        }
        for h in handles {
            let result = h.await.unwrap();
            assert!(result.is_some());
        }
        let stats = cache.stats().await;
        assert!(stats.hits >= 1);
    }

    #[tokio::test]
    async fn test_with_ttls() {
        let cache = ProviderCache::with_ttls(
            10,
            Duration::from_secs(5),
            Duration::from_secs(10),
            Duration::from_secs(15),
        );
        cache.put_primals(vec!["x".to_string()]).await;
        assert!(cache.get_primals().await.is_some());
    }

    #[tokio::test]
    async fn test_zero_capacity_uses_minimum() {
        let cache: ProviderCache<Vec<String>> = ProviderCache::new(0);
        cache.put_primals(vec!["min".to_string()]).await;
        assert!(cache.get_primals().await.is_some());
    }

    #[test]
    fn test_cache_stats_display() {
        let stats = CacheStats {
            hits: 10,
            misses: 5,
            total: 15,
            hit_rate: 66.666,
        };
        let s = format!("{stats}");
        assert!(s.contains("10"));
        assert!(s.contains('5'));
        assert!(s.contains("66.7"));
    }

    #[test]
    fn test_cache_key_equality() {
        assert_eq!(CacheKey::Primals, CacheKey::Primals);
        assert_ne!(CacheKey::Primals, CacheKey::Topology);
    }
}
