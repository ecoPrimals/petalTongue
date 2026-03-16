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

    /// Event bus broadcast failed.
    #[error("event bus error: {0}")]
    EventBus(String),

    /// No modalities available.
    #[error("no modalities available")]
    NoModalities,

    /// Modality not found.
    #[error("modality not found: {0}")]
    ModalityNotFound(String),

    /// No GPU compute provider available.
    #[error("no GPU compute provider available")]
    NoGpuCompute,

    /// No migration found for schema version.
    #[error("no migration found for {0} → {1}")]
    NoMigration(String, String),

    /// Invalid version format.
    #[error("invalid version format: {0}")]
    InvalidVersionFormat(String),

    /// JSON parse/serialize error.
    #[error("JSON error: {0}")]
    Json(String),

    /// Config directory error.
    #[error("Could not determine config directory: {0}")]
    ConfigDir(String),
}

impl From<tokio::task::JoinError> for PetalTongueError {
    fn from(err: tokio::task::JoinError) -> Self {
        Self::Internal(err.to_string())
    }
}

impl<T> From<std::sync::PoisonError<T>> for PetalTongueError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self::LockPoisoned(err.to_string())
    }
}

/// Result type alias for petalTongue operations
pub type Result<T> = std::result::Result<T, PetalTongueError>;
