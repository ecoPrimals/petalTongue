// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed errors for `BiomeOS` API client operations.

use thiserror::Error;

/// Errors from `BiomeOS` API client operations.
#[derive(Debug, Error)]
pub enum BiomeOsClientError {
    /// Fixture mode requested but `test-fixtures` feature is not enabled.
    #[error(
        "Fixture mode requires test-fixtures feature. Use real biomeOS connection or build with --features test-fixtures."
    )]
    FixtureModeUnavailable,

    /// Network request failed (connection, timeout, etc.)
    #[error("Failed to connect to biomeOS at {url}: {source}")]
    Network {
        /// URL that failed
        url: String,
        /// Underlying error
        #[source]
        source: reqwest::Error,
    },

    /// Server returned an error status
    #[error("biomeOS API returned error status: {status}\nURL: {url}")]
    ServerError {
        /// HTTP status code
        status: u16,
        /// Request URL
        url: String,
    },

    /// Failed to parse API response (JSON decode error)
    #[error("Failed to parse biomeOS response: {0}")]
    Parse(String),

    /// JSON-RPC client: socket not found
    #[error("biomeOS socket not found: {0}")]
    SocketNotFound(String),

    /// JSON-RPC client: failed to connect to Unix socket
    #[error("Failed to connect to biomeOS: {0}")]
    Connect(String),

    /// JSON-RPC client: I/O error
    #[error("biomeOS I/O error: {0}")]
    Io(String),

    /// JSON-RPC client: server returned JSON-RPC error
    #[error("biomeOS JSON-RPC error: {0}")]
    JsonRpcError(String),

    /// JSON-RPC client: no result field in response
    #[error("No result field in JSON-RPC response")]
    NoResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display_fixture_mode_unavailable() {
        let err = BiomeOsClientError::FixtureModeUnavailable;
        let s = err.to_string();
        assert!(s.contains("Fixture mode"));
        assert!(s.contains("test-fixtures"));
    }

    #[test]
    fn test_error_display_server_error() {
        let err = BiomeOsClientError::ServerError {
            status: 404,
            url: "http://test/api".to_string(),
        };
        let s = err.to_string();
        assert!(s.contains("404"));
        assert!(s.contains("http://test/api"));
    }

    #[test]
    fn test_error_display_parse() {
        let err = BiomeOsClientError::Parse("invalid json".to_string());
        let s = err.to_string();
        assert!(s.contains("invalid json"));
    }

    #[test]
    fn test_error_display_socket_not_found() {
        let err = BiomeOsClientError::SocketNotFound("path not found".to_string());
        let s = err.to_string();
        assert!(s.contains("path not found"));
    }

    #[test]
    fn test_error_display_connect() {
        let err = BiomeOsClientError::Connect("connection failed".to_string());
        let s = err.to_string();
        assert!(s.contains("connection failed"));
    }

    #[test]
    fn test_error_display_io() {
        let err = BiomeOsClientError::Io("io error".to_string());
        let s = err.to_string();
        assert!(s.contains("io error"));
    }

    #[test]
    fn test_error_display_jsonrpc_error() {
        let err = BiomeOsClientError::JsonRpcError("rpc failed".to_string());
        let s = err.to_string();
        assert!(s.contains("rpc failed"));
    }

    #[test]
    fn test_error_display_no_result() {
        let err = BiomeOsClientError::NoResult;
        let s = err.to_string();
        assert!(s.contains("No result"));
    }

    #[test]
    fn test_error_impl_std_error() {
        use std::error::Error;
        let err = BiomeOsClientError::Parse("test".to_string());
        assert!(err.source().is_none());
        let err2 = BiomeOsClientError::NoResult;
        assert!(err2.source().is_none());
    }
}
