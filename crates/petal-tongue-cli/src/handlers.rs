// SPDX-License-Identifier: AGPL-3.0-or-later
//! CLI command handlers and executors.

use colored::Colorize;

use crate::error::CliError;
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
pub async fn run(command: Commands) -> Result<(), CliError> {
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
async fn list_instances() -> Result<(), CliError> {
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

async fn show_instance(instance_id_str: &str) -> Result<(), CliError> {
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

async fn raise_instance(instance_id_str: &str) -> Result<(), CliError> {
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

async fn ping_instance(instance_id_str: &str) -> Result<(), CliError> {
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

#[expect(
    clippy::unused_async,
    reason = "CLI entry point; async for future IPC integration"
)]
async fn gc_instances(force: bool) -> Result<(), CliError> {
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

async fn status_instances() -> Result<(), CliError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::Commands;
    use petal_tongue_core::{Instance, InstanceId, InstanceRegistry};
    use std::fs;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_list_empty_registry() {
        let temp = tempfile::tempdir().unwrap();
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
            "XDG_DATA_HOME",
            temp.path().to_str().unwrap(),
            || {
                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(run(Commands::List))
                });
                assert!(result.is_ok());
            },
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_status_empty_registry() {
        let temp = tempfile::tempdir().unwrap();
        let data_home = temp.path().join("data");
        fs::create_dir_all(&data_home).unwrap();
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
            "XDG_DATA_HOME",
            data_home.to_str().unwrap(),
            || {
                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(run(Commands::Status))
                });
                assert!(result.is_ok());
            },
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_gc_empty_registry() {
        let temp = tempfile::tempdir().unwrap();
        let data_home = temp.path().join("data");
        fs::create_dir_all(&data_home).unwrap();
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
            "XDG_DATA_HOME",
            data_home.to_str().unwrap(),
            || {
                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(run(Commands::Gc { force: false }))
                });
                assert!(result.is_ok());
            },
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_gc_force_with_dead_instances() {
        let temp = tempfile::tempdir().unwrap();
        let app_dir = temp.path().join("petaltongue");
        fs::create_dir_all(&app_dir).unwrap();
        let reg_path = app_dir.join("instances.ron");
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
            &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
            || {
                let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440000").unwrap();
                let mut inst = Instance::new(id, Some("dead-test".to_string())).unwrap();
                inst.pid = 99_999_999;
                let mut registry = InstanceRegistry::new();
                registry.register(inst).unwrap();

                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(run(Commands::Gc { force: true }))
                });
                assert!(result.is_ok());

                let loaded = InstanceRegistry::load_from(&reg_path).unwrap();
                assert_eq!(loaded.count(), 0);
            },
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_gc_dry_run_with_dead_instances() {
        let temp = tempfile::tempdir().unwrap();
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
            &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
            || {
                let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440001").unwrap();
                let mut inst = Instance::new(id, Some("dead".to_string())).unwrap();
                inst.pid = 99_999_999;
                let mut registry = InstanceRegistry::new();
                registry.register(inst).unwrap();

                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(run(Commands::Gc { force: false }))
                });
                assert!(result.is_ok());

                let loaded = InstanceRegistry::load().unwrap();
                assert_eq!(loaded.count(), 1);
            },
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_list_with_instances() {
        let temp = tempfile::tempdir().unwrap();
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
            &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
            || {
                let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440002").unwrap();
                let inst = Instance::new(id, Some("alive".to_string())).unwrap();
                let mut registry = InstanceRegistry::new();
                registry.register(inst).unwrap();

                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(run(Commands::List))
                });
                assert!(result.is_ok());
            },
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_list_with_instance_having_window_id() {
        let temp = tempfile::tempdir().unwrap();
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
            &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
            || {
                let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440004").unwrap();
                let mut inst = Instance::new(id, Some("windowed".to_string())).unwrap();
                inst.set_window_id(0x0012_3456);
                let mut registry = InstanceRegistry::new();
                registry.register(inst).unwrap();

                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(run(Commands::List))
                });
                assert!(result.is_ok());
            },
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_list_with_instance_no_name() {
        let temp = tempfile::tempdir().unwrap();
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
            &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
            || {
                let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440008").unwrap();
                let inst = Instance::new(id, None).unwrap();
                let mut registry = InstanceRegistry::new();
                registry.register(inst).unwrap();

                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(run(Commands::List))
                });
                assert!(result.is_ok());
            },
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_list_with_mixed_alive_and_dead_instances() {
        let temp = tempfile::tempdir().unwrap();
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
            &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
            || {
                let id1 = InstanceId::parse("550e8400-e29b-41d4-a716-446655440006").unwrap();
                let inst1 = Instance::new(id1, Some("alive".to_string())).unwrap();
                let id2 = InstanceId::parse("550e8400-e29b-41d4-a716-446655440007").unwrap();
                let mut inst2 = Instance::new(id2, Some("dead".to_string())).unwrap();
                inst2.pid = 99_999_999;
                let mut registry = InstanceRegistry::new();
                registry.register(inst1).unwrap();
                registry.register(inst2).unwrap();

                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(run(Commands::List))
                });
                assert!(result.is_ok());
            },
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_status_with_alive_instance_unreachable() {
        let temp = tempfile::tempdir().unwrap();
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
            &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
            || {
                let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440005").unwrap();
                let inst = Instance::new(id, Some("alive-unreachable".to_string())).unwrap();
                let mut registry = InstanceRegistry::new();
                registry.register(inst).unwrap();

                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(run(Commands::Status))
                });
                assert!(result.is_ok());
            },
        );
    }

    #[tokio::test]
    async fn test_run_show_invalid_instance_id() {
        let result = run(Commands::Show {
            instance_id: "not-a-valid-uuid".to_string(),
        })
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_raise_invalid_instance_id() {
        let result = run(Commands::Raise {
            instance_id: "bad-id".to_string(),
        })
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_ping_invalid_instance_id() {
        let result = run(Commands::Ping {
            instance_id: "invalid".to_string(),
        })
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_show_valid_uuid_no_socket() {
        let result = run(Commands::Show {
            instance_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        })
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_raise_valid_uuid_no_socket() {
        let result = run(Commands::Raise {
            instance_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        })
        .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_ping_valid_uuid_no_socket() {
        let result = run(Commands::Ping {
            instance_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        })
        .await;
        assert!(result.is_err());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_status_with_dead_instance() {
        let temp = tempfile::tempdir().unwrap();
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_vars(
            &[("XDG_DATA_HOME", Some(temp.path().to_str().unwrap()))],
            || {
                let id = InstanceId::parse("550e8400-e29b-41d4-a716-446655440003").unwrap();
                let mut inst = Instance::new(id, None).unwrap();
                inst.pid = 99_999_999;
                let mut registry = InstanceRegistry::new();
                registry.register(inst).unwrap();

                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(run(Commands::Status))
                });
                assert!(result.is_ok());
            },
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_run_show_empty_instance_id() {
        let temp = tempfile::tempdir().unwrap();
        petal_tongue_core::test_fixtures::env_test_helpers::with_env_var(
            "XDG_DATA_HOME",
            temp.path().to_str().unwrap(),
            || {
                let result = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(run(Commands::Show {
                        instance_id: String::new(),
                    }))
                });
                assert!(result.is_err());
            },
        );
    }

    #[test]
    fn test_commands_match_variants() {
        drop(Commands::List);
        drop(Commands::Show {
            instance_id: "test".to_string(),
        });
        drop(Commands::Raise {
            instance_id: "test".to_string(),
        });
        drop(Commands::Ping {
            instance_id: "test".to_string(),
        });
        drop(Commands::Gc { force: false });
        drop(Commands::Status);
    }
}
