//! Common configuration for primals.
//!
//! Previously sourced from sourdough-core, now self-contained for independence.

use serde::{Deserialize, Serialize};

/// Common configuration shared by all primals.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommonConfig {
    /// Name of the primal instance
    pub name: String,

    /// Host to bind to
    #[serde(default = "default_host")]
    pub host: String,

    /// Port to bind to
    #[serde(default = "default_port")]
    pub port: u16,

    /// Log level
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self {
            name: "primal".to_string(),
            host: default_host(),
            port: default_port(),
            log_level: default_log_level(),
        }
    }
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_log_level() -> String {
    "info".to_string()
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
}
