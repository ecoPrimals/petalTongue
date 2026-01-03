//! petalTongue CLI - Manage petalTongue instances from the command line
//!
//! # Commands
//!
//! - `petaltongue list` - List all running instances
//! - `petaltongue show <id>` - Show details of an instance
//! - `petaltongue raise <id>` - Bring an instance window to front
//! - `petaltongue ping <id>` - Check if instance is responsive
//! - `petaltongue gc` - Clean up dead instances from registry

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use petal_tongue_core::{InstanceId, InstanceRegistry};
use petal_tongue_ipc::{IpcClient, IpcCommand, IpcResponse};
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "petaltongue")]
#[command(about = "petalTongue instance manager", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all running instances
    List,

    /// Show detailed information about an instance
    Show {
        /// Instance ID (UUID or prefix)
        #[arg(value_name = "INSTANCE_ID")]
        instance_id: String,
    },

    /// Bring instance window to front
    Raise {
        /// Instance ID (UUID or prefix)
        #[arg(value_name = "INSTANCE_ID")]
        instance_id: String,
    },

    /// Ping an instance to check if it's responsive
    Ping {
        /// Instance ID (UUID or prefix)
        #[arg(value_name = "INSTANCE_ID")]
        instance_id: String,
    },

    /// Clean up dead instances from registry
    Gc {
        /// Actually remove dead instances (default: dry-run)
        #[arg(short, long)]
        force: bool,
    },

    /// Show status of all instances
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => list_instances().await,
        Commands::Show { instance_id } => show_instance(&instance_id).await,
        Commands::Raise { instance_id } => raise_instance(&instance_id).await,
        Commands::Ping { instance_id } => ping_instance(&instance_id).await,
        Commands::Gc { force } => gc_instances(force).await,
        Commands::Status => status_instances().await,
    }
}

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
            format!("({})", status_text).dimmed()
        );

        if let Some(name) = &instance.name {
            println!("     Name: {}", name);
        }

        println!("     PID:  {}", instance.pid);

        if let Some(wid) = instance.window_id {
            println!("     Window: 0x{:x}", wid);
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
                println!("  Name:     {}", name);
            }

            if let Some(wid) = status.window_id {
                println!("  Window ID: 0x{:x}", wid);
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
            println!("   Error: {}", e);
        }
    }

    Ok(())
}

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
                        if let Some(name) = &status.name {
                            format!(" | {}", name)
                        } else {
                            String::new()
                        }
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

/// Resolve instance ID from string (supports prefixes)
fn resolve_instance_id(id_str: &str) -> Result<InstanceId> {
    // Try to parse as UUID string and create InstanceId
    if let Ok(uuid) = uuid::Uuid::parse_str(id_str) {
        let id_string = uuid.to_string();
        return InstanceId::from_str(&id_string)
            .map_err(|e| anyhow::anyhow!("Invalid instance ID: {}", e));
    }

    // Try prefix match
    let registry = InstanceRegistry::load().context("Failed to load registry")?;
    let instances = registry.list();

    let matches: Vec<_> = instances
        .iter()
        .filter(|i| i.id.as_str().starts_with(id_str))
        .collect();

    match matches.len() {
        0 => anyhow::bail!("No instance found matching '{}'", id_str),
        1 => Ok(matches[0].id.clone()),
        _ => anyhow::bail!(
            "Ambiguous instance ID '{}' matches {} instances",
            id_str,
            matches.len()
        ),
    }
}
