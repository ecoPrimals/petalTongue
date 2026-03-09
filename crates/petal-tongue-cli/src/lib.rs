// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
//! petalTongue CLI - Manage petalTongue instances from the command line
//!
//! Library crate providing CLI instance management. Consumed by the
//! petalTongue `UniBin` via the `status` / `cli` mode.
//!
//! # Commands
//!
//! - `list` - List all running instances
//! - `show <id>` - Show details of an instance
//! - `raise <id>` - Bring an instance window to front
//! - `ping <id>` - Check if instance is responsive
//! - `gc` - Clean up dead instances from registry

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use petal_tongue_core::{InstanceId, InstanceRegistry};
use petal_tongue_ipc::{IpcClient, IpcCommand, IpcResponse};

/// CLI argument parser for petalTongue instance management.
#[derive(Debug, Parser)]
#[command(name = "petaltongue")]
#[command(about = "petalTongue instance manager", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
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

#[allow(clippy::unused_async)]
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

#[allow(clippy::unused_async)]
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
                            format!(" | {name}")
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

/// Parse CLI arguments (for testing)
#[cfg(test)]
pub fn parse_args(args: &[&str]) -> std::result::Result<Commands, clap::Error> {
    let cli = Cli::try_parse_from(args)?;
    Ok(cli.command)
}

/// Resolve instance ID from string (supports prefixes)
fn resolve_instance_id(id_str: &str) -> Result<InstanceId> {
    // Try to parse as UUID string and create InstanceId
    if let Ok(uuid) = uuid::Uuid::parse_str(id_str) {
        let id_string = uuid.to_string();
        return InstanceId::parse(&id_string)
            .map_err(|e| anyhow::anyhow!("Invalid instance ID: {e}"));
    }

    // Try prefix match
    let registry = InstanceRegistry::load().context("Failed to load registry")?;
    let instances = registry.list();

    let matches: Vec<_> = instances
        .iter()
        .filter(|i| i.id.as_str().starts_with(id_str))
        .collect();

    match matches.len() {
        0 => anyhow::bail!("No instance found matching '{id_str}'"),
        1 => Ok(matches[0].id.clone()),
        _ => anyhow::bail!(
            "Ambiguous instance ID '{}' matches {} instances",
            id_str,
            matches.len()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_list() {
        let cmd = parse_args(&["petaltongue", "list"]).unwrap();
        assert!(matches!(cmd, Commands::List));
    }

    #[test]
    fn test_parse_status() {
        let cmd = parse_args(&["petaltongue", "status"]).unwrap();
        assert!(matches!(cmd, Commands::Status));
    }

    #[test]
    fn test_parse_show() {
        let cmd = parse_args(&["petaltongue", "show", "abc-123"]).unwrap();
        match &cmd {
            Commands::Show { instance_id } => assert_eq!(instance_id, "abc-123"),
            _ => panic!("Expected Show command"),
        }
    }

    #[test]
    fn test_parse_gc() {
        let cmd = parse_args(&["petaltongue", "gc"]).unwrap();
        match &cmd {
            Commands::Gc { force } => assert!(!*force),
            _ => panic!("Expected Gc command"),
        }
    }

    #[test]
    fn test_parse_gc_force() {
        let cmd = parse_args(&["petaltongue", "gc", "--force"]).unwrap();
        match &cmd {
            Commands::Gc { force } => assert!(*force),
            _ => panic!("Expected Gc command"),
        }
    }

    #[test]
    fn test_parse_ping() {
        let cmd = parse_args(&["petaltongue", "ping", "uuid-here"]).unwrap();
        match &cmd {
            Commands::Ping { instance_id } => assert_eq!(instance_id, "uuid-here"),
            _ => panic!("Expected Ping command"),
        }
    }

    #[test]
    fn test_parse_raise() {
        let cmd = parse_args(&["petaltongue", "raise", "inst-id"]).unwrap();
        match &cmd {
            Commands::Raise { instance_id } => assert_eq!(instance_id, "inst-id"),
            _ => panic!("Expected Raise command"),
        }
    }

    #[test]
    fn test_help_text_generation() {
        let result = Cli::try_parse_from(["petaltongue", "--help"]);
        let err = result.expect_err("--help should produce Err with help text");
        let help = err.to_string();
        assert!(help.contains("petaltongue"));
        assert!(help.contains("list") || help.contains("List"));
        assert!(help.contains("show") || help.contains("Show"));
    }

    #[test]
    fn test_version_output() {
        let result = Cli::try_parse_from(["petaltongue", "--version"]);
        let err = result.expect_err("--version should produce Err with version");
        let version = err.to_string();
        assert!(version.contains("petaltongue"));
        assert!(version.contains('.') || version.chars().any(|c| c.is_ascii_digit()));
    }

    #[test]
    fn test_list_subcommand_help() {
        let result = Cli::try_parse_from(["petaltongue", "list", "--help"]);
        let err = result.expect_err("list --help should produce Err with help");
        let help = err.to_string();
        assert!(help.contains("list") || help.contains("List"));
    }

    #[test]
    fn test_gc_subcommand_with_force_flag() {
        let cmd = parse_args(&["petaltongue", "gc", "-f"]).unwrap();
        match &cmd {
            Commands::Gc { force } => assert!(*force),
            _ => panic!("Expected Gc command"),
        }
    }

    #[test]
    fn test_parse_args_missing_subcommand_fails() {
        let result = parse_args(&["petaltongue"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_unknown_subcommand_fails() {
        let result = parse_args(&["petaltongue", "unknown-cmd"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_show_with_empty_id() {
        let cmd = parse_args(&["petaltongue", "show", ""]).unwrap();
        match &cmd {
            Commands::Show { instance_id } => assert_eq!(instance_id, ""),
            _ => panic!("Expected Show command"),
        }
    }

    #[test]
    fn test_parse_args_raise_with_uuid() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let cmd = parse_args(&["petaltongue", "raise", uuid]).unwrap();
        match &cmd {
            Commands::Raise { instance_id } => assert_eq!(instance_id, uuid),
            _ => panic!("Expected Raise command"),
        }
    }

    #[test]
    fn test_parse_args_ping_with_short_prefix() {
        let cmd = parse_args(&["petaltongue", "ping", "550e"]).unwrap();
        match &cmd {
            Commands::Ping { instance_id } => assert_eq!(instance_id, "550e"),
            _ => panic!("Expected Ping command"),
        }
    }

    #[test]
    fn test_run_command_dispatch_list() {
        let cmd = parse_args(&["petaltongue", "list"]).unwrap();
        assert!(matches!(cmd, Commands::List));
    }

    #[test]
    fn test_run_command_dispatch_status() {
        let cmd = parse_args(&["petaltongue", "status"]).unwrap();
        assert!(matches!(cmd, Commands::Status));
    }

    #[test]
    fn test_cli_struct_has_command_field() {
        let cli = Cli::try_parse_from(["petaltongue", "list"]).unwrap();
        assert!(matches!(cli.command, Commands::List));
    }

    #[test]
    fn test_show_subcommand_accepts_instance_id_arg() {
        let cmd = parse_args(&["petaltongue", "show", "my-instance-123"]).unwrap();
        match &cmd {
            Commands::Show { instance_id } => assert_eq!(instance_id, "my-instance-123"),
            _ => panic!("Expected Show"),
        }
    }
}
