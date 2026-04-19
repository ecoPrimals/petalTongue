// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! petalTongue ecoBud - Production `UniBin`
//!
//! # Architecture
//!
//! `UniBin`: 1 binary, 6 subcommands (ui, tui, web, headless, server, status)
//! ecoBin: 100% Pure Rust (ui uses egui/eframe for platform windowing)
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
    long_about = "ecoBud v1.0: UniBin + Pure Rust\n\nFully concurrent, modern Rust architecture"
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
    /// Launch native desktop display
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

    /// Run IPC server (Unix socket JSON-RPC) without display
    Server {
        /// TCP port for newline-delimited JSON-RPC (optional, UDS always active)
        #[arg(long)]
        port: Option<u16>,

        /// Unix domain socket path override (default: $XDG_RUNTIME_DIR/biomeos/petaltongue-{family}.sock)
        #[arg(long)]
        socket: Option<String>,
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
            tracing::info!(mode = "ui", "Launching desktop display mode");
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
            let bind_addr = bind.unwrap_or_else(|| config.network.web_addr().to_string());

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
            let bind_addr = bind.unwrap_or_else(|| config.network.headless_addr().to_string());

            tracing::info!(
                mode = "headless",
                bind = %bind_addr,
                workers,
                "Launching headless API server (Pure Rust!)"
            );
            headless_mode::run(&bind_addr, workers, data_service).await
        }
        Commands::Server { port, socket } => {
            tracing::info!(mode = "server", tcp_port = ?port, socket = ?socket, "Launching IPC server (no display)");
            server_mode::run(data_service, port, socket).await
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
/// Uses capability-based discovery to find the registration service (any primal
/// providing the "discovery" capability).
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
mod tests;
