// SPDX-License-Identifier: AGPL-3.0-or-later
//! Doom-specific error types.

use thiserror::Error;

/// Doom-specific errors.
#[derive(Debug, Error)]
pub enum DoomError {
    #[error("Doom engine initialization failed: {0}")]
    InitializationFailed(String),

    #[error("WAD file not found: {0}")]
    WadNotFound(String),

    #[error("Invalid WAD file: {0}")]
    InvalidWad(String),

    #[error("Doom engine error: {0}")]
    EngineError(String),
}

/// Convenience alias for Doom operations.
pub type Result<T> = std::result::Result<T, DoomError>;
