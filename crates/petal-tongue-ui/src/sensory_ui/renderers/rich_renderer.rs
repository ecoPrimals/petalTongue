// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rich Sensory UI (High-res desktop with precision input).

use eframe::egui;

use super::formatting::{format_capabilities_count, format_cpu_metrics, format_memory_metrics};
use crate::sensory_ui::manager::SensoryUIRenderer;

/// Rich renderer for high-resolution desktop with precision input.
pub struct RichSensoryUI;

impl RichSensoryUI {
    pub const fn new() -> Self {
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
                        ui.label(format_capabilities_count(primal.capabilities.len()));
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
                    ui.label(format_cpu_metrics(m.system.cpu_percent.into()));
                    ui.end_row();

                    ui.label("Memory:");
                    ui.add(
                        egui::ProgressBar::new(m.system.memory_percent / 100.0)
                            .show_percentage()
                            .desired_width(150.0),
                    );
                    ui.label(format_memory_metrics(m.system.memory_percent.into()));
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::sensory_ui::manager::SensoryUIManager;
    use petal_tongue_core::sensory_capabilities::{
        KeyboardInputCapability, PointerInputCapability, SensoryCapabilities,
        VisualOutputCapability,
    };
    use petal_tongue_core::{PrimalHealthStatus, PrimalId};

    #[test]
    fn test_rich_renderer_with_capabilities_headless() {
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::TwoD {
                resolution: (1920, 1080),
                refresh_rate: 60,
                color_depth: 8,
                size_mm: None,
            }],
            pointer_inputs: vec![PointerInputCapability::TwoD {
                precision: 1.5,
                has_wheel: true,
                has_pressure: false,
                button_count: 3,
            }],
            keyboard_inputs: vec![KeyboardInputCapability::Physical {
                layout: "QWERTY".to_string(),
                has_numpad: true,
                modifier_keys: 4,
            }],
            ..Default::default()
        };
        let mut manager = SensoryUIManager::with_capabilities(caps);
        let primals = vec![petal_tongue_core::PrimalInfo::new(
            PrimalId::from("p1"),
            "Rich Primal",
            "Compute",
            "http://localhost:8080",
            vec![
                "compute".to_string(),
                "storage".to_string(),
                "network".to_string(),
            ],
            PrimalHealthStatus::Healthy,
            0,
        )];

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_primal_list(ui, &primals);
            });
        });
    }

    #[test]
    fn test_rich_renderer_with_metrics_and_proprioception() {
        use petal_tongue_core::metrics::{NeuralApiMetrics, SystemResourceMetrics};
        use petal_tongue_core::proprioception::{HealthData, HealthStatus};

        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::TwoD {
                resolution: (1920, 1080),
                refresh_rate: 60,
                color_depth: 8,
                size_mm: None,
            }],
            pointer_inputs: vec![PointerInputCapability::TwoD {
                precision: 1.5,
                has_wheel: true,
                has_pressure: false,
                button_count: 3,
            }],
            keyboard_inputs: vec![KeyboardInputCapability::Physical {
                layout: "QWERTY".to_string(),
                has_numpad: true,
                modifier_keys: 4,
            }],
            ..Default::default()
        };
        let mut manager = SensoryUIManager::with_capabilities(caps);
        let metrics = petal_tongue_core::SystemMetrics {
            timestamp: chrono::Utc::now(),
            system: SystemResourceMetrics {
                cpu_percent: 88.0,
                memory_used_mb: 1800,
                memory_total_mb: 2048,
                memory_percent: 88.0,
                uptime_seconds: 86400,
            },
            neural_api: NeuralApiMetrics {
                family_id: "t".to_string(),
                active_primals: 5,
                graphs_available: 3,
                active_executions: 2,
            },
        };
        let mut proprio = petal_tongue_core::ProprioceptionData::empty("t");
        proprio.health = HealthData {
            percentage: 100.0,
            status: HealthStatus::Healthy,
        };
        proprio.confidence = 100.0;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_metrics(ui, Some(&metrics));
                manager.render_proprioception(ui, Some(&proprio));
            });
        });
    }

    #[test]
    fn test_rich_renderer_topology_with_edges() {
        use petal_tongue_core::{GraphEngine, PrimalHealthStatus};

        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::TwoD {
                resolution: (1920, 1080),
                refresh_rate: 60,
                color_depth: 8,
                size_mm: None,
            }],
            pointer_inputs: vec![PointerInputCapability::TwoD {
                precision: 1.5,
                has_wheel: true,
                has_pressure: false,
                button_count: 3,
            }],
            keyboard_inputs: vec![KeyboardInputCapability::Physical {
                layout: "QWERTY".to_string(),
                has_numpad: true,
                modifier_keys: 4,
            }],
            ..Default::default()
        };
        let mut manager = SensoryUIManager::with_capabilities(caps);
        let mut graph = GraphEngine::new();
        graph.add_node(petal_tongue_core::PrimalInfo::new(
            PrimalId::from("a"),
            "A",
            "Compute",
            "http://a",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ));
        graph.add_node(petal_tongue_core::PrimalInfo::new(
            PrimalId::from("b"),
            "B",
            "Compute",
            "http://b",
            vec![],
            PrimalHealthStatus::Healthy,
            0,
        ));

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_topology(ui, &graph);
            });
        });
    }
}
