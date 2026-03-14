// SPDX-License-Identifier: AGPL-3.0-only
//! Doom Stats Panel - Display game metrics
//!
//! Phase 1.4: Shows real-time Doom game statistics including:
//! - Map name
//! - View mode
//! - Player position and orientation
//! - Frame count
//! - Game state

use crate::panel_registry::{PanelFactory, PanelInstance};
use crate::scenario::CustomPanelConfig;
use doom_core::{DoomInstance, DoomState, GameStats, ViewMode};
use std::sync::{Arc, RwLock};

/// Panel that displays Doom game statistics
pub struct DoomStatsPanel {
    doom: Arc<RwLock<DoomInstance>>,
    last_stats: Option<GameStats>,
}

impl DoomStatsPanel {
    /// Create a new Doom stats panel connected to a Doom instance
    pub const fn new(doom: Arc<RwLock<DoomInstance>>) -> Self {
        Self {
            doom,
            last_stats: None,
        }
    }

    fn update_stats(&mut self) {
        if let Ok(doom) = self.doom.read() {
            self.last_stats = Some(doom.stats());
        }
    }
}

impl PanelInstance for DoomStatsPanel {
    fn title(&self) -> &'static str {
        "Game Stats"
    }

    fn on_open(&mut self) -> anyhow::Result<()> {
        tracing::info!("Doom Stats Panel opened");
        self.update_stats();
        Ok(())
    }

    fn on_close(&mut self) -> anyhow::Result<()> {
        tracing::info!("Doom Stats Panel closed");
        Ok(())
    }

    fn on_event(&mut self, _event: &egui::Event) {
        // Stats panel doesn't handle events
    }

    fn render(&mut self, ui: &mut egui::Ui) {
        // Update stats every frame
        self.update_stats();

        if let Some(stats) = &self.last_stats {
            ui.heading("🎮 Doom Stats");
            ui.separator();

            // Game state
            let state_text = match stats.state {
                DoomState::Uninitialized => "❌ Uninitialized",
                DoomState::Loading => "⏳ Loading",
                DoomState::Menu => "📋 Menu",
                DoomState::Playing => "▶️ Playing",
                DoomState::Paused => "⏸️ Paused",
                DoomState::Error => "❌ Error",
            };
            ui.label(format!("State: {state_text}"));

            ui.separator();

            // Map info
            if let Some(map_name) = &stats.current_map {
                ui.label(format!("Map: {map_name} (Hangar)"));
            } else {
                ui.label("Map: None");
            }

            // View mode
            let view_text = match stats.view_mode {
                ViewMode::FirstPerson => "First-Person (3D)",
                ViewMode::TopDown => "Top-Down (2D)",
            };
            ui.label(format!("View: {view_text}"));

            ui.separator();

            // Player position
            if let (Some(x), Some(y), Some(angle)) =
                (stats.player_x, stats.player_y, stats.player_angle)
            {
                ui.label("Player Position:");
                ui.label(format!("  X: {x:.0}"));
                ui.label(format!("  Y: {y:.0}"));
                ui.label(format!("  Angle: {:.0}°", angle.to_degrees()));
            } else {
                ui.label("Player: Not initialized");
            }

            ui.separator();

            // Performance
            ui.label(format!("Frame: {}", stats.frame_count));
            ui.label(format!(
                "Resolution: {}x{}",
                stats.dimensions.0, stats.dimensions.1
            ));
        } else {
            ui.label("⏳ Waiting for game data...");
        }
    }

    fn wants_keyboard_input(&self) -> bool {
        false // Stats panel doesn't need keyboard input
    }

    fn wants_mouse_input(&self) -> bool {
        false // Stats panel doesn't need mouse input
    }
}

/// Factory for creating Doom Stats panels
pub struct DoomStatsPanelFactory {
    doom: Arc<RwLock<DoomInstance>>,
}

impl DoomStatsPanelFactory {
    /// Create a new factory that produces Doom stats panels
    pub const fn new(doom: Arc<RwLock<DoomInstance>>) -> Self {
        Self { doom }
    }
}

impl PanelFactory for DoomStatsPanelFactory {
    fn panel_type(&self) -> &'static str {
        "doom_stats"
    }

    fn create(
        &self,
        _config: &CustomPanelConfig,
    ) -> crate::panel_registry::Result<Box<dyn PanelInstance>> {
        Ok(Box::new(DoomStatsPanel::new(Arc::clone(&self.doom))))
    }

    fn description(&self) -> &'static str {
        "Displays real-time Doom game statistics"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_panel_creation() {
        let doom = Arc::new(RwLock::new(DoomInstance::new(640, 480).unwrap()));
        let panel = DoomStatsPanel::new(doom);
        assert!(!panel.wants_keyboard_input());
        assert!(!panel.wants_mouse_input());
    }

    #[test]
    fn test_stats_panel_factory() {
        let doom = Arc::new(RwLock::new(DoomInstance::new(640, 480).unwrap()));
        let factory = DoomStatsPanelFactory::new(doom);
        assert_eq!(factory.panel_type(), "doom_stats");
        assert_eq!(
            factory.description(),
            "Displays real-time Doom game statistics"
        );

        let config = CustomPanelConfig {
            panel_type: "doom_stats".to_string(),
            title: "Test Stats".to_string(),
            width: None,
            height: None,
            fullscreen: false,
            config: serde_json::json!({}),
        };
        let panel = factory.create(&config);
        assert!(panel.is_ok());
    }
}
