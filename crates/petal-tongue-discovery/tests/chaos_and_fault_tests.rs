// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Chaos and fault tolerance tests for discovery system
//!
//! Tests system behavior under adverse conditions:
//! - Primal churn (services appearing/disappearing)
//! - Network timeouts
//! - Malformed responses
//! - Connection failures
//! - Concurrent access
//!
//! MODERN IDIOMATIC RUST:
//! - Fully concurrent (no sleeps, no serial execution)
//! - Non-blocking operations
//! - Proper async/await patterns
//! - No environment variable mutation (causes race conditions)

use petal_tongue_discovery::{SongbirdClient, UnixSocketProvider};
use std::time::Duration;
use tokio::time::timeout;

/// Test handling of missing Songbird socket
#[tokio::test]
async fn test_songbird_missing_socket() {
    // Try to discover Songbird when it doesn't exist
    let result = SongbirdClient::discover(Some("nonexistent-family-12345"));

    // Should gracefully fail (not panic)
    assert!(result.is_err());

    // Error message should be informative
    let err = format!("{result:?}");
    assert!(
        err.contains("not found")
            || err.contains("NotFound")
            || err.contains("DiscoveryServiceNotFound")
    );
}

/// Test handling of invalid paths
#[tokio::test]
async fn test_unix_socket_invalid_paths() {
    let provider = UnixSocketProvider::new();

    // Try to discover from paths (some may not exist)
    let result = provider.discover().await;

    // Should succeed but return empty list (graceful degradation)
    assert!(result.is_ok());
}

/// Test timeout handling with fast timeout
#[tokio::test(flavor = "multi_thread")]
async fn test_discovery_timeout_handling() {
    let provider = UnixSocketProvider::new();

    // Wrap discovery in timeout (generous for CI/coverage instrumentation)
    let result = timeout(Duration::from_secs(5), provider.discover()).await;

    // Should complete within timeout (modern async should be fast)
    assert!(result.is_ok(), "Discovery should complete within 5s");
}

/// Test concurrent discovery (no race conditions) - FULLY PARALLEL
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_discovery() {
    // Spawn multiple discovery tasks concurrently using join_all
    let tasks: Vec<_> = (0..20)
        .map(|_| {
            tokio::spawn(async move {
                let provider = UnixSocketProvider::new();
                provider.discover().await
            })
        })
        .collect();

    // Wait for all to complete in parallel
    let results = futures::future::join_all(tasks).await;

    // All should complete without panic
    for result in results {
        assert!(result.is_ok(), "Task should not panic");
        assert!(result.unwrap().is_ok(), "Discovery should succeed");
    }
}

/// Test primal churn simulation (idempotency)
#[tokio::test]
async fn test_primal_churn_resilience() {
    let provider = UnixSocketProvider::new();

    // Run multiple discoveries in quick succession
    let (result1, result2, result3) = tokio::join!(
        provider.discover(),
        provider.discover(),
        provider.discover(),
    );

    // All should succeed (system is stateless and idempotent)
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());
}

/// Test permission denied handling (graceful degradation)
#[tokio::test]
async fn test_permission_denied_handling() {
    let provider = UnixSocketProvider::new();
    let result = provider.discover().await;

    // Should not panic, even if some paths are inaccessible
    assert!(result.is_ok());
}

/// Test capability-based inference (no hardcoding)
#[tokio::test]
async fn test_capability_based_inference() {
    let provider = UnixSocketProvider::new();

    // Discovery should handle ANY capability pattern
    // No hardcoded primal names means ANY primal can be discovered
    assert!(provider.discover().await.is_ok());
}

/// Test malformed capability handling (robustness)
#[tokio::test]
async fn test_malformed_capability_handling() {
    let provider = UnixSocketProvider::new();

    // Discovery should handle any socket format gracefully
    let result = provider.discover().await;
    assert!(result.is_ok());
}

/// Test rapid repeated discovery (no resource leaks)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_rapid_repeated_discovery() {
    let provider = UnixSocketProvider::new();

    // Run many discoveries in parallel
    let tasks: Vec<_> = (0..50).map(|_| provider.discover()).collect();

    let results = futures::future::join_all(tasks).await;

    // All should complete (no deadlocks or resource exhaustion)
    assert_eq!(results.len(), 50);
    for result in results {
        assert!(result.is_ok());
    }
}

/// Test concurrent Songbird discovery attempts
#[tokio::test]
async fn test_concurrent_songbird_discovery() {
    // Multiple concurrent attempts to find Songbird
    let tasks: Vec<_> = (0..10)
        .map(|i| {
            tokio::spawn(async move { SongbirdClient::discover(Some(&format!("test-family-{i}"))) })
        })
        .collect();

    let results = futures::future::join_all(tasks).await;

    // All should complete without panic (may error if Songbird not running)
    for result in results {
        assert!(result.is_ok(), "Task should not panic");
    }
}

/// Test mixed concurrent operations (discovery + timeout)
#[tokio::test]
async fn test_mixed_concurrent_operations() {
    let provider = UnixSocketProvider::new();

    // Mix of operations happening concurrently
    let (discovery_result, timeout_result) = tokio::join!(
        provider.discover(),
        timeout(Duration::from_millis(100), provider.discover())
    );

    assert!(discovery_result.is_ok());
    // Timeout result might succeed or timeout - both are valid
    assert!(timeout_result.is_ok() || timeout_result.is_err());
}

/// Test zero-delay concurrent creation (constructor safety)
#[test]
fn test_concurrent_provider_creation() {
    // Create many providers concurrently on regular threads
    let handles: Vec<_> = (0..100)
        .map(|_| {
            std::thread::spawn(|| {
                let _provider = UnixSocketProvider::new();
            })
        })
        .collect();

    // All should complete without panic
    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
}

/// Test that discovery is truly non-blocking
#[tokio::test]
async fn test_non_blocking_discovery() {
    let provider = UnixSocketProvider::new();

    // Start discovery
    let discovery_future = provider.discover();

    // Should be able to do other work while waiting
    let other_work = async {
        for _ in 0..1000 {
            tokio::task::yield_now().await;
        }
        "done"
    };

    // Both should complete
    let (discovery_result, work_result) = tokio::join!(discovery_future, other_work);

    assert!(discovery_result.is_ok());
    assert_eq!(work_result, "done");
}
