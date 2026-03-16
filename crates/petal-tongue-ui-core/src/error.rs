// SPDX-License-Identifier: AGPL-3.0-only
//! UI core error types.

use thiserror::Error;

/// Errors from UI core operations.
#[derive(Debug, Error)]
pub enum UiCoreError {
    /// Graph lock poisoned.
    #[error("graph lock poisoned: {0}")]
    LockPoisoned(String),

    /// Interactive mode not supported.
    #[error("Interactive mode not supported for {0}")]
    InteractiveNotSupported(String),

    /// Failed to create pixmap.
    #[error("Failed to create pixmap {0}x{1}")]
    PixmapCreationFailed(u32, u32),

    /// PNG encode failed.
    #[error("PNG encode failed: {0}")]
    PngEncodeFailed(String),

    /// Canvas UI only supports binary export (PNG).
    #[error("Canvas UI only supports binary export (PNG)")]
    CanvasExportOnlyPng,

    /// JSON serialize error.
    #[error("JSON error: {0}")]
    Json(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl<T> From<std::sync::PoisonError<T>> for UiCoreError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self::LockPoisoned(err.to_string())
    }
}

/// Result type alias for UI core operations.
pub type Result<T> = std::result::Result<T, UiCoreError>;
