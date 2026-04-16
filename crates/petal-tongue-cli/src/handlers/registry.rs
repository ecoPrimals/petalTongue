// SPDX-License-Identifier: AGPL-3.0-or-later
//! Handlers that use the on-disk [`InstanceRegistry`] (list, gc, status summary).

use colored::Colorize;

use crate::error::CliError;
use petal_tongue_core::InstanceRegistry;
use petal_tongue_ipc::{IpcClient, IpcCommand, IpcResponse};

#[expect(
    clippy::unused_async,
    reason = "CLI entry point; async for future IPC integration"
)]
pub(super) async fn list_instances() -> Result<(), CliError> {
    let registry = InstanceRegistry::load()?;

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

#[expect(
    clippy::unused_async,
    reason = "CLI entry point; async for future IPC integration"
)]
pub(super) async fn gc_instances(force: bool) -> Result<(), CliError> {
    let mut registry = InstanceRegistry::load()?;

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
        registry.save()?;

        println!("{} Removed {} dead instances", "✓".green(), removed);
    } else {
        println!("{} Run with --force to actually remove them", "!".yellow());
    }

    Ok(())
}

pub(super) async fn status_instances() -> Result<(), CliError> {
    let registry = InstanceRegistry::load()?;

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
