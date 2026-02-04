//! Platform-Agnostic Configuration System
//!
//! TRUE PRIMAL principle: Zero hardcoding, XDG-compliant, environment-driven
//!
//! This module provides comprehensive configuration management that adapts
//! to the host environment without hardcoded assumptions.

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// 2. Config file (if specified via PETALTONGUE_CONFIG)
    /// 3. XDG config dir (~/.config/petaltongue/config.toml)
    /// 4. Defaults (lowest)
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut config = Self::default();

        // Try to load from file (if specified or in XDG config dir)
        if let Ok(path) = std::env::var("PETALTONGUE_CONFIG") {
            config = config.merge(Self::from_file(&path)?);
        } else if let Ok(xdg_config) = Self::xdg_config_path() {
            if xdg_config.exists() {
                config = config.merge(Self::from_file(&xdg_config)?);
            }
        }

        // Override with environment variables
        config.apply_env_overrides()?;

        // Validate
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from TOML file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path)?;
        toml::from_str(&contents).map_err(|e| ConfigError::LoadError(e.to_string()))
    }

    /// Get XDG config path
    fn xdg_config_path() -> Result<PathBuf, ConfigError> {
        use crate::platform_dirs;

        let config_dir =
            platform_dirs::config_dir().map_err(|e| ConfigError::EnvError(e.to_string()))?;

        Ok(config_dir.join("petaltongue").join("config.toml"))
    }

    /// Merge configurations (other overrides self)
    pub fn merge(mut self, other: Self) -> Self {
        self.network = self.network.merge(other.network);
        self.paths = self.paths.merge(other.paths);
        self.discovery = self.discovery.merge(other.discovery);
        self.thresholds = self.thresholds.merge(other.thresholds);
        self.performance = self.performance.merge(other.performance);
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
            return Err(ConfigError::ValidationError("Invalid web_port: cannot be 0".to_string()));
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

impl Default for Config {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            paths: PathsConfig::default(),
            discovery: DiscoveryConfig::default(),
            thresholds: ThresholdsConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn merge(self, other: Self) -> Self {
        other // Simple override for now; could be more sophisticated
    }

    /// Get web socket address
    pub fn web_addr(&self) -> SocketAddr {
        SocketAddr::new(self.web_bind, self.web_port)
    }

    /// Get headless socket address
    pub fn headless_addr(&self) -> SocketAddr {
        SocketAddr::new(self.headless_bind, self.headless_port)
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            web_bind: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            web_port: 3000,
            headless_bind: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            headless_port: 8080,
            workers: 4,
        }
    }
}

/// Path configuration (XDG-compliant)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    /// Runtime directory for sockets (XDG_RUNTIME_DIR)
    pub runtime_dir: Option<PathBuf>,

    /// Data directory for persistent storage (XDG_DATA_HOME)
    pub data_dir: Option<PathBuf>,

    /// Config directory (XDG_CONFIG_HOME)
    pub config_dir: Option<PathBuf>,

    /// Cache directory (XDG_CACHE_HOME)
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
    pub fn runtime_dir(&self) -> Result<PathBuf, ConfigError> {
        if let Some(ref dir) = self.runtime_dir {
            return Ok(dir.clone());
        }

        use crate::platform_dirs;
        platform_dirs::runtime_dir().map_err(|e| ConfigError::EnvError(e.to_string()))
    }

    /// Get data directory (with fallback)
    pub fn data_dir(&self) -> Result<PathBuf, ConfigError> {
        if let Some(ref dir) = self.data_dir {
            return Ok(dir.clone());
        }

        use crate::platform_dirs;
        platform_dirs::data_dir().map_err(|e| ConfigError::EnvError(e.to_string()))
    }
}

impl Default for PathsConfig {
    fn default() -> Self {
        Self {
            runtime_dir: None, // Will use XDG at runtime
            data_dir: None,
            config_dir: None,
            cache_dir: None,
        }
    }
}

/// Discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn merge(self, other: Self) -> Self {
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
    fn merge(self, other: Self) -> Self {
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
    fn merge(self, other: Self) -> Self {
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

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.network.web_port, 3000);
        assert_eq!(config.network.headless_port, 8080);
        assert_eq!(config.thresholds.health_threshold, 80.0);
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
}
