// SPDX-License-Identifier: AGPL-3.0-or-later
//! Standard Sensory UI (Desktop with mouse + keyboard).

use eframe::egui;

use super::formatting::{
    format_avg_degree, format_cpu_metrics, format_memory_metrics, format_proprioception_summary,
    format_topology_edges, format_topology_nodes,
};
use crate::sensory_ui::manager::SensoryUIRenderer;

/// Standard renderer for desktop with mouse and keyboard.
pub struct StandardSensoryUI;

impl StandardSensoryUI {
    pub const fn new() -> Self {
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
            ui.label(format_topology_nodes(stats.node_count));
            ui.label(format_topology_edges(stats.edge_count));
            ui.label(format_avg_degree(stats.avg_degree));
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

                ui.label(format_cpu_metrics(m.system.cpu_percent.into()));
                ui.add(egui::ProgressBar::new(m.system.cpu_percent / 100.0).show_percentage());

                ui.label(format_memory_metrics(m.system.memory_percent.into()));
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
            let pairs = format_proprioception_summary(
                p.health.percentage,
                &format!("{:?}", p.health.status),
                p.confidence,
            );
            ui.vertical(|ui| {
                ui.label(egui::RichText::new("Proprioception").heading());
                for (label, value) in pairs {
                    ui.label(format!("{label}: {value}"));
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensory_ui::manager::SensoryUIManager;
    use petal_tongue_core::sensory_capabilities::{
        KeyboardInputCapability, PointerInputCapability, SensoryCapabilities,
        VisualOutputCapability,
    };

    #[test]
    fn test_standard_renderer_headless() {
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
        let mut graph = petal_tongue_core::GraphEngine::new();
        graph.add_node(petal_tongue_core::PrimalInfo::new(
            petal_tongue_core::PrimalId::from("test"),
            "Test Primal",
            "Compute",
            "http://localhost",
            vec![],
            petal_tongue_core::PrimalHealthStatus::Healthy,
            0,
        ));

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_topology(ui, &graph);
                manager.render_proprioception(ui, None);
            });
        });
    }

    #[test]
    fn test_standard_renderer_with_metrics_and_proprioception() {
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
                cpu_percent: 33.0,
                memory_used_mb: 768,
                memory_total_mb: 2048,
                memory_percent: 37.5,
                uptime_seconds: 3600,
            },
            neural_api: NeuralApiMetrics {
                family_id: "t".to_string(),
                active_primals: 3,
                graphs_available: 2,
                active_executions: 0,
            },
        };
        let mut proprio = petal_tongue_core::ProprioceptionData::empty("t");
        proprio.health = HealthData {
            percentage: 50.0,
            status: HealthStatus::Critical,
        };
        proprio.confidence = 60.0;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_metrics(ui, Some(&metrics));
                manager.render_proprioception(ui, Some(&proprio));
            });
        });
    }
}
