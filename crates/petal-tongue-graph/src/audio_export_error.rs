// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed errors for audio export operations.

use std::path::PathBuf;
use thiserror::Error;

/// Errors from audio file export operations.
#[derive(Debug, Error)]
pub enum AudioExportError {
    /// Failed to create WAV file
    #[error("Failed to create WAV file: {path} - {source}")]
    CreateFile {
        /// Path that failed
        path: PathBuf,
        /// Underlying hound error
        #[source]
        source: hound::Error,
    },

    /// Failed to write or finalize WAV file
    #[error("WAV file I/O error: {0}")]
    WavIo(#[from] hound::Error),
}
