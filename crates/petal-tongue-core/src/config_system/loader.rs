// SPDX-License-Identifier: AGPL-3.0-or-later

//! Load, merge, and validate [`Config`](super::types::Config) (environment and TOML).

use std::path::PathBuf;
use std::time::Duration;

use super::types::{
    Config, ConfigError, DiscoveryConfig, NetworkConfig, PerformanceConfig, ThresholdsConfig,
};

impl Config {
    /// Load configuration from environment
    ///
    /// Priority:
    /// 1. Environment variables (highest)
    /// 2. Config file (if specified via `PETALTONGUE_CONFIG`)
    /// 3. XDG config dir (~/.config/petaltongue/config.toml)
    /// 4. Defaults (lowest)
    ///
    /// # Errors
    ///
    /// Returns an error if a config file exists but cannot be read or parsed, if
    /// environment overrides (e.g. `PETALTONGUE_WEB_PORT`) are invalid, or if
    /// validation fails (e.g. `web_port` is 0 or `health_threshold` > 100).
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Try to load from file (if specified or in XDG config dir)
        if let Ok(path) = std::env::var("PETALTONGUE_CONFIG") {
            config = config.merge(Self::from_file(&path)?);
        } else if let Ok(xdg_config) = Self::xdg_config_path()
            && xdg_config.exists()
        {
            config = config.merge(Self::from_file(&xdg_config)?);
        }

        // Override with environment variables
        config.apply_env_overrides()?;

        // Validate
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from TOML file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read, or if the TOML content is invalid.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)?;
        toml::from_str(&contents).map_err(|e| ConfigError::LoadError(e.to_string()))
    }

    /// Get XDG config path
    fn xdg_config_path() -> Result<PathBuf, ConfigError> {
        use crate::platform_dirs;

        let config_dir =
            platform_dirs::config_dir().map_err(|e| ConfigError::EnvError(e.to_string()))?;

        Ok(config_dir
            .join(crate::constants::APP_DIR_NAME)
            .join("config.toml"))
    }

    /// Merge configurations (other overrides self)
    #[must_use]
    pub fn merge(mut self, other: Self) -> Self {
        self.network = NetworkConfig::merge(self.network, other.network);
        self.paths = self.paths.merge(other.paths);
        self.discovery = DiscoveryConfig::merge(self.discovery, other.discovery);
        self.thresholds = ThresholdsConfig::merge(self.thresholds, other.thresholds);
        self.performance = PerformanceConfig::merge(self.performance, other.performance);
        self
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        // Network overrides
        if let Ok(port) = std::env::var("PETALTONGUE_WEB_PORT") {
            self.network.web_port = port
                .parse()
                .map_err(|_| ConfigError::EnvError("Invalid WEB_PORT".to_string()))?;
        }
        if let Ok(port) = std::env::var("PETALTONGUE_HEADLESS_PORT") {
            self.network.headless_port = port
                .parse()
                .map_err(|_| ConfigError::EnvError("Invalid HEADLESS_PORT".to_string()))?;
        }

        // Discovery overrides
        if let Ok(timeout) = std::env::var("PETALTONGUE_DISCOVERY_TIMEOUT") {
            let ms = timeout
                .parse()
                .map_err(|_| ConfigError::EnvError("Invalid DISCOVERY_TIMEOUT".to_string()))?;
            self.discovery.timeout = Duration::from_millis(ms);
        }

        Ok(())
    }

    /// Validate configuration
    pub(crate) fn validate(&self) -> Result<(), ConfigError> {
        // Validate ports are in valid range (u16 already guarantees <= 65535)
        if self.network.web_port == 0 {
            return Err(ConfigError::ValidationError(
                "Invalid web_port: cannot be 0".to_string(),
            ));
        }

        // Validate thresholds are percentages
        if self.thresholds.health_threshold > 100.0 {
            return Err(ConfigError::ValidationError(
                "health_threshold must be <= 100".to_string(),
            ));
        }

        Ok(())
    }
}
