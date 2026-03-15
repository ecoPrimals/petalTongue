// SPDX-License-Identifier: AGPL-3.0-only
//! Doom Panel Factory - Registers Doom as an available panel type

use super::doom_panel::DoomPanel;
use crate::panel_registry::{PanelFactory, PanelInstance, Result};
use crate::scenario::CustomPanelConfig;
use std::sync::Arc;

/// Factory for creating Doom panels
pub struct DoomPanelFactory;

impl PanelFactory for DoomPanelFactory {
    fn panel_type(&self) -> &'static str {
        "doom_game"
    }

    fn create(&self, config: &CustomPanelConfig) -> Result<Box<dyn PanelInstance>> {
        tracing::info!("Creating Doom panel: {}", config.title);

        // Extract width/height from config, with defaults
        let _width = config.width.unwrap_or(640);
        let _height = config.height.unwrap_or(480);

        // Parse panel-specific config (optional)
        let show_debug = config.config.as_object().is_none_or(|cfg| {
            cfg.get("show_debug")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true)
        });

        let mut panel = DoomPanel::new();
        if !show_debug {
            panel.toggle_debug();
        }

        Ok(Box::new(DoomPanelWrapper {
            panel,
            title: config.title.clone(),
        }))
    }

    fn description(&self) -> &'static str {
        "Classic Doom game (1993) - Platform capability test"
    }
}

/// Wrapper to adapt `DoomPanel` to `PanelInstance` trait
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
#[must_use]
pub fn create_doom_factory() -> Arc<dyn PanelFactory> {
    Arc::new(DoomPanelFactory)
}

#[cfg(all(test, feature = "doom"))]
mod tests {
    use super::*;

    fn config_with_show_debug(show_debug: bool) -> CustomPanelConfig {
        CustomPanelConfig {
            panel_type: "doom_game".to_string(),
            title: "Test Doom".to_string(),
            width: Some(320),
            height: Some(240),
            fullscreen: false,
            config: if show_debug {
                serde_json::Value::Null
            } else {
                serde_json::json!({"show_debug": false})
            },
        }
    }

    #[test]
    fn factory_panel_type() {
        let factory = DoomPanelFactory;
        assert_eq!(factory.panel_type(), "doom_game");
    }

    #[test]
    fn factory_description() {
        let factory = DoomPanelFactory;
        assert!(factory.description().contains("Doom"));
    }

    #[test]
    fn factory_create_returns_panel() {
        let factory = DoomPanelFactory;
        let config = config_with_show_debug(true);
        let panel = factory.create(&config).unwrap();
        assert_eq!(panel.title(), "Test Doom");
        assert!(panel.wants_keyboard_input());
        assert!(panel.wants_mouse_input());
        assert!(panel.wants_exclusive_input());
    }

    #[test]
    fn factory_create_with_show_debug_false() {
        let factory = DoomPanelFactory;
        let config = config_with_show_debug(false);
        let panel = factory.create(&config).unwrap();
        assert_eq!(panel.title(), "Test Doom");
    }

    #[test]
    fn factory_create_uses_width_height_defaults() {
        let factory = DoomPanelFactory;
        let config = CustomPanelConfig {
            panel_type: "doom_game".to_string(),
            title: "Minimal".to_string(),
            width: None,
            height: None,
            fullscreen: false,
            config: serde_json::Value::Null,
        };
        let panel = factory.create(&config).unwrap();
        assert_eq!(panel.title(), "Minimal");
    }

    #[test]
    fn create_doom_factory_returns_arc() {
        let factory = create_doom_factory();
        assert_eq!(factory.panel_type(), "doom_game");
    }
}
