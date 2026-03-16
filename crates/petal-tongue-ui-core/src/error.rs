// SPDX-License-Identifier: AGPL-3.0-or-later
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_core_error_display_variants() {
        assert!(
            UiCoreError::LockPoisoned("test".into())
                .to_string()
                .contains("poisoned")
        );
        assert!(
            UiCoreError::InteractiveNotSupported("Canvas".into())
                .to_string()
                .contains("Interactive")
        );
        assert!(
            UiCoreError::PixmapCreationFailed(0, 0)
                .to_string()
                .contains("pixmap")
        );
        assert!(
            UiCoreError::PngEncodeFailed("err".into())
                .to_string()
                .contains("PNG")
        );
        assert!(
            UiCoreError::CanvasExportOnlyPng
                .to_string()
                .contains("binary")
        );
        assert!(UiCoreError::Json("err".into()).to_string().contains("JSON"));
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        assert!(UiCoreError::Io(io_err).to_string().contains("IO"));
    }

    #[test]
    fn from_poison_error() {
        let mutex = std::sync::Mutex::new(0);
        let _ = std::panic::catch_unwind(|| {
            let _g = mutex.lock().unwrap();
            panic!("poison");
        });
        let result = mutex.lock();
        assert!(result.is_err());
        let err: UiCoreError = result.unwrap_err().into();
        assert!(err.to_string().contains("poisoned"));
    }
}
