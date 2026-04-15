// SPDX-License-Identifier: AGPL-3.0-or-later

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
    pub(crate) const fn merge(_base: Self, other: Self) -> Self {
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
    pub(crate) fn merge(self, other: Self) -> Self {
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
    pub(crate) const fn merge(_base: Self, other: Self) -> Self {
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
    pub(crate) const fn merge(_base: Self, other: Self) -> Self {
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
    pub(crate) const fn merge(_base: Self, other: Self) -> Self {
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
