// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed errors for BTSP handshake operations.

use std::path::PathBuf;
use thiserror::Error;

/// Error during a BTSP provider RPC call or handshake step.
#[derive(Debug, Error)]
pub enum BtspHandshakeError {
    /// Failed to connect to the BearDog security provider socket.
    #[error("BTSP provider {path}: {source}")]
    ProviderConnect {
        /// Path to the provider socket that was unreachable.
        path: PathBuf,
        /// Underlying I/O error from the connection attempt.
        source: std::io::Error,
    },

    /// I/O error during handshake frame read/write.
    #[error("{context}: {source}")]
    Io {
        /// Which handshake step failed.
        context: &'static str,
        /// Underlying I/O error.
        source: std::io::Error,
    },

    /// JSON serialization or deserialization failed.
    #[error("{context}: {source}")]
    Json {
        /// Which serialization step failed.
        context: &'static str,
        /// Underlying serde error.
        source: serde_json::Error,
    },

    /// EOF received before a complete message.
    #[error("EOF before {expected}")]
    UnexpectedEof {
        /// What was expected before the stream ended.
        expected: &'static str,
    },

    /// The provider returned a JSON-RPC error object.
    #[error("BTSP provider error: {0}")]
    ProviderRpcError(serde_json::Value),

    /// The provider response contained no `result` field.
    #[error("no result in provider response")]
    NoResult,

    /// BearDog verification rejected the handshake.
    #[error("BTSP verify failed: {reason}")]
    VerifyFailed {
        /// Rejection reason from the security provider.
        reason: String,
    },

    /// Phase 3 key derivation failed (HKDF or cipher init).
    #[error("Phase 3 key derivation: {context}")]
    KeyDerivationFailed {
        /// Which derivation step failed.
        context: &'static str,
    },

    /// Phase 3 AEAD encrypt/decrypt operation failed.
    #[error("Phase 3 crypto: {context}")]
    Phase3Crypto {
        /// What failed (encrypt, decrypt, or frame validation).
        context: &'static str,
    },
}
