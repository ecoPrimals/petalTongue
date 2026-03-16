// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed errors for primal registration with Songbird.

use thiserror::Error;

/// Errors from primal registration and heartbeat operations.
#[derive(Debug, Error)]
pub enum PrimalRegistrationError {
    /// I/O error (connect, write, read)
    #[error("Songbird I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Failed to serialize or parse JSON
    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Songbird returned a JSON-RPC error
    #[error("Songbird returned error: {0}")]
    SongbirdError(serde_json::Value),
}
