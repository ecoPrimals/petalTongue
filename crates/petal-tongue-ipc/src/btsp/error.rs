// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed errors for BTSP handshake operations.

use std::path::PathBuf;
use thiserror::Error;

/// Error during a BTSP provider RPC call or handshake step.
#[derive(Debug, Error)]
pub enum BtspHandshakeError {
    /// Failed to connect to the security provider socket.
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

    /// Security provider rejected the handshake verification.
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "test code")]

    use super::*;
    use std::io;
    use std::path::PathBuf;

    #[test]
    fn provider_connect_display() {
        let err = BtspHandshakeError::ProviderConnect {
            path: PathBuf::from("/run/provider.sock"),
            source: io::Error::new(io::ErrorKind::NotFound, "no such file"),
        };
        let msg = err.to_string();
        assert!(msg.contains("BTSP provider"));
        assert!(msg.contains("/run/provider.sock"));
        assert!(msg.contains("no such file"));
    }

    #[test]
    fn io_display() {
        let err = BtspHandshakeError::Io {
            context: "reading hello",
            source: io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"),
        };
        assert_eq!(err.to_string(), "reading hello: broken pipe");
    }

    #[test]
    fn json_display() {
        let json_err = serde_json::from_str::<()>("bad").unwrap_err();
        let err = BtspHandshakeError::Json {
            context: "decode frame",
            source: json_err,
        };
        let msg = err.to_string();
        assert!(msg.starts_with("decode frame: "));
    }

    #[test]
    fn unexpected_eof_display() {
        let err = BtspHandshakeError::UnexpectedEof {
            expected: "handshake response",
        };
        assert_eq!(err.to_string(), "EOF before handshake response");
    }

    #[test]
    fn provider_rpc_error_display() {
        let err = BtspHandshakeError::ProviderRpcError(serde_json::json!({
            "code": -32000,
            "message": "session expired"
        }));
        let msg = err.to_string();
        assert!(msg.starts_with("BTSP provider error: "));
        assert!(msg.contains("session expired"));
    }

    #[test]
    fn no_result_display() {
        let err = BtspHandshakeError::NoResult;
        assert_eq!(err.to_string(), "no result in provider response");
    }

    #[test]
    fn verify_failed_display() {
        let err = BtspHandshakeError::VerifyFailed {
            reason: "invalid token".to_owned(),
        };
        assert_eq!(err.to_string(), "BTSP verify failed: invalid token");
    }

    #[test]
    fn key_derivation_failed_display() {
        let err = BtspHandshakeError::KeyDerivationFailed {
            context: "HKDF expand",
        };
        assert_eq!(err.to_string(), "Phase 3 key derivation: HKDF expand");
    }

    #[test]
    fn phase3_crypto_display() {
        let err = BtspHandshakeError::Phase3Crypto {
            context: "decrypt frame",
        };
        assert_eq!(err.to_string(), "Phase 3 crypto: decrypt frame");
    }
}
