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
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::sensory_ui::manager::{SensoryUIManager, SensoryUIRenderer};
    use petal_tongue_core::PrimalHealthStatus;
    use petal_tongue_core::PrimalInfo;
    use petal_tongue_core::sensory_capabilities::UIComplexity as SensoryUIComplexity;
    use petal_tongue_core::sensory_capabilities::{
        PointerInputCapability, SensoryCapabilities, TouchInputCapability, VisualOutputCapability,
    };

    /// Touch without keyboard maps to [`SensoryUIComplexity::Simple`] and uses [`SimpleSensoryUI`].
    fn caps_simple_complexity() -> SensoryCapabilities {
        SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::TwoD {
                resolution: (800, 600),
                refresh_rate: 60,
                color_depth: 8,
                size_mm: None,
            }],
            touch_inputs: vec![TouchInputCapability {
                max_touch_points: 5,
                supports_pressure: false,
                supports_hover: false,
                screen_size_mm: None,
            }],
            ..Default::default()
        }
    }

    #[test]
    fn simple_sensory_ui_new_is_const() {
        const _R: SimpleSensoryUI = SimpleSensoryUI::new();
    }

    #[test]
    fn simple_renderer_trait_exercises_all_paths_directly() {
        use petal_tongue_core::metrics::{NeuralApiMetrics, SystemResourceMetrics};
        use petal_tongue_core::proprioception::{HealthData, HealthStatus};

        let primals = vec![
            PrimalInfo::new(
                "a",
                "Alpha",
                "t",
                "e",
                vec![],
                PrimalHealthStatus::Healthy,
                0,
            ),
            PrimalInfo::new(
                "b",
                "Beta",
                "t",
                "e",
                vec![],
                PrimalHealthStatus::Warning,
                0,
            ),
        ];
        let metrics = petal_tongue_core::SystemMetrics {
            timestamp: chrono::Utc::now(),
            system: SystemResourceMetrics {
                cpu_percent: 12.5,
                memory_used_mb: 512,
                memory_total_mb: 4096,
                memory_percent: 33.3,
                uptime_seconds: 10,
            },
            neural_api: NeuralApiMetrics {
                family_id: "fam".to_string(),
                active_primals: 1,
                graphs_available: 0,
                active_executions: 0,
            },
        };
        let mut proprio_some = petal_tongue_core::ProprioceptionData::empty("fam");
        proprio_some.health = HealthData {
            percentage: 42.0,
            status: HealthStatus::Degraded,
        };
        proprio_some.confidence = 77.5;

        let graph = petal_tongue_core::GraphEngine::new();
        let mut renderer = SimpleSensoryUI::new();

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                renderer.render_primal_list(ui, &[]);
                renderer.render_metrics(ui, None);
                renderer.render_proprioception(ui, None);

                renderer.render_primal_list(ui, &primals);
                renderer.render_topology(ui, &graph);
                renderer.render_metrics(ui, Some(&metrics));
                renderer.render_proprioception(ui, Some(&proprio_some));

                let mut critical = petal_tongue_core::ProprioceptionData::empty("x");
                critical.health = HealthData {
                    percentage: 1.0,
                    status: HealthStatus::Critical,
                };
                critical.confidence = 0.0;
                renderer.render_proprioception(ui, Some(&critical));
            });
        });
    }

    #[test]
    fn simple_complexity_manager_dispatches_simple_renderer() {
        let graph = petal_tongue_core::GraphEngine::new();
        let mut manager = SensoryUIManager::with_capabilities(caps_simple_complexity());
        assert_eq!(manager.ui_complexity(), SensoryUIComplexity::Simple);

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_primal_list(ui, &[]);
                manager.render_topology(ui, &graph);
                manager.render_metrics(ui, None);
                manager.render_proprioception(ui, None);
            });
        });
    }

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
