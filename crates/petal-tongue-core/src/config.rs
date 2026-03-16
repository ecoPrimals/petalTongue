// SPDX-License-Identifier: AGPL-3.0-or-later
//! petalTongue configuration.

use crate::common_config::CommonConfig;
use crate::constants::PRIMAL_NAME;
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
                name: PRIMAL_NAME.to_string(),
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
    pub const fn refresh_interval(&self) -> Duration {
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

const fn default_refresh_interval() -> u64 {
    5
}

const fn default_sample_rate() -> u32 {
    48000
}

const fn default_max_fps() -> u32 {
    60
}

const fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PetalTongueConfig::default();
        assert_eq!(config.common.name, "petalTongue");
        assert!(config.biomeos_url.is_none());
        assert_eq!(config.refresh_interval_secs, 5);
        assert_eq!(config.audio_sample_rate, 48000);
        assert_eq!(config.max_fps, 60);
        assert!(config.audio_enabled);
        assert!(!config.mock_mode);
    }

    #[test]
    fn test_refresh_interval() {
        let config = PetalTongueConfig::default();
        let d = config.refresh_interval();
        assert_eq!(d.as_secs(), 5);
    }

    #[test]
    fn test_config_serialization() {
        let config = PetalTongueConfig::default();
        let toml = toml::to_string(&config).unwrap();
        let parsed: PetalTongueConfig = toml::from_str(&toml).unwrap();
        assert_eq!(parsed.refresh_interval_secs, config.refresh_interval_secs);
    }

    #[test]
    fn test_config_with_overrides() {
        let toml = r#"
            name = "custom"
            refresh_interval_secs = 10
            audio_sample_rate = 44100
            max_fps = 30
            audio_enabled = false
            mock_mode = true
        "#;
        let config: PetalTongueConfig = toml::from_str(toml).unwrap();
        assert_eq!(config.refresh_interval_secs, 10);
        assert_eq!(config.audio_sample_rate, 44100);
        assert_eq!(config.max_fps, 30);
        assert!(!config.audio_enabled);
        assert!(config.mock_mode);
    }

    #[test]
    fn test_biomeos_url_from_config() {
        let config = PetalTongueConfig {
            biomeos_url: Some("http://localhost:9999".to_string()),
            ..PetalTongueConfig::default()
        };
        let url = config.biomeos_url();
        assert_eq!(url, "http://localhost:9999");
    }

    #[test]
    fn test_biomeos_url_returns_string() {
        let config = PetalTongueConfig::default();
        let url = config.biomeos_url();
        assert!(url.len() < 256, "url should be reasonable length");
    }

    #[test]
    fn test_config_common_flattened() {
        let config = PetalTongueConfig::default();
        assert_eq!(config.common.name, "petalTongue");
        assert!(!config.common.host.is_empty());
    }
}
