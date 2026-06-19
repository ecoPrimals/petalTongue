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
//! │  ├─ Provider (BiomeOSProvider or OfflineDeviceProvider)     │
//! │  ├─ DevicePanel (device management; shared UIEventHandler)  │
//! │  ├─ PrimalPanel (primal status)                             │
//! │  ├─ NicheDesigner (niche creation)                          │
//! │  └─ JSON-RPC Methods (biomeOS API)                          │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use crate::biomeos_integration::{BiomeOSProvider, Device, NicheTemplate, Primal};
use crate::device_panel::DevicePanel;
use crate::error::Result;
use crate::niche_designer::NicheDesigner;
#[cfg(feature = "offline-demo")]
use crate::offline_device_provider::OfflineDeviceProvider;
use crate::primal_panel::PrimalPanel;
use crate::ui_events::UIEventHandler;
use egui::Ui;
use petal_tongue_core::constants;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// biomeOS UI Manager - Main integration point
pub struct BiomeOSUIManager {
    /// Live biomeOS provider (when ecosystem is reachable)
    biomeos_provider: Option<BiomeOSProvider>,
    /// Offline provider — only when `offline-demo` feature enabled and biomeOS unavailable
    #[cfg(feature = "offline-demo")]
    offline_provider: Option<OfflineDeviceProvider>,
    /// Operating in offline/degraded mode (no live ecosystem connection)
    offline_mode: bool,

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

        let biomeos_provider = BiomeOSProvider::discover()
            .await
            .inspect_err(|e| tracing::debug!("biomeOS provider discovery: {e}"))
            .ok()
            .flatten();

        let offline_mode = biomeos_provider.is_none() && cfg!(feature = "offline-demo");

        if offline_mode {
            info!(
                "Offline mode: biomeOS unavailable — serving degraded sample data (offline-demo enabled)"
            );
        } else if biomeos_provider.is_none() {
            info!(
                "Ecosystem unavailable — empty panels (use --features offline-demo for sample data)"
            );
        } else {
            info!("✅ Connected to biomeOS");
        }

        #[cfg(feature = "offline-demo")]
        let offline_provider = if offline_mode {
            Some(OfflineDeviceProvider::new())
        } else {
            None
        };

        Self {
            biomeos_provider,
            #[cfg(feature = "offline-demo")]
            offline_provider,
            offline_mode,
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

        let (devices, primals, templates) = if self.offline_mode {
            #[cfg(feature = "offline-demo")]
            {
                if self.offline_provider.is_none() {
                    self.offline_provider = Some(OfflineDeviceProvider::new());
                }

                if let Some(offline) = &self.offline_provider {
                    (
                        offline.get_devices(),
                        offline.get_primals_extended(),
                        offline.get_niche_templates(),
                    )
                } else {
                    warn!("Offline provider not available");
                    return Ok(());
                }
            }
            #[cfg(not(feature = "offline-demo"))]
            {
                tracing::error!(
                    "offline_mode set without offline-demo feature — returning empty data"
                );
                return Ok(());
            }
        } else if let Some(provider) = &self.biomeos_provider {
            let devices = provider.get_devices().await?;
            let primals = provider.get_primals_extended().await?;
            let templates = provider.get_niche_templates().await?;
            (devices, primals, templates)
        } else {
            warn!("No ecosystem connection — panels remain empty");
            return Ok(());
        };

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
        ui.heading("🌸 biomeOS Device & Niche Management");
        ui.separator();

        self.render_provider_status(ui);
        ui.add_space(8.0);

        self.render_tab_bar(ui);
        ui.separator();
        ui.add_space(8.0);

        match self.current_tab {
            UITab::Devices => self.device_panel.ui(ui),
            UITab::Primals => self.primal_panel.ui(ui),
            UITab::NicheDesigner => self.niche_designer.ui(ui),
        }
    }

    /// Render provider status indicator
    fn render_provider_status(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if self.offline_mode {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    "⚠ Ecosystem unavailable (offline/degraded)",
                );
                ui.label("(serving cached sample data)");
            } else if self.biomeos_provider.is_some() {
                ui.colored_label(egui::Color32::GREEN, "✓ Connected to biomeOS");
            } else {
                ui.colored_label(egui::Color32::RED, "✗ No ecosystem connection");
                ui.label("(panels empty until biomeOS is reachable)");
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

    /// Check if operating in offline/degraded mode (no live ecosystem connection).
    #[must_use]
    pub const fn is_offline_mode(&self) -> bool {
        self.offline_mode
    }

    /// Get current tab
    #[must_use]
    pub const fn current_tab(&self) -> UITab {
        self.current_tab
    }
}

/// JSON-RPC Methods for biomeOS Integration
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
        {
            let mut manager = self.manager.write().await;
            manager.current_tab = UITab::Devices;
        }
        Ok(())
    }

    /// Show primal panel
    pub async fn show_primal_panel(&self) -> Result<()> {
        {
            let mut manager = self.manager.write().await;
            manager.current_tab = UITab::Primals;
        }
        Ok(())
    }

    /// Show niche designer
    pub async fn show_niche_designer(&self) -> Result<()> {
        {
            let mut manager = self.manager.write().await;
            manager.current_tab = UITab::NicheDesigner;
        }
        Ok(())
    }

    /// Get device list
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        let manager = self.manager.read().await;
        if manager.offline_mode {
            #[cfg(feature = "offline-demo")]
            {
                Ok(manager
                    .offline_provider
                    .as_ref()
                    .map(OfflineDeviceProvider::get_devices)
                    .unwrap_or_default())
            }
            #[cfg(not(feature = "offline-demo"))]
            Ok(Vec::new())
        } else if let Some(provider) = &manager.biomeos_provider {
            provider.get_devices().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get primal list
    pub async fn get_primals_extended(&self) -> Result<Vec<Primal>> {
        let manager = self.manager.read().await;
        if manager.offline_mode {
            #[cfg(feature = "offline-demo")]
            {
                Ok(manager
                    .offline_provider
                    .as_ref()
                    .map(OfflineDeviceProvider::get_primals_extended)
                    .unwrap_or_default())
            }
            #[cfg(not(feature = "offline-demo"))]
            Ok(Vec::new())
        } else if let Some(provider) = &manager.biomeos_provider {
            provider.get_primals_extended().await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get niche templates
    pub async fn get_niche_templates(&self) -> Result<Vec<NicheTemplate>> {
        let manager = self.manager.read().await;
        if manager.offline_mode {
            #[cfg(feature = "offline-demo")]
            {
                Ok(manager
                    .offline_provider
                    .as_ref()
                    .map(OfflineDeviceProvider::get_niche_templates)
                    .unwrap_or_default())
            }
            #[cfg(not(feature = "offline-demo"))]
            Ok(Vec::new())
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
#[path = "biomeos_ui_manager_tests.rs"]
mod tests;
