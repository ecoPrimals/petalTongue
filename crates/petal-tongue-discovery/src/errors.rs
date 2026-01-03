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
