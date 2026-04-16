// SPDX-License-Identifier: AGPL-3.0-or-later
//! Handlers that talk to a running instance over IPC.

use colored::Colorize;

use crate::error::CliError;
use crate::resolve::resolve_instance_id;
use petal_tongue_ipc::{IpcClient, IpcCommand, IpcResponse};

pub(super) async fn show_instance(instance_id_str: &str) -> Result<(), CliError> {
    let instance_id = resolve_instance_id(instance_id_str)?;
    let client = IpcClient::new(&instance_id)?;

    let response = client
        .send(IpcCommand::GetStatus)
        .await
        .map_err(CliError::IpcStatus)?;

    match response {
        IpcResponse::Status(status) => {
            println!("{}", "Instance Status:".bold());
            println!();
            println!("  ID:       {}", status.instance_id.as_str().bright_blue());
            println!("  PID:      {}", status.pid);
            println!("  Uptime:   {}s", status.uptime_seconds);
            println!("  Nodes:    {}", status.node_count);
            println!("  Edges:    {}", status.edge_count);
            println!(
                "  Window:   {}",
                if status.window_visible {
                    "visible".green()
                } else {
                    "hidden".yellow()
                }
            );

            if let Some(name) = status.name {
                println!("  Name:     {name}");
            }

            if let Some(wid) = status.window_id {
                println!("  Window ID: 0x{wid:x}");
            }

            if !status.metadata.is_empty() {
                println!();
                println!("  Metadata:");
                for (key, value) in &status.metadata {
                    println!("    {}: {}", key.dimmed(), value);
                }
            }
        }
        IpcResponse::Error { message } => {
            println!("{} {}", "Error:".red().bold(), message);
        }
        _ => {
            println!("{}", "Unexpected response from instance".red());
        }
    }

    Ok(())
}

pub(super) async fn raise_instance(instance_id_str: &str) -> Result<(), CliError> {
    let instance_id = resolve_instance_id(instance_id_str)?;
    let client = IpcClient::new(&instance_id)?;

    let response = client
        .send(IpcCommand::Show)
        .await
        .map_err(CliError::IpcRaise)?;

    match response {
        IpcResponse::Success => {
            println!(
                "{} Instance {} raised",
                "✓".green(),
                instance_id.as_str().bright_blue()
            );
        }
        IpcResponse::Error { message } => {
            println!("{} {}", "Error:".red().bold(), message);
        }
        _ => {
            println!("{}", "Unexpected response from instance".red());
        }
    }

    Ok(())
}

pub(super) async fn ping_instance(instance_id_str: &str) -> Result<(), CliError> {
    let instance_id = resolve_instance_id(instance_id_str)?;
    let client = IpcClient::new(&instance_id)?;

    match client.ping().await {
        Ok(()) => {
            println!(
                "{} Instance {} is {}",
                "●".green(),
                instance_id.as_str().bright_blue(),
                "responsive".green()
            );
        }
        Err(e) => {
            println!(
                "{} Instance {} is {}",
                "●".red(),
                instance_id.as_str().bright_blue(),
                "unresponsive".red()
            );
            println!("   Error: {e}");
        }
    }

    Ok(())
}
