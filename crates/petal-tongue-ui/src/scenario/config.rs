// SPDX-License-Identifier: AGPL-3.0-only
//! UI configuration types for scenarios
//!
//! Defines UI-related configuration structures including panels,
//! animations, performance settings, and custom panel configurations.

use crate::error::Result;
use crate::scenario_error::ScenarioError;
use serde::{Deserialize, Serialize};

/// UI configuration settings for a scenario
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme name (e.g., "dark", "light", "system")
    #[serde(default)]
    pub theme: String,
    /// Layout mode: "canvas-only", "dashboard-centered", "full-dashboard"
    #[serde(default)]
    pub layout: String,
    /// Which panels are visible
    #[serde(default)]
    pub show_panels: PanelVisibility,
    /// Animation settings
    #[serde(default)]
    pub animations: AnimationConfig,
    /// Performance tuning settings
    #[serde(default)]
    pub performance: PerformanceConfig,
    /// Feature flags for optional functionality
    #[serde(default)]
    pub features: FeatureFlags,
    /// Custom panels (e.g., Doom, web browser, video player)
    #[serde(default)]
    pub custom_panels: Vec<CustomPanelConfig>,
    /// Whether to show the awakening overlay on startup (default: true)
    #[serde(default = "default_true")]
    pub awakening_enabled: bool,
    /// Initial zoom level: "fit" to auto-fit, or a numeric string like "1.0"
    #[serde(default)]
    pub initial_zoom: String,
}

const fn default_true() -> bool {
    true
}

impl UiConfig {
    /// Validate UI configuration
    pub fn validate(&self) -> Result<()> {
        // Validate custom panels
        for (idx, panel) in self.custom_panels.iter().enumerate() {
            panel.validate().map_err(|e| ScenarioError::PanelConfig {
                message: e.to_string(),
                panel_index: Some(idx),
                panel_type: Some(panel.panel_type.clone()),
            })?;
        }

        // Validate performance config
        if self.performance.target_fps > 0 && self.performance.target_fps < 10 {
            tracing::warn!(
                "⚠️  Target FPS ({}) is very low, may cause sluggish UI",
                self.performance.target_fps
            );
        }

        if self.performance.target_fps > 240 {
            tracing::warn!(
                "⚠️  Target FPS ({}) is very high, may waste resources",
                self.performance.target_fps
            );
        }

        Ok(())
    }
}

/// Panel visibility settings - controls which UI panels are shown
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelVisibility {
    /// Show left sidebar with navigation
    pub left_sidebar: bool,
    /// Show right sidebar with details
    pub right_sidebar: bool,
    /// Show top menu bar
    pub top_menu: bool,
    /// Show system metrics dashboard
    pub system_dashboard: bool,
    /// Show audio controls panel
    pub audio_panel: bool,
    /// Show trust relationship dashboard
    pub trust_dashboard: bool,
    /// Show proprioception (self-awareness) panel
    pub proprioception: bool,
    /// Show graph statistics panel
    pub graph_stats: bool,
}

impl Default for PanelVisibility {
    fn default() -> Self {
        // Default: show everything (backward compatible)
        Self {
            left_sidebar: true,
            right_sidebar: true,
            top_menu: true,
            system_dashboard: true,
            audio_panel: true,
            trust_dashboard: true,
            proprioception: true,
            graph_stats: true,
        }
    }
}

/// Feature flags for optional functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FeatureFlags {
    /// Enable audio sonification of graph events
    pub audio_sonification: bool,
    /// Automatically refresh data from providers
    pub auto_refresh: bool,
    /// Enable Neural API integration for metrics
    pub neural_api: bool,
    /// Enable tutorial/demo mode with guided interactions
    pub tutorial_mode: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        // Default: enable all features (backward compatible)
        Self {
            audio_sonification: true,
            auto_refresh: true,
            neural_api: false, // Disabled by default (requires external service)
            tutorial_mode: false,
        }
    }
}

/// Custom panel configuration (for embedded apps like Doom, web browsers, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPanelConfig {
    /// Panel type identifier (e.g., "`doom_game`", "`web_view`", "`video_player`")
    #[serde(rename = "type")]
    pub panel_type: String,

    /// Panel title
    pub title: String,

    /// Panel width (optional, defaults to fit)
    #[serde(default)]
    pub width: Option<usize>,

    /// Panel height (optional, defaults to fit)
    #[serde(default)]
    pub height: Option<usize>,

    /// Fullscreen mode
    #[serde(default)]
    pub fullscreen: bool,

    /// Panel-specific configuration (JSON value for flexibility)
    #[serde(default)]
    pub config: serde_json::Value,
}

impl CustomPanelConfig {
    /// Validate panel configuration
    pub fn validate(&self) -> Result<()> {
        // Check panel type
        if self.panel_type.trim().is_empty() {
            return Err(ScenarioError::PanelConfig {
                message: "Panel type cannot be empty (e.g., 'doom_game', 'web_view')".to_string(),
                panel_index: None,
                panel_type: None,
            }
            .into());
        }

        // Check title
        if self.title.trim().is_empty() {
            return Err(ScenarioError::PanelConfig {
                message: format!("Panel '{}' has empty title", self.panel_type),
                panel_index: None,
                panel_type: Some(self.panel_type.clone()),
            }
            .into());
        }

        // Validate dimensions
        if let Some(width) = self.width {
            if width == 0 {
                return Err(ScenarioError::PanelConfig {
                    message: format!("Panel '{}' has zero width", self.title),
                    panel_index: None,
                    panel_type: Some(self.panel_type.clone()),
                }
                .into());
            }
            if width > 7680 {
                // Reasonable max: 8K resolution
                tracing::warn!(
                    "⚠️  Panel '{}' has unusually large width: {}",
                    self.title,
                    width
                );
            }
        }

        if let Some(height) = self.height {
            if height == 0 {
                return Err(ScenarioError::PanelConfig {
                    message: format!("Panel '{}' has zero height", self.title),
                    panel_index: None,
                    panel_type: Some(self.panel_type.clone()),
                }
                .into());
            }
            if height > 4320 {
                // Reasonable max: 8K resolution
                tracing::warn!(
                    "⚠️  Panel '{}' has unusually large height: {}",
                    self.title,
                    height
                );
            }
        }

        Ok(())
    }
}

/// Animation configuration for UI effects
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnimationConfig {
    /// Master switch for all animations
    #[serde(default)]
    pub enabled: bool,
    /// Animate nodes with subtle breathing effect
    #[serde(default)]
    pub breathing_nodes: bool,
    /// Animate connection lines with pulse effects
    #[serde(default)]
    pub connection_pulses: bool,
    /// Smooth transitions between UI states
    #[serde(default)]
    pub smooth_transitions: bool,
    /// Celebratory effects for achievements
    #[serde(default)]
    pub celebration_effects: bool,
}

/// Performance configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceConfig {
    #[serde(default)]
    pub target_fps: u32,
    #[serde(default)]
    pub vsync: bool,
    #[serde(default)]
    pub hardware_acceleration: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_config_default() {
        let c = UiConfig::default();
        assert!(c.theme.is_empty());
        assert!(c.layout.is_empty());
        assert!(c.initial_zoom.is_empty());
    }

    #[test]
    fn panel_visibility_default() {
        let p = PanelVisibility::default();
        assert!(p.left_sidebar);
        assert!(p.right_sidebar);
        assert!(p.top_menu);
        assert!(p.system_dashboard);
    }

    #[test]
    fn feature_flags_default() {
        let f = FeatureFlags::default();
        assert!(f.audio_sonification);
        assert!(f.auto_refresh);
        assert!(!f.neural_api);
        assert!(!f.tutorial_mode);
    }

    #[test]
    fn ui_config_validate_ok() {
        let c = UiConfig::default();
        assert!(c.validate().is_ok());
    }

    #[test]
    fn custom_panel_validate_empty_type_fails() {
        let p = CustomPanelConfig {
            panel_type: "".to_string(),
            title: "T".to_string(),
            width: None,
            height: None,
            fullscreen: false,
            config: serde_json::Value::Null,
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn custom_panel_validate_empty_title_fails() {
        let p = CustomPanelConfig {
            panel_type: "doom".to_string(),
            title: "".to_string(),
            width: None,
            height: None,
            fullscreen: false,
            config: serde_json::Value::Null,
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn custom_panel_validate_zero_width_fails() {
        let p = CustomPanelConfig {
            panel_type: "doom".to_string(),
            title: "Doom".to_string(),
            width: Some(0),
            height: None,
            fullscreen: false,
            config: serde_json::Value::Null,
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn custom_panel_validate_ok() {
        let p = CustomPanelConfig {
            panel_type: "doom_game".to_string(),
            title: "Doom".to_string(),
            width: Some(320),
            height: Some(200),
            fullscreen: false,
            config: serde_json::Value::Null,
        };
        assert!(p.validate().is_ok());
    }

    #[test]
    fn animation_config_default() {
        let a = AnimationConfig::default();
        assert!(!a.enabled);
        assert!(!a.breathing_nodes);
    }

    #[test]
    fn performance_config_default() {
        let p = PerformanceConfig::default();
        assert_eq!(p.target_fps, 0);
        assert!(!p.vsync);
    }
}
