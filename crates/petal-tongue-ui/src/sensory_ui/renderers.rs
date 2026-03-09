// SPDX-License-Identifier: AGPL-3.0-only
//! Sensory UI renderer implementations for each complexity level.

use eframe::egui;

use super::manager::SensoryUIRenderer;

// ============================================================================
// Minimal Sensory UI (Audio-only, very limited capabilities)
// ============================================================================

/// Minimal renderer for audio-only or very limited capabilities.
pub(super) struct MinimalSensoryUI;

impl MinimalSensoryUI {
    pub(super) fn new() -> Self {
        Self
    }
}

impl SensoryUIRenderer for MinimalSensoryUI {
    fn render_primal_list(&mut self, ui: &mut egui::Ui, primals: &[petal_tongue_core::PrimalInfo]) {
        // Minimal text-only list
        ui.label(format!("{} primals detected", primals.len()));
        for primal in primals {
            ui.label(format!("• {}", primal.name));
        }
    }

    fn render_topology(
        &mut self,
        ui: &mut egui::Ui,
        graph_engine: &petal_tongue_core::GraphEngine,
    ) {
        let stats = graph_engine.stats();
        ui.label(format!(
            "Topology: {} nodes, {} edges",
            stats.node_count, stats.edge_count
        ));
    }

    fn render_metrics(
        &mut self,
        ui: &mut egui::Ui,
        metrics: Option<&petal_tongue_core::SystemMetrics>,
    ) {
        if let Some(m) = metrics {
            ui.label(format!("CPU: {:.1}%", m.system.cpu_percent));
            ui.label(format!("Memory: {:.1}%", m.system.memory_percent));
        }
    }

    fn render_proprioception(
        &mut self,
        ui: &mut egui::Ui,
        proprioception: Option<&petal_tongue_core::ProprioceptionData>,
    ) {
        if let Some(p) = proprioception {
            ui.label(format!("Health: {:.0}%", p.health.percentage));
            ui.label(format!("Status: {}", p.health.status));
        }
    }
}

// ============================================================================
// Simple Sensory UI (Small screen, touch, limited capabilities)
// ============================================================================

/// Simple renderer for small screens and touch input.
pub(super) struct SimpleSensoryUI;

impl SimpleSensoryUI {
    pub(super) fn new() -> Self {
        Self
    }
}

impl SensoryUIRenderer for SimpleSensoryUI {
    fn render_primal_list(&mut self, ui: &mut egui::Ui, primals: &[petal_tongue_core::PrimalInfo]) {
        // Touch-friendly large tap targets
        for primal in primals {
            ui.group(|ui| {
                ui.set_min_height(40.0); // Large touch target
                ui.label(egui::RichText::new(&primal.name).size(16.0));
            });
            ui.add_space(4.0);
        }
    }

    fn render_topology(
        &mut self,
        ui: &mut egui::Ui,
        graph_engine: &petal_tongue_core::GraphEngine,
    ) {
        let stats = graph_engine.stats();
        ui.group(|ui| {
            ui.label(egui::RichText::new("Topology").heading());
            ui.label(format!("Nodes: {}", stats.node_count));
            ui.label(format!("Edges: {}", stats.edge_count));
        });
    }

    fn render_metrics(
        &mut self,
        ui: &mut egui::Ui,
        metrics: Option<&petal_tongue_core::SystemMetrics>,
    ) {
        if let Some(m) = metrics {
            ui.group(|ui| {
                ui.label(format!("CPU {:.1}%", m.system.cpu_percent));
                ui.add(egui::ProgressBar::new(m.system.cpu_percent / 100.0));

                ui.label(format!("Mem {:.1}%", m.system.memory_percent));
                ui.add(egui::ProgressBar::new(m.system.memory_percent / 100.0));
            });
        }
    }

    fn render_proprioception(
        &mut self,
        ui: &mut egui::Ui,
        proprioception: Option<&petal_tongue_core::ProprioceptionData>,
    ) {
        if let Some(p) = proprioception {
            ui.group(|ui| {
                ui.label(egui::RichText::new("System Health").heading());
                ui.label(format!("{:.0}%", p.health.percentage));
                ui.label(format!("{:?}", p.health.status));
                ui.add(egui::ProgressBar::new(p.health.percentage / 100.0));
            });
        }
    }
}

// ============================================================================
// Standard Sensory UI (Desktop with mouse + keyboard)
// ============================================================================

/// Standard renderer for desktop with mouse and keyboard.
pub(super) struct StandardSensoryUI;

impl StandardSensoryUI {
    pub(super) fn new() -> Self {
        Self
    }
}

impl SensoryUIRenderer for StandardSensoryUI {
    fn render_primal_list(&mut self, ui: &mut egui::Ui, primals: &[petal_tongue_core::PrimalInfo]) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for primal in primals {
                ui.group(|ui| {
                    ui.label(egui::RichText::new(&primal.name).strong());
                    ui.label(format!("Type: {}", primal.primal_type));
                });
            }
        });
    }

    fn render_topology(
        &mut self,
        ui: &mut egui::Ui,
        graph_engine: &petal_tongue_core::GraphEngine,
    ) {
        let stats = graph_engine.stats();
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Topology").heading());
            ui.label(format!("Nodes: {}", stats.node_count));
            ui.label(format!("Edges: {}", stats.edge_count));
            ui.label(format!("Avg Degree: {:.1}", stats.avg_degree));
        });
    }

    fn render_metrics(
        &mut self,
        ui: &mut egui::Ui,
        metrics: Option<&petal_tongue_core::SystemMetrics>,
    ) {
        if let Some(m) = metrics {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("System Metrics").heading());

                ui.label(format!("CPU: {:.1}%", m.system.cpu_percent));
                ui.add(egui::ProgressBar::new(m.system.cpu_percent / 100.0).show_percentage());

                ui.label(format!("Memory: {:.1}%", m.system.memory_percent));
                ui.add(egui::ProgressBar::new(m.system.memory_percent / 100.0).show_percentage());
            });
        }
    }

    fn render_proprioception(
        &mut self,
        ui: &mut egui::Ui,
        proprioception: Option<&petal_tongue_core::ProprioceptionData>,
    ) {
        if let Some(p) = proprioception {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Proprioception").heading());
                ui.label(format!("Health: {:.0}%", p.health.percentage));
                ui.label(format!("Status: {:?}", p.health.status));
                ui.label(format!("Confidence: {:.0}%", p.confidence));
            });
        }
    }
}

// ============================================================================
// Rich Sensory UI (High-res desktop with precision input)
// ============================================================================

/// Rich renderer for high-resolution desktop with precision input.
pub(super) struct RichSensoryUI;

impl RichSensoryUI {
    pub(super) fn new() -> Self {
        Self
    }
}

impl SensoryUIRenderer for RichSensoryUI {
    fn render_primal_list(&mut self, ui: &mut egui::Ui, primals: &[petal_tongue_core::PrimalInfo]) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("primals_grid")
                .striped(true)
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Name").strong());
                    ui.label(egui::RichText::new("Type").strong());
                    ui.label(egui::RichText::new("Capabilities").strong());
                    ui.end_row();

                    for primal in primals {
                        ui.label(&primal.name);
                        ui.label(&primal.primal_type);
                        ui.label(format!("{} caps", primal.capabilities.len()));
                        ui.end_row();
                    }
                });
        });
    }

    fn render_topology(
        &mut self,
        ui: &mut egui::Ui,
        graph_engine: &petal_tongue_core::GraphEngine,
    ) {
        let stats = graph_engine.stats();
        ui.vertical(|ui| {
            ui.label(egui::RichText::new("Network Topology").heading());

            egui::Grid::new("topology_stats").show(ui, |ui| {
                ui.label("Nodes:");
                ui.label(stats.node_count.to_string());
                ui.end_row();

                ui.label("Edges:");
                ui.label(stats.edge_count.to_string());
                ui.end_row();

                ui.label("Avg Degree:");
                ui.label(format!("{:.1}", stats.avg_degree));
                ui.end_row();
            });
        });
    }

    fn render_metrics(
        &mut self,
        ui: &mut egui::Ui,
        metrics: Option<&petal_tongue_core::SystemMetrics>,
    ) {
        if let Some(m) = metrics {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("System Metrics").heading());

                egui::Grid::new("metrics_grid").show(ui, |ui| {
                    ui.label("CPU:");
                    ui.add(
                        egui::ProgressBar::new(m.system.cpu_percent / 100.0)
                            .show_percentage()
                            .desired_width(150.0),
                    );
                    ui.label(format!("{:.1}%", m.system.cpu_percent));
                    ui.end_row();

                    ui.label("Memory:");
                    ui.add(
                        egui::ProgressBar::new(m.system.memory_percent / 100.0)
                            .show_percentage()
                            .desired_width(150.0),
                    );
                    ui.label(format!("{:.1}%", m.system.memory_percent));
                    ui.end_row();
                });
            });
        }
    }

    fn render_proprioception(
        &mut self,
        ui: &mut egui::Ui,
        proprioception: Option<&petal_tongue_core::ProprioceptionData>,
    ) {
        if let Some(p) = proprioception {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("SAME DAVE Proprioception").heading());

                egui::Grid::new("proprio_grid").show(ui, |ui| {
                    ui.label("Health:");
                    ui.add(
                        egui::ProgressBar::new(p.health.percentage / 100.0)
                            .show_percentage()
                            .desired_width(150.0),
                    );
                    ui.label(format!("{:?}", p.health.status));
                    ui.end_row();

                    ui.label("Confidence:");
                    ui.add(
                        egui::ProgressBar::new(p.confidence / 100.0)
                            .show_percentage()
                            .desired_width(150.0),
                    );
                    ui.label(format!("{:.0}%", p.confidence));
                    ui.end_row();
                });
            });
        }
    }
}

// ============================================================================
// Immersive Sensory UI (VR/AR with spatial audio and haptics)
// ============================================================================

/// Immersive renderer for VR/AR with spatial audio and haptics.
pub(super) struct ImmersiveSensoryUI;

impl ImmersiveSensoryUI {
    pub(super) fn new() -> Self {
        Self
    }
}

impl SensoryUIRenderer for ImmersiveSensoryUI {
    fn render_primal_list(&mut self, ui: &mut egui::Ui, primals: &[petal_tongue_core::PrimalInfo]) {
        // In full VR implementation, this would render in 3D space
        // For now, use rich 2D rendering
        ui.label(egui::RichText::new("🌌 Immersive Mode").heading());
        ui.label("(3D spatial rendering would appear here)");

        for primal in primals {
            ui.label(format!("🔮 {}", primal.name));
        }
    }

    fn render_topology(
        &mut self,
        ui: &mut egui::Ui,
        graph_engine: &petal_tongue_core::GraphEngine,
    ) {
        let stats = graph_engine.stats();
        ui.label(egui::RichText::new("🕸️ 3D Topology").heading());
        ui.label("(3D graph rendering would appear here)");
        ui.label(format!(
            "Nodes: {} • Edges: {}",
            stats.node_count, stats.edge_count
        ));
    }

    fn render_metrics(
        &mut self,
        ui: &mut egui::Ui,
        metrics: Option<&petal_tongue_core::SystemMetrics>,
    ) {
        if let Some(m) = metrics {
            ui.label(egui::RichText::new("📊 Spatial Metrics").heading());
            ui.label("(Floating 3D metrics panels would appear here)");
            ui.label(format!(
                "CPU: {:.1}% | Memory: {:.1}%",
                m.system.cpu_percent, m.system.memory_percent
            ));
        }
    }

    fn render_proprioception(
        &mut self,
        ui: &mut egui::Ui,
        proprioception: Option<&petal_tongue_core::ProprioceptionData>,
    ) {
        if let Some(p) = proprioception {
            ui.label(egui::RichText::new("🧠 Holographic Health").heading());
            ui.label("(3D health visualization would appear here)");
            ui.label(format!(
                "Health: {:.0}% | Confidence: {:.0}%",
                p.health.percentage, p.confidence
            ));
        }
    }
}
