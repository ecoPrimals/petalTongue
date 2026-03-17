// SPDX-License-Identifier: AGPL-3.0-or-later
//! Device-specific UI renderers.

use petal_tongue_core::{PrimalInfo, RenderingCapabilities};

use super::AdaptiveUIRenderer;
use super::formatting::{
    count_healthy_primals, format_cli_primal_line, format_cli_primal_status,
    format_desktop_primal_indicator, format_metrics_line, format_phone_primal_color_rgb,
    format_phone_primal_icon, format_topology_node_count, format_watch_health_summary,
    format_watch_topology_count, watch_health_all_ok,
};

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
        ui.heading("🕸️ Topology");
        ui.separator();
        ui.label(format!("Connected primals: {}", primals.len()));
        ui.label("(Full graph visualization would go here)");
    }

    fn render_metrics(&self, ui: &mut egui::Ui, metrics_data: &str, _caps: &RenderingCapabilities) {
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
        ui.label(format!("📱 {} primals connected", primals.len()));
        ui.small("Tap primal for details");
    }

    fn render_metrics(
        &self,
        ui: &mut egui::Ui,
        _metrics_data: &str,
        _caps: &RenderingCapabilities,
    ) {
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

pub(super) fn create_renderer(
    device: petal_tongue_core::DeviceType,
) -> Box<dyn AdaptiveUIRenderer> {
    use petal_tongue_core::DeviceType;

    match device {
        DeviceType::Desktop => Box::new(DesktopUIRenderer::new()),
        DeviceType::Phone => Box::new(PhoneUIRenderer::new()),
        DeviceType::Watch => Box::new(WatchUIRenderer::new()),
        DeviceType::CLI => Box::new(CliUIRenderer::new()),
        DeviceType::Tablet => Box::new(TabletUIRenderer::new()),
        DeviceType::TV => Box::new(TvUIRenderer::new()),
        DeviceType::Unknown => Box::new(DesktopUIRenderer::new()),
    }
}
