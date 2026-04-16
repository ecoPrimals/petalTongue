// SPDX-License-Identifier: AGPL-3.0-or-later
//! CLI command handlers and executors.

mod ipc;
mod registry;

#[cfg(test)]
mod tests;

use crate::commands::Commands;
use crate::error::CliError;

/// Execute a CLI command.
///
/// # Errors
///
/// Returns an error if the command fails (registry unavailable,
/// instance not found, IPC failure, etc.).
pub async fn run(command: Commands) -> Result<(), CliError> {
    match command {
        Commands::List => registry::list_instances().await,
        Commands::Show { instance_id } => ipc::show_instance(&instance_id).await,
        Commands::Raise { instance_id } => ipc::raise_instance(&instance_id).await,
        Commands::Ping { instance_id } => ipc::ping_instance(&instance_id).await,
        Commands::Gc { force } => registry::gc_instances(force).await,
        Commands::Status => registry::status_instances().await,
    }
}
