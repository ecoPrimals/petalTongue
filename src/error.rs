// SPDX-License-Identifier: AGPL-3.0-or-later
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

    /// eframe/display error.
    #[error("eframe error: {0}")]
    Eframe(String),

    /// UI mode not available (when built without `ui` feature).
    #[cfg_attr(
        feature = "ui",
        expect(dead_code, reason = "variant unreachable when ui feature enabled")
    )]
    #[error(
        "UI mode not available in this build. Try tui or web mode, or rebuild with --features ui"
    )]
    UiNotAvailable,

    /// TUI launch error.
    #[error("TUI error: {0}")]
    Tui(String),

    /// Task panic (e.g. `spawn_blocking`).
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_graph_lock_poisoned_constructor() {
        let err = AppError::GraphLockPoisoned("lock poisoned".to_string());
        assert!(matches!(err, AppError::GraphLockPoisoned(_)));
        assert!(err.to_string().contains("Graph lock poisoned"));
        assert!(err.to_string().contains("lock poisoned"));
    }

    #[test]
    fn test_refresh_lock_poisoned_constructor() {
        let err = AppError::RefreshLockPoisoned("refresh failed".to_string());
        assert!(matches!(err, AppError::RefreshLockPoisoned(_)));
        assert!(err.to_string().contains("Refresh time lock poisoned"));
        assert!(err.to_string().contains("refresh failed"));
    }

    #[test]
    fn test_neural_api_constructor() {
        let err = AppError::NeuralApi("connection refused".to_string());
        assert!(matches!(err, AppError::NeuralApi(_)));
        assert!(err.to_string().contains("Neural API error"));
        assert!(err.to_string().contains("connection refused"));
    }

    #[test]
    fn test_eframe_constructor() {
        let err = AppError::Eframe("window creation failed".to_string());
        assert!(matches!(err, AppError::Eframe(_)));
        assert!(err.to_string().contains("eframe error"));
        assert!(err.to_string().contains("window creation failed"));
    }

    #[test]
    fn test_ui_not_available_constructor() {
        let err = AppError::UiNotAvailable;
        assert!(matches!(err, AppError::UiNotAvailable));
        let msg = err.to_string();
        assert!(msg.contains("UI mode not available"));
        assert!(msg.contains("tui"));
        assert!(msg.contains("web"));
        assert!(msg.contains("--features ui"));
    }

    #[test]
    fn test_tui_constructor() {
        let err = AppError::Tui("terminal init failed".to_string());
        assert!(matches!(err, AppError::Tui(_)));
        assert!(err.to_string().contains("TUI error"));
        assert!(err.to_string().contains("terminal init failed"));
    }

    #[test]
    fn test_task_panic_constructor() {
        let err = AppError::TaskPanic("worker panicked".to_string());
        assert!(matches!(err, AppError::TaskPanic(_)));
        assert!(err.to_string().contains("Task panicked"));
        assert!(err.to_string().contains("worker panicked"));
    }

    #[test]
    fn test_other_constructor() {
        let err = AppError::Other("generic error".to_string());
        assert!(matches!(err, AppError::Other(_)));
        assert_eq!(err.to_string(), "generic error");
    }

    #[test]
    fn test_from_poison_error() {
        let mutex = Arc::new(Mutex::new(0));
        let mutex_clone = Arc::clone(&mutex);
        let handle = std::thread::spawn(move || {
            let _guard = mutex_clone.lock().unwrap();
            panic!("poison for test");
        });
        let _ = handle.join();
        let result = mutex.lock();
        let poison_err = result.unwrap_err();
        let app_err: AppError = poison_err.into();
        assert!(matches!(app_err, AppError::GraphLockPoisoned(_)));
        assert!(app_err.to_string().contains("Graph lock poisoned"));
        assert!(app_err.to_string().contains("poison"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Other(_)));
        assert!(app_err.to_string().contains("file not found"));
    }

    #[test]
    fn test_from_io_error_permission_denied() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let app_err: AppError = io_err.into();
        assert!(matches!(app_err, AppError::Other(_)));
        assert!(app_err.to_string().contains("access denied"));
    }

    #[test]
    fn test_app_error_debug() {
        let err = AppError::UiNotAvailable;
        let debug_str = format!("{err:?}");
        assert!(debug_str.contains("UiNotAvailable"));
    }
}
