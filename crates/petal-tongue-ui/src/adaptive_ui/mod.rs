// SPDX-License-Identifier: AGPL-3.0-or-later
//! Adaptive UI components that respond to device capabilities
//!
//! This module implements the `AdaptiveRenderer` trait for different device types,
//! providing optimized UI experiences for Desktop, Phone, Watch, and CLI.
//!
//! # Architecture
//!
//! ```text
//! RenderingCapabilities (from device detection)
//!           ↓
//!    AdaptiveUIManager
//!           ↓
//!     ┌─────┴─────┬─────────┬─────────┐
//!     ↓           ↓         ↓         ↓
//! DesktopUI   PhoneUI   WatchUI   CliUI
//! (Full)      (Minimal) (Essential) (Text)
//! ```
//!
//! # Example
//!
//! ```no_run
//! use petal_tongue_core::RenderingCapabilities;
//! use petal_tongue_ui::adaptive_ui::AdaptiveUIManager;
//!
//! let caps = RenderingCapabilities::detect();
//! let ui_manager = AdaptiveUIManager::new(caps);
//! // UI automatically adapts to device!
//! ```

mod formatting;
mod renderers;

use petal_tongue_core::{DeviceType, PrimalInfo, RenderingCapabilities, UIComplexity};

/// Manages adaptive UI rendering across different devices
pub struct AdaptiveUIManager {
    capabilities: RenderingCapabilities,
    renderer: renderers::AdaptiveUIRendererImpl,
}

impl AdaptiveUIManager {
    /// Create new adaptive UI manager with device detection
    pub fn new(capabilities: RenderingCapabilities) -> Self {
        let device = formatting::effective_device_for_rendering(capabilities.device_type);
        if capabilities.device_type == DeviceType::Unknown {
            tracing::warn!("Unknown device type, defaulting to desktop UI");
        }
        let renderer = renderers::create_renderer(device);

        Self {
            capabilities,
            renderer,
        }
    }

    /// Get current device type
    #[must_use]
    pub const fn device_type(&self) -> DeviceType {
        self.capabilities.device_type
    }

    /// Get UI complexity level
    #[must_use]
    pub const fn ui_complexity(&self) -> UIComplexity {
        self.capabilities.ui_complexity
    }

    /// Render primal list with device-specific optimizations
    pub fn render_primal_list(&self, ui: &mut egui::Ui, primals: &[PrimalInfo]) {
        self.renderer
            .render_primal_list(ui, primals, &self.capabilities);
    }

    /// Render topology view with device-specific optimizations
    pub fn render_topology(&self, ui: &mut egui::Ui, primals: &[PrimalInfo]) {
        self.renderer
            .render_topology(ui, primals, &self.capabilities);
    }

    /// Render metrics with device-specific optimizations
    pub fn render_metrics(&self, ui: &mut egui::Ui, metrics_data: &str) {
        self.renderer
            .render_metrics(ui, metrics_data, &self.capabilities);
    }
}

/// Trait for device-specific UI renderers
pub(crate) trait AdaptiveUIRenderer: Send + Sync {
    fn render_primal_list(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        caps: &RenderingCapabilities,
    );
    fn render_topology(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        caps: &RenderingCapabilities,
    );
    fn render_metrics(&self, ui: &mut egui::Ui, metrics_data: &str, caps: &RenderingCapabilities);
}

#[cfg(test)]
mod tests;
