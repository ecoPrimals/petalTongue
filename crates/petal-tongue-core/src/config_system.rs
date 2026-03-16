// SPDX-License-Identifier: AGPL-3.0-or-later
//! Platform-Agnostic Configuration System
//!
//! TRUE PRIMAL principle: Zero hardcoding, XDG-compliant, environment-driven
//!
//! This module provides comprehensive configuration management that adapts
//! to the host environment without hardcoded assumptions.

use crate::constants::{DEFAULT_HEADLESS_PORT, DEFAULT_WEB_PORT};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

/// Configuration error types
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to load configuration file from disk
    #[error("Failed to load config file: {0}")]
    LoadError(String),

    /// Configuration values failed validation (e.g., port out of range)
    #[error("Invalid configuration: {0}")]
    ValidationError(String),

    /// Environment variable parsing or access error
    #[error("Environment variable error: {0}")]
    EnvError(String),

    /// File system I/O error during configuration operations
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Complete petalTongue configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Network configuration (ports, bindings)
    pub network: NetworkConfig,

    /// Path configuration (XDG-compliant)
    pub paths: PathsConfig,

    /// Discovery configuration (timeouts, fallbacks)
    pub discovery: DiscoveryConfig,

    /// Health and metric thresholds
    pub thresholds: ThresholdsConfig,

    /// Performance limits
    pub performance: PerformanceConfig,
}

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
    fn validate(&self) -> Result<(), ConfigError> {
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

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    /// Web mode bind address
    pub web_bind: IpAddr,
    /// Web mode port
    pub web_port: u16,

    /// Headless mode bind address
    pub headless_bind: IpAddr,
    /// Headless mode port
    pub headless_port: u16,

    /// Number of worker threads
    pub workers: usize,
}

impl NetworkConfig {
    const fn merge(_base: Self, other: Self) -> Self {
        other // Simple override for now; could be more sophisticated
    }

    /// Get web socket address
    #[must_use]
    pub const fn web_addr(&self) -> SocketAddr {
        SocketAddr::new(self.web_bind, self.web_port)
    }

    /// Get headless socket address
    #[must_use]
    pub const fn headless_addr(&self) -> SocketAddr {
        SocketAddr::new(self.headless_bind, self.headless_port)
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            web_bind: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            web_port: DEFAULT_WEB_PORT,
            headless_bind: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
            headless_port: DEFAULT_HEADLESS_PORT,
            workers: 4,
        }
    }
}

/// Path configuration (XDG-compliant)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathsConfig {
    /// Runtime directory for sockets (`XDG_RUNTIME_DIR`)
    pub runtime_dir: Option<PathBuf>,

    /// Data directory for persistent storage (`XDG_DATA_HOME`)
    pub data_dir: Option<PathBuf>,

    /// Config directory (`XDG_CONFIG_HOME`)
    pub config_dir: Option<PathBuf>,

    /// Cache directory (`XDG_CACHE_HOME`)
    pub cache_dir: Option<PathBuf>,
}

impl PathsConfig {
    fn merge(self, other: Self) -> Self {
        Self {
            runtime_dir: other.runtime_dir.or(self.runtime_dir),
            data_dir: other.data_dir.or(self.data_dir),
            config_dir: other.config_dir.or(self.config_dir),
            cache_dir: other.cache_dir.or(self.cache_dir),
        }
    }

    /// Get runtime directory (with fallback)
    ///
    /// # Errors
    ///
    /// Returns an error if no explicit path is set and the XDG runtime directory
    /// cannot be determined (e.g. `XDG_RUNTIME_DIR` is unset on Linux).
    pub fn runtime_dir(&self) -> Result<PathBuf, ConfigError> {
        use crate::platform_dirs;

        if let Some(ref dir) = self.runtime_dir {
            return Ok(dir.clone());
        }

        platform_dirs::runtime_dir().map_err(|e| ConfigError::EnvError(e.to_string()))
    }

    /// Get data directory (with fallback)
    ///
    /// # Errors
    ///
    /// Returns an error if no explicit path is set and the XDG data directory
    /// cannot be determined (e.g. `XDG_DATA_HOME` is unset and home dir is unknown).
    pub fn data_dir(&self) -> Result<PathBuf, ConfigError> {
        use crate::platform_dirs;

        if let Some(ref dir) = self.data_dir {
            return Ok(dir.clone());
        }

        platform_dirs::data_dir().map_err(|e| ConfigError::EnvError(e.to_string()))
    }
}

/// Discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DiscoveryConfig {
    /// Discovery timeout
    pub timeout: Duration,

    /// Retry attempts
    pub retry_attempts: usize,

    /// Retry delay
    pub retry_delay: Duration,

    /// Cache TTL
    pub cache_ttl: Duration,
}

impl DiscoveryConfig {
    const fn merge(_base: Self, other: Self) -> Self {
        other
    }
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(200),
            retry_attempts: 3,
            retry_delay: Duration::from_millis(100),
            cache_ttl: Duration::from_secs(60),
        }
    }
}

/// Health and metric thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThresholdsConfig {
    /// Health threshold percentage (0-100)
    pub health_threshold: f32,

    /// Confidence threshold percentage (0-100)
    pub confidence_threshold: f32,

    /// CPU warning threshold percentage (0-100)
    pub cpu_warning: f32,

    /// CPU critical threshold percentage (0-100)
    pub cpu_critical: f32,

    /// Memory warning threshold percentage (0-100)
    pub memory_warning: f32,

    /// Memory critical threshold percentage (0-100)
    pub memory_critical: f32,
}

impl ThresholdsConfig {
    const fn merge(_base: Self, other: Self) -> Self {
        other
    }
}

impl Default for ThresholdsConfig {
    fn default() -> Self {
        Self {
            health_threshold: 80.0,
            confidence_threshold: 80.0,
            cpu_warning: 50.0,
            cpu_critical: 80.0,
            memory_warning: 50.0,
            memory_critical: 80.0,
        }
    }
}

/// Performance limits
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PerformanceConfig {
    /// Maximum FPS for rendering
    pub max_fps: u32,

    /// Maximum frame buffer size (bytes)
    pub max_frame_size: usize,

    /// Maximum resolution width
    pub max_width: u32,

    /// Maximum resolution height
    pub max_height: u32,
}

impl PerformanceConfig {
    const fn merge(_base: Self, other: Self) -> Self {
        other
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_fps: 240,
            max_frame_size: 16 * 1024 * 1024, // 16 MB
            max_width: 7680,                  // 8K width
            max_height: 4320,                 // 8K height
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::env_test_helpers;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.network.web_port, 3000);
        assert_eq!(config.network.headless_port, 8080);
        assert!((config.thresholds.health_threshold - 80.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());

        config.network.web_port = 0;
        assert!(config.validate().is_err());

        config.network.web_port = 3000;
        config.thresholds.health_threshold = 150.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_merge() {
        let base = Config::default();
        let mut override_cfg = Config::default();
        override_cfg.network.web_port = 9000;

        let merged = base.merge(override_cfg);
        assert_eq!(merged.network.web_port, 9000);
    }

    #[test]
    fn test_network_socket_addrs() {
        let config = NetworkConfig::default();
        let web_addr = config.web_addr();
        assert_eq!(web_addr.port(), 3000);

        let headless_addr = config.headless_addr();
        assert_eq!(headless_addr.port(), 8080);
    }

    #[test]
    fn test_config_from_file_valid() {
        let temp = std::env::temp_dir().join("petaltongue-config-test.toml");
        let contents = r#"
[network]
web_bind = "0.0.0.0"
web_port = 4000
headless_bind = "0.0.0.0"
headless_port = 9000
workers = 4

[thresholds]
health_threshold = 75.0
"#;
        std::fs::write(&temp, contents).expect("write temp config");
        let config = Config::from_file(&temp).expect("load config");
        assert_eq!(config.network.web_port, 4000);
        assert_eq!(config.network.headless_port, 9000);
        assert!((config.thresholds.health_threshold - 75.0).abs() < f32::EPSILON);
        let _ = std::fs::remove_file(&temp);
    }

    #[test]
    fn test_config_from_file_nonexistent() {
        let result = Config::from_file("/nonexistent/path/config.toml");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::IoError(_)));
    }

    #[test]
    fn test_config_from_file_invalid_toml() {
        let temp = std::env::temp_dir().join("petaltongue-invalid-config.toml");
        std::fs::write(&temp, "invalid toml {{{").expect("write temp");
        let result = Config::from_file(&temp);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::LoadError(_)));
        let _ = std::fs::remove_file(&temp);
    }

    #[test]
    fn test_config_from_env_with_overrides() {
        let temp = std::env::temp_dir().join("petaltongue-env-override-test.toml");
        let contents = r#"
[network]
web_bind = "0.0.0.0"
web_port = 3000
headless_bind = "0.0.0.0"
headless_port = 8080
workers = 4
"#;
        std::fs::write(&temp, contents).expect("write");
        let path = temp.to_str().expect("path");
        env_test_helpers::with_env_vars(
            &[
                ("PETALTONGUE_CONFIG", Some(path)),
                ("PETALTONGUE_WEB_PORT", Some("5000")),
                ("PETALTONGUE_HEADLESS_PORT", Some("9090")),
                ("PETALTONGUE_DISCOVERY_TIMEOUT", Some("500")),
            ],
            || {
                let config = Config::from_env().expect("from_env");
                assert_eq!(config.network.web_port, 5000);
                assert_eq!(config.network.headless_port, 9090);
                assert_eq!(config.discovery.timeout.as_millis(), 500);
            },
        );
        let _ = std::fs::remove_file(&temp);
    }

    #[test]
    fn test_config_from_env_invalid_web_port() {
        let temp = std::env::temp_dir().join("petaltongue-invalid-port-test.toml");
        let contents = r#"
[network]
web_bind = "0.0.0.0"
web_port = 3000
headless_bind = "0.0.0.0"
headless_port = 8080
workers = 4
"#;
        std::fs::write(&temp, contents).expect("write");
        let path = temp.to_str().expect("path");
        env_test_helpers::with_env_vars(
            &[
                ("PETALTONGUE_CONFIG", Some(path)),
                ("PETALTONGUE_WEB_PORT", Some("not-a-number")),
            ],
            || {
                let result = Config::from_env();
                assert!(result.is_err());
                let err = result.unwrap_err();
                assert!(
                    matches!(err, ConfigError::EnvError(_)),
                    "expected EnvError, got {err:?}"
                );
            },
        );
        let _ = std::fs::remove_file(&temp);
    }

    #[test]
    fn test_paths_config_runtime_dir_explicit() {
        let paths = PathsConfig {
            runtime_dir: Some(PathBuf::from("/custom/runtime")),
            data_dir: None,
            config_dir: None,
            cache_dir: None,
        };
        let dir = paths.runtime_dir().expect("runtime_dir");
        assert!(dir.to_string_lossy().contains("custom"));
    }

    #[test]
    fn test_paths_config_data_dir_explicit() {
        let paths = PathsConfig {
            runtime_dir: None,
            data_dir: Some(PathBuf::from("/custom/data")),
            config_dir: None,
            cache_dir: None,
        };
        let dir = paths.data_dir().expect("data_dir");
        assert!(dir.to_string_lossy().contains("custom"));
    }

    #[test]
    fn test_paths_config_merge_via_config() {
        let mut base = Config::default();
        base.paths.runtime_dir = Some(PathBuf::from("/base/runtime"));
        let mut other = Config::default();
        other.paths.runtime_dir = Some(PathBuf::from("/other/runtime"));
        other.paths.data_dir = Some(PathBuf::from("/other/data"));
        let merged = base.merge(other);
        assert!(
            merged
                .paths
                .runtime_dir
                .as_ref()
                .is_some_and(|p| p.to_string_lossy().contains("other"))
        );
        assert!(
            merged
                .paths
                .data_dir
                .as_ref()
                .is_some_and(|p| p.to_string_lossy().contains("other"))
        );
    }

    #[test]
    fn test_discovery_config_default() {
        let d = DiscoveryConfig::default();
        assert_eq!(d.timeout.as_millis(), 200);
        assert_eq!(d.retry_attempts, 3);
        assert_eq!(d.retry_delay.as_millis(), 100);
        assert_eq!(d.cache_ttl.as_secs(), 60);
    }

    #[test]
    fn test_thresholds_config_default() {
        let t = ThresholdsConfig::default();
        assert!((t.health_threshold - 80.0).abs() < f32::EPSILON);
        assert!((t.cpu_warning - 50.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_performance_config_default() {
        let p = PerformanceConfig::default();
        assert_eq!(p.max_fps, 240);
        assert_eq!(p.max_width, 7680);
        assert_eq!(p.max_height, 4320);
    }

    #[test]
    fn test_config_error_display() {
        let err = ConfigError::LoadError("parse error".to_string());
        assert!(err.to_string().contains("parse error"));

        let err = ConfigError::ValidationError("port 0".to_string());
        assert!(err.to_string().contains("port 0"));

        let err = ConfigError::EnvError("bad var".to_string());
        assert!(err.to_string().contains("bad var"));
    }
}
