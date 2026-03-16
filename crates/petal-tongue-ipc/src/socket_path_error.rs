// SPDX-License-Identifier: AGPL-3.0-only
//! Typed errors for socket path resolution.

use std::path::PathBuf;
use thiserror::Error;

/// Errors from socket path resolution.
#[derive(Debug, Error)]
pub enum SocketPathError {
    /// Failed to create directory
    #[error("Failed to create directory: {0}")]
    CreateDir(#[from] std::io::Error),

    /// Runtime directory does not exist
    #[error("Runtime directory does not exist: {path}. Will fall back to /tmp/")]
    RuntimeDirNotFound {
        /// Path that was checked
        path: PathBuf,
    },

    /// Failed to run or parse 'id -u' command
    #[error("Failed to get current UID: {0}")]
    GetUid(String),
}
