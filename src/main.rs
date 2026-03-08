// SPDX-License-Identifier: AGPL-3.0-only
//! petalTongue ecoBud - Production UniBin
//!
//! # Architecture
//!
//! UniBin: 1 binary, 5 modes
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

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use petal_tongue_core::config_system::Config;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cli_mode;
mod data_service;
mod headless_mode;
mod tui_mode;
mod ui_mode;
mod web_mode;

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
        /// Bind address (default: from config or 0.0.0.0:<web_port>)
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
        /// Bind address (default: from config or 0.0.0.0:<headless_port>)
        #[arg(long)]
        bind: Option<String>,

        /// Number of worker threads
        #[arg(long, default_value = "4")]
        workers: usize,
    },

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
async fn main() -> Result<()> {
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
    let config = Config::from_env().context("Failed to load configuration")?;
    tracing::info!(
        web_port = config.network.web_port,
        headless_port = config.network.headless_port,
        "✅ Configuration loaded"
    );

    // Initialize DataService ONCE (single source of truth for all modes)
    tracing::info!("📊 Initializing unified DataService...");
    let mut data_service = data_service::DataService::new();
    data_service
        .init()
        .await
        .context("Failed to initialize DataService")?;
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
fn init_tracing(level: &str, format: &str) -> Result<()> {
    // Parse log level
    let env_filter =
        tracing_subscriber::EnvFilter::try_new(level).context("Failed to parse log level")?;

    // Build subscriber based on format
    match format {
        "json" => {
            // JSON logging for production (requires json feature)
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().with_target(true))
                .try_init()
                .context("Failed to initialize JSON logging")?;
        }
        "compact" => {
            // Compact logging
            tracing_subscriber::registry()
                .with(env_filter)
                .with(tracing_subscriber::fmt::layer().compact())
                .try_init()
                .context("Failed to initialize compact logging")?;
        }
        _ => {
            // Pretty logging (default)
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
                .context("Failed to initialize pretty logging")?;
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
    fn test_cli_parse_status() {
        let cli = Cli::parse_from(["petaltongue", "status"]);
        assert!(matches!(cli.command, Commands::Status { .. }));
    }

    // All tests run in parallel (default)
    // No sleeps needed - tests are deterministic
}
