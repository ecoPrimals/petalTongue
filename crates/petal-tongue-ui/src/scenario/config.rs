// SPDX-License-Identifier: AGPL-3.0-only
//! UI configuration types for scenarios
//!
//! Defines UI-related configuration structures including panels,
//! animations, performance settings, and custom panel configurations.

use anyhow::{Context, Result};
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
}

impl UiConfig {
    /// Validate UI configuration
    pub fn validate(&self) -> Result<()> {
        // Validate custom panels
        for (idx, panel) in self.custom_panels.iter().enumerate() {
            panel
                .validate()
                .with_context(|| format!("Custom panel {idx} validation failed"))?;
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
            anyhow::bail!("Panel type cannot be empty (e.g., 'doom_game', 'web_view')");
        }

        // Check title
        if self.title.trim().is_empty() {
            anyhow::bail!("Panel '{}' has empty title", self.panel_type);
        }

        // Validate dimensions
        if let Some(width) = self.width {
            if width == 0 {
                anyhow::bail!("Panel '{}' has zero width", self.title);
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
                anyhow::bail!("Panel '{}' has zero height", self.title);
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
