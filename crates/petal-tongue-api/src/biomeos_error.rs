// SPDX-License-Identifier: AGPL-3.0-only
//! Typed errors for `BiomeOS` API client operations.

use thiserror::Error;

/// Errors from `BiomeOS` API client operations.
#[derive(Debug, Error)]
pub enum BiomeOsClientError {
    /// Mock mode requested but test-fixtures feature is not enabled
    #[error(
        "Mock mode requires test-fixtures feature. Use real biomeOS connection or build with --features test-fixtures."
    )]
    MockModeUnavailable,

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
