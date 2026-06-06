// SPDX-License-Identifier: AGPL-3.0-or-later
//! Error types for headless petalTongue binary.

use petal_tongue_ui_core::UiCoreError;
use thiserror::Error;

/// Errors from the headless binary.
#[derive(Debug, Error)]
pub enum HeadlessError {
    /// Graph lock poisoned.
    #[error("Graph lock poisoned: {0}")]
    LockPoisoned(String),

    /// UI core error.
    #[error(transparent)]
    UiCore(#[from] UiCoreError),

    /// IO error (file write, directory creation).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Scenario file load/parse error.
    #[error("Scenario load error: {0}")]
    ScenarioLoad(String),
}

impl<T> From<std::sync::PoisonError<T>> for HeadlessError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self::LockPoisoned(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headless_error_display_variants() {
        let err = HeadlessError::LockPoisoned("lock fail".into());
        assert!(err.to_string().contains("lock fail"));

        let err = HeadlessError::ScenarioLoad("bad scenario".into());
        assert!(err.to_string().contains("bad scenario"));

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err = HeadlessError::Io(io_err);
        assert!(err.to_string().contains("missing"));
    }

    #[test]
    fn poison_error_converts() {
        use std::sync::{Arc, RwLock};
        let lock = Arc::new(RwLock::new(42));
        let l2 = Arc::clone(&lock);
        let h = std::thread::spawn(move || {
            let _g = l2.write().unwrap();
            panic!("intentional poison");
        });
        let _ = h.join();
        let poison_err = lock.write().unwrap_err();
        let err: HeadlessError = poison_err.into();
        assert!(matches!(err, HeadlessError::LockPoisoned(_)));
    }
}
