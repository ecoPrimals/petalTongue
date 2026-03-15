// SPDX-License-Identifier: AGPL-3.0-only
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
    DeviceType, PrimalHealthStatus, PrimalInfo, RenderingCapabilities, UIComplexity,
};

// ============================================================================
// Pure layout/adaptation logic (testable, no egui)
// ============================================================================

/// Device type to use for rendering (Unknown defaults to Desktop)
#[must_use]
pub(crate) const fn effective_device_for_rendering(device_type: DeviceType) -> DeviceType {
    match device_type {
        DeviceType::Unknown => DeviceType::Desktop,
        other => other,
    }
}

/// CLI-style status text for primal health
#[must_use]
pub(crate) const fn format_cli_primal_status(health: PrimalHealthStatus) -> &'static str {
    match health {
        PrimalHealthStatus::Healthy => "OK",
        PrimalHealthStatus::Warning => "WARN",
        PrimalHealthStatus::Critical => "CRIT",
        PrimalHealthStatus::Unknown => "UNKN",
    }
}

/// Count primals with healthy status
#[must_use]
pub(crate) fn count_healthy_primals(primals: &[PrimalInfo]) -> usize {
    primals
        .iter()
        .filter(|p| matches!(p.health, PrimalHealthStatus::Healthy))
        .count()
}

/// Watch-style health summary (healthy/total)
#[must_use]
pub(crate) fn format_watch_health_summary(healthy: usize, total: usize) -> String {
    if healthy == total {
        format!("✅ {healthy}/{total} OK")
    } else {
        format!("⚠️ {healthy}/{total}")
    }
}

#[must_use]
pub(crate) const fn watch_health_all_ok(healthy: usize, total: usize) -> bool {
    healthy == total
}

#[must_use]
pub(crate) fn format_cli_primal_line(status: &str, name: &str) -> String {
    format!("[{status}] {name}")
}

#[must_use]
pub(crate) fn format_topology_node_count(count: usize) -> String {
    format!("Topology: {count} nodes")
}

#[must_use]
pub(crate) fn format_metrics_line(metrics_data: &str) -> String {
    format!("Metrics: {metrics_data}")
}

#[must_use]
pub(crate) fn format_watch_topology_count(count: usize) -> String {
    format!("🕸️ {count}")
}

/// Phone-style status color RGB for primal health
#[must_use]
pub(crate) const fn format_phone_primal_color_rgb(health: PrimalHealthStatus) -> [u8; 3] {
    match health {
        PrimalHealthStatus::Healthy => [0, 255, 0],
        PrimalHealthStatus::Warning => [255, 255, 0],
        PrimalHealthStatus::Critical => [255, 0, 0],
        PrimalHealthStatus::Unknown => [128, 128, 128],
    }
}

/// Phone-style status icon for primal health
#[must_use]
pub(crate) const fn format_phone_primal_icon(health: PrimalHealthStatus) -> &'static str {
    match health {
        PrimalHealthStatus::Healthy => "✅",
        PrimalHealthStatus::Warning => "⚠️",
        PrimalHealthStatus::Critical => "❌",
        PrimalHealthStatus::Unknown => "❓",
    }
}

/// Desktop/tablet status indicator (text, rgb color)
#[must_use]
pub(crate) const fn format_desktop_primal_indicator(
    health: PrimalHealthStatus,
) -> (&'static str, [u8; 3]) {
    match health {
        PrimalHealthStatus::Healthy => ("●", [0, 255, 0]),
        PrimalHealthStatus::Warning => ("●", [255, 255, 0]),
        PrimalHealthStatus::Critical => ("●", [255, 0, 0]),
        PrimalHealthStatus::Unknown => ("○", [128, 128, 128]),
    }
}

/// Manages adaptive UI rendering across different devices
pub struct AdaptiveUIManager {
    capabilities: RenderingCapabilities,
    renderer: Box<dyn AdaptiveUIRenderer>,
}

impl AdaptiveUIManager {
    /// Create new adaptive UI manager with device detection
    pub fn new(capabilities: RenderingCapabilities) -> Self {
        let device = effective_device_for_rendering(capabilities.device_type);
        if capabilities.device_type == DeviceType::Unknown {
            tracing::warn!("Unknown device type, defaulting to desktop UI");
        }
        let renderer: Box<dyn AdaptiveUIRenderer> = match device {
            DeviceType::Desktop => Box::new(DesktopUIRenderer::new()),
            DeviceType::Phone => Box::new(PhoneUIRenderer::new()),
            DeviceType::Watch => Box::new(WatchUIRenderer::new()),
            DeviceType::CLI => Box::new(CliUIRenderer::new()),
            DeviceType::Tablet => Box::new(TabletUIRenderer::new()),
            DeviceType::TV => Box::new(TvUIRenderer::new()),
            DeviceType::Unknown => Box::new(DesktopUIRenderer::new()),
        };

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
    const fn new() -> Self {
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
                        let (text, rgb) = format_desktop_primal_indicator(primal.health);
                        let color = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                        ui.colored_label(color, text);

                        ui.vertical(|ui| {
                            ui.strong(&primal.name);
                            ui.label(format!("Type: {}", primal.primal_type));
                            ui.label(format!("Endpoint: {}", primal.endpoint));

                            if !primal.capabilities.is_empty() {
                                ui.horizontal_wrapped(|ui| {
                                    ui.label("Capabilities:");
                                    for cap in &primal.capabilities {
                                        ui.small(format!("🔹 {cap}"));
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
    const fn new() -> Self {
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
                    let icon = format_phone_primal_icon(primal.health);
                    let rgb = format_phone_primal_color_rgb(primal.health);
                    let color = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
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
    const fn new() -> Self {
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
        let healthy = count_healthy_primals(primals);
        let total = primals.len();
        let summary = format_watch_health_summary(healthy, total);
        let color = if watch_health_all_ok(healthy, total) {
            egui::Color32::GREEN
        } else {
            egui::Color32::YELLOW
        };
        ui.colored_label(color, summary);
    }

    fn render_topology(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        ui.label(format_watch_topology_count(primals.len()));
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
    const fn new() -> Self {
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
        for primal in primals {
            let status = format_cli_primal_status(primal.health);
            ui.monospace(format_cli_primal_line(status, &primal.name));
        }
    }

    fn render_topology(
        &self,
        ui: &mut egui::Ui,
        primals: &[PrimalInfo],
        _caps: &RenderingCapabilities,
    ) {
        ui.monospace(format_topology_node_count(primals.len()));
    }

    fn render_metrics(&self, ui: &mut egui::Ui, metrics_data: &str, _caps: &RenderingCapabilities) {
        ui.monospace(format_metrics_line(metrics_data));
    }
}

// ============================================================================
// Tablet UI Renderer (Simplified Complexity)
// ============================================================================

struct TabletUIRenderer;

impl TabletUIRenderer {
    const fn new() -> Self {
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
                    let (text, rgb) = format_desktop_primal_indicator(primal.health);
                    let color = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
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
    const fn new() -> Self {
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
                let (text, rgb) = format_desktop_primal_indicator(primal.health);
                let color = egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
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

    #[test]
    fn test_tablet_renderer_selection() {
        let mut caps = RenderingCapabilities::detect();
        caps.device_type = DeviceType::Tablet;
        let manager = AdaptiveUIManager::new(caps);
        assert_eq!(manager.device_type(), DeviceType::Tablet);
    }

    #[test]
    fn test_cli_renderer_selection() {
        let mut caps = RenderingCapabilities::detect();
        caps.device_type = DeviceType::CLI;
        caps.ui_complexity = UIComplexity::Essential;
        let manager = AdaptiveUIManager::new(caps);
        assert_eq!(manager.device_type(), DeviceType::CLI);
        assert_eq!(manager.ui_complexity(), UIComplexity::Essential);
    }

    #[test]
    fn test_tv_renderer_selection() {
        let mut caps = RenderingCapabilities::detect();
        caps.device_type = DeviceType::TV;
        let manager = AdaptiveUIManager::new(caps);
        assert_eq!(manager.device_type(), DeviceType::TV);
    }

    #[test]
    fn test_ui_complexity_levels() {
        let mut caps = RenderingCapabilities::detect();
        caps.ui_complexity = UIComplexity::Full;
        let manager = AdaptiveUIManager::new(caps);
        assert_eq!(manager.ui_complexity(), UIComplexity::Full);

        let mut caps = RenderingCapabilities::detect();
        caps.ui_complexity = UIComplexity::Simplified;
        let manager = AdaptiveUIManager::new(caps);
        assert_eq!(manager.ui_complexity(), UIComplexity::Simplified);
    }

    #[test]
    fn test_all_device_types_create_manager() {
        for device_type in [
            DeviceType::Desktop,
            DeviceType::Phone,
            DeviceType::Watch,
            DeviceType::CLI,
            DeviceType::Tablet,
            DeviceType::TV,
            DeviceType::Unknown,
        ] {
            let mut caps = RenderingCapabilities::detect();
            caps.device_type = device_type;
            let manager = AdaptiveUIManager::new(caps);
            assert_eq!(manager.device_type(), device_type);
        }
    }

    #[test]
    fn test_unknown_device_uses_desktop_renderer() {
        let mut caps = RenderingCapabilities::detect();
        caps.device_type = DeviceType::Unknown;
        let manager = AdaptiveUIManager::new(caps);
        // Unknown should default to desktop-like behavior (no panic)
        assert_eq!(manager.device_type(), DeviceType::Unknown);
    }

    #[test]
    fn test_effective_device_for_rendering() {
        assert_eq!(
            effective_device_for_rendering(DeviceType::Unknown),
            DeviceType::Desktop
        );
        assert_eq!(
            effective_device_for_rendering(DeviceType::Phone),
            DeviceType::Phone
        );
        assert_eq!(
            effective_device_for_rendering(DeviceType::Desktop),
            DeviceType::Desktop
        );
    }

    #[test]
    fn test_format_cli_primal_status() {
        assert_eq!(format_cli_primal_status(PrimalHealthStatus::Healthy), "OK");
        assert_eq!(
            format_cli_primal_status(PrimalHealthStatus::Warning),
            "WARN"
        );
        assert_eq!(
            format_cli_primal_status(PrimalHealthStatus::Critical),
            "CRIT"
        );
        assert_eq!(
            format_cli_primal_status(PrimalHealthStatus::Unknown),
            "UNKN"
        );
    }

    #[test]
    fn test_format_watch_health_summary() {
        assert_eq!(format_watch_health_summary(5, 5), "✅ 5/5 OK");
        assert_eq!(format_watch_health_summary(3, 5), "⚠️ 3/5");
    }

    #[test]
    fn test_format_phone_primal_icon() {
        assert_eq!(format_phone_primal_icon(PrimalHealthStatus::Healthy), "✅");
        assert_eq!(format_phone_primal_icon(PrimalHealthStatus::Warning), "⚠️");
        assert_eq!(format_phone_primal_icon(PrimalHealthStatus::Critical), "❌");
        assert_eq!(format_phone_primal_icon(PrimalHealthStatus::Unknown), "❓");
    }

    #[test]
    fn test_format_desktop_primal_indicator() {
        let (text, rgb) = format_desktop_primal_indicator(PrimalHealthStatus::Healthy);
        assert_eq!(text, "●");
        assert_eq!(rgb, [0, 255, 0]);

        let (text, rgb) = format_desktop_primal_indicator(PrimalHealthStatus::Warning);
        assert_eq!(text, "●");
        assert_eq!(rgb, [255, 255, 0]);

        let (text, rgb) = format_desktop_primal_indicator(PrimalHealthStatus::Critical);
        assert_eq!(text, "●");
        assert_eq!(rgb, [255, 0, 0]);

        let (text, rgb) = format_desktop_primal_indicator(PrimalHealthStatus::Unknown);
        assert_eq!(text, "○");
        assert_eq!(rgb, [128, 128, 128]);
    }

    #[test]
    fn test_format_watch_health_summary_edge_cases() {
        assert_eq!(format_watch_health_summary(0, 5), "⚠️ 0/5");
        assert_eq!(format_watch_health_summary(1, 1), "✅ 1/1 OK");
        assert_eq!(format_watch_health_summary(0, 0), "✅ 0/0 OK");
    }

    #[test]
    fn test_format_phone_primal_color_rgb() {
        let rgb = format_phone_primal_color_rgb(PrimalHealthStatus::Healthy);
        assert_eq!(rgb, [0, 255, 0]);
        let rgb = format_phone_primal_color_rgb(PrimalHealthStatus::Warning);
        assert_eq!(rgb, [255, 255, 0]);
        let rgb = format_phone_primal_color_rgb(PrimalHealthStatus::Critical);
        assert_eq!(rgb, [255, 0, 0]);
        let rgb = format_phone_primal_color_rgb(PrimalHealthStatus::Unknown);
        assert_eq!(rgb, [128, 128, 128]);
    }

    #[test]
    fn test_watch_health_all_ok() {
        assert!(watch_health_all_ok(5, 5));
        assert!(!watch_health_all_ok(3, 5));
    }

    #[test]
    fn test_format_cli_primal_line() {
        assert_eq!(format_cli_primal_line("OK", "primal1"), "[OK] primal1");
    }

    #[test]
    fn test_format_topology_node_count() {
        assert_eq!(format_topology_node_count(10), "Topology: 10 nodes");
    }

    #[test]
    fn test_format_metrics_line() {
        assert_eq!(format_metrics_line("cpu: 50%"), "Metrics: cpu: 50%");
    }

    #[test]
    fn test_format_watch_topology_count() {
        assert_eq!(format_watch_topology_count(3), "🕸️ 3");
    }

    #[test]
    fn test_count_healthy_primals() {
        use petal_tongue_core::{PrimalId, PrimalInfo};
        let primals = vec![
            PrimalInfo::new(
                PrimalId::from("a"),
                "a",
                "",
                "http://localhost",
                vec![],
                PrimalHealthStatus::Healthy,
                0,
            ),
            PrimalInfo::new(
                PrimalId::from("b"),
                "b",
                "",
                "http://localhost",
                vec![],
                PrimalHealthStatus::Warning,
                0,
            ),
        ];
        assert_eq!(count_healthy_primals(&primals), 1);
        assert_eq!(count_healthy_primals(&[]), 0);
    }
}
