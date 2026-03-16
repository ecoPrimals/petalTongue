// SPDX-License-Identifier: AGPL-3.0-only
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! petalTongue ecoBud - Production `UniBin`
//!
//! # Architecture
//!
//! `UniBin`: 1 binary, 5 modes
//! ecoBin: 80% (4/5 modes Pure Rust)
//!
//! # Concurrency
//!
//! All modes are fully concurrent:
//! - No blocking operations
//! - Proper async/await patterns
//! - Channel-based communication
//! - Atomic synchronization
//!
//! # Testing
//!
//! - All tests run in parallel
//! - No sleeps (use proper sync primitives)
//! - Test failures = production issues

use clap::{Parser, Subcommand};
use petal_tongue_core::config_system::Config;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli_mode;
mod data_service;
mod error;
mod headless_mode;
mod server_mode;
mod tui_mode;
mod ui_mode;
mod web_mode;

use crate::error::AppError;

#[derive(Parser)]
#[command(name = "petaltongue")]
#[command(
    version,
    about = "🌸 petalTongue - Universal UI & Visualization System"
)]
#[command(
    long_about = "ecoBud v1.0: UniBin + 80% ecoBin\n\nFully concurrent, modern Rust architecture"
)]
struct Cli {
    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Log format (json, pretty, compact)
    #[arg(long, default_value = "pretty")]
    log_format: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Launch native desktop GUI
    #[command(visible_alias = "gui")]
    Ui {
        /// Scenario JSON file to load
        #[arg(long)]
        scenario: Option<String>,

        /// Disable audio sonification
        #[arg(long)]
        no_audio: bool,
    },

    /// Launch terminal user interface (Pure Rust! ✅)
    Tui {
        /// Scenario JSON file to load
        #[arg(long)]
        scenario: Option<String>,

        /// Refresh rate in Hz
        #[arg(long, default_value = "60")]
        refresh_rate: u32,
    },

    /// Launch web UI server (Pure Rust backend! ✅)
    Web {
        /// Bind address (default: from config or 0.0.0.0:<`web_port`>)
        #[arg(long)]
        bind: Option<String>,

        /// Scenario JSON file to load
        #[arg(long)]
        scenario: Option<String>,

        /// Number of worker threads
        #[arg(long, default_value = "4")]
        workers: usize,
    },

    /// Run headless API server (Pure Rust! ✅)
    Headless {
        /// Bind address (default: from config or 0.0.0.0:<`headless_port`>)
        #[arg(long)]
        bind: Option<String>,

        /// Number of worker threads
        #[arg(long, default_value = "4")]
        workers: usize,
    },

    /// Run IPC server (Unix socket JSON-RPC) without GUI
    Server,

    /// Show status and system info (Pure Rust! ✅)
    Status {
        /// Show detailed information
        #[arg(long)]
        verbose: bool,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let cli = Cli::parse();

    // Initialize structured logging (no println!, proper tracing)
    init_tracing(&cli.log_level, &cli.log_format)?;

    // Log startup
    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        command = ?cli.command,
        "🌸 petalTongue starting"
    );

    // Load configuration (environment-driven, XDG-compliant)
    tracing::info!("⚙️ Loading configuration from environment...");
    let config = Config::from_env().map_err(|e| AppError::Other(e.to_string()))?;
    tracing::info!(
        web_port = config.network.web_port,
        headless_port = config.network.headless_port,
        "✅ Configuration loaded"
    );

    // Initialize DataService ONCE (single source of truth for all modes)
    tracing::info!("📊 Initializing unified DataService...");
    let mut data_service = data_service::DataService::new();
    data_service.init().await?;
    let data_service = std::sync::Arc::new(data_service);
    tracing::info!("✅ DataService initialized - all modes will use same data source");

    // Register with ecosystem discovery service (capability-based, no hardcoded primal names)
    tracing::info!("🔍 Registering with ecosystem discovery service...");
    register_with_discovery_service().await;

    // Execute command (all modes are fully async)
    let result = match cli.command {
        Commands::Ui { scenario, no_audio } => {
            tracing::info!(mode = "ui", "Launching desktop GUI mode");
            ui_mode::run(scenario, no_audio, data_service).await
        }
        Commands::Tui {
            scenario,
            refresh_rate,
        } => {
            tracing::info!(
                mode = "tui",
                refresh_rate,
                "Launching terminal UI mode (Pure Rust!)"
            );
            tui_mode::run(scenario, refresh_rate, data_service).await
        }
        Commands::Web {
            bind,
            scenario,
            workers,
        } => {
            // Use explicit bind address or fall back to config (capability-based, no hardcoding)
            let bind_addr = bind.unwrap_or_else(|| format!("0.0.0.0:{}", config.network.web_port));

            tracing::info!(
                mode = "web",
                bind = %bind_addr,
                workers,
                "Launching web UI server (Pure Rust!)"
            );
            web_mode::run(&bind_addr, scenario, workers, data_service).await
        }
        Commands::Headless { bind, workers } => {
            // Use explicit bind address or fall back to config (capability-based, no hardcoding)
            let bind_addr =
                bind.unwrap_or_else(|| format!("0.0.0.0:{}", config.network.headless_port));

            tracing::info!(
                mode = "headless",
                bind = %bind_addr,
                workers,
                "Launching headless API server (Pure Rust!)"
            );
            headless_mode::run(&bind_addr, workers, data_service).await
        }
        Commands::Server => {
            tracing::info!(mode = "server", "Launching IPC server (no GUI)");
            server_mode::run(data_service).await
        }
        Commands::Status { verbose, format } => {
            tracing::info!(
                mode = "status",
                verbose,
                format,
                "Querying system status (Pure Rust!)"
            );
            cli_mode::status(verbose, &format, data_service).await
        }
    };

    // Handle result
    match result {
        Ok(()) => {
            tracing::info!("🌸 petalTongue shutdown gracefully");
            Ok(())
        }
        Err(e) => {
            tracing::error!(error = ?e, "🌸 petalTongue encountered an error");
            Err(e)
        }
    }
}

/// Initialize structured logging with proper filtering
fn init_tracing(level: &str, format: &str) -> Result<(), AppError> {
    // Parse log level
    let env_filter = tracing_subscriber::EnvFilter::try_new(level)
        .map_err(|e| AppError::Other(format!("Failed to parse log level: {e}")))?;

    // Build subscriber based on format
    match format {
        "json" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().with_target(true))
                .try_init()
                .map_err(|e| AppError::Other(format!("Failed to initialize JSON logging: {e}")))?;
        }
        "compact" => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().compact())
                .try_init()
                .map_err(|e| {
                    AppError::Other(format!("Failed to initialize compact logging: {e}"))
                })?;
        }
        _ => {
            tracing_subscriber::registry()
                .with(env_filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_target(true)
                        .with_thread_ids(true)
                        .with_file(true)
                        .with_line_number(true),
                )
                .try_init()
                .map_err(|e| {
                    AppError::Other(format!("Failed to initialize pretty logging: {e}"))
                })?;
        }
    }

    Ok(())
}

/// Register petalTongue with the ecosystem discovery service
///
/// This implements the `ipc.register` standard from `PRIMAL_IPC_PROTOCOL.md`.
/// Uses capability-based discovery to find the registration service (could be Songbird
/// or any other primal providing the "discovery" capability).
///
/// # TRUE PRIMAL: Capability-Based Registration
/// - Discovers the registration service at runtime (no hardcoded primal name)
/// - Gracefully handles service unavailability (standalone mode works fine)
/// - Self-knowledge only: petalTongue knows its own capabilities, not others
async fn register_with_discovery_service() {
    use petal_tongue_ipc::primal_registration::{PrimalRegistration, RegistrationManager};

    // Create petalTongue registration (self-knowledge only)
    let registration = PrimalRegistration::petaltongue();

    tracing::debug!(
        "📝 Registration: {} v{} with {} capabilities",
        registration.name,
        registration.version,
        registration.capabilities.len()
    );

    // Create registration manager (handles discovery service lookup)
    let manager = RegistrationManager::new(registration);

    // Attempt registration with discovered service (gracefully handles failure)
    manager.register_on_startup().await;

    // Spawn heartbeat task (maintains discovery presence)
    let _heartbeat_handle = manager.spawn_heartbeat_task();

    // Note: Heartbeat task runs in background until process exit
    // It automatically handles reconnection if discovery service restarts
    tracing::debug!("✅ Primal registration complete (heartbeat task spawned)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_ui() {
        let cli = Cli::parse_from(["petaltongue", "ui"]);
        assert!(matches!(cli.command, Commands::Ui { .. }));
    }

    #[test]
    fn test_cli_parse_tui() {
        let cli = Cli::parse_from(["petaltongue", "tui"]);
        assert!(matches!(cli.command, Commands::Tui { .. }));
    }

    #[test]
    fn test_cli_parse_web() {
        let cli = Cli::parse_from(["petaltongue", "web"]);
        assert!(matches!(cli.command, Commands::Web { .. }));
    }

    #[test]
    fn test_cli_parse_headless() {
        let cli = Cli::parse_from(["petaltongue", "headless"]);
        assert!(matches!(cli.command, Commands::Headless { .. }));
    }

    #[test]
    fn test_cli_parse_server() {
        let cli = Cli::parse_from(["petaltongue", "server"]);
        assert!(matches!(cli.command, Commands::Server));
    }

    #[test]
    fn test_cli_parse_status() {
        let cli = Cli::parse_from(["petaltongue", "status"]);
        assert!(matches!(cli.command, Commands::Status { .. }));
    }

    #[test]
    fn test_cli_parse_ui_with_scenario() {
        let cli = Cli::parse_from(["petaltongue", "ui", "--scenario", "test.json"]);
        let Commands::Ui { scenario, no_audio } = cli.command else {
            unreachable!("CLI parsed 'ui' subcommand")
        };
        assert_eq!(scenario.as_deref(), Some("test.json"));
        assert!(!no_audio);
    }

    #[test]
    fn test_cli_parse_ui_no_audio() {
        let cli = Cli::parse_from(["petaltongue", "ui", "--no-audio"]);
        let Commands::Ui { no_audio, .. } = cli.command else {
            unreachable!("CLI parsed 'ui' subcommand")
        };
        assert!(no_audio);
    }

    #[test]
    fn test_cli_parse_tui_with_refresh_rate() {
        let cli = Cli::parse_from(["petaltongue", "tui", "--refresh-rate", "30"]);
        let Commands::Tui { refresh_rate, .. } = cli.command else {
            unreachable!("CLI parsed 'tui' subcommand")
        };
        assert_eq!(refresh_rate, 30);
    }

    #[test]
    fn test_cli_parse_tui_default_refresh_rate() {
        let cli = Cli::parse_from(["petaltongue", "tui"]);
        let Commands::Tui { refresh_rate, .. } = cli.command else {
            unreachable!("CLI parsed 'tui' subcommand")
        };
        assert_eq!(refresh_rate, 60);
    }

    #[test]
    fn test_cli_parse_tui_with_scenario() {
        let cli = Cli::parse_from(["petaltongue", "tui", "--scenario", "demo.json"]);
        let Commands::Tui { scenario, .. } = cli.command else {
            unreachable!("CLI parsed 'tui' subcommand")
        };
        assert_eq!(scenario.as_deref(), Some("demo.json"));
    }

    #[test]
    fn test_cli_parse_status_default_verbose() {
        let cli = Cli::parse_from(["petaltongue", "status"]);
        let Commands::Status { verbose, .. } = cli.command else {
            unreachable!("CLI parsed 'status' subcommand")
        };
        assert!(!verbose);
    }

    #[test]
    fn test_cli_parse_status_verbose() {
        let cli = Cli::parse_from(["petaltongue", "status", "--verbose"]);
        let Commands::Status { verbose, .. } = cli.command else {
            unreachable!("CLI parsed 'status' subcommand")
        };
        assert!(verbose);
    }

    #[test]
    fn test_cli_parse_status_text_format() {
        let cli = Cli::parse_from(["petaltongue", "status", "--format", "text"]);
        let Commands::Status { format, .. } = cli.command else {
            unreachable!("CLI parsed 'status' subcommand")
        };
        assert_eq!(format, "text");
    }

    #[test]
    fn test_cli_log_level_trace() {
        let cli = Cli::parse_from(["petaltongue", "--log-level", "trace", "status"]);
        assert_eq!(cli.log_level, "trace");
    }

    #[test]
    fn test_cli_log_level_warn() {
        let cli = Cli::parse_from(["petaltongue", "--log-level", "warn", "status"]);
        assert_eq!(cli.log_level, "warn");
    }

    #[test]
    fn test_cli_log_format_json() {
        let cli = Cli::parse_from(["petaltongue", "--log-format", "json", "status"]);
        assert_eq!(cli.log_format, "json");
    }

    #[test]
    fn test_cli_parse_web_with_bind() {
        let cli = Cli::parse_from(["petaltongue", "web", "--bind", "127.0.0.1:9090"]);
        let Commands::Web { bind, workers, .. } = cli.command else {
            unreachable!("CLI parsed 'web' subcommand")
        };
        assert_eq!(bind.as_deref(), Some("127.0.0.1:9090"));
        assert_eq!(workers, 4);
    }

    #[test]
    fn test_cli_parse_web_with_workers() {
        let cli = Cli::parse_from(["petaltongue", "web", "--workers", "8"]);
        let Commands::Web { workers, .. } = cli.command else {
            unreachable!("CLI parsed 'web' subcommand")
        };
        assert_eq!(workers, 8);
    }

    #[test]
    fn test_cli_parse_headless_with_all_options() {
        let cli = Cli::parse_from([
            "petaltongue",
            "headless",
            "--bind",
            "0.0.0.0:7070",
            "--workers",
            "2",
        ]);
        let Commands::Headless { bind, workers } = cli.command else {
            unreachable!("CLI parsed 'headless' subcommand")
        };
        assert_eq!(bind.as_deref(), Some("0.0.0.0:7070"));
        assert_eq!(workers, 2);
    }

    #[test]
    fn test_cli_parse_status_verbose_json() {
        let cli = Cli::parse_from(["petaltongue", "status", "--verbose", "--format", "json"]);
        let Commands::Status { verbose, format } = cli.command else {
            unreachable!("CLI parsed 'status' subcommand")
        };
        assert!(verbose);
        assert_eq!(format, "json");
    }

    #[test]
    fn test_cli_default_log_level() {
        let cli = Cli::parse_from(["petaltongue", "status"]);
        assert_eq!(cli.log_level, "info");
        assert_eq!(cli.log_format, "pretty");
    }

    #[test]
    fn test_cli_custom_log_level() {
        let cli = Cli::parse_from(["petaltongue", "--log-level", "debug", "status"]);
        assert_eq!(cli.log_level, "debug");
    }

    #[test]
    fn test_cli_gui_alias() {
        let cli = Cli::parse_from(["petaltongue", "gui"]);
        assert!(matches!(cli.command, Commands::Ui { .. }));
    }

    #[test]
    fn test_cli_parse_ui_with_scenario_and_no_audio() {
        let cli = Cli::parse_from(["petaltongue", "ui", "--scenario", "demo.json", "--no-audio"]);
        let Commands::Ui { scenario, no_audio } = cli.command else {
            unreachable!("CLI parsed 'ui' subcommand")
        };
        assert_eq!(scenario.as_deref(), Some("demo.json"));
        assert!(no_audio);
    }

    #[test]
    fn test_cli_parse_web_with_scenario() {
        let cli = Cli::parse_from(["petaltongue", "web", "--scenario", "tutorial.json"]);
        let Commands::Web { scenario, .. } = cli.command else {
            unreachable!("CLI parsed 'web' subcommand")
        };
        assert_eq!(scenario.as_deref(), Some("tutorial.json"));
    }

    #[test]
    fn test_cli_custom_log_format() {
        let cli = Cli::parse_from(["petaltongue", "--log-format", "compact", "status"]);
        assert_eq!(cli.log_format, "compact");
    }

    #[test]
    fn test_cli_parse_headless_default_workers() {
        let cli = Cli::parse_from(["petaltongue", "headless"]);
        let Commands::Headless { workers, .. } = cli.command else {
            unreachable!("CLI parsed 'headless' subcommand")
        };
        assert_eq!(workers, 4);
    }

    #[test]
    fn test_cli_parse_status_default_format() {
        let cli = Cli::parse_from(["petaltongue", "status"]);
        let Commands::Status { format, .. } = cli.command else {
            unreachable!("CLI parsed 'status' subcommand")
        };
        assert_eq!(format, "text");
    }

    #[test]
    fn test_commands_debug() {
        let cmd = Commands::Status {
            verbose: false,
            format: "text".to_string(),
        };
        let debug_str = format!("{cmd:?}");
        assert!(debug_str.contains("Status"));
    }

    // init_tracing tests - tracing can only be initialized once per process.
    // Run test_init_tracing_invalid_level first (doesn't init).
    // test_init_tracing_formats exercises all format branches and double-init error.

    #[test]
    fn test_init_tracing_invalid_level() {
        // "crate=invalid_level" - invalid level fails at EnvFilter::try_new (does not call try_init)
        let result = init_tracing("crate=invalid_level", "pretty");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("log level") || err_msg.contains("parse"));
    }

    #[test]
    fn test_init_tracing_formats() {
        // Tracing can only be initialized once per process. Run formats test first (single-threaded
        // or before any other init). If another test already initialized, all inits here will fail.
        // 1. JSON format init (first successful init in this test)
        let result = init_tracing("info", "json");
        assert!(
            result.is_ok(),
            "first init with json format should succeed: {result:?}"
        );

        // 2. Compact format - fails because already initialized (exercises compact branch)
        let result = init_tracing("debug", "compact");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("compact") || err_msg.contains("init"),
            "expected compact init error: {err_msg}"
        );

        // 3. Pretty format - fails because already initialized (exercises default branch)
        let result = init_tracing("info", "pretty");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("pretty") || err_msg.contains("init"),
            "expected pretty init error: {err_msg}"
        );
    }

    #[tokio::test]
    async fn test_register_with_discovery_service_completes() {
        // Registration runs to completion (gracefully handles service unavailability)
        register_with_discovery_service().await;
    }

    #[test]
    fn test_config_from_env_loads() {
        let config = Config::from_env().expect("Config::from_env should load");
        assert!(config.network.web_port > 0);
        assert!(config.network.headless_port > 0);
    }

    #[test]
    fn test_web_bind_addr_fallback_format() {
        let port = 3000u16;
        let addr = format!("0.0.0.0:{port}");
        assert_eq!(addr, "0.0.0.0:3000");
    }

    #[test]
    fn test_web_bind_addr_explicit_override() {
        let bind = Some("127.0.0.1:9090".to_string());
        let addr = bind.as_deref().unwrap_or("0.0.0.0:3000");
        assert_eq!(addr, "127.0.0.1:9090");
    }

    #[test]
    fn test_headless_bind_addr_fallback_format() {
        let port = 8080u16;
        let addr = format!("0.0.0.0:{port}");
        assert_eq!(addr, "0.0.0.0:8080");
    }

    #[test]
    fn test_subcommand_routing_web_bind_resolution() {
        // When --bind is omitted, main uses config.network.web_port
        let cli = Cli::parse_from(["petaltongue", "web"]);
        let Commands::Web { bind, .. } = cli.command else {
            unreachable!("parsed web")
        };
        assert!(bind.is_none(), "default web has no explicit bind");
    }

    #[test]
    fn test_subcommand_routing_headless_bind_resolution() {
        let cli = Cli::parse_from(["petaltongue", "headless"]);
        let Commands::Headless { bind, .. } = cli.command else {
            unreachable!("parsed headless")
        };
        assert!(bind.is_none(), "default headless has no explicit bind");
    }
}
