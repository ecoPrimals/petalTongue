// SPDX-License-Identifier: AGPL-3.0-only
//! Fault Injection Tests for biomeOS UI
//!
//! These tests verify error handling, recovery, and resilience
//! when things go wrong.

use petal_tongue_ui::biomeos_ui_manager::{BiomeOSUIManager, BiomeOSUIRPC};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};

/// Fault Test: Manager survives panics in spawned tasks
#[tokio::test]
async fn test_fault_panic_recovery() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));

    // Spawn a task that panics
    let mgr_clone = manager.clone();
    let handle = tokio::spawn(async move {
        let _rpc = BiomeOSUIRPC::new(mgr_clone);
        panic!("Intentional panic for testing");
    });

    // Task should panic
    assert!(handle.await.is_err());

    // But manager should still work
    let rpc = BiomeOSUIRPC::new(manager);
    assert!(rpc.get_devices().await.is_ok());
}

/// Fault Test: Graceful handling of repeated errors
#[tokio::test]
async fn test_fault_repeated_operations() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager);

    // Repeatedly call operations (should not fail with mock provider)
    for _ in 0..1000 {
        assert!(rpc.get_devices().await.is_ok());
        assert!(rpc.get_primals_extended().await.is_ok());
        assert!(rpc.get_niche_templates().await.is_ok());
    }
}

/// Fault Test: Recovery from lock contention
#[tokio::test]
async fn test_fault_lock_contention() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));

    // Create heavy contention
    let mut handles = vec![];

    for _ in 0..100 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let rpc = BiomeOSUIRPC::new(mgr);
            // Hold read lock for a bit
            let _ = rpc.get_devices().await;
            sleep(Duration::from_micros(100)).await;
        });
        handles.push(handle);
    }

    // All should eventually complete
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}

/// Fault Test: Manager state consistency after errors
#[tokio::test]
async fn test_fault_state_consistency() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager.clone());

    // Get initial state
    let initial_devices = rpc.get_devices().await.unwrap();

    // Perform operations that might fail
    for _ in 0..100 {
        let _ = rpc.refresh().await;
        let _ = rpc.show_device_panel().await;
    }

    // State should still be consistent
    let final_devices = rpc.get_devices().await.unwrap();
    assert_eq!(initial_devices.len(), final_devices.len());
}

/// Fault Test: Rapid manager creation under stress
#[tokio::test]
async fn test_fault_rapid_initialization() {
    // Create many managers rapidly
    let mut handles = vec![];

    for _ in 0..50 {
        let handle = tokio::spawn(async {
            let manager = BiomeOSUIManager::new().await;
            assert!(manager.is_mock_mode());
        });
        handles.push(handle);
    }

    // All should succeed
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}

/// Fault Test: Tab switching under load
#[tokio::test]
async fn test_fault_tab_switching_stress() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager.clone());

    // Rapid tab switching while accessing data
    let mut handles = vec![];

    // Switchers
    for _ in 0..25 {
        let rpc_clone = BiomeOSUIRPC::new(manager.clone());
        let handle = tokio::spawn(async move {
            for i in 0..100 {
                match i % 3 {
                    0 => {
                        let _ = rpc_clone.show_device_panel().await;
                    }
                    1 => {
                        let _ = rpc_clone.show_primal_panel().await;
                    }
                    _ => {
                        let _ = rpc_clone.show_niche_designer().await;
                    }
                }
            }
        });
        handles.push(handle);
    }

    // Data accessors
    for _ in 0..25 {
        let rpc_clone = BiomeOSUIRPC::new(manager.clone());
        let handle = tokio::spawn(async move {
            for _ in 0..100 {
                let _ = rpc_clone.get_devices().await;
            }
        });
        handles.push(handle);
    }

    // All should complete
    for handle in handles {
        assert!(handle.await.is_ok());
    }

    // Final state should be valid
    assert!(rpc.get_devices().await.is_ok());
}

/// Fault Test: Memory safety under concurrent access
#[tokio::test]
async fn test_fault_memory_safety() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));

    // Create many RPC instances
    let mut rpcs = vec![];
    for _ in 0..100 {
        rpcs.push(BiomeOSUIRPC::new(manager.clone()));
    }

    // Use them all concurrently
    let mut handles = vec![];
    for rpc in rpcs {
        let handle = tokio::spawn(async move {
            for _ in 0..50 {
                let _ = rpc.get_devices().await;
            }
        });
        handles.push(handle);
    }

    // All should complete safely
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}

/// Fault Test: Recovery from event processing errors
#[tokio::test]
async fn test_fault_event_processing_recovery() {
    let mut manager = BiomeOSUIManager::new().await;

    // Process events repeatedly
    for _ in 0..1000 {
        manager.process_events().await;
    }

    // Should still be functional
    assert!(manager.is_mock_mode());
}

/// Fault Test: Refresh throttling under stress
#[tokio::test]
async fn test_fault_refresh_throttling() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));

    // Spam refresh from multiple tasks
    let mut handles = vec![];

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

    // All should handle throttling gracefully
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}

/// Fault Test: Data consistency after rapid changes
#[tokio::test]
async fn test_fault_data_consistency() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager.clone());

    // Initial data
    let initial_devices = rpc.get_devices().await.unwrap();
    let device_count = initial_devices.len();

    // Perform many operations
    for _ in 0..500 {
        let _ = rpc.refresh().await;
        let _ = rpc.show_device_panel().await;
        let _ = rpc.show_primal_panel().await;
    }

    // Data count should remain consistent (mock provider)
    let final_devices = rpc.get_devices().await.unwrap();
    assert_eq!(device_count, final_devices.len());
}

/// Fault Test: Concurrent manager operations
#[tokio::test]
async fn test_fault_concurrent_manager_operations() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));

    // Spawn tasks doing different operations
    let mut handles = vec![];

    for i in 0..100 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let rpc = BiomeOSUIRPC::new(mgr);

            match i % 4 {
                0 => {
                    for _ in 0..10 {
                        let _ = rpc.get_devices().await;
                    }
                }
                1 => {
                    for _ in 0..10 {
                        let _ = rpc.get_primals_extended().await;
                    }
                }
                2 => {
                    for _ in 0..10 {
                        let _ = rpc.refresh().await;
                    }
                }
                _ => {
                    for _ in 0..10 {
                        let _ = rpc.show_device_panel().await;
                        let _ = rpc.show_primal_panel().await;
                    }
                }
            }
        });
        handles.push(handle);
    }

    // All should complete
    for handle in handles {
        assert!(handle.await.is_ok());
    }
}

/// Fault Test: Sustained stress test
#[tokio::test]
async fn test_fault_sustained_stress() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));

    // Run for 2 seconds under high load
    let start = std::time::Instant::now();
    let mut total_ops = 0;

    while start.elapsed() < Duration::from_secs(2) {
        let rpc = BiomeOSUIRPC::new(manager.clone());

        // Mix of operations
        let _ = rpc.get_devices().await;
        let _ = rpc.get_primals_extended().await;
        let _ = rpc.show_device_panel().await;
        let _ = rpc.refresh().await;

        total_ops += 4;
    }

    // Should handle many operations
    assert!(total_ops > 200, "Should handle sustained load");

    // Should still be functional
    let rpc = BiomeOSUIRPC::new(manager);
    assert!(rpc.get_devices().await.is_ok());
}
