// SPDX-License-Identifier: AGPL-3.0-only
//! petalTongue error types.

use thiserror::Error;

/// Errors specific to petalTongue.
#[derive(Debug, Error)]
pub enum PetalTongueError {
    /// Configuration error.
    #[error("configuration error: {0}")]
    Config(String),

    /// Graph engine error.
    #[error("graph engine error: {0}")]
    GraphEngine(String),

    /// Renderer error.
    #[error("renderer error: {0}")]
    Renderer(String),

    /// `BiomeOS` API client error.
    #[error("BiomeOS API error: {0}")]
    BiomeOSApi(#[from] reqwest::Error),

    /// Discovery error.
    #[error("primal discovery failed: {0}")]
    Discovery(String),

    /// Lock poisoned error.
    #[error("lock poisoned: {0}")]
    LockPoisoned(String),

    /// Audio system error.
    #[error("audio system error: {0}")]
    Audio(String),

    /// Telemetry error.
    #[error("telemetry error: {0}")]
    Telemetry(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl<T> From<std::sync::PoisonError<T>> for PetalTongueError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self::LockPoisoned(err.to_string())
    }
}

/// Result type alias for petalTongue operations
pub type Result<T> = std::result::Result<T, PetalTongueError>;
