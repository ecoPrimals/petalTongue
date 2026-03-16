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
}

impl<T> From<std::sync::PoisonError<T>> for HeadlessError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        Self::LockPoisoned(err.to_string())
    }
}
