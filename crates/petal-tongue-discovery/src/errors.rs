// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rich error types for discovery operations
//!
//! Modern, idiomatic error handling with `thiserror` and full context chains.

use std::time::Duration;
use thiserror::Error;

/// Discovery-specific errors with rich context
#[derive(Error, Debug)]
pub enum DiscoveryError {
    // --- Discovery / Provider not found ---
    /// No providers could be discovered
    #[error("No providers found after trying {attempted} source(s): {sources}")]
    NoProvidersFound { attempted: usize, sources: String },

    /// Discovery service (Songbird) not found
    #[error("Discovery service not found. Is it running? (looking for {socket_name})")]
    DiscoveryServiceNotFound { socket_name: String },

    /// Neural API not found
    #[error("Neural API not found. Is biomeOS nucleus serve running? (looking for {socket_name})")]
    NeuralApiNotFound { socket_name: String },

    /// No JSON-RPC providers found
    #[error("No JSON-RPC providers found: {message}")]
    NoJsonRpcProvidersFound { message: String },

    /// All providers failed
    #[error("All {count} provider(s) failed")]
    AllProvidersFailed { count: usize },

    // --- Health / Provider errors ---
    /// Provider failed health check
    #[error("Provider '{name}' at {endpoint} failed health check")]
    HealthCheckFailed {
        name: String,
        endpoint: String,
        #[source]
        source: HealthCheckSource,
    },

    /// Provider returned HTTP error status
    #[error("Provider returned error status: {status}")]
    ProviderHttpError {
        status: u16,
        endpoint: Option<String>,
    },

    // --- Timeouts ---
    /// Discovery operation timed out
    #[error("Discovery timeout after {duration:?}")]
    Timeout { duration: Duration },

    /// Connection timeout
    #[error("Connection timeout to {endpoint}")]
    ConnectionTimeout { endpoint: String },

    /// Write timeout
    #[error("Write timeout to {endpoint}")]
    WriteTimeout { endpoint: String },

    /// Read timeout
    #[error("Read timeout from {endpoint}")]
    ReadTimeout { endpoint: String },

    /// RPC/timeout for JSON-RPC call
    #[error("RPC timeout: {context}")]
    RpcTimeout { context: String },

    /// Operation timed out (generic)
    #[error("Operation timed out after {duration:?}")]
    OperationTimedOut { duration: Duration },

    // --- Network / HTTP ---
    /// HTTP request error
    #[error("HTTP request failed")]
    HttpError(#[from] reqwest::Error),

    /// mDNS discovery error
    #[error("mDNS discovery failed: {0}")]
    MdnsError(String),

    /// Not a DNS response packet
    #[error("Not a DNS response packet")]
    NotDnsResponse,

    /// No port advertised in mDNS service
    #[error("No port advertised in mDNS service - refusing to assume default")]
    NoPortAdvertisedInMdns,

    // --- I/O ---
    /// I/O error
    #[error("I/O error")]
    Io(#[from] std::io::Error),

    /// Failed to read file
    #[error("Failed to read file: {path}")]
    FileReadError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    // --- JSON / Parsing ---
    /// JSON parse/serialize error
    #[error("JSON error")]
    Json(#[from] serde_json::Error),

    /// JSON-RPC protocol error
    #[error("JSON-RPC error: {message}")]
    JsonRpcError { code: Option<i32>, message: String },

    /// Request ID mismatch in JSON-RPC response
    #[error("Request ID mismatch: expected {expected}, got {actual}")]
    RequestIdMismatch {
        expected: u64,
        actual: serde_json::Value,
    },

    /// No result in JSON-RPC response
    #[error("No result in response{context}")]
    NoResultInResponse { context: String },

    /// Expected array in response
    #[error("Expected array{context}")]
    ExpectedArray { context: String },

    /// Missing required field
    #[error("Missing '{field}' field{context}")]
    MissingField { field: String, context: String },

    /// Scenario/ecosystem parse error
    #[error("Scenario parse error: {message}")]
    ScenarioParseError { message: String },

    /// Failed to parse proprioception or similar structured data
    #[error("Failed to parse {data_type}: {message}")]
    ParseError { data_type: String, message: String },

    // --- DNS parsing ---
    /// DNS parse error
    #[error("DNS parse error: {message}")]
    DnsParseError { message: String },

    // --- Configuration ---
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

    /// Retry attempts exhausted
    #[error("All retry attempts failed")]
    RetryExhausted,

    /// Upstream integration error (e.g. device management JSON-RPC), when no more specific variant applies
    #[error("Integration error: {0}")]
    Integration(String),
}

/// Concrete source types for health-check failures (replaces `Box<dyn Error>`).
#[derive(Error, Debug)]
pub enum HealthCheckSource {
    /// I/O error (e.g. Unix socket connect failure)
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// HTTP request error
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    /// JSON parse error on health-check response
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Upstream integration or other failure (message only)
    #[error("{0}")]
    Upstream(String),
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
            source: std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "Connection refused",
            )
            .into(),
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

    #[test]
    fn test_discovery_failure_display_source() {
        let failure = DiscoveryFailure::new("HTTP", "Connection refused");
        assert_eq!(failure.source, "HTTP");
        assert_eq!(failure.error, "Connection refused");
    }

    #[test]
    fn test_discovery_failure_with_string_into() {
        let failure = DiscoveryFailure::new("mDNS".to_string(), "timeout".to_string());
        assert_eq!(failure.source, "mDNS");
        assert_eq!(failure.error, "timeout");
    }

    #[test]
    fn test_discovery_result_type() {
        let ok_result: DiscoveryResult<i32> = Ok(42);
        let val = match ok_result {
            Ok(v) => v,
            Err(e) => panic!("expected Ok, got {e:?}"),
        };
        assert_eq!(val, 42);

        let err_result: DiscoveryResult<i32> =
            Err(DiscoveryError::ConfigError("bad config".to_string()));
        assert!(err_result.is_err());
    }

    #[test]
    fn test_error_display_format() {
        let err = DiscoveryError::InvalidUrl {
            url: "bad://url".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("Invalid URL"));
        assert!(display.contains("bad://url"));
    }

    #[test]
    fn test_error_source_chain() {
        let inner = std::io::Error::other("inner error");
        let err = DiscoveryError::HealthCheckFailed {
            name: "test".to_string(),
            endpoint: "http://localhost".to_string(),
            source: inner.into(),
        };
        let display = err.to_string();
        assert!(display.contains("test"));
        assert!(display.contains("http://localhost"));
        assert!(display.contains("health check"));
    }
}
