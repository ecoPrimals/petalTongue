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

use petal_tongue_core::{
    AdaptiveRenderer, DeviceType, PrimalInfo, RenderingCapabilities, UIComplexity,
};

/// Manages adaptive UI rendering across different devices
pub struct AdaptiveUIManager {
    capabilities: RenderingCapabilities,
    renderer: Box<dyn AdaptiveUIRenderer>,
}

impl AdaptiveUIManager {
    /// Create new adaptive UI manager with device detection
    pub fn new(capabilities: RenderingCapabilities) -> Self {
        let renderer: Box<dyn AdaptiveUIRenderer> = match capabilities.device_type {
            DeviceType::Desktop => Box::new(DesktopUIRenderer::new()),
            DeviceType::Phone => Box::new(PhoneUIRenderer::new()),
            DeviceType::Watch => Box::new(WatchUIRenderer::new()),
            DeviceType::CLI => Box::new(CliUIRenderer::new()),
            DeviceType::Tablet => Box::new(TabletUIRenderer::new()),
            DeviceType::TV => Box::new(TvUIRenderer::new()),
            DeviceType::Unknown => {
                // Default to desktop for unknown devices
                tracing::warn!("Unknown device type, defaulting to desktop UI");
                Box::new(DesktopUIRenderer::new())
            }
        };

        Self {
            capabilities,
            renderer,
        }
    }

    /// Get current device type
    pub fn device_type(&self) -> DeviceType {
        self.capabilities.device_type
    }

    /// Get UI complexity level
    pub fn ui_complexity(&self) -> UIComplexity {
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
trait AdaptiveUIRenderer: Send + Sync {
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

// ============================================================================
// Desktop UI Renderer (Full Complexity)
// ============================================================================

struct DesktopUIRenderer;

impl DesktopUIRenderer {
    fn new() -> Self {
        Self
    }
}

impl AdaptiveUIRenderer for DesktopUIRenderer {
    fn render_primal_list(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        // Desktop: Full feature set with detailed cards
        ui.heading("🌸 Primals");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            for primal in primals {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        // Status indicator
                        let (color, text) = match primal.health {
                            petal_tongue_core::PrimalHealthStatus::Healthy => {
                                (egui::Color32::GREEN, "●")
                            }
                            petal_tongue_core::PrimalHealthStatus::Warning => {
                                (egui::Color32::YELLOW, "●")
                            }
                            petal_tongue_core::PrimalHealthStatus::Critical => {
                                (egui::Color32::RED, "●")
                            }
                            petal_tongue_core::PrimalHealthStatus::Unknown => {
                                (egui::Color32::GRAY, "○")
                            }
                        };
                        ui.colored_label(color, text);

                        ui.vertical(|ui| {
                            ui.strong(&primal.name);
                            ui.label(format!("Type: {}", primal.primal_type));
                            ui.label(format!("Endpoint: {}", primal.endpoint));

                            if !primal.capabilities.is_empty() {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label("Capabilities:");
                                    for cap in &primal.capabilities {
                                        ui.small(format!("🔹 {}", cap));
                                    }
                                });
                            }
                        });
                    });
                });
                ui.add_space(4.0);
            }
        });
    }

    fn render_topology(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        // Desktop: Full graph visualization
        ui.heading("🕸️ Topology");
        ui.separator();
        ui.label(format!("Connected primals: {}", primals.len()));
        ui.label("(Full graph visualization would go here)");
    }

    fn render_metrics(&self, ui: &mut egui::Ui, metrics_data: &str, _caps: &RenderingCapabilities) {
        // Desktop: Detailed metrics with charts
        ui.heading("📊 Metrics");
        ui.separator();
        ui.label(metrics_data);
    }
}

// ============================================================================
// Phone UI Renderer (Minimal Complexity)
// ============================================================================

struct PhoneUIRenderer;

impl PhoneUIRenderer {
    fn new() -> Self {
        Self
    }
}

impl AdaptiveUIRenderer for PhoneUIRenderer {
    fn render_primal_list(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        // Phone: Simplified list, touch-optimized
        ui.heading("🌸 Primals");

        egui::ScrollArea::vertical().show(ui, |ui| {
            for primal in primals {
                ui.horizontal(|ui| {
                    // Larger touch targets
                    let (color, icon) = match primal.health {
                        petal_tongue_core::PrimalHealthStatus::Healthy => {
                            (egui::Color32::GREEN, "✅")
                        }
                        petal_tongue_core::PrimalHealthStatus::Warning => {
                            (egui::Color32::YELLOW, "⚠️")
                        }
                        petal_tongue_core::PrimalHealthStatus::Critical => {
                            (egui::Color32::RED, "❌")
                        }
                        petal_tongue_core::PrimalHealthStatus::Unknown => {
                            (egui::Color32::GRAY, "❓")
                        }
                    };

                    ui.colored_label(color, icon);
                    ui.label(&primal.name);
                });
                ui.separator();
            }
        });
    }

    fn render_topology(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        // Phone: Simple count, tap for details
        ui.label(format!("📱 {} primals connected", primals.len()));
        ui.small("Tap primal for details");
    }

    fn render_metrics(
        &self,
        ui: &mut egui::Ui,
        _metrics_data: &str,
        _caps: &RenderingCapabilities,
    ) {
        // Phone: Key metrics only
        ui.label("📊 Metrics");
        ui.small("(Simplified view)");
    }
}

// ============================================================================
// Watch UI Renderer (Essential Complexity)
// ============================================================================

struct WatchUIRenderer;

impl WatchUIRenderer {
    fn new() -> Self {
        Self
    }
}

impl AdaptiveUIRenderer for WatchUIRenderer {
    fn render_primal_list(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        // Watch: Glanceable summary only
        let healthy = primals
            .iter()
            .filter(|p| matches!(p.health, petal_tongue_core::PrimalHealthStatus::Healthy))
            .count();
        let total = primals.len();

        if healthy == total {
            ui.colored_label(egui::Color32::GREEN, format!("✅ {}/{} OK", healthy, total));
        } else {
            ui.colored_label(egui::Color32::YELLOW, format!("⚠️ {}/{}", healthy, total));
        }
    }

    fn render_topology(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        // Watch: Icon + count only
        ui.label(format!("🕸️ {}", primals.len()));
    }

    fn render_metrics(
        &self,
        ui: &mut egui::Ui,
        _metrics_data: &str,
        _caps: &RenderingCapabilities,
    ) {
        // Watch: Single most important metric
        ui.label("📊 OK");
    }
}

// ============================================================================
// CLI UI Renderer (Text-only)
// ============================================================================

struct CliUIRenderer;

impl CliUIRenderer {
    fn new() -> Self {
        Self
    }
}

impl AdaptiveUIRenderer for CliUIRenderer {
    fn render_primal_list(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        // CLI: Plain text list
        for primal in primals {
            let status = match primal.health {
                petal_tongue_core::PrimalHealthStatus::Healthy => "OK",
                petal_tongue_core::PrimalHealthStatus::Warning => "WARN",
                petal_tongue_core::PrimalHealthStatus::Critical => "CRIT",
                petal_tongue_core::PrimalHealthStatus::Unknown => "UNKN",
            };
            ui.monospace(format!("[{}] {}", status, primal.name));
        }
    }

    fn render_topology(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        ui.monospace(format!("Topology: {} nodes", primals.len()));
    }

    fn render_metrics(&self, ui: &mut egui::Ui, metrics_data: &str, _caps: &RenderingCapabilities) {
        ui.monospace(format!("Metrics: {}", metrics_data));
    }
}

// ============================================================================
// Tablet UI Renderer (Simplified Complexity)
// ============================================================================

struct TabletUIRenderer;

impl TabletUIRenderer {
    fn new() -> Self {
        Self
    }
}

impl AdaptiveUIRenderer for TabletUIRenderer {
    fn render_primal_list(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        // Tablet: Similar to desktop but with larger touch targets
        ui.heading("🌸 Primals");

        egui::ScrollArea::vertical().show(ui, |ui| {
            for primal in primals {
                ui.horizontal(|ui| {
                    let (color, text) = match primal.health {
                        petal_tongue_core::PrimalHealthStatus::Healthy => {
                            (egui::Color32::GREEN, "●")
                        }
                        _ => (egui::Color32::GRAY, "○"),
                    };
                    ui.colored_label(color, text);
                    ui.label(&primal.name);
                    ui.label(&primal.primal_type);
                });
                ui.separator();
            }
        });
    }

    fn render_topology(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        ui.heading("🕸️ Topology");
        ui.label(format!("{} primals", primals.len()));
    }

    fn render_metrics(&self, ui: &mut egui::Ui, metrics_data: &str, _caps: &RenderingCapabilities) {
        ui.heading("📊 Metrics");
        ui.label(metrics_data);
    }
}

// ============================================================================
// TV UI Renderer (Simplified Complexity, 10-foot UI)
// ============================================================================

struct TvUIRenderer;

impl TvUIRenderer {
    fn new() -> Self {
        Self
    }
}

impl AdaptiveUIRenderer for TvUIRenderer {
    fn render_primal_list(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        // TV: Large text, high contrast, 10-foot UI
        ui.heading(egui::RichText::new("🌸 PRIMALS").size(32.0));
        ui.add_space(20.0);

        for primal in primals {
            ui.horizontal(|ui| {
                let (color, text) = match primal.health {
                    petal_tongue_core::PrimalHealthStatus::Healthy => (egui::Color32::GREEN, "● "),
                    _ => (egui::Color32::GRAY, "○ "),
                };
                ui.colored_label(color, egui::RichText::new(text).size(24.0));
                ui.label(egui::RichText::new(&primal.name).size(24.0));
            });
            ui.add_space(10.0);
        }
    }

    fn render_topology(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        ui.heading(egui::RichText::new("🕸️ TOPOLOGY").size(32.0));
        ui.label(egui::RichText::new(format!("{} PRIMALS", primals.len())).size(24.0));
    }

    fn render_metrics(&self, ui: &mut egui::Ui, metrics_data: &str, _caps: &RenderingCapabilities) {
        ui.heading(egui::RichText::new("📊 METRICS").size(32.0));
        ui.label(egui::RichText::new(metrics_data).size(20.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_ui_manager_creation() {
        let caps = RenderingCapabilities::detect();
        let manager = AdaptiveUIManager::new(caps.clone());
        assert_eq!(manager.device_type(), caps.device_type);
        assert_eq!(manager.ui_complexity(), caps.ui_complexity);
    }

    #[test]
    fn test_desktop_renderer_selection() {
        let mut caps = RenderingCapabilities::detect();
        caps.device_type = DeviceType::Desktop;
        let manager = AdaptiveUIManager::new(caps);
        assert_eq!(manager.device_type(), DeviceType::Desktop);
    }

    #[test]
    fn test_phone_renderer_selection() {
        let mut caps = RenderingCapabilities::detect();
        caps.device_type = DeviceType::Phone;
        caps.ui_complexity = UIComplexity::Minimal;
        let manager = AdaptiveUIManager::new(caps);
        assert_eq!(manager.device_type(), DeviceType::Phone);
        assert_eq!(manager.ui_complexity(), UIComplexity::Minimal);
    }

    #[test]
    fn test_watch_renderer_selection() {
        let mut caps = RenderingCapabilities::detect();
        caps.device_type = DeviceType::Watch;
        caps.ui_complexity = UIComplexity::Essential;
        let manager = AdaptiveUIManager::new(caps);
        assert_eq!(manager.device_type(), DeviceType::Watch);
        assert_eq!(manager.ui_complexity(), UIComplexity::Essential);
    }

    #[test]
    fn test_unknown_device_defaults_to_desktop() {
        let mut caps = RenderingCapabilities::detect();
        caps.device_type = DeviceType::Unknown;
        let _manager = AdaptiveUIManager::new(caps);
        // Should not panic, defaults to desktop
    }
}
