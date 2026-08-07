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

mod bootstrap;
mod cli;
mod cli_mode;
mod content_render;
mod data_service;
mod dispatch;
mod error;
mod headless_mode;
#[cfg(feature = "ui")]
mod live_mode;
mod notebook_render;
mod registration;
mod server_mode;
mod signal;
#[cfg(feature = "tui")]
mod tui_mode;
#[cfg(feature = "ui")]
mod ui_mode;
mod viz_data;
mod web_mode;

use clap::Parser;
use petal_tongue_core::config_system::Config;

use crate::bootstrap::init_tracing;
use crate::cli::{Cli, Commands};
use crate::dispatch::{dispatch_async, parse_ipc_bind_host};
#[cfg(all(feature = "ui", unix))]
use crate::dispatch::resolve_server_transport;
use crate::error::AppError;

#[expect(
    clippy::too_many_lines,
    reason = "UniBin entry point: CLI parse, transport resolve, mode dispatch"
)]
fn main() -> Result<(), AppError> {
    let cli = Cli::parse();

    // Propagate --family-id to the IPC layer so BTSP/socket resolution uses it.
    // CLI flag takes precedence over FAMILY_ID env var.
    if let Some(ref fid) = cli.family_id {
        petal_tongue_ipc::socket_path::set_family_id_override(fid.clone());
    }

    init_tracing(&cli.log_level, &cli.log_format)?;

    let global_socket = cli.socket.clone();
    let global_port = cli.port;

    let transport_endpoint = match petal_tongue_core::transport::TransportEndpoint::from_env() {
        Ok(Some(ep)) => {
            tracing::info!(transport = %ep, "TRANSPORT_ENDPOINT injected by launcher");
            Some(ep)
        }
        Ok(None) => None,
        Err(e) => {
            tracing::warn!(error = %e, "invalid TRANSPORT_ENDPOINT, ignoring");
            None
        }
    };

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        command = ?cli.command,
        family_id = cli.family_id.as_deref().unwrap_or("(env)"),
        socket = global_socket.as_deref().unwrap_or("(default)"),
        transport = transport_endpoint.as_ref().map(ToString::to_string).as_deref().unwrap_or("(cli/default)"),
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
    let runtime = rt_builder.build()?;

    let (cli_tcp_port, cli_bind_host) = match &cli.command {
        Commands::Server { port, bind, .. } | Commands::Live { port, bind, .. } => (
            (*port).or(global_port),
            parse_ipc_bind_host(bind.as_deref()),
        ),
        _ => (
            global_port,
            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
        ),
    };

    // Async setup: config, data service, discovery registration
    let (config, data_service) = runtime.block_on(async {
        tracing::info!("⚙️ Loading configuration from environment...");
        let config = Config::from_env()?;
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

        tracing::info!("Registering with ecosystem discovery service...");
        registration::register_with_discovery_service(cli_tcp_port, cli_bind_host).await;

        registration::announce_to_neural_api().await;

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

        #[cfg(all(feature = "ui", unix))]
        Commands::Live {
            scenario,
            no_audio,
            port,
            bind,
            socket,
        } => {
            let (merged_socket, merged_port, bind_host) = resolve_server_transport(
                transport_endpoint.as_ref(),
                socket,
                port,
                global_socket,
                global_port,
                bind.as_deref(),
            );
            tracing::info!(mode = "live", tcp_port = ?merged_port, ?bind_host, socket = ?merged_socket, "Launching NUCLEUS interactive mode (IPC + GUI)");
            live_mode::run_on_main_thread(
                scenario,
                no_audio,
                &data_service,
                merged_port,
                bind_host,
                merged_socket,
                &runtime,
            )
        }
        #[cfg(not(all(feature = "ui", unix)))]
        Commands::Live { .. } => Err(AppError::UiNotAvailable),

        other => runtime.block_on(dispatch_async(
            other,
            config,
            data_service,
            global_socket,
            global_port,
            transport_endpoint,
        )),
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

#[cfg(test)]
mod tests;
