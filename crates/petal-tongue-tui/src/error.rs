// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed errors for TUI operations.

use thiserror::Error;

/// Errors from TUI operations.
#[derive(Debug, Error)]
pub enum TuiError {
    /// I/O or terminal error (raw mode, alternate screen, cursor, etc.)
    #[error("{context}: {source}")]
    Terminal {
        /// Human-readable context
        context: &'static str,
        /// Underlying error
        #[source]
        source: std::io::Error,
    },

    /// I/O error (event reading, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Event channel closed
    #[error("Event channel closed")]
    ChannelClosed(#[from] tokio::sync::mpsc::error::SendError<crate::events::TUIEvent>),
}

impl TuiError {
    /// Create a terminal error with context
    pub(crate) const fn terminal(context: &'static str, source: std::io::Error) -> Self {
        Self::Terminal { context, source }
    }
}
