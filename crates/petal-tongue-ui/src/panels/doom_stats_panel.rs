// SPDX-License-Identifier: AGPL-3.0-or-later
//! Doom Stats Panel - Display game metrics
//!
//! Shows real-time Doom game statistics including:
//! - Map name
//! - View mode
//! - Player position and orientation
//! - Frame count
//! - Game state

use crate::panel_registry::{PanelFactory, PanelInstance};
use crate::panels::doom_stats_display::prepare_doom_stats_display;
use crate::scenario::CustomPanelConfig;
use doom_core::{DoomInstance, GameStats};
use std::sync::{Arc, RwLock};

/// Panel that presents Doom game statistics
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

    fn on_open(&mut self) -> crate::error::Result<()> {
        tracing::info!("Doom Stats Panel opened");
        self.update_stats();
        Ok(())
    }

    fn on_close(&mut self) -> crate::error::Result<()> {
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
            let display = prepare_doom_stats_display(stats);

            ui.heading("🎮 Doom Stats");
            ui.separator();

            ui.label(format!("State: {}", display.state_text));
            ui.separator();
            ui.label(format!("Map: {}", display.map_name));
            ui.label(format!("View: {}", display.view_mode));
            ui.separator();

            if let Some(position) = &display.position {
                ui.label("Player Position:");
                for line in position.lines() {
                    ui.label(line);
                }
            } else {
                ui.label("Player: Not initialized");
            }

            ui.separator();
            ui.label(format!("Frame: {}", display.frame_count));
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
    ) -> crate::panel_registry::Result<crate::panel_registry::PanelInstanceImpl> {
        Ok(crate::panel_registry::PanelInstanceImpl::DoomStats(
            DoomStatsPanel::new(Arc::clone(&self.doom)),
        ))
    }

    fn description(&self) -> &'static str {
        "Displays real-time Doom game statistics"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stats_panel_title() {
        let doom = Arc::new(RwLock::new(DoomInstance::new(640, 480).unwrap()));
        let panel = DoomStatsPanel::new(doom);
        assert_eq!(panel.title(), "Game Stats");
    }

    #[test]
    fn test_stats_panel_on_open_on_close() {
        let doom = Arc::new(RwLock::new(DoomInstance::new(640, 480).unwrap()));
        let mut panel = DoomStatsPanel::new(doom);
        assert!(panel.on_open().is_ok());
        assert!(panel.on_close().is_ok());
    }

    #[test]
    fn test_stats_panel_on_event() {
        let doom = Arc::new(RwLock::new(DoomInstance::new(640, 480).unwrap()));
        let mut panel = DoomStatsPanel::new(doom);
        panel.on_event(&egui::Event::PointerMoved(egui::Pos2::ZERO));
    }

    #[test]
    fn test_stats_panel_update() {
        let doom = Arc::new(RwLock::new(DoomInstance::new(640, 480).unwrap()));
        let mut panel = DoomStatsPanel::new(doom);
        panel.update();
    }

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
