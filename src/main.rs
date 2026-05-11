// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
//! petalTongue ecoBud - Production `UniBin`
//!
//! # Architecture
//!
//! `UniBin`: 1 binary, 7 subcommands (ui, tui, web, headless, server, live, status)
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
#[cfg(feature = "ui")]
mod live_mode;
mod notebook_render;
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
    ///
    /// When `--docroot` is provided, serves static files from that directory
    /// as a fallback for any path not matched by the API routes. This enables
    /// sovereign static site serving (sporePrint, Zola builds, etc.).
    Web {
        /// TCP port (UniBin standard: `--port` binds `0.0.0.0:PORT`)
        #[arg(long)]
        port: Option<u16>,

        /// Bind address override (takes precedence over --port)
        #[arg(long)]
        bind: Option<String>,

        /// Scenario JSON file to load
        #[arg(long)]
        scenario: Option<String>,

        /// Static file document root for catch-all serving (e.g., Zola build output)
        #[arg(long, env = "PETALTONGUE_DOCROOT")]
        docroot: Option<String>,

        /// Content backend: "filesystem" (default) or "nestgate" (content-addressed)
        #[arg(long, env = "PETALTONGUE_WEB_BACKEND", default_value = "filesystem")]
        backend: String,

        /// Also start UDS JSON-RPC IPC server alongside HTTP (NUCLEUS dual-port mode)
        #[arg(long)]
        ipc: bool,

        /// TCP port for IPC JSON-RPC when --ipc is active (optional, UDS always active)
        #[arg(long)]
        ipc_port: Option<u16>,

        /// Number of worker threads (configures tokio runtime)
        #[arg(long, default_value = "4")]
        workers: usize,

        /// Hide code cells when rendering .ipynb notebooks (outputs only)
        #[arg(long, env = "PETALTONGUE_STRIP_SOURCES")]
        strip_sources: bool,

        /// Cache-Control max-age in seconds for static files (0 = no cache header)
        #[arg(long, env = "PETALTONGUE_CACHE_TTL")]
        cache_ttl: Option<u64>,

        /// SPA mode: serve index.html for missing paths (client-side routing)
        #[arg(long, env = "PETALTONGUE_SPA")]
        spa: bool,

        /// CORS allowed origins (comma-separated, or "*" for all)
        #[arg(long, env = "PETALTONGUE_ALLOWED_ORIGINS", value_delimiter = ',')]
        allowed_origins: Vec<String>,
    },

    /// Run headless API server (Pure Rust! ✅)
    Headless {
        /// TCP port (UniBin standard: `--port` binds `0.0.0.0:PORT`)
        #[arg(long)]
        port: Option<u16>,

        /// Bind address override (takes precedence over --port)
        #[arg(long)]
        bind: Option<String>,

        /// Number of worker threads
        #[arg(long, default_value = "4")]
        workers: usize,
    },

    /// Run IPC server (Unix socket JSON-RPC) without display
    ///
    /// Socket path priority: --socket flag > PETALTONGUE_SOCKET env > XDG default
    Server {
        /// TCP port for newline-delimited JSON-RPC (optional, UDS always active)
        #[arg(long)]
        port: Option<u16>,

        /// TCP bind host (default: 127.0.0.1; use 0.0.0.0 for Docker/network)
        #[arg(long, env = "PETALTONGUE_IPC_HOST")]
        bind: Option<String>,

        /// Unix domain socket path override (or set PETALTONGUE_SOCKET env var)
        #[arg(long, env = "PETALTONGUE_SOCKET")]
        socket: Option<String>,
    },

    /// Launch native desktop display with IPC server (NUCLEUS interactive mode)
    Live {
        /// Scenario JSON file to load
        #[arg(long)]
        scenario: Option<String>,

        /// Disable audio sonification
        #[arg(long)]
        no_audio: bool,

        /// TCP port for newline-delimited JSON-RPC (optional, UDS always active)
        #[arg(long)]
        port: Option<u16>,

        /// TCP bind host (default: 127.0.0.1; use 0.0.0.0 for Docker/network)
        #[arg(long, env = "PETALTONGUE_IPC_HOST")]
        bind: Option<String>,

        /// Unix domain socket path override (or set PETALTONGUE_SOCKET env var)
        #[arg(long, env = "PETALTONGUE_SOCKET")]
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

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();

    init_tracing(&cli.log_level, &cli.log_format)?;

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        command = ?cli.command,
        "🌸 petalTongue starting"
    );

    let cli_workers = match &cli.command {
        Commands::Web { workers, .. } | Commands::Headless { workers, .. } => Some(*workers),
        _ => None,
    };

    let mut rt_builder = tokio::runtime::Builder::new_multi_thread();
    rt_builder.enable_all();
    if let Some(w) = cli_workers {
        rt_builder.worker_threads(w);
    }
    let runtime = rt_builder
        .build()
        .map_err(|e| AppError::Other(format!("Failed to create tokio runtime: {e}")))?;

    let (cli_tcp_port, cli_bind_host) = match &cli.command {
        Commands::Server { port, bind, .. } | Commands::Live { port, bind, .. } => {
            (*port, parse_ipc_bind_host(bind.as_deref()))
        }
        _ => (None, std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
    };

    // Async setup: config, data service, discovery registration
    let (config, data_service) = runtime.block_on(async {
        tracing::info!("⚙️ Loading configuration from environment...");
        let config = Config::from_env().map_err(|e| AppError::Other(e.to_string()))?;
        tracing::info!(
            web_port = config.network.web_port,
            headless_port = config.network.headless_port,
            "✅ Configuration loaded"
        );

        tracing::info!("📊 Initializing unified DataService...");
        let mut data_service = data_service::DataService::new();
        data_service.init().await?;
        let data_service = std::sync::Arc::new(data_service);
        tracing::info!("✅ DataService initialized - all modes will use same data source");

        tracing::info!("🔍 Registering with ecosystem discovery service...");
        register_with_discovery_service(cli_tcp_port, cli_bind_host).await;

        Ok::<_, AppError>((config, data_service))
    })?;

    // PG-40 fix: UI modes (ui, live) run eframe on the main thread.
    // winit requires main-thread event loop init on Linux (X11/Wayland).
    // Non-UI modes dispatch async via runtime.block_on().
    let result = match cli.command {
        #[cfg(feature = "ui")]
        Commands::Ui { scenario, no_audio } => {
            tracing::info!(mode = "ui", "Launching desktop display mode");
            ui_mode::run_on_main_thread(scenario, no_audio, &data_service)
        }
        #[cfg(not(feature = "ui"))]
        Commands::Ui { .. } => Err(AppError::UiNotAvailable),

        #[cfg(feature = "ui")]
        Commands::Live {
            scenario,
            no_audio,
            port,
            bind,
            socket,
        } => {
            let bind_host = parse_ipc_bind_host(bind.as_deref());
            tracing::info!(mode = "live", tcp_port = ?port, ?bind_host, socket = ?socket, "Launching NUCLEUS interactive mode (IPC + GUI)");
            live_mode::run_on_main_thread(
                scenario,
                no_audio,
                &data_service,
                port,
                bind_host,
                socket,
                &runtime,
            )
        }
        #[cfg(not(feature = "ui"))]
        Commands::Live { .. } => Err(AppError::UiNotAvailable),

        other => runtime.block_on(dispatch_async(other, config, data_service)),
    };

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

/// Dispatch non-GUI commands on the async runtime.
async fn dispatch_async(
    command: Commands,
    config: Config,
    data_service: std::sync::Arc<data_service::DataService>,
) -> Result<(), AppError> {
    match command {
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
            port,
            bind,
            scenario,
            docroot,
            backend,
            ipc,
            ipc_port,
            workers,
            strip_sources,
            cache_ttl,
            spa,
            allowed_origins,
        } => {
            dispatch_web(
                port,
                bind,
                scenario,
                docroot,
                backend,
                ipc,
                ipc_port,
                workers,
                strip_sources,
                cache_ttl,
                spa,
                allowed_origins,
                config,
                data_service,
            )
            .await
        }
        Commands::Headless {
            port,
            bind,
            workers,
        } => {
            let bind_addr = resolve_bind(bind, port, || config.network.headless_addr().to_string());
            tracing::info!(
                mode = "headless",
                bind = %bind_addr,
                workers,
                "Launching headless API server (Pure Rust!)"
            );
            headless_mode::run(&bind_addr, workers, data_service).await
        }
        Commands::Server { port, bind, socket } => {
            let bind_host = parse_ipc_bind_host(bind.as_deref());
            tracing::info!(mode = "server", tcp_port = ?port, ?bind_host, socket = ?socket, "Launching IPC server (no display)");
            server_mode::run(data_service, port, bind_host, socket).await
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
        Commands::Ui { .. } | Commands::Live { .. } => {
            unreachable!("GUI modes handled on main thread")
        }
    }
}

/// Dispatch `web` command — extracted to keep `dispatch_async` under the line limit.
#[expect(
    clippy::too_many_arguments,
    reason = "web dispatch aggregates CLI + config params"
)]
async fn dispatch_web(
    port: Option<u16>,
    bind: Option<String>,
    scenario: Option<String>,
    docroot: Option<String>,
    backend: String,
    ipc: bool,
    ipc_port: Option<u16>,
    workers: usize,
    strip_sources: bool,
    cache_ttl: Option<u64>,
    spa: bool,
    allowed_origins: Vec<String>,
    config: Config,
    data_service: std::sync::Arc<data_service::DataService>,
) -> Result<(), AppError> {
    let bind_addr = resolve_bind(bind, port, || config.network.web_addr().to_string());
    let effective_docroot = docroot.or_else(|| {
        config
            .web
            .docroot
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
    });
    let effective_backend = if backend == "filesystem" {
        config.web.backend.clone()
    } else {
        backend
    };
    let effective_strip = strip_sources || config.web.strip_sources;
    let effective_cache_ttl = cache_ttl.unwrap_or(config.web.cache_ttl_secs);
    let effective_spa = spa || config.web.spa;
    let effective_origins = if allowed_origins.is_empty() {
        config.web.allowed_origins.clone()
    } else {
        allowed_origins
    };

    tracing::info!(
        mode = "web",
        bind = %bind_addr,
        docroot = ?effective_docroot,
        backend = %effective_backend,
        ipc,
        ipc_port = ?ipc_port,
        workers,
        strip_sources = effective_strip,
        cache_ttl = effective_cache_ttl,
        spa = effective_spa,
        allowed_origins = ?effective_origins,
        "Launching web UI server (Pure Rust!)"
    );

    if ipc {
        let ipc_service = std::sync::Arc::clone(&data_service);
        let ipc_tcp = ipc_port;
        tokio::spawn(async move {
            if let Err(e) = server_mode::run(
                ipc_service,
                ipc_tcp,
                std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                None,
            )
            .await
            {
                tracing::error!("IPC server error (web+ipc mode): {e}");
            }
        });
        tracing::info!("🔌 IPC server co-started alongside web (PT-4 dual-port mode)");
    }

    let cfg = web_mode::WebConfig {
        bind: &bind_addr,
        scenario,
        docroot: effective_docroot,
        backend: &effective_backend,
        workers,
        strip_sources: effective_strip,
        cache_ttl_secs: effective_cache_ttl,
        spa: effective_spa,
        allowed_origins: effective_origins,
    };
    web_mode::run(cfg, data_service).await
}

/// Resolve bind address from `--bind` (explicit), `--port` (UniBin standard), or config default.
fn resolve_bind(
    bind: Option<String>,
    port: Option<u16>,
    default: impl FnOnce() -> String,
) -> String {
    if let Some(b) = bind {
        return b;
    }
    if let Some(p) = port {
        return format!("0.0.0.0:{p}");
    }
    default()
}

/// Parse an IPC TCP bind host from the `--bind` flag or `PETALTONGUE_IPC_HOST` env.
///
/// PG-55: secure default `127.0.0.1`. Docker/network-facing deployments
/// use `--bind 0.0.0.0`. Matches Squirrel SQ-04 / coralReef `--bind` pattern.
fn parse_ipc_bind_host(bind: Option<&str>) -> std::net::IpAddr {
    bind.and_then(|s| s.parse().ok())
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
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

/// Register petalTongue with the ecosystem discovery service.
///
/// This implements the `ipc.register` standard from `PRIMAL_IPC_PROTOCOL.md`.
/// Uses capability-based discovery to find the registration service (any primal
/// providing the "discovery" capability).
///
/// When `tcp_port` is `Some`, the registration advertises the TCP endpoint
/// so Songbird can return it for tier-1 `ipc.resolve` routing.
///
/// # TRUE PRIMAL: Capability-Based Registration
/// - Discovers the registration service at runtime (no hardcoded primal name)
/// - Gracefully handles service unavailability (standalone mode works fine)
/// - Self-knowledge only: petalTongue knows its own capabilities, not others
async fn register_with_discovery_service(tcp_port: Option<u16>, tcp_bind_host: std::net::IpAddr) {
    use petal_tongue_ipc::primal_registration::{PrimalRegistration, RegistrationManager};

    let mut registration = PrimalRegistration::petaltongue();
    if let Some(port) = tcp_port {
        registration = registration.with_tcp_endpoint(tcp_bind_host, port);
    }

    tracing::debug!(
        "📝 Registration: {} v{} with {} capabilities, transports={:?}",
        registration.name,
        registration.version,
        registration.capabilities.len(),
        registration.transports,
    );

    let manager = RegistrationManager::new(registration);
    manager.register_on_startup().await;
    let _heartbeat_handle = manager.spawn_heartbeat_task();

    tracing::debug!("✅ Primal registration complete (heartbeat task spawned)");
}

#[cfg(test)]
mod tests;
