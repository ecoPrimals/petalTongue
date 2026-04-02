// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed errors for primal registration with the discovery service.

use thiserror::Error;

/// Errors from primal registration and heartbeat operations.
#[derive(Debug, Error)]
pub enum PrimalRegistrationError {
    /// I/O error (connect, write, read)
    #[error("Registration I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to serialize or parse JSON
    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Discovery service returned a JSON-RPC error
    #[error("Discovery service returned error: {0}")]
    RegistrationRejected(serde_json::Value),
}
