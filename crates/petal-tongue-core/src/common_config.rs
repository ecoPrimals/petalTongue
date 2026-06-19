// SPDX-License-Identifier: AGPL-3.0-or-later
//! Common configuration for primals.
//!
//! Previously sourced from sourdough-core, now self-contained for independence.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Common configuration shared by all primals.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommonConfig {
    /// Name of the primal instance
    pub name: String,

    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: Cow<'static, str>,

    /// Port to bind to
    #[serde(default = "default_port")]
    pub port: u16,

    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: Cow<'static, str>,
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self {
            name: "primal".to_owned(),
            host: default_host(),
            port: default_port(),
            log_level: default_log_level(),
        }
    }
}

const fn default_host() -> Cow<'static, str> {
    Cow::Borrowed(crate::constants::DEFAULT_LOOPBACK_HOST)
}

const fn default_port() -> u16 {
    crate::constants::DEFAULT_HEADLESS_PORT
}

const fn default_log_level() -> Cow<'static, str> {
    Cow::Borrowed("info")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CommonConfig::default();
        assert_eq!(config.name, "primal");
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = CommonConfig {
            name: "test-primal".to_owned(),
            host: Cow::Borrowed("0.0.0.0"),
            port: 9000,
            log_level: Cow::Borrowed("debug"),
        };
        let json = serde_json::to_string(&config).expect("serialize");
        let restored: CommonConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(config.name, restored.name);
        assert_eq!(config.host, restored.host);
        assert_eq!(config.port, restored.port);
        assert_eq!(config.log_level, restored.log_level);
    }

    #[test]
    fn test_config_deserialize_with_defaults() {
        let json = r#"{"name": "minimal"}"#;
        let config: CommonConfig = serde_json::from_str(json).expect("deserialize");
        assert_eq!(config.name, "minimal");
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, "info");
    }
}
