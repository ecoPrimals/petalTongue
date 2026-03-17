// SPDX-License-Identifier: AGPL-3.0-or-later
//! Simple Sensory UI (Small display surface, touch, limited capabilities).

use eframe::egui;

use super::formatting::{
    format_cpu_metrics, format_memory_metrics, format_proprioception_summary,
    format_topology_edges, format_topology_nodes,
};
use crate::sensory_ui::manager::SensoryUIRenderer;

/// Simple renderer for small display surfaces and touch input.
pub struct SimpleSensoryUI;

impl SimpleSensoryUI {
    pub const fn new() -> Self {
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
            ui.label(format_topology_nodes(stats.node_count));
            ui.label(format_topology_edges(stats.edge_count));
        });
    }

    fn render_metrics(
        &mut self,
        ui: &mut egui::Ui,
        metrics: Option<&petal_tongue_core::SystemMetrics>,
    ) {
        if let Some(m) = metrics {
            ui.group(|ui| {
                ui.label(format_cpu_metrics(m.system.cpu_percent.into()));
                ui.add(egui::ProgressBar::new(m.system.cpu_percent / 100.0));

                ui.label(format_memory_metrics(m.system.memory_percent.into()));
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
            let pairs = format_proprioception_summary(
                p.health.percentage,
                &format!("{:?}", p.health.status),
                p.confidence,
            );
            ui.group(|ui| {
                ui.label(egui::RichText::new("System Health").heading());
                for (label, value) in pairs {
                    ui.label(format!("{label}: {value}"));
                }
                ui.add(egui::ProgressBar::new(p.health.percentage / 100.0));
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
        PointerInputCapability, SensoryCapabilities, VisualOutputCapability,
    };

    #[test]
    fn test_simple_renderer_headless() {
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::TwoD {
                resolution: (800, 600),
                refresh_rate: 60,
                color_depth: 8,
                size_mm: None,
            }],
            pointer_inputs: vec![PointerInputCapability::TwoD {
                precision: 1.0,
                has_wheel: false,
                has_pressure: false,
                button_count: 1,
            }],
            ..Default::default()
        };
        let mut manager = SensoryUIManager::with_capabilities(caps);
        let graph = petal_tongue_core::GraphEngine::new();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_topology(ui, &graph);
                manager.render_metrics(ui, None);
            });
        });
    }

    #[test]
    fn test_simple_renderer_with_metrics_and_proprioception() {
        use petal_tongue_core::metrics::{NeuralApiMetrics, SystemResourceMetrics};
        use petal_tongue_core::proprioception::{HealthData, HealthStatus};

        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::TwoD {
                resolution: (800, 600),
                refresh_rate: 60,
                color_depth: 8,
                size_mm: None,
            }],
            pointer_inputs: vec![PointerInputCapability::TwoD {
                precision: 1.0,
                has_wheel: false,
                has_pressure: false,
                button_count: 1,
            }],
            ..Default::default()
        };
        let mut manager = SensoryUIManager::with_capabilities(caps);
        let metrics = petal_tongue_core::SystemMetrics {
            timestamp: chrono::Utc::now(),
            system: SystemResourceMetrics {
                cpu_percent: 60.0,
                memory_used_mb: 1024,
                memory_total_mb: 2048,
                memory_percent: 50.0,
                uptime_seconds: 500,
            },
            neural_api: NeuralApiMetrics {
                family_id: "t".to_string(),
                active_primals: 2,
                graphs_available: 1,
                active_executions: 1,
            },
        };
        let mut proprio = petal_tongue_core::ProprioceptionData::empty("t");
        proprio.health = HealthData {
            percentage: 95.0,
            status: HealthStatus::Healthy,
        };
        proprio.confidence = 99.0;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_metrics(ui, Some(&metrics));
                manager.render_proprioception(ui, Some(&proprio));
            });
        });
    }

    #[test]
    fn test_simple_renderer_empty_topology() {
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::TwoD {
                resolution: (800, 600),
                refresh_rate: 60,
                color_depth: 8,
                size_mm: None,
            }],
            pointer_inputs: vec![PointerInputCapability::TwoD {
                precision: 1.0,
                has_wheel: false,
                has_pressure: false,
                button_count: 1,
            }],
            ..Default::default()
        };
        let mut manager = SensoryUIManager::with_capabilities(caps);
        let graph = petal_tongue_core::GraphEngine::new();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_topology(ui, &graph);
            });
        });
    }
}
