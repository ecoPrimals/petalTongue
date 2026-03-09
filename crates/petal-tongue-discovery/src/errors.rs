// SPDX-License-Identifier: AGPL-3.0-only
//! Rich error types for discovery operations
//!
//! Modern, idiomatic error handling with `thiserror` and full context chains.

use std::time::Duration;
use thiserror::Error;

/// Discovery-specific errors with rich context
#[derive(Error, Debug)]
pub enum DiscoveryError {
    /// No providers could be discovered
    #[error("No providers found after trying {attempted} source(s): {sources}")]
    NoProvidersFound { attempted: usize, sources: String },

    /// Provider failed health check
    #[error("Provider '{name}' at {endpoint} failed health check")]
    HealthCheckFailed {
        name: String,
        endpoint: String,
        #[source]
        source: anyhow::Error,
    },

    /// Discovery operation timed out
    #[error("Discovery timeout after {duration:?}")]
    Timeout { duration: Duration },

    /// All providers failed
    #[error("All {count} provider(s) failed")]
    AllProvidersFailed { count: usize },

    /// HTTP request error
    #[error("HTTP request failed")]
    HttpError(#[from] reqwest::Error),

    /// mDNS discovery error
    #[error("mDNS discovery failed: {0}")]
    MdnsError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Invalid URL
    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },

    /// Provider returned invalid data
    #[error("Provider '{name}' returned invalid data: {reason}")]
    InvalidData { name: String, reason: String },

    /// Connection pool exhausted
    #[error("Connection pool exhausted for {endpoint}")]
    PoolExhausted { endpoint: String },
}

/// Result type for discovery operations
pub type DiscoveryResult<T> = Result<T, DiscoveryError>;

/// Failure information for graceful degradation
#[derive(Debug, Clone)]
pub struct DiscoveryFailure {
    /// Source that failed (mDNS, HTTP, env, etc.)
    pub source: String,
    /// Error message
    pub error: String,
    /// When the failure occurred
    pub timestamp: std::time::Instant,
}

impl DiscoveryFailure {
    pub fn new(source: impl Into<String>, error: impl std::fmt::Display) -> Self {
        Self {
            source: source.into(),
            error: error.to_string(),
            timestamp: std::time::Instant::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_providers_found_error() {
        let err = DiscoveryError::NoProvidersFound {
            attempted: 3,
            sources: "mDNS, HTTP, env".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("No providers found"));
        assert!(msg.contains("3 source(s)"));
        assert!(msg.contains("mDNS, HTTP, env"));
    }

    #[test]
    fn test_health_check_failed_error() {
        let err = DiscoveryError::HealthCheckFailed {
            name: "biomeOS".to_string(),
            endpoint: "http://localhost:3000".to_string(),
            source: anyhow::anyhow!("Connection refused"),
        };
        let msg = err.to_string();
        assert!(msg.contains("biomeOS"));
        assert!(msg.contains("http://localhost:3000"));
        assert!(msg.contains("failed health check"));
    }

    #[test]
    fn test_timeout_error() {
        let err = DiscoveryError::Timeout {
            duration: Duration::from_secs(5),
        };
        let msg = err.to_string();
        assert!(msg.contains("timeout"));
        assert!(msg.contains("5s"));
    }

    #[test]
    fn test_all_providers_failed_error() {
        let err = DiscoveryError::AllProvidersFailed { count: 3 };
        let msg = err.to_string();
        assert!(msg.contains("All"));
        assert!(msg.contains('3'));
        assert!(msg.contains("failed"));
    }

    #[test]
    fn test_mdns_error() {
        let err = DiscoveryError::MdnsError("Network unreachable".to_string());
        let msg = err.to_string();
        assert!(msg.contains("mDNS"));
        assert!(msg.contains("Network unreachable"));
    }

    #[test]
    fn test_config_error() {
        let err = DiscoveryError::ConfigError("Invalid timeout value".to_string());
        let msg = err.to_string();
        assert!(msg.contains("Configuration error"));
        assert!(msg.contains("Invalid timeout value"));
    }

    #[test]
    fn test_invalid_url_error() {
        let err = DiscoveryError::InvalidUrl {
            url: "not a valid url".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Invalid URL"));
        assert!(msg.contains("not a valid url"));
    }

    #[test]
    fn test_invalid_data_error() {
        let err = DiscoveryError::InvalidData {
            name: "TestProvider".to_string(),
            reason: "Missing required field 'endpoint'".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("TestProvider"));
        assert!(msg.contains("invalid data"));
        assert!(msg.contains("Missing required field"));
    }

    #[test]
    fn test_pool_exhausted_error() {
        let err = DiscoveryError::PoolExhausted {
            endpoint: "http://busy-server:8080".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("Connection pool exhausted"));
        assert!(msg.contains("http://busy-server:8080"));
    }

    #[test]
    fn test_discovery_failure_creation() {
        let failure = DiscoveryFailure::new("mDNS", "Timeout after 5s");

        assert_eq!(failure.source, "mDNS");
        assert_eq!(failure.error, "Timeout after 5s");
        assert!(failure.timestamp.elapsed().as_secs() < 1);
    }

    #[test]
    fn test_discovery_failure_with_complex_error() {
        let source_error = anyhow::anyhow!("Complex error with context");
        let failure = DiscoveryFailure::new("HTTP", source_error);

        assert_eq!(failure.source, "HTTP");
        assert!(failure.error.contains("Complex error"));
    }

    #[test]
    fn test_error_debug_format() {
        let err = DiscoveryError::NoProvidersFound {
            attempted: 2,
            sources: "test".to_string(),
        };
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("NoProvidersFound"));
        assert!(debug_str.contains("attempted"));
    }
}
