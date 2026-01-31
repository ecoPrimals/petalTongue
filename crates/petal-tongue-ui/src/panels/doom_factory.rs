//! Doom Panel Factory - Registers Doom as an available panel type

use super::doom_panel::DoomPanel;
use crate::panel_registry::{PanelError, PanelFactory, PanelInstance, Result};
use crate::scenario::CustomPanelConfig;
use std::sync::Arc;

/// Factory for creating Doom panels
pub struct DoomPanelFactory;

impl PanelFactory for DoomPanelFactory {
    fn panel_type(&self) -> &str {
        "doom_game"
    }

    fn create(&self, config: &CustomPanelConfig) -> Result<Box<dyn PanelInstance>> {
        tracing::info!("Creating Doom panel: {}", config.title);

        // Extract width/height from config, with defaults
        let width = config.width.unwrap_or(640);
        let height = config.height.unwrap_or(480);

        // Parse panel-specific config (optional)
        let show_debug = if let Some(cfg) = config.config.as_object() {
            cfg.get("show_debug")
                .and_then(|v| v.as_bool())
                .unwrap_or(true)
        } else {
            true
        };

        let mut panel = DoomPanel::new();
        if !show_debug {
            panel.toggle_debug();
        }

        Ok(Box::new(DoomPanelWrapper {
            panel,
            title: config.title.clone(),
        }))
    }

    fn description(&self) -> &str {
        "Classic Doom game (1993) - Platform capability test"
    }
}

/// Wrapper to adapt DoomPanel to PanelInstance trait
struct DoomPanelWrapper {
    panel: DoomPanel,
    title: String,
}

impl PanelInstance for DoomPanelWrapper {
    fn render(&mut self, ui: &mut egui::Ui) {
        self.panel.render(ui);
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn update(&mut self) {
        // DoomPanel handles its own updates in render()
    }

    // 🎮 Doom needs ALL the input!
    fn wants_keyboard_input(&self) -> bool {
        true // WASD, arrows, keys
    }

    fn wants_mouse_input(&self) -> bool {
        true // Click to fire, mouse to turn
    }

    fn wants_exclusive_input(&self) -> bool {
        true // Games need exclusive input
    }
}

/// Convenience function to create a Doom panel factory
pub fn create_doom_factory() -> Arc<dyn PanelFactory> {
    Arc::new(DoomPanelFactory)
}
