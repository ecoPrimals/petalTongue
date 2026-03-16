// SPDX-License-Identifier: AGPL-3.0-only
//! Typed errors for entropy capture and streaming.

use thiserror::Error;

/// Errors from entropy capture and streaming operations.
#[derive(Debug, Error)]
pub enum EntropyError {
    /// Failed to serialize entropy data
    #[error("Failed to serialize entropy: {0}")]
    Serialize(#[from] serde_json::Error),

    /// Failed to encrypt entropy data
    #[error("Encryption failed")]
    Encrypt,

    /// Failed to decrypt entropy data
    #[error("Decryption failed")]
    Decrypt,

    /// Failed to build HTTP client or send request
    #[error("Network request failed: {0}")]
    Network(#[from] reqwest::Error),

    /// Server rejected the entropy submission
    #[error("Server rejected entropy: {status} - {body}")]
    ServerRejected {
        /// HTTP status code from the server
        status: u16,
        /// Response body from the server
        body: String,
    },
}
