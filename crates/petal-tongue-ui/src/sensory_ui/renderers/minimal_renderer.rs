// SPDX-License-Identifier: AGPL-3.0-or-later
//! Minimal Sensory UI (Audio-only, very limited capabilities).

use eframe::egui;

use super::formatting::{
    format_cpu_metrics, format_memory_metrics, format_proprioception_summary,
    format_topology_summary,
};
use crate::sensory_ui::manager::SensoryUIRenderer;

/// Minimal renderer for audio-only or very limited capabilities.
pub struct MinimalSensoryUI;

impl MinimalSensoryUI {
    pub const fn new() -> Self {
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
        ui.label(format_topology_summary(stats.node_count, stats.edge_count));
    }

    fn render_metrics(
        &mut self,
        ui: &mut egui::Ui,
        metrics: Option<&petal_tongue_core::SystemMetrics>,
    ) {
        if let Some(m) = metrics {
            ui.label(format_cpu_metrics(m.system.cpu_percent.into()));
            ui.label(format_memory_metrics(m.system.memory_percent.into()));
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
            for (label, value) in pairs {
                ui.label(format!("{label}: {value}"));
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::sensory_ui::manager::SensoryUIManager;
    use petal_tongue_core::sensory_capabilities::{AudioOutputCapability, SensoryCapabilities};
    use petal_tongue_core::{PrimalHealthStatus, PrimalId};

    #[test]
    fn test_minimal_renderer_headless() {
        let caps = SensoryCapabilities {
            audio_outputs: vec![AudioOutputCapability::Stereo {
                sample_rate: 48000,
                bit_depth: 16,
            }],
            ..Default::default()
        };
        let mut manager = SensoryUIManager::with_capabilities(caps);
        let primals: Vec<petal_tongue_core::PrimalInfo> = vec![];

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_primal_list(ui, &primals);
                manager.render_metrics(ui, None);
                manager.render_proprioception(ui, None);
            });
        });
    }

    #[test]
    fn test_minimal_renderer_with_metrics_and_proprioception() {
        use petal_tongue_core::metrics::{NeuralApiMetrics, SystemResourceMetrics};
        use petal_tongue_core::proprioception::{HealthData, HealthStatus};

        let caps = SensoryCapabilities {
            audio_outputs: vec![AudioOutputCapability::Stereo {
                sample_rate: 48000,
                bit_depth: 16,
            }],
            ..Default::default()
        };
        let mut manager = SensoryUIManager::with_capabilities(caps);
        let metrics = petal_tongue_core::SystemMetrics {
            timestamp: chrono::Utc::now(),
            system: SystemResourceMetrics {
                cpu_percent: 25.0,
                memory_used_mb: 512,
                memory_total_mb: 2048,
                memory_percent: 25.0,
                uptime_seconds: 100,
            },
            neural_api: NeuralApiMetrics {
                family_id: "t".to_string(),
                active_primals: 1,
                graphs_available: 1,
                active_executions: 0,
            },
        };
        let mut proprio = petal_tongue_core::ProprioceptionData::empty("t");
        proprio.health = HealthData {
            percentage: 75.0,
            status: HealthStatus::Degraded,
        };
        proprio.confidence = 80.0;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_metrics(ui, Some(&metrics));
                manager.render_proprioception(ui, Some(&proprio));
            });
        });
    }

    #[test]
    fn test_minimal_renderer_with_primals() {
        let caps = SensoryCapabilities {
            audio_outputs: vec![AudioOutputCapability::Stereo {
                sample_rate: 48000,
                bit_depth: 16,
            }],
            ..Default::default()
        };
        let mut manager = SensoryUIManager::with_capabilities(caps);
        let primals = vec![
            petal_tongue_core::PrimalInfo::new(
                PrimalId::from("p1"),
                "Primal One",
                "Compute",
                "http://localhost:8080",
                vec![],
                PrimalHealthStatus::Healthy,
                0,
            ),
            petal_tongue_core::PrimalInfo::new(
                PrimalId::from("p2"),
                "Primal Two",
                "Storage",
                "http://localhost:8081",
                vec!["store".to_string()],
                PrimalHealthStatus::Warning,
                1,
            ),
        ];

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_primal_list(ui, &primals);
            });
        });
    }
}
