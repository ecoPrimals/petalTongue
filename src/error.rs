// SPDX-License-Identifier: AGPL-3.0-only
//! Application-level error types.

use thiserror::Error;

/// Errors from the petaltongue application.
#[derive(Debug, Error)]
pub enum AppError {
    /// Graph lock poisoned.
    #[error("Graph lock poisoned: {0}")]
    GraphLockPoisoned(String),

    /// Refresh time lock poisoned.
    #[error("Refresh time lock poisoned: {0}")]
    RefreshLockPoisoned(String),

    /// Neural API error.
    #[error("Neural API error: {0}")]
    NeuralApi(String),

    /// eframe/GUI error.
    #[error("eframe error: {0}")]
    Eframe(String),

    /// UI mode not available (when built without `ui` feature).
    #[allow(dead_code)]
    #[error("UI mode not available in this build")]
    UiNotAvailable,

    /// TUI launch error.
    #[error("TUI error: {0}")]
    Tui(String),

    /// Task panic (e.g. spawn_blocking).
    #[error("Task panicked: {0}")]
    TaskPanic(String),

    /// Wrapped error from dependencies.
    #[error("{0}")]
    Other(String),
}

impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self::GraphLockPoisoned(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::Other(err.to_string())
    }
}
