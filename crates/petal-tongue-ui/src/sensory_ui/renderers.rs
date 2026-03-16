// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sensory UI renderer implementations for each complexity level.

use eframe::egui;

use super::manager::SensoryUIRenderer;

// ============================================================================
// Pure formatting functions (testable, no egui)
// ============================================================================

/// Format topology summary as single line
#[must_use]
pub fn format_topology_summary(node_count: usize, edge_count: usize) -> String {
    format!("Topology: {node_count} nodes, {edge_count} edges")
}

/// Format CPU metric
#[must_use]
pub fn format_cpu_metrics(cpu_percent: f64) -> String {
    format!("CPU: {cpu_percent:.1}%")
}

/// Format memory metric
#[must_use]
pub fn format_memory_metrics(memory_percent: f64) -> String {
    format!("Memory: {memory_percent:.1}%")
}

/// Format proprioception as label-value pairs for display
#[must_use]
pub fn format_proprioception_summary(
    health_percentage: f32,
    status: &str,
    confidence: f32,
) -> Vec<(String, String)> {
    vec![
        ("Health".to_string(), format!("{health_percentage:.0}%")),
        ("Status".to_string(), status.to_string()),
        ("Confidence".to_string(), format!("{confidence:.1}")),
    ]
}

/// Format nodes count
#[must_use]
pub fn format_topology_nodes(node_count: usize) -> String {
    format!("Nodes: {node_count}")
}

/// Format edges count
#[must_use]
pub fn format_topology_edges(edge_count: usize) -> String {
    format!("Edges: {edge_count}")
}

/// Format average degree
#[must_use]
pub fn format_avg_degree(avg_degree: f32) -> String {
    format!("Avg Degree: {avg_degree:.1}")
}

/// Format capabilities count
#[must_use]
pub fn format_capabilities_count(count: usize) -> String {
    format!("{count} caps")
}

/// Format combined CPU and memory for compact display
#[must_use]
pub fn format_cpu_memory_combined(cpu_percent: f64, memory_percent: f64) -> String {
    format!("CPU: {cpu_percent:.1}% | Memory: {memory_percent:.1}%")
}

/// Format health and confidence for compact display
#[must_use]
pub fn format_health_confidence(health_percentage: f32, confidence: f32) -> String {
    format!("Health: {health_percentage:.0}% | Confidence: {confidence:.0}%")
}

// ============================================================================
// Minimal Sensory UI (Audio-only, very limited capabilities)
// ============================================================================

/// Minimal renderer for audio-only or very limited capabilities.
pub(super) struct MinimalSensoryUI;

impl MinimalSensoryUI {
    pub(super) const fn new() -> Self {
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

// ============================================================================
// Simple Sensory UI (Small display surface, touch, limited capabilities)
// ============================================================================

/// Simple renderer for small display surfaces and touch input.
pub(super) struct SimpleSensoryUI;

impl SimpleSensoryUI {
    pub(super) const fn new() -> Self {
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

// ============================================================================
// Standard Sensory UI (Desktop with mouse + keyboard)
// ============================================================================

/// Standard renderer for desktop with mouse and keyboard.
pub(super) struct StandardSensoryUI;

impl StandardSensoryUI {
    pub(super) const fn new() -> Self {
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

// ============================================================================
// Rich Sensory UI (High-res desktop with precision input)
// ============================================================================

/// Rich renderer for high-resolution desktop with precision input.
pub(super) struct RichSensoryUI;

impl RichSensoryUI {
    pub(super) const fn new() -> Self {
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

// ============================================================================
// Immersive Sensory UI (VR/AR with spatial audio and haptics)
// ============================================================================

/// Immersive renderer for VR/AR with spatial audio and haptics.
pub(super) struct ImmersiveSensoryUI;

impl ImmersiveSensoryUI {
    pub(super) const fn new() -> Self {
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

    #[test]
    fn test_format_topology_summary() {
        assert_eq!(
            format_topology_summary(5, 12),
            "Topology: 5 nodes, 12 edges"
        );
        assert_eq!(format_topology_summary(0, 0), "Topology: 0 nodes, 0 edges");
    }

    #[test]
    fn test_format_cpu_metrics() {
        assert_eq!(format_cpu_metrics(45.2), "CPU: 45.2%");
        assert_eq!(format_cpu_metrics(0.0), "CPU: 0.0%");
        assert_eq!(format_cpu_metrics(100.0), "CPU: 100.0%");
    }

    #[test]
    fn test_format_memory_metrics() {
        assert_eq!(format_memory_metrics(62.5), "Memory: 62.5%");
        assert_eq!(format_memory_metrics(0.0), "Memory: 0.0%");
    }

    #[test]
    fn test_format_proprioception_summary() {
        let pairs = format_proprioception_summary(85.0, "Healthy", 92.5);
        assert_eq!(pairs.len(), 3);
        assert_eq!(pairs[0], ("Health".to_string(), "85%".to_string()));
        assert_eq!(pairs[1], ("Status".to_string(), "Healthy".to_string()));
        assert_eq!(pairs[2], ("Confidence".to_string(), "92.5".to_string()));
    }

    #[test]
    fn test_format_topology_nodes_edges() {
        assert_eq!(format_topology_nodes(10), "Nodes: 10");
        assert_eq!(format_topology_edges(25), "Edges: 25");
    }

    #[test]
    fn test_format_avg_degree() {
        assert_eq!(format_avg_degree(2.5), "Avg Degree: 2.5");
        assert_eq!(format_avg_degree(0.0), "Avg Degree: 0.0");
    }

    #[test]
    fn test_format_capabilities_count() {
        assert_eq!(format_capabilities_count(3), "3 caps");
        assert_eq!(format_capabilities_count(0), "0 caps");
    }

    #[test]
    fn test_format_cpu_memory_combined() {
        assert_eq!(
            format_cpu_memory_combined(45.0, 62.5),
            "CPU: 45.0% | Memory: 62.5%"
        );
    }

    #[test]
    fn test_format_health_confidence() {
        assert_eq!(
            format_health_confidence(85.0, 92.0),
            "Health: 85% | Confidence: 92%"
        );
    }

    #[test]
    fn test_format_proprioception_summary_edge_cases() {
        let pairs = format_proprioception_summary(0.0, "Offline", 0.0);
        assert_eq!(pairs[0].1, "0%");
        assert_eq!(pairs[2].1, "0.0");
        let pairs = format_proprioception_summary(100.0, "Healthy", 100.0);
        assert_eq!(pairs[0].1, "100%");
        assert_eq!(pairs[2].1, "100.0");
    }

    #[test]
    fn test_format_cpu_metrics_edge_cases() {
        assert_eq!(format_cpu_metrics(99.9), "CPU: 99.9%");
        assert_eq!(format_cpu_metrics(0.1), "CPU: 0.1%");
    }

    #[test]
    fn test_format_memory_metrics_edge_cases() {
        assert_eq!(format_memory_metrics(100.0), "Memory: 100.0%");
    }

    #[test]
    fn test_format_capabilities_count_single() {
        assert_eq!(format_capabilities_count(1), "1 caps");
    }

    #[test]
    fn test_format_cpu_memory_combined_edge_cases() {
        assert_eq!(
            format_cpu_memory_combined(0.0, 0.0),
            "CPU: 0.0% | Memory: 0.0%"
        );
        assert_eq!(
            format_cpu_memory_combined(100.0, 100.0),
            "CPU: 100.0% | Memory: 100.0%"
        );
    }

    /// Headless egui: `MinimalSensoryUI` renders without panic (via manager)
    #[test]
    fn test_minimal_renderer_headless() {
        use crate::sensory_ui::manager::SensoryUIManager;
        use petal_tongue_core::sensory_capabilities::{AudioOutputCapability, SensoryCapabilities};

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

    /// Headless egui: `RichSensoryUI` renders primals with capabilities (hits `format_capabilities_count`)
    #[test]
    fn test_rich_renderer_with_capabilities_headless() {
        use crate::sensory_ui::manager::SensoryUIManager;
        use petal_tongue_core::sensory_capabilities::{
            KeyboardInputCapability, PointerInputCapability, SensoryCapabilities,
            VisualOutputCapability,
        };
        use petal_tongue_core::{PrimalHealthStatus, PrimalId};

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

    /// Headless egui: `SimpleSensoryUI` renders with topology and metrics
    #[test]
    fn test_simple_renderer_headless() {
        use crate::sensory_ui::manager::SensoryUIManager;
        use petal_tongue_core::sensory_capabilities::{
            PointerInputCapability, SensoryCapabilities, VisualOutputCapability,
        };

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

    /// Headless egui: `StandardSensoryUI` renders with topology and proprioception
    #[test]
    fn test_standard_renderer_headless() {
        use crate::sensory_ui::manager::SensoryUIManager;
        use petal_tongue_core::sensory_capabilities::{
            KeyboardInputCapability, PointerInputCapability, SensoryCapabilities,
            VisualOutputCapability,
        };

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

    /// Headless egui: `ImmersiveSensoryUI` renders (hits `format_cpu_memory_combined`, `format_health_confidence`)
    #[test]
    fn test_immersive_renderer_headless() {
        use crate::sensory_ui::manager::SensoryUIManager;
        use petal_tongue_core::metrics::{NeuralApiMetrics, SystemResourceMetrics};
        use petal_tongue_core::proprioception::{HealthData, HealthStatus};
        use petal_tongue_core::sensory_capabilities::{
            AudioOutputCapability, HapticOutputCapability, SensoryCapabilities,
            VisualOutputCapability,
        };

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
}
