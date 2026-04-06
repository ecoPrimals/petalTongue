// SPDX-License-Identifier: AGPL-3.0-or-later
//! Immersive Sensory UI (VR/AR with spatial audio and haptics).

use eframe::egui;

use super::formatting::{
    format_cpu_memory_combined, format_health_confidence, format_topology_edges,
    format_topology_nodes,
};
use crate::sensory_ui::manager::SensoryUIRenderer;

/// Immersive renderer for VR/AR with spatial audio and haptics.
pub struct ImmersiveSensoryUI;

impl ImmersiveSensoryUI {
    pub const fn new() -> Self {
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
            "{} • {}",
            format_topology_nodes(stats.node_count),
            format_topology_edges(stats.edge_count)
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
            ui.label(format_cpu_memory_combined(
                m.system.cpu_percent.into(),
                m.system.memory_percent.into(),
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
            ui.label(format_health_confidence(p.health.percentage, p.confidence));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensory_ui::manager::SensoryUIManager;
    use petal_tongue_core::metrics::{NeuralApiMetrics, SystemResourceMetrics};
    use petal_tongue_core::proprioception::{HealthData, HealthStatus};
    use petal_tongue_core::sensory_capabilities::{
        AudioOutputCapability, HapticOutputCapability, SensoryCapabilities, VisualOutputCapability,
    };

    #[test]
    fn test_immersive_renderer_headless() {
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::ThreeD {
                resolution_per_eye: (2160, 1200),
                field_of_view: (110.0, 90.0),
                refresh_rate: 90,
                has_depth_tracking: true,
                has_hand_tracking: true,
            }],
            audio_outputs: vec![AudioOutputCapability::Spatial {
                channels: 6,
                sample_rate: 48000,
                has_head_tracking: true,
            }],
            haptic_outputs: vec![HapticOutputCapability::SimpleVibration {
                intensity_levels: 255,
            }],
            ..Default::default()
        };
        let mut manager = SensoryUIManager::with_capabilities(caps);
        let metrics = petal_tongue_core::SystemMetrics {
            timestamp: chrono::Utc::now(),
            system: SystemResourceMetrics {
                cpu_percent: 42.0,
                memory_used_mb: 1024,
                memory_total_mb: 2048,
                memory_percent: 50.0,
                uptime_seconds: 7200,
            },
            neural_api: NeuralApiMetrics {
                family_id: "test".to_string(),
                active_primals: 3,
                graphs_available: 2,
                active_executions: 1,
            },
        };
        let mut proprio = petal_tongue_core::ProprioceptionData::empty("test");
        proprio.health = HealthData {
            percentage: 90.0,
            status: HealthStatus::Healthy,
        };
        proprio.confidence = 95.0;

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_metrics(ui, Some(&metrics));
                manager.render_proprioception(ui, Some(&proprio));
            });
        });
    }

    #[test]
    fn test_immersive_renderer_empty_primals() {
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::ThreeD {
                resolution_per_eye: (2160, 1200),
                field_of_view: (110.0, 90.0),
                refresh_rate: 90,
                has_depth_tracking: true,
                has_hand_tracking: true,
            }],
            audio_outputs: vec![AudioOutputCapability::Spatial {
                channels: 6,
                sample_rate: 48000,
                has_head_tracking: true,
            }],
            haptic_outputs: vec![HapticOutputCapability::SimpleVibration {
                intensity_levels: 255,
            }],
            ..Default::default()
        };
        let mut manager = SensoryUIManager::with_capabilities(caps);
        let primals: Vec<petal_tongue_core::PrimalInfo> = vec![];

        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                manager.render_primal_list(ui, &primals);
            });
        });
    }
}
