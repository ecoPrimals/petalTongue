// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

#[tokio::test]
async fn test_biomeos_ui_manager_creation() {
    let manager = BiomeOSUIManager::new().await;

    // Should default to mock mode if biomeOS not available
    assert_eq!(manager.current_tab, UITab::Devices);
}

#[tokio::test]
async fn test_tab_switching() {
    let mut manager = BiomeOSUIManager::new().await;

    assert_eq!(manager.current_tab, UITab::Devices);

    manager.current_tab = UITab::Primals;
    assert_eq!(manager.current_tab, UITab::Primals);

    manager.current_tab = UITab::NicheDesigner;
    assert_eq!(manager.current_tab, UITab::NicheDesigner);
}

#[tokio::test]
async fn test_fixture_mode() {
    let manager = BiomeOSUIManager::new().await;

    #[cfg(feature = "mock")]
    assert!(manager.is_fixture_mode());
    #[cfg(not(feature = "mock"))]
    assert!(!manager.is_fixture_mode());
}

#[tokio::test]
async fn test_rpc_interface() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager.clone());

    // Test tab switching via RPC
    rpc.show_primal_panel().await.unwrap();
    assert_eq!(manager.read().await.current_tab, UITab::Primals);

    rpc.show_niche_designer().await.unwrap();
    assert_eq!(manager.read().await.current_tab, UITab::NicheDesigner);

    rpc.show_device_panel().await.unwrap();
    assert_eq!(manager.read().await.current_tab, UITab::Devices);
}

#[tokio::test]
async fn test_rpc_data_access() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager);

    // Test data access via RPC (mock data only when mock feature enabled)
    let devices = rpc.get_devices().await.unwrap();
    let primals = rpc.get_primals_extended().await.unwrap();
    let templates = rpc.get_niche_templates().await.unwrap();

    #[cfg(feature = "mock")]
    {
        assert!(!devices.is_empty(), "Should have mock devices");
        assert!(!primals.is_empty(), "Should have mock primals");
        assert!(!templates.is_empty(), "Should have mock templates");
    }
    #[cfg(not(feature = "mock"))]
    {
        assert!(devices.is_empty());
        assert!(primals.is_empty());
        assert!(templates.is_empty());
    }
}

#[tokio::test]
async fn test_refresh_throttling() {
    let mut manager = BiomeOSUIManager::new().await;

    // First refresh should succeed
    assert!(manager.refresh().await.is_ok());

    // Immediate second refresh should skip (throttled)
    let last_refresh = manager.last_refresh;
    assert!(manager.refresh().await.is_ok());
    assert_eq!(manager.last_refresh, last_refresh); // Should not have updated
}

#[tokio::test]
async fn test_panel_access() {
    let manager = BiomeOSUIManager::new().await;

    // Test panel accessors
    let _ = manager.device_panel();
    let _ = manager.primal_panel();
    let _ = manager.niche_designer();
}

#[tokio::test]
async fn test_ui_render_headless() {
    let mut manager = BiomeOSUIManager::new().await;
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            manager.ui(ui);
        });
    });
}

#[tokio::test]
async fn test_ui_render_all_tabs_via_rpc() {
    let manager = Arc::new(RwLock::new(BiomeOSUIManager::new().await));
    let rpc = BiomeOSUIRPC::new(manager.clone());
    rpc.show_primal_panel().await.unwrap();
    let mut mgr = manager.write().await;
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            mgr.ui(ui);
        });
    });
    drop(mgr);
    rpc.show_niche_designer().await.unwrap();
    let mut mgr = manager.write().await;
    let _ = ctx.run(egui::RawInput::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            mgr.ui(ui);
        });
    });
}

#[tokio::test]
async fn test_uitab_debug() {
    assert!(format!("{:?}", UITab::Devices).contains("Devices"));
    assert!(format!("{:?}", UITab::Primals).contains("Primals"));
    assert!(format!("{:?}", UITab::NicheDesigner).contains("NicheDesigner"));
}

#[tokio::test]
async fn test_uitab_equality() {
    assert_eq!(UITab::Devices, UITab::Devices);
    assert_ne!(UITab::Devices, UITab::Primals);
}
