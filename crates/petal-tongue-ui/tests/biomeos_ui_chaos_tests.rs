// SPDX-License-Identifier: AGPL-3.0-only
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Chaos Tests for biomeOS UI
//!
//! These tests verify system behavior under adverse conditions,
//! resource constraints, and unexpected inputs.

use petal_tongue_ui::biomeos_ui_manager::{BiomeOSUIManager, BiomeOSUIRPC};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep, timeout};

/// Chaos Test: Rapid concurrent operations
#[tokio::test]
async fn test_chaos_concurrent_hammering() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));

    // Spawn 100 concurrent tasks doing random operations
    let mut handles = vec![];

    for i in 0..100 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let rpc = BiomeOSUIRPC::new(mgr);

            // Random operations
            match i % 7 {
                0 => {
                    let _ = rpc.get_devices().await;
                }
                1 => {
                    let _ = rpc.get_primals_extended().await;
                }
                2 => {
                    let _ = rpc.get_niche_templates().await;
                }
                3 => {
                    let _ = rpc.show_device_panel().await;
                }
                4 => {
                    let _ = rpc.show_primal_panel().await;
                }
                5 => {
                    let _ = rpc.show_niche_designer().await;
                }
                _ => {
                    let _ = rpc.refresh().await;
                }
            }
        });
        handles.push(handle);
    }

    // All tasks should complete without panicking
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}

/// Chaos Test: Rapid refresh spamming
#[tokio::test]
async fn test_chaos_refresh_spam() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager);

    // Spam refresh 1000 times
    for _ in 0..1000 {
        let _ = rpc.refresh().await;
    }

    // Should handle throttling gracefully
    assert!(rpc.get_devices().await.is_ok());
}

/// Chaos Test: Interleaved reads and writes
#[tokio::test]
async fn test_chaos_interleaved_operations() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));

    // Spawn readers
    let mut handles = vec![];
    for _ in 0..50 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let rpc = BiomeOSUIRPC::new(mgr);
            for _ in 0..100 {
                let _ = rpc.get_devices().await;
            }
        });
        handles.push(handle);
    }

    // Spawn writers
    for _ in 0..50 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let rpc = BiomeOSUIRPC::new(mgr);
            for _ in 0..100 {
                let _ = rpc.refresh().await;
            }
        });
        handles.push(handle);
    }

    // All should complete
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}

/// Chaos Test: Rapid manager creation/destruction
#[tokio::test]
async fn test_chaos_manager_churn() {
    for _ in 0..100 {
        let manager = BiomeOSUIManager::new().await;
        // Mock mode only when mock feature enabled and biomeOS unavailable
        #[cfg(feature = "mock")]
        assert!(manager.is_mock_mode());
        #[cfg(not(feature = "mock"))]
        assert!(!manager.is_mock_mode());
        drop(manager);
    }
}

/// Chaos Test: Task cancellation during operations
#[tokio::test]
async fn test_chaos_task_cancellation() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));

    let mut handles = vec![];

    // Spawn tasks that we'll cancel
    for _ in 0..20 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let rpc = BiomeOSUIRPC::new(mgr);
            loop {
                let _ = rpc.refresh().await;
                sleep(Duration::from_millis(1)).await;
            }
        });
        handles.push(handle);
    }

    // Let them run briefly
    sleep(Duration::from_millis(100)).await;

    // Cancel all tasks
    for handle in handles {
        handle.abort();
    }

    // Manager should still be usable
    let rpc = BiomeOSUIRPC::new(manager);
    assert!(rpc.get_devices().await.is_ok());
}

/// Chaos Test: Timeout scenarios
#[tokio::test]
async fn test_chaos_timeout_handling() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager);

    // All operations should complete within timeout
    for _ in 0..50 {
        let result = timeout(Duration::from_secs(1), rpc.get_devices()).await;
        assert!(result.is_ok(), "Operation should not timeout");
    }
}

/// Chaos Test: Memory pressure simulation
#[tokio::test]
async fn test_chaos_memory_pressure() {
    // Create many managers simultaneously
    let mut managers = vec![];

    for _ in 0..50 {
        let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
        managers.push(manager);
    }

    // Use them all
    for manager in &managers {
        let rpc = BiomeOSUIRPC::new(manager.clone());
        assert!(rpc.get_devices().await.is_ok());
    }

    // Clean up
    managers.clear();
}

/// Chaos Test: Rapid state changes
#[tokio::test]
async fn test_chaos_rapid_state_changes() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager.clone());

    // Rapidly change state 10,000 times
    for i in 0..10000 {
        match i % 3 {
            0 => {
                rpc.show_device_panel().await.unwrap();
            }
            1 => {
                rpc.show_primal_panel().await.unwrap();
            }
            _ => {
                rpc.show_niche_designer().await.unwrap();
            }
        }
    }

    // Should still work
    assert!(rpc.get_devices().await.is_ok());
}

/// Chaos Test: Concurrent panel access
#[tokio::test]
async fn test_chaos_concurrent_panel_access() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));

    let mut handles = vec![];

    // Many readers accessing panels simultaneously
    for i in 0..100 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..50 {
                let rpc = BiomeOSUIRPC::new(mgr.clone());
                match i % 3 {
                    0 => {
                        let _ = rpc.get_devices().await;
                    }
                    1 => {
                        let _ = rpc.get_primals_extended().await;
                    }
                    _ => {
                        let _ = rpc.get_niche_templates().await;
                    }
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        assert!(handle.await.is_ok());
    }
}

/// Chaos Test: Sustained high load
#[tokio::test]
async fn test_chaos_sustained_load() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));

    // Run high load for 1 second
    let start = std::time::Instant::now();
    let mut count = 0;

    while start.elapsed() < Duration::from_secs(1) {
        let rpc = BiomeOSUIRPC::new(manager.clone());
        let _ = rpc.get_devices().await;
        count += 1;
    }

    // Should handle many operations
    assert!(count > 100, "Should handle at least 100 ops/sec");
}
