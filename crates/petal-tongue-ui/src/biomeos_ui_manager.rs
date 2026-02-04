//! biomeOS UI Integration Module
//!
//! This module wires together all the UI components (DevicePanel, PrimalPanel,
//! NicheDesigner) and provides a unified interface for biomeOS integration.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ BiomeOSUIManager                                            │
//! │  ├─ Provider (BiomeOSProvider or MockProvider)              │
//! │  ├─ EventHandler (centralized event dispatch)               │
//! │  ├─ DevicePanel (device management)                         │
//! │  ├─ PrimalPanel (primal status)                             │
//! │  ├─ NicheDesigner (niche creation)                          │
//! │  └─ JSON-RPC Methods (biomeOS API)                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use crate::biomeos_integration::{BiomeOSProvider, Device, NicheTemplate, Primal};
use crate::device_panel::DevicePanel;
use crate::mock_device_provider::MockDeviceProvider;
use crate::niche_designer::NicheDesigner;
use crate::primal_panel::PrimalPanel;
use crate::ui_events::UIEventHandler;
use anyhow::Result;
use egui::Ui;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// biomeOS UI Manager - Main integration point
pub struct BiomeOSUIManager {
    /// Provider for data (BiomeOS or Mock)
    biomeos_provider: Option<BiomeOSProvider>,
    /// Mock provider - lazily initialized only when needed for graceful degradation
    mock_provider: Option<MockDeviceProvider>,
    use_mock: bool,

    /// Event handler
    event_handler: Arc<RwLock<UIEventHandler>>,

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

        let use_mock = biomeos_provider.is_none();

        if use_mock {
            info!("📦 Using mock provider (biomeOS not available)");
        } else {
            info!("✅ Connected to biomeOS");
        }

        // Lazy initialization: only create mock provider when needed for graceful degradation
        let mock_provider = if use_mock {
            Some(MockDeviceProvider::new())
        } else {
            None
        };

        Self {
            biomeos_provider,
            mock_provider,
            use_mock,
            event_handler: event_handler.clone(),
            device_panel: DevicePanel::new(event_handler.clone()),
            primal_panel: PrimalPanel::new(event_handler.clone()),
            niche_designer: NicheDesigner::new(event_handler.clone()),
            current_tab: UITab::Devices,
            last_refresh: std::time::Instant::now(),
            refresh_interval: std::time::Duration::from_secs(2),
        }
    }

    /// Refresh data from provider
    pub async fn refresh(&mut self) -> Result<()> {
        if self.last_refresh.elapsed() < self.refresh_interval {
            return Ok(());
        }

        let (devices, primals, templates) = if self.use_mock {
            // Use mock provider (methods are not async)
            // Lazily create mock provider if not already initialized
            if self.mock_provider.is_none() {
                self.mock_provider = Some(MockDeviceProvider::new());
            }

            if let Some(mock) = &self.mock_provider {
                let devices = mock.get_devices();
                let primals = mock.get_primals_extended();
                let templates = mock.get_niche_templates();
                (devices, primals, templates)
            } else {
                warn!("Mock provider not available");
                return Ok(());
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
            if self.use_mock {
                ui.colored_label(egui::Color32::YELLOW, "⚠ Mock Mode");
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
    pub fn device_panel(&self) -> &DevicePanel {
        &self.device_panel
    }

    /// Get reference to primal panel
    pub fn primal_panel(&self) -> &PrimalPanel {
        &self.primal_panel
    }

    /// Get reference to niche designer
    pub fn niche_designer(&self) -> &NicheDesigner {
        &self.niche_designer
    }

    /// Check if using mock mode
    pub fn is_mock_mode(&self) -> bool {
        self.use_mock
    }

    /// Get current tab
    pub fn current_tab(&self) -> UITab {
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
    pub async fn show_device_panel(&self) -> Result<()> {
        let mut manager = self.manager.write().await;
        manager.current_tab = UITab::Devices;
        Ok(())
    }

    /// Show primal panel
    pub async fn show_primal_panel(&self) -> Result<()> {
        let mut manager = self.manager.write().await;
        manager.current_tab = UITab::Primals;
        Ok(())
    }

    /// Show niche designer
    pub async fn show_niche_designer(&self) -> Result<()> {
        let mut manager = self.manager.write().await;
        manager.current_tab = UITab::NicheDesigner;
        Ok(())
    }

    /// Get device list
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        let manager = self.manager.read().await;
        if manager.use_mock {
            // Use mock provider for graceful degradation
            Ok(manager
                .mock_provider
                .as_ref()
                .map(|m| m.get_devices())
                .unwrap_or_default())
        } else if let Some(provider) = &manager.biomeos_provider {
            provider.get_devices().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get primal list
    pub async fn get_primals_extended(&self) -> Result<Vec<Primal>> {
        let manager = self.manager.read().await;
        if manager.use_mock {
            // Use mock provider for graceful degradation
            Ok(manager
                .mock_provider
                .as_ref()
                .map(|m| m.get_primals_extended())
                .unwrap_or_default())
        } else if let Some(provider) = &manager.biomeos_provider {
            provider.get_primals_extended().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get niche templates
    pub async fn get_niche_templates(&self) -> Result<Vec<NicheTemplate>> {
        let manager = self.manager.read().await;
        if manager.use_mock {
            // Use mock provider for graceful degradation
            Ok(manager
                .mock_provider
                .as_ref()
                .map(|m| m.get_niche_templates())
                .unwrap_or_default())
        } else if let Some(provider) = &manager.biomeos_provider {
            provider.get_niche_templates().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Refresh all data
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
    async fn test_mock_mode() {
        let manager = BiomeOSUIManager::new().await;

        // In test environment, should use mock mode
        assert!(manager.is_mock_mode());
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

        // Test data access via RPC
        let devices = rpc.get_devices().await.unwrap();
        assert!(!devices.is_empty(), "Should have mock devices");

        let primals = rpc.get_primals_extended().await.unwrap();
        assert!(!primals.is_empty(), "Should have mock primals");

        let templates = rpc.get_niche_templates().await.unwrap();
        assert!(!templates.is_empty(), "Should have mock templates");
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
}
