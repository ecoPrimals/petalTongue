// SPDX-License-Identifier: AGPL-3.0-or-later
//! CLI error types.

use thiserror::Error;

/// Errors from CLI operations.
#[derive(Debug, Error)]
pub enum CliError {
    /// Instance registry error (load, save, gc).
    #[error("Registry error: {0}")]
    Registry(#[from] petal_tongue_core::InstanceError),

    /// Failed to connect to instance via IPC.
    #[error("Failed to connect to instance (is it running?): {0}")]
    IpcConnect(#[from] petal_tongue_ipc::IpcClientError),

    /// Failed to get instance status.
    #[error("Failed to get instance status: {0}")]
    IpcStatus(petal_tongue_ipc::IpcClientError),

    /// Failed to raise instance.
    #[error("Failed to raise instance: {0}")]
    IpcRaise(petal_tongue_ipc::IpcClientError),

    /// Invalid instance ID.
    #[error("Invalid instance ID: {0}")]
    InvalidInstanceId(String),

    /// No instance found matching the given ID.
    #[error("No instance found matching '{0}'")]
    NoInstanceFound(String),

    /// Ambiguous instance ID matches multiple instances.
    #[error("Ambiguous instance ID '{0}' matches {1} instances")]
    AmbiguousInstanceId(String, usize),
}
