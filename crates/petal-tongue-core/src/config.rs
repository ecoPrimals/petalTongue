//! petalTongue configuration.

use crate::common_config::CommonConfig;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for petalTongue.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PetalTongueConfig {
    /// Common configuration.
    #[serde(flatten)]
    pub common: CommonConfig,

    /// `BiomeOS` discovery endpoint (if not using auto-discovery via Songbird)
    /// If None, will attempt to discover via Songbird
    #[serde(default)]
    pub biomeos_url: Option<String>,

    /// Auto-refresh interval for topology updates (seconds)
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval_secs: u64,

    /// Audio sample rate for sonification
    #[serde(default = "default_sample_rate")]
    pub audio_sample_rate: u32,

    /// Maximum FPS for rendering
    #[serde(default = "default_max_fps")]
    pub max_fps: u32,

    /// Enable audio output
    #[serde(default = "default_true")]
    pub audio_enabled: bool,

    /// Enable mock mode for testing (when `BiomeOS` unavailable)
    #[serde(default)]
    pub mock_mode: bool,
}

impl Default for PetalTongueConfig {
    fn default() -> Self {
        Self {
            common: CommonConfig {
                name: "petalTongue".to_string(),
                ..CommonConfig::default()
            },
            biomeos_url: None,
            refresh_interval_secs: default_refresh_interval(),
            audio_sample_rate: default_sample_rate(),
            max_fps: default_max_fps(),
            audio_enabled: true,
            mock_mode: false,
        }
    }
}

impl PetalTongueConfig {
    /// Get refresh interval as Duration
    #[must_use]
    pub fn refresh_interval(&self) -> Duration {
        Duration::from_secs(self.refresh_interval_secs)
    }

    /// Get `BiomeOS` URL from environment or config
    #[must_use]
    pub fn biomeos_url(&self) -> String {
        std::env::var("BIOMEOS_URL")
            .ok()
            .or_else(|| self.biomeos_url.clone())
        .unwrap_or_else(|| {
            tracing::warn!(
                "No BiomeOS URL configured - discovery will rely on mDNS/HTTP probing"
            );
            String::new() // Empty = not configured, will discover at runtime
        })
    }
}

fn default_refresh_interval() -> u64 {
    5
}

fn default_sample_rate() -> u32 {
    48000
}

fn default_max_fps() -> u32 {
    60
}

fn default_true() -> bool {
    true
}
