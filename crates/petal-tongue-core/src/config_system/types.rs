// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::constants::{DEFAULT_HEADLESS_PORT, DEFAULT_WEB_PORT};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
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

    /// Web mode configuration (PT-3: static serving, backend, caching)
    pub web: WebServeConfig,
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

/// Default web content backend identifier.
pub const DEFAULT_WEB_BACKEND: &str = "filesystem";

/// Default index file served for directory requests.
pub const DEFAULT_WEB_INDEX_FILE: &str = "index.html";

/// Web serving mode configuration (PT-3).
///
/// Controls static file serving, backend selection, caching, CORS, SPA routing,
/// and notebook rendering for `web` mode.
/// CLI flags (`--docroot`, `--backend`) take precedence over config values.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WebServeConfig {
    /// Document root directory for static file serving (PT-1 catch-all).
    /// When set, all requests not matched by API routes fall through to
    /// `tower_http::ServeDir` serving files from this path.
    pub docroot: Option<PathBuf>,

    /// Content backend: `"filesystem"` (default) or `"content-provider"`.
    pub backend: Cow<'static, str>,

    /// Default index file name served when a directory is requested.
    pub index_file: Cow<'static, str>,

    /// Static file cache TTL in seconds (for `Cache-Control` headers).
    pub cache_ttl_secs: u64,

    /// Hide code input cells when rendering `.ipynb` notebooks (show outputs only).
    pub strip_sources: bool,

    /// SPA (single-page application) mode: serve `index.html` for missing paths
    /// instead of 404, enabling client-side routing (React, Vue, Svelte, etc.).
    pub spa: bool,

    /// CORS allowed origins. Empty = same-origin only. `["*"]` = allow all.
    /// Example: `["https://primals.eco", "http://localhost:3000"]`
    pub allowed_origins: Vec<String>,
}

impl Default for WebServeConfig {
    fn default() -> Self {
        Self {
            docroot: None,
            backend: Cow::Borrowed(DEFAULT_WEB_BACKEND),
            index_file: Cow::Borrowed(DEFAULT_WEB_INDEX_FILE),
            cache_ttl_secs: 3600,
            strip_sources: false,
            spa: false,
            allowed_origins: Vec::new(),
        }
    }
}

impl WebServeConfig {
    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            docroot: other.docroot.or(self.docroot),
            backend: if other.backend.as_ref() == DEFAULT_WEB_BACKEND
                && self.backend.as_ref() != DEFAULT_WEB_BACKEND
            {
                self.backend
            } else {
                other.backend
            },
            index_file: other.index_file,
            cache_ttl_secs: other.cache_ttl_secs,
            strip_sources: other.strip_sources || self.strip_sources,
            spa: other.spa || self.spa,
            allowed_origins: if other.allowed_origins.is_empty() {
                self.allowed_origins
            } else {
                other.allowed_origins
            },
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
            cache_ttl: Duration::from_mins(1),
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test code")]

    use super::*;
    use crate::constants::{DEFAULT_HEADLESS_PORT, DEFAULT_WEB_PORT};
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    #[test]
    fn config_error_display_all_variants() {
        let err = ConfigError::LoadError("parse error".to_string());
        assert!(err.to_string().contains("Failed to load config file"));
        assert!(err.to_string().contains("parse error"));

        let err = ConfigError::ValidationError("port 0".to_string());
        assert!(err.to_string().contains("Invalid configuration"));
        assert!(err.to_string().contains("port 0"));

        let err = ConfigError::EnvError("bad var".to_string());
        assert!(err.to_string().contains("Environment variable error"));
        assert!(err.to_string().contains("bad var"));

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing file");
        let err = ConfigError::IoError(io_err);
        assert!(err.to_string().contains("IO error"));
        assert!(err.to_string().contains("missing file"));
    }

    #[test]
    fn config_default_nested_values() {
        let config = Config::default();

        assert_eq!(config.network.web_port, DEFAULT_WEB_PORT);
        assert_eq!(config.network.headless_port, DEFAULT_HEADLESS_PORT);
        assert_eq!(config.network.workers, 4);

        assert!(config.paths.runtime_dir.is_none());
        assert!(config.paths.data_dir.is_none());
        assert!(config.paths.config_dir.is_none());
        assert!(config.paths.cache_dir.is_none());

        assert_eq!(config.discovery.timeout, Duration::from_millis(200));
        assert_eq!(config.discovery.retry_attempts, 3);
        assert_eq!(config.discovery.retry_delay, Duration::from_millis(100));
        assert_eq!(config.discovery.cache_ttl, Duration::from_mins(1));

        assert_eq!(config.thresholds.health_threshold, 80.0);
        assert_eq!(config.thresholds.confidence_threshold, 80.0);
        assert_eq!(config.thresholds.cpu_warning, 50.0);
        assert_eq!(config.thresholds.cpu_critical, 80.0);
        assert_eq!(config.thresholds.memory_warning, 50.0);
        assert_eq!(config.thresholds.memory_critical, 80.0);

        assert_eq!(config.performance.max_fps, 240);
        assert_eq!(config.performance.max_frame_size, 16 * 1024 * 1024);
        assert_eq!(config.performance.max_width, 7680);
        assert_eq!(config.performance.max_height, 4320);

        assert!(config.web.docroot.is_none());
        assert_eq!(config.web.backend, "filesystem");
        assert_eq!(config.web.index_file, "index.html");
        assert_eq!(config.web.cache_ttl_secs, 3600);
        assert!(!config.web.strip_sources);
        assert!(!config.web.spa);
        assert!(config.web.allowed_origins.is_empty());
    }

    #[test]
    fn network_config_default_ports_and_addrs() {
        let network = NetworkConfig::default();

        assert_eq!(network.web_port, 3000);
        assert_eq!(network.web_port, DEFAULT_WEB_PORT);
        assert_eq!(network.headless_port, 8080);
        assert_eq!(network.headless_port, DEFAULT_HEADLESS_PORT);

        let expected_bind = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
        assert_eq!(
            network.web_addr(),
            SocketAddr::new(expected_bind, DEFAULT_WEB_PORT)
        );
        assert_eq!(
            network.headless_addr(),
            SocketAddr::new(expected_bind, DEFAULT_HEADLESS_PORT)
        );
    }

    #[test]
    fn network_config_merge_other_overrides_base() {
        let base = NetworkConfig {
            web_port: 3000,
            headless_port: 8080,
            workers: 4,
            ..Default::default()
        };
        let other = NetworkConfig {
            web_port: 4000,
            headless_port: 9000,
            workers: 8,
            ..Default::default()
        };

        let merged = NetworkConfig::merge(base, other);
        assert_eq!(merged.web_port, 4000);
        assert_eq!(merged.headless_port, 9000);
        assert_eq!(merged.workers, 8);
    }

    #[test]
    fn web_serve_config_default_values() {
        let web = WebServeConfig::default();

        assert!(web.docroot.is_none());
        assert_eq!(web.backend, "filesystem");
        assert_eq!(web.index_file, "index.html");
        assert_eq!(web.cache_ttl_secs, 3600);
        assert!(!web.strip_sources);
        assert!(!web.spa);
        assert!(web.allowed_origins.is_empty());
    }

    #[test]
    fn web_serve_config_merge_rules() {
        let base = WebServeConfig {
            docroot: Some(PathBuf::from("/base/docroot")),
            backend: Cow::Borrowed("content-provider"),
            index_file: Cow::Borrowed("base.html"),
            cache_ttl_secs: 60,
            strip_sources: true,
            spa: false,
            allowed_origins: vec!["https://base.example".to_owned()],
        };
        let other = WebServeConfig {
            docroot: Some(PathBuf::from("/other/docroot")),
            backend: Cow::Borrowed(DEFAULT_WEB_BACKEND),
            index_file: Cow::Borrowed("other.html"),
            cache_ttl_secs: 120,
            strip_sources: false,
            spa: true,
            allowed_origins: vec!["https://other.example".to_owned()],
        };

        let merged = base.merge(other);
        assert_eq!(merged.docroot, Some(PathBuf::from("/other/docroot")));
        assert_eq!(merged.backend, "content-provider");
        assert_eq!(merged.index_file, "other.html");
        assert_eq!(merged.cache_ttl_secs, 120);
        assert!(merged.strip_sources);
        assert!(merged.spa);
        assert_eq!(
            merged.allowed_origins,
            vec!["https://other.example".to_owned()]
        );

        let base = WebServeConfig {
            strip_sources: true,
            spa: true,
            allowed_origins: vec!["https://keep.example".to_owned()],
            ..Default::default()
        };
        let other = WebServeConfig {
            strip_sources: false,
            spa: false,
            allowed_origins: Vec::new(),
            ..Default::default()
        };
        let merged = base.merge(other);
        assert!(merged.strip_sources);
        assert!(merged.spa);
        assert_eq!(
            merged.allowed_origins,
            vec!["https://keep.example".to_owned()]
        );
    }

    #[test]
    fn paths_config_merge_other_overrides_none_falls_back() {
        let base = PathsConfig {
            runtime_dir: Some(PathBuf::from("/base/runtime")),
            data_dir: Some(PathBuf::from("/base/data")),
            config_dir: Some(PathBuf::from("/base/config")),
            cache_dir: Some(PathBuf::from("/base/cache")),
        };
        let other = PathsConfig {
            runtime_dir: None,
            data_dir: Some(PathBuf::from("/other/data")),
            config_dir: None,
            cache_dir: Some(PathBuf::from("/other/cache")),
        };

        let merged = base.merge(other);
        assert_eq!(merged.runtime_dir, Some(PathBuf::from("/base/runtime")));
        assert_eq!(merged.data_dir, Some(PathBuf::from("/other/data")));
        assert_eq!(merged.config_dir, Some(PathBuf::from("/base/config")));
        assert_eq!(merged.cache_dir, Some(PathBuf::from("/other/cache")));
    }

    #[test]
    fn paths_config_explicit_runtime_dir() {
        let paths = PathsConfig {
            runtime_dir: Some(PathBuf::from("/explicit/runtime")),
            ..Default::default()
        };

        assert_eq!(
            paths.runtime_dir().unwrap(),
            PathBuf::from("/explicit/runtime")
        );
    }

    #[test]
    fn discovery_config_default_values() {
        let discovery = DiscoveryConfig::default();

        assert_eq!(discovery.timeout, Duration::from_millis(200));
        assert_eq!(discovery.retry_attempts, 3);
        assert_eq!(discovery.retry_delay, Duration::from_millis(100));
        assert_eq!(discovery.cache_ttl, Duration::from_mins(1));
    }

    #[test]
    fn thresholds_config_default_values() {
        let thresholds = ThresholdsConfig::default();

        assert_eq!(thresholds.health_threshold, 80.0);
        assert_eq!(thresholds.confidence_threshold, 80.0);
        assert_eq!(thresholds.cpu_warning, 50.0);
        assert_eq!(thresholds.cpu_critical, 80.0);
        assert_eq!(thresholds.memory_warning, 50.0);
        assert_eq!(thresholds.memory_critical, 80.0);
    }

    #[test]
    fn performance_config_default_values() {
        let performance = PerformanceConfig::default();

        assert_eq!(performance.max_fps, 240);
        assert_eq!(performance.max_frame_size, 16 * 1024 * 1024);
        assert_eq!(performance.max_width, 7680);
        assert_eq!(performance.max_height, 4320);
    }

    #[test]
    fn config_serde_roundtrip() {
        let original = Config::default();
        let json = serde_json::to_string(&original).unwrap();
        let restored: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.network.web_port, original.network.web_port);
        assert_eq!(
            restored.network.headless_port,
            original.network.headless_port
        );
        assert_eq!(restored.network.workers, original.network.workers);
        assert_eq!(restored.paths.runtime_dir, original.paths.runtime_dir);
        assert_eq!(restored.paths.data_dir, original.paths.data_dir);
        assert_eq!(restored.discovery.timeout, original.discovery.timeout);
        assert_eq!(
            restored.discovery.retry_attempts,
            original.discovery.retry_attempts
        );
        assert_eq!(
            restored.thresholds.health_threshold,
            original.thresholds.health_threshold
        );
        assert_eq!(restored.performance.max_fps, original.performance.max_fps);
        assert_eq!(
            restored.performance.max_frame_size,
            original.performance.max_frame_size
        );
        assert_eq!(restored.web.backend, original.web.backend);
        assert_eq!(restored.web.index_file, original.web.index_file);
        assert_eq!(restored.web.cache_ttl_secs, original.web.cache_ttl_secs);
    }
}
