// SPDX-License-Identifier: AGPL-3.0-only
//! End-to-End Integration Tests for biomeOS UI
//!
//! These tests verify the complete flow from provider discovery through
//! UI interaction to event handling.

use petal_tongue_ui::biomeos_ui_manager::{BiomeOSUIManager, BiomeOSUIRPC, UITab};
use petal_tongue_ui::niche_designer::ValidationResult;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};

/// E2E Test: Complete device assignment flow
#[tokio::test]
async fn test_e2e_device_assignment_flow() {
    // Create manager
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager.clone());

    // 1. Get initial data (mock data only when mock feature enabled)
    let devices = rpc.get_devices().await.unwrap();
    let primals = rpc.get_primals_extended().await.unwrap();

    #[cfg(feature = "mock")]
    {
        assert!(!devices.is_empty(), "Should have devices");
        assert!(!primals.is_empty(), "Should have primals");
    }
    #[cfg(not(feature = "mock"))]
    let _ = (&devices, &primals);

    // 2. Switch to device panel
    rpc.show_device_panel().await.unwrap();
    assert_eq!(manager.read().await.current_tab(), UITab::Devices);

    // 3. Switch to primal panel
    rpc.show_primal_panel().await.unwrap();
    assert_eq!(manager.read().await.current_tab(), UITab::Primals);

    // 4. Verify data refreshes
    rpc.refresh().await.unwrap();

    // Success - complete flow works
}

/// E2E Test: Niche creation workflow
#[tokio::test]
async fn test_e2e_niche_creation_workflow() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager.clone());

    // 1. Get templates (mock only when mock feature enabled)
    let templates = rpc.get_niche_templates().await.unwrap();
    #[cfg(feature = "mock")]
    assert!(!templates.is_empty(), "Should have templates");
    #[cfg(not(feature = "mock"))]
    let _ = &templates;

    // 2. Switch to niche designer
    rpc.show_niche_designer().await.unwrap();
    assert_eq!(manager.read().await.current_tab(), UITab::NicheDesigner);

    // 3. Verify niche designer has templates
    {
        let mgr = manager.read().await;
        let designer = mgr.niche_designer();
        // Designer should have data after refresh
        assert!(designer.validation_result() == &ValidationResult::Valid);
    }

    // Success - niche creation flow works
}

/// E2E Test: Multi-tab navigation with data consistency
#[tokio::test]
async fn test_e2e_tab_navigation_consistency() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager.clone());

    // Refresh initial data
    rpc.refresh().await.unwrap();

    let initial_devices = rpc.get_devices().await.unwrap();
    let initial_primals = rpc.get_primals_extended().await.unwrap();

    // Navigate through tabs
    rpc.show_device_panel().await.unwrap();
    sleep(Duration::from_millis(10)).await;

    rpc.show_primal_panel().await.unwrap();
    sleep(Duration::from_millis(10)).await;

    rpc.show_niche_designer().await.unwrap();
    sleep(Duration::from_millis(10)).await;

    // Data should remain consistent
    let final_devices = rpc.get_devices().await.unwrap();
    let final_primals = rpc.get_primals_extended().await.unwrap();

    assert_eq!(initial_devices.len(), final_devices.len());
    assert_eq!(initial_primals.len(), final_primals.len());
}

/// E2E Test: Provider fallback behavior
#[tokio::test]
async fn test_e2e_provider_fallback() {
    let manager = BiomeOSUIManager::new().await;

    // Mock mode only when mock feature enabled and biomeOS unavailable
    #[cfg(feature = "mock")]
    assert!(manager.is_mock_mode());
    #[cfg(not(feature = "mock"))]
    assert!(!manager.is_mock_mode());

    let manager = Arc::new(RwLock::new(manager));
    let rpc = BiomeOSUIRPC::new(manager.clone());

    let devices = rpc.get_devices().await.unwrap();
    let primals = rpc.get_primals_extended().await.unwrap();
    let templates = rpc.get_niche_templates().await.unwrap();

    #[cfg(feature = "mock")]
    {
        assert!(!devices.is_empty());
        assert!(!primals.is_empty());
        assert!(!templates.is_empty());
    }
    #[cfg(not(feature = "mock"))]
    let _ = (&devices, &primals, &templates);
}

/// E2E Test: Concurrent access to manager
#[tokio::test]
async fn test_e2e_concurrent_access() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));

    // Spawn multiple concurrent tasks
    let mut handles = vec![];

    for i in 0..10 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let rpc = BiomeOSUIRPC::new(mgr);

            // Each task does a different operation
            match i % 3 {
                0 => {
                    rpc.get_devices().await.unwrap();
                }
                1 => {
                    rpc.get_primals_extended().await.unwrap();
                }
                _ => {
                    rpc.get_niche_templates().await.unwrap();
                }
            }
        });
        handles.push(handle);
    }

    // All tasks should complete successfully
    for handle in handles {
        handle.await.unwrap();
    }
}

/// E2E Test: Event propagation through system
#[tokio::test]
async fn test_e2e_event_propagation() {
    let mut manager = BiomeOSUIManager::new().await;

    // Refresh to load initial data
    manager.refresh().await.unwrap();

    // Process events
    manager.process_events().await;

    // Verify panels received data
    let device_panel = manager.device_panel();
    let primal_panel = manager.primal_panel();
    let niche_designer = manager.niche_designer();

    // All panels should have processed their initial data
    assert!(device_panel.selected_device().is_none()); // No selection initially
    assert!(primal_panel.selected_primal().is_none()); // No selection initially
    assert_eq!(niche_designer.validation_result(), &ValidationResult::Valid); // No template selected, so valid
}

/// E2E Test: Full lifecycle from startup to shutdown
#[tokio::test]
async fn test_e2e_full_lifecycle() {
    // 1. Startup
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager.clone());

    // 2. Initial data load
    rpc.refresh().await.unwrap();

    // 3. Navigate through all tabs
    rpc.show_device_panel().await.unwrap();
    rpc.show_primal_panel().await.unwrap();
    rpc.show_niche_designer().await.unwrap();

    // 4. Access all data types
    let devices = rpc.get_devices().await.unwrap();
    let primals = rpc.get_primals_extended().await.unwrap();
    let templates = rpc.get_niche_templates().await.unwrap();

    #[cfg(feature = "mock")]
    {
        assert!(!devices.is_empty());
        assert!(!primals.is_empty());
        assert!(!templates.is_empty());
    }
    #[cfg(not(feature = "mock"))]
    let _ = (&devices, &primals, &templates);

    // 5. Final refresh
    rpc.refresh().await.unwrap();

    // 6. Shutdown (implicit via drop)
    drop(rpc);
    drop(manager);

    // Success - full lifecycle complete
}

/// E2E Test: Rapid tab switching
#[tokio::test]
async fn test_e2e_rapid_tab_switching() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager.clone());

    // Rapidly switch tabs 100 times
    for i in 0..100 {
        match i % 3 {
            0 => rpc.show_device_panel().await.unwrap(),
            1 => rpc.show_primal_panel().await.unwrap(),
            _ => rpc.show_niche_designer().await.unwrap(),
        }
    }

    // Should still be in valid state
    #[cfg(feature = "mock")]
    assert!(manager.read().await.is_mock_mode());
    #[cfg(not(feature = "mock"))]
    assert!(!manager.read().await.is_mock_mode());
}

/// E2E Test: Data consistency across refreshes
#[tokio::test]
async fn test_e2e_data_consistency() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager.clone());

    // First refresh
    rpc.refresh().await.unwrap();
    let devices1 = rpc.get_devices().await.unwrap();

    // Wait for throttle to expire
    sleep(Duration::from_secs(3)).await;

    // Second refresh
    rpc.refresh().await.unwrap();
    let devices2 = rpc.get_devices().await.unwrap();

    // Data should be consistent (mock returns same; empty stays empty)
    assert_eq!(devices1.len(), devices2.len());
    if !devices1.is_empty() {
        assert_eq!(devices1[0].id, devices2[0].id);
    }
}
