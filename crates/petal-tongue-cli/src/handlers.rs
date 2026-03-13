// SPDX-License-Identifier: AGPL-3.0-only
//! CLI command handlers and executors.

use anyhow::{Context, Result};
use colored::Colorize;
use petal_tongue_core::InstanceRegistry;
use petal_tongue_ipc::{IpcClient, IpcCommand, IpcResponse};

use crate::commands::Commands;
use crate::resolve::resolve_instance_id;

/// Execute a CLI command.
///
/// # Errors
///
/// Returns an error if the command fails (registry unavailable,
/// instance not found, IPC failure, etc.).
pub async fn run(command: Commands) -> Result<()> {
    match command {
        Commands::List => list_instances().await,
        Commands::Show { instance_id } => show_instance(&instance_id).await,
        Commands::Raise { instance_id } => raise_instance(&instance_id).await,
        Commands::Ping { instance_id } => ping_instance(&instance_id).await,
        Commands::Gc { force } => gc_instances(force).await,
        Commands::Status => status_instances().await,
    }
}

#[expect(
    clippy::unused_async,
    reason = "CLI entry point; async for future IPC integration"
)]
async fn list_instances() -> Result<()> {
    let registry = InstanceRegistry::load().context("Failed to load instance registry")?;

    println!("📋 Active petalTongue instances:\n");
    let instances = registry.list();

    if instances.is_empty() {
        println!("{}", "No instances running".yellow());
        return Ok(());
    }

    println!("{}", "petalTongue Instances:".bold());
    println!();

    for instance in &instances {
        let alive = instance.is_alive();
        let status_icon = if alive { "●".green() } else { "●".red() };
        let status_text = if alive { "alive" } else { "dead" };

        println!(
            "  {} {} {}",
            status_icon,
            instance.id.as_str().bright_blue(),
            format!("({status_text})").dimmed()
        );

        if let Some(name) = &instance.name {
            println!("     Name: {name}");
        }

        println!("     PID:  {}", instance.pid);

        if let Some(wid) = instance.window_id {
            println!("     Window: 0x{wid:x}");
        }

        println!(
            "     Socket: {}",
            instance.socket_path.display().to_string().dimmed()
        );

        println!();
    }

    let alive_count = instances.iter().filter(|i| i.is_alive()).count();
    println!(
        "Total: {} instances ({} alive, {} dead)",
        instances.len(),
        alive_count.to_string().green(),
        (instances.len() - alive_count).to_string().red()
    );

    Ok(())
}

async fn show_instance(instance_id_str: &str) -> Result<()> {
    let instance_id = resolve_instance_id(instance_id_str)?;
    let client =
        IpcClient::new(&instance_id).context("Failed to connect to instance (is it running?)")?;

    let response = client
        .send(IpcCommand::GetStatus)
        .await
        .context("Failed to get instance status")?;

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

async fn raise_instance(instance_id_str: &str) -> Result<()> {
    let instance_id = resolve_instance_id(instance_id_str)?;
    let client =
        IpcClient::new(&instance_id).context("Failed to connect to instance (is it running?)")?;

    let response = client
        .send(IpcCommand::Show)
        .await
        .context("Failed to raise instance")?;

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

async fn ping_instance(instance_id_str: &str) -> Result<()> {
    let instance_id = resolve_instance_id(instance_id_str)?;
    let client =
        IpcClient::new(&instance_id).context("Failed to connect to instance (is it running?)")?;

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

#[expect(
    clippy::unused_async,
    reason = "CLI entry point; async for future IPC integration"
)]
async fn gc_instances(force: bool) -> Result<()> {
    let mut registry = InstanceRegistry::load().context("Failed to load instance registry")?;

    let all_instances = registry.list();
    let dead_instances: Vec<_> = all_instances
        .iter()
        .filter(|i| !i.is_alive())
        .map(|i| i.id.clone())
        .collect();

    if dead_instances.is_empty() {
        println!("{}", "No dead instances to clean up".green());
        return Ok(());
    }

    println!("Found {} dead instances:", dead_instances.len());
    for id in &dead_instances {
        println!("  {} {}", "●".red(), id.as_str().dimmed());
    }
    println!();

    if force {
        let removed = registry.gc()?;
        registry
            .save()
            .context("Failed to save registry after cleanup")?;

        println!("{} Removed {} dead instances", "✓".green(), removed);
    } else {
        println!("{} Run with --force to actually remove them", "!".yellow());
    }

    Ok(())
}

async fn status_instances() -> Result<()> {
    let registry = InstanceRegistry::load().context("Failed to load instance registry")?;

    let instances = registry.list();

    if instances.is_empty() {
        println!("{}", "No instances running".yellow());
        return Ok(());
    }

    println!("{}", "Instance Status Summary:".bold());
    println!();

    for instance in instances {
        let alive = instance.is_alive();

        if !alive {
            println!(
                "  {} {} {}",
                "●".red(),
                instance.id.as_str().dimmed(),
                "(dead)".red()
            );
            continue;
        }

        // Try to get detailed status via IPC
        match IpcClient::new(&instance.id) {
            Ok(client) => match client.send(IpcCommand::GetStatus).await {
                Ok(IpcResponse::Status(status)) => {
                    println!(
                        "  {} {} {} | {} nodes, {} edges{}",
                        "●".green(),
                        instance.id.as_str().bright_blue(),
                        format!("({}s)", status.uptime_seconds).dimmed(),
                        status.node_count,
                        status.edge_count,
                        status
                            .name
                            .as_ref()
                            .map_or_else(String::new, |name| format!(" | {name}"))
                    );
                }
                _ => {
                    println!(
                        "  {} {} {}",
                        "●".yellow(),
                        instance.id.as_str().bright_blue(),
                        "(no status)".yellow()
                    );
                }
            },
            Err(_) => {
                println!(
                    "  {} {} {}",
                    "●".yellow(),
                    instance.id.as_str().bright_blue(),
                    "(unreachable)".yellow()
                );
            }
        }
    }

    Ok(())
}
