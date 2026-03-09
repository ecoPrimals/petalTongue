// SPDX-License-Identifier: AGPL-3.0-only
//! Sensory-Based Adaptive UI System
//!
//! This module replaces device-type based rendering with capability-based
//! rendering. Instead of asking "what device is this?", we ask "what
//! capabilities does this device have?"
//!
//! # Architecture
//!
//! - **Discover** capabilities at runtime (visual, audio, haptic, inputs)
//! - **Determine** UI complexity from capabilities (Minimal → Immersive)
//! - **Adapt** rendering based on complexity
//! - **Hot-reload** when capabilities change (VR headset plugged in)

use eframe::egui;
use petal_tongue_core::{CapabilityError, SensoryCapabilities};
// Import as SensoryUIComplexity to avoid conflict with adaptive_rendering::UIComplexity
use petal_tongue_core::sensory_capabilities::UIComplexity as SensoryUIComplexity;
use std::time::Instant;

/// Sensory-based adaptive UI manager
///
/// This replaces the old `AdaptiveUIManager` which used device types.
/// Now we use discovered capabilities instead.
pub struct SensoryUIManager {
    capabilities: SensoryCapabilities,
    ui_complexity: SensoryUIComplexity,
    renderer: Box<dyn SensoryUIRenderer>,
    last_discovery: Instant,
}

impl SensoryUIManager {
    /// Create a new sensory UI manager with discovered capabilities
    pub fn new() -> Result<Self, CapabilityError> {
        let capabilities = SensoryCapabilities::discover()?;
        let ui_complexity = capabilities.determine_ui_complexity();

        let renderer = Self::create_renderer(ui_complexity);

        Ok(Self {
            capabilities,
            ui_complexity,
            renderer,
            last_discovery: Instant::now(),
        })
    }

    /// Create appropriate renderer for UI complexity level
    fn create_renderer(complexity: SensoryUIComplexity) -> Box<dyn SensoryUIRenderer> {
        match complexity {
            SensoryUIComplexity::Minimal => Box::new(MinimalSensoryUI::new()),
            SensoryUIComplexity::Simple => Box::new(SimpleSensoryUI::new()),
            SensoryUIComplexity::Standard => Box::new(StandardSensoryUI::new()),
            SensoryUIComplexity::Rich => Box::new(RichSensoryUI::new()),
            SensoryUIComplexity::Immersive => Box::new(ImmersiveSensoryUI::new()),
        }
    }

    /// Get current UI complexity
    #[must_use]
    pub fn ui_complexity(&self) -> SensoryUIComplexity {
        self.ui_complexity
    }

    /// Get capabilities description
    #[must_use]
    pub fn capabilities_description(&self) -> String {
        self.capabilities.describe()
    }

    /// Re-discover capabilities (for hot-reload when hardware changes)
    pub fn rediscover(&mut self) -> Result<(), CapabilityError> {
        // Only rediscover every 5 seconds to avoid overhead
        if self.last_discovery.elapsed().as_secs() < 5 {
            return Ok(());
        }

        let new_capabilities = SensoryCapabilities::discover()?;
        let new_complexity = new_capabilities.determine_ui_complexity();

        // Hot-swap renderer if complexity changed
        if new_complexity != self.ui_complexity {
            tracing::info!(
                "Capability change detected: {} → {}",
                self.ui_complexity,
                new_complexity
            );

            self.renderer = Self::create_renderer(new_complexity);
            self.ui_complexity = new_complexity;
        }

        self.capabilities = new_capabilities;
        self.last_discovery = Instant::now();

        Ok(())
    }

    /// Create manager with given capabilities (for testing - bypasses discovery)
    #[cfg(test)]
    pub fn with_capabilities(capabilities: SensoryCapabilities) -> Self {
        let ui_complexity = capabilities.determine_ui_complexity();
        let renderer = Self::create_renderer(ui_complexity);
        Self {
            capabilities,
            ui_complexity,
            renderer,
            last_discovery: Instant::now(),
        }
    }

    /// Render the primal list
    pub fn render_primal_list(
        &mut self,
        ui: &mut egui::Ui,
        primals: &[petal_tongue_core::PrimalInfo],
    ) {
        self.renderer.render_primal_list(ui, primals);
    }

    /// Render the topology view
    pub fn render_topology(
        &mut self,
        ui: &mut egui::Ui,
        graph_engine: &petal_tongue_core::GraphEngine,
    ) {
        self.renderer.render_topology(ui, graph_engine);
    }

    /// Render the metrics panel
    pub fn render_metrics(
        &mut self,
        ui: &mut egui::Ui,
        metrics: Option<&petal_tongue_core::SystemMetrics>,
    ) {
        self.renderer.render_metrics(ui, metrics);
    }

    /// Render the proprioception panel
    pub fn render_proprioception(
        &mut self,
        ui: &mut egui::Ui,
        proprioception: Option<&petal_tongue_core::ProprioceptionData>,
    ) {
        self.renderer.render_proprioception(ui, proprioception);
    }
}

/// Trait for sensory-based UI renderers
///
/// Each complexity level has a different renderer implementation
/// that adapts to the available capabilities.
pub trait SensoryUIRenderer: Send {
    /// Render the primal list
    fn render_primal_list(&mut self, ui: &mut egui::Ui, primals: &[petal_tongue_core::PrimalInfo]);

    /// Render the topology view
    fn render_topology(&mut self, ui: &mut egui::Ui, graph_engine: &petal_tongue_core::GraphEngine);

    /// Render the metrics panel
    fn render_metrics(
        &mut self,
        ui: &mut egui::Ui,
        metrics: Option<&petal_tongue_core::SystemMetrics>,
    );

    /// Render the proprioception panel
    fn render_proprioception(
        &mut self,
        ui: &mut egui::Ui,
        proprioception: Option<&petal_tongue_core::ProprioceptionData>,
    );
}

// ============================================================================
// Minimal Sensory UI (Audio-only, very limited capabilities)
// ============================================================================

struct MinimalSensoryUI;

impl MinimalSensoryUI {
    fn new() -> Self {
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

struct SimpleSensoryUI;

impl SimpleSensoryUI {
    fn new() -> Self {
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

struct StandardSensoryUI;

impl StandardSensoryUI {
    fn new() -> Self {
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

struct RichSensoryUI;

impl RichSensoryUI {
    fn new() -> Self {
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

struct ImmersiveSensoryUI;

impl ImmersiveSensoryUI {
    fn new() -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::sensory_capabilities::{
        AudioOutputCapability, KeyboardInputCapability, PointerInputCapability,
        VisualOutputCapability,
    };

    #[test]
    fn test_with_capabilities_simple() {
        // Touch + small screen => Simple
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::TwoD {
                resolution: (800, 480),
                refresh_rate: 60,
                color_depth: 8,
                size_mm: Some((120, 72)),
            }],
            touch_inputs: vec![],
            pointer_inputs: vec![PointerInputCapability::TwoD {
                precision: 2.0,
                has_wheel: false,
                has_pressure: false,
                button_count: 1,
            }],
            ..Default::default()
        };
        let manager = SensoryUIManager::with_capabilities(caps);
        assert_eq!(manager.ui_complexity(), SensoryUIComplexity::Simple);
    }

    #[test]
    fn test_with_capabilities_standard() {
        // Desktop mouse + keyboard => Standard
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::TwoD {
                resolution: (1280, 720),
                refresh_rate: 60,
                color_depth: 8,
                size_mm: None,
            }],
            pointer_inputs: vec![PointerInputCapability::TwoD {
                precision: 1.0,
                has_wheel: true,
                has_pressure: false,
                button_count: 3,
            }],
            keyboard_inputs: vec![KeyboardInputCapability::Physical {
                layout: "QWERTY".to_string(),
                has_numpad: false,
                modifier_keys: 3,
            }],
            ..Default::default()
        };
        let manager = SensoryUIManager::with_capabilities(caps);
        assert_eq!(manager.ui_complexity(), SensoryUIComplexity::Standard);
    }

    #[test]
    fn test_with_capabilities_immersive() {
        // VR/AR + spatial audio + haptics => Immersive
        use petal_tongue_core::sensory_capabilities::{AudioOutputCapability as AOC, HapticOutputCapability};
        let caps = SensoryCapabilities {
            visual_outputs: vec![VisualOutputCapability::ThreeD {
                resolution_per_eye: (2160, 1200),
                field_of_view: (110.0, 90.0),
                refresh_rate: 90,
                has_depth_tracking: true,
                has_hand_tracking: true,
            }],
            audio_outputs: vec![AOC::Spatial {
                channels: 6,
                sample_rate: 48000,
                has_head_tracking: true,
            }],
            haptic_outputs: vec![HapticOutputCapability::SimpleVibration {
                intensity_levels: 255,
            }],
            ..Default::default()
        };
        let manager = SensoryUIManager::with_capabilities(caps);
        assert_eq!(manager.ui_complexity(), SensoryUIComplexity::Immersive);
    }

    #[test]
    fn test_rediscover_no_change_within_5_seconds() {
        let caps = SensoryCapabilities {
            audio_outputs: vec![AudioOutputCapability::Stereo {
                sample_rate: 48000,
                bit_depth: 16,
            }],
            keyboard_inputs: vec![KeyboardInputCapability::Physical {
                layout: "QWERTY".to_string(),
                has_numpad: false,
                modifier_keys: 3,
            }],
            ..Default::default()
        };
        let mut manager = SensoryUIManager::with_capabilities(caps);
        let initial_complexity = manager.ui_complexity();
        // Rediscover immediately - should skip due to 5 second throttle
        let result = manager.rediscover();
        assert!(result.is_ok());
        assert_eq!(manager.ui_complexity(), initial_complexity);
    }

    #[test]
    fn test_capabilities_description() {
        let caps = SensoryCapabilities {
            audio_outputs: vec![AudioOutputCapability::Stereo {
                sample_rate: 48000,
                bit_depth: 16,
            }],
            ..Default::default()
        };
        let manager = SensoryUIManager::with_capabilities(caps);
        let desc = manager.capabilities_description();
        assert!(!desc.is_empty());
    }

    #[test]
    fn test_with_capabilities_minimal() {
        let caps = SensoryCapabilities {
            audio_outputs: vec![AudioOutputCapability::Stereo {
                sample_rate: 48000,
                bit_depth: 16,
            }],
            keyboard_inputs: vec![KeyboardInputCapability::Physical {
                layout: "QWERTY".to_string(),
                has_numpad: false,
                modifier_keys: 3,
            }],
            ..Default::default()
        };
        let manager = SensoryUIManager::with_capabilities(caps);
        assert_eq!(manager.ui_complexity(), SensoryUIComplexity::Minimal);
        let desc = manager.capabilities_description();
        assert!(desc.contains("audio") || !desc.is_empty());
    }

    #[test]
    fn test_with_capabilities_rich() {
        // High-res + precision pointer + keyboard => Rich
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
        let manager = SensoryUIManager::with_capabilities(caps);
        assert_eq!(manager.ui_complexity(), SensoryUIComplexity::Rich);
    }
}
