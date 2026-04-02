// SPDX-License-Identifier: AGPL-3.0-or-later
//! biomeOS UI Integration Module
//!
//! This module wires together all the UI components (`DevicePanel`, `PrimalPanel`,
//! `NicheDesigner`) and provides a unified interface for biomeOS integration.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ BiomeOSUIManager                                            │
//! │  ├─ Provider (BiomeOSProvider or DemoDeviceProvider when mock)  │
//! │  ├─ DevicePanel (device management; shared UIEventHandler)   │
//! │  ├─ PrimalPanel (primal status)                             │
//! │  ├─ NicheDesigner (niche creation)                          │
//! │  └─ JSON-RPC Methods (biomeOS API)                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use crate::biomeos_integration::{BiomeOSProvider, Device, NicheTemplate, Primal};
#[cfg(feature = "mock")]
use crate::demo_device_provider::DemoDeviceProvider;
use crate::device_panel::DevicePanel;
use crate::error::Result;
use crate::niche_designer::NicheDesigner;
use crate::primal_panel::PrimalPanel;
use crate::ui_events::UIEventHandler;
use egui::Ui;
use petal_tongue_core::constants;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// biomeOS UI Manager - Main integration point
pub struct BiomeOSUIManager {
    /// Provider for data (`BiomeOS` or Mock)
    biomeos_provider: Option<BiomeOSProvider>,
    /// Demo provider - only when `mock` feature enabled (graceful fallback when biomeOS unavailable)
    #[cfg(feature = "mock")]
    demo_provider: Option<DemoDeviceProvider>,
    use_fixtures: bool,

    /// UI Panels
    device_panel: DevicePanel,
    primal_panel: PrimalPanel,
    niche_designer: NicheDesigner,

    /// Current tab
    current_tab: UITab,

    /// Last refresh time
    last_refresh: std::time::Instant,
    refresh_interval: std::time::Duration,
}

/// UI Tab selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UITab {
    /// Device management tab
    Devices,
    /// Primal status tab
    Primals,
    /// Niche designer tab
    NicheDesigner,
}

impl BiomeOSUIManager {
    /// Create a new biomeOS UI manager
    #[must_use]
    pub async fn new() -> Self {
        info!("🌸 Creating biomeOS UI manager");

        let event_handler = Arc::new(RwLock::new(UIEventHandler::new()));

        // Try to discover biomeOS provider
        let biomeos_provider = BiomeOSProvider::discover().await.ok().flatten();

        // Demo fallback only when biomeOS unavailable AND mock feature enabled
        let use_fixtures = biomeos_provider.is_none() && cfg!(feature = "mock");

        if use_fixtures {
            info!("📦 Using demo provider (biomeOS not available, mock feature enabled)");
        } else if biomeos_provider.is_none() {
            info!("⚠️ biomeOS not available - empty panels (use --features mock for demo data)");
        } else {
            info!("✅ Connected to biomeOS");
        }

        // Lazy initialization: only create demo provider when needed (mock feature + biomeOS unavailable)
        #[cfg(feature = "mock")]
        let demo_provider = if use_fixtures {
            Some(DemoDeviceProvider::new())
        } else {
            None
        };

        Self {
            biomeos_provider,
            #[cfg(feature = "mock")]
            demo_provider,
            use_fixtures,
            device_panel: DevicePanel::new(event_handler.clone()),
            primal_panel: PrimalPanel::new(event_handler.clone()),
            niche_designer: NicheDesigner::new(event_handler),
            current_tab: UITab::Devices,
            last_refresh: std::time::Instant::now(),
            refresh_interval: constants::default_refresh_interval(),
        }
    }

    /// Refresh data from provider
    ///
    /// # Errors
    ///
    /// Returns an error if the biomeOS provider fails to fetch devices, primals, or templates.
    pub async fn refresh(&mut self) -> Result<()> {
        if self.last_refresh.elapsed() < self.refresh_interval {
            return Ok(());
        }

        let (devices, primals, templates) = if self.use_fixtures {
            #[cfg(feature = "mock")]
            {
                // Use demo provider (methods are not async)
                if self.demo_provider.is_none() {
                    self.demo_provider = Some(DemoDeviceProvider::new());
                }

                if let Some(demo) = &self.demo_provider {
                    let devices = demo.get_devices();
                    let primals = demo.get_primals_extended();
                    let templates = demo.get_niche_templates();
                    (devices, primals, templates)
                } else {
                    warn!("Demo provider not available");
                    return Ok(());
                }
            }
            #[cfg(not(feature = "mock"))]
            {
                unreachable!("use_fixtures requires mock feature");
            }
        } else if let Some(provider) = &self.biomeos_provider {
            // Use biomeOS provider (methods are async)
            let devices = provider.get_devices().await?;
            let primals = provider.get_primals_extended().await?;
            let templates = provider.get_niche_templates().await?;
            (devices, primals, templates)
        } else {
            warn!("No provider available");
            return Ok(());
        };

        // Update panels
        self.device_panel.refresh(devices).await;
        self.primal_panel.refresh(primals.clone()).await;
        self.niche_designer.refresh(templates, primals).await;

        self.last_refresh = std::time::Instant::now();

        Ok(())
    }

    /// Process events
    pub async fn process_events(&mut self) {
        self.device_panel.process_events().await;
        self.primal_panel.process_events().await;
        self.niche_designer.process_events().await;
    }

    /// Render the UI
    pub fn ui(&mut self, ui: &mut Ui) {
        // Header
        ui.heading("🌸 biomeOS Device & Niche Management");
        ui.separator();

        // Provider status
        self.render_provider_status(ui);
        ui.add_space(8.0);

        // Tab bar
        self.render_tab_bar(ui);
        ui.separator();
        ui.add_space(8.0);

        // Current tab content
        match self.current_tab {
            UITab::Devices => self.device_panel.ui(ui),
            UITab::Primals => self.primal_panel.ui(ui),
            UITab::NicheDesigner => self.niche_designer.ui(ui),
        }
    }

    /// Render provider status indicator
    fn render_provider_status(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if self.use_fixtures {
                ui.colored_label(egui::Color32::YELLOW, "⚠ Fixture Mode (offline)");
                ui.label("(biomeOS not connected)");
            } else {
                ui.colored_label(egui::Color32::GREEN, "✓ Connected to biomeOS");
            }
        });
    }

    /// Render tab bar
    fn render_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.current_tab == UITab::Devices, "🖥️ Devices")
                .clicked()
            {
                self.current_tab = UITab::Devices;
            }

            if ui
                .selectable_label(self.current_tab == UITab::Primals, "🌸 Primals")
                .clicked()
            {
                self.current_tab = UITab::Primals;
            }

            if ui
                .selectable_label(
                    self.current_tab == UITab::NicheDesigner,
                    "🎨 Niche Designer",
                )
                .clicked()
            {
                self.current_tab = UITab::NicheDesigner;
            }
        });
    }

    /// Get reference to device panel
    #[must_use]
    pub const fn device_panel(&self) -> &DevicePanel {
        &self.device_panel
    }

    /// Get reference to primal panel
    #[must_use]
    pub const fn primal_panel(&self) -> &PrimalPanel {
        &self.primal_panel
    }

    /// Get reference to niche designer
    #[must_use]
    pub const fn niche_designer(&self) -> &NicheDesigner {
        &self.niche_designer
    }

    /// Check if using fixture mode (deterministic offline data).
    #[must_use]
    pub const fn is_fixture_mode(&self) -> bool {
        self.use_fixtures
    }

    /// Get current tab
    #[must_use]
    pub const fn current_tab(&self) -> UITab {
        self.current_tab
    }
}

/// JSON-RPC Methods for biomeOS Integration
///
/// These methods provide a programmatic interface for biomeOS to interact
/// with the UI components.
pub struct BiomeOSUIRPC {
    manager: Arc<RwLock<BiomeOSUIManager>>,
}

impl BiomeOSUIRPC {
    /// Create a new RPC interface
    #[must_use]
    pub fn new(manager: Arc<RwLock<BiomeOSUIManager>>) -> Self {
        info!("📡 Creating biomeOS UI RPC interface");
        Self { manager }
    }

    /// Show device panel
    ///
    /// # Errors
    ///
    /// Returns an error if the manager lock cannot be acquired.
    pub async fn show_device_panel(&self) -> Result<()> {
        {
            let mut manager = self.manager.write().await;
            manager.current_tab = UITab::Devices;
        }
        Ok(())
    }

    /// Show primal panel
    ///
    /// # Errors
    ///
    /// Returns an error if the manager lock cannot be acquired.
    pub async fn show_primal_panel(&self) -> Result<()> {
        {
            let mut manager = self.manager.write().await;
            manager.current_tab = UITab::Primals;
        }
        Ok(())
    }

    /// Show niche designer
    ///
    /// # Errors
    ///
    /// Returns an error if the manager lock cannot be acquired.
    pub async fn show_niche_designer(&self) -> Result<()> {
        {
            let mut manager = self.manager.write().await;
            manager.current_tab = UITab::NicheDesigner;
        }
        Ok(())
    }

    /// Get device list
    ///
    /// # Errors
    ///
    /// Returns an error if the biomeOS provider fails to fetch devices.
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        let manager = self.manager.read().await;
        if manager.use_fixtures {
            #[cfg(feature = "mock")]
            {
                Ok(manager
                    .demo_provider
                    .as_ref()
                    .map(super::demo_device_provider::DemoDeviceProvider::get_devices)
                    .unwrap_or_default())
            }
            #[cfg(not(feature = "mock"))]
            Ok(Vec::new())
        } else if let Some(provider) = &manager.biomeos_provider {
            provider.get_devices().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get primal list
    ///
    /// # Errors
    ///
    /// Returns an error if the biomeOS provider fails to fetch primals.
    pub async fn get_primals_extended(&self) -> Result<Vec<Primal>> {
        let manager = self.manager.read().await;
        if manager.use_fixtures {
            #[cfg(feature = "mock")]
            {
                Ok(manager
                    .demo_provider
                    .as_ref()
                    .map(super::demo_device_provider::DemoDeviceProvider::get_primals_extended)
                    .unwrap_or_default())
            }
            #[cfg(not(feature = "mock"))]
            Ok(Vec::new())
        } else if let Some(provider) = &manager.biomeos_provider {
            provider.get_primals_extended().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get niche templates
    ///
    /// # Errors
    ///
    /// Returns an error if the biomeOS provider fails to fetch templates.
    pub async fn get_niche_templates(&self) -> Result<Vec<NicheTemplate>> {
        let manager = self.manager.read().await;
        if manager.use_fixtures {
            #[cfg(feature = "mock")]
            {
                Ok(manager
                    .demo_provider
                    .as_ref()
                    .map(super::demo_device_provider::DemoDeviceProvider::get_niche_templates)
                    .unwrap_or_default())
            }
            #[cfg(not(feature = "mock"))]
            Ok(Vec::new())
        } else if let Some(provider) = &manager.biomeos_provider {
            provider.get_niche_templates().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Refresh all data
    ///
    /// # Errors
    ///
    /// Returns an error if the manager refresh fails.
    pub async fn refresh(&self) -> Result<()> {
        let mut manager = self.manager.write().await;
        manager.refresh().await
    }
}

#[cfg(test)]
mod tests {
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
}
