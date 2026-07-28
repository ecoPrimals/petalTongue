// SPDX-License-Identifier: AGPL-3.0-or-later

use petal_tongue_core::config_system::Config;

use crate::bootstrap::discover_compositions;
use crate::cli::Commands;
use crate::cli_mode;
use crate::data_service;
use crate::error::AppError;
use crate::headless_mode;
use crate::server_mode;
#[cfg(feature = "tui")]
use crate::tui_mode;
use crate::web_mode;

/// Dispatch non-GUI commands on the async runtime.
pub async fn dispatch_async(
    command: Commands,
    config: Config,
    data_service: std::sync::Arc<data_service::DataService>,
    global_socket: Option<String>,
    global_port: Option<u16>,
    transport_endpoint: Option<petal_tongue_core::transport::TransportEndpoint>,
) -> Result<(), AppError> {
    match command {
        Commands::Tui {
            scenario,
            refresh_rate,
        } => {
            #[cfg(feature = "tui")]
            {
                tracing::info!(
                    mode = "tui",
                    refresh_rate,
                    "Launching terminal UI mode (Pure Rust!)"
                );
                tui_mode::run(scenario, refresh_rate, data_service).await
            }
            #[cfg(not(feature = "tui"))]
            {
                let _ = (scenario, refresh_rate, data_service);
                Err(AppError::Tui(
                    "TUI mode requires the 'tui' feature (enable with --features tui)".to_owned(),
                ))
            }
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
                global_socket,
            )
            .await
        }
        Commands::Headless {
            port,
            bind,
            workers,
        } => {
            let bind_addr = resolve_bind(bind, port.or(global_port), || {
                config.network.headless_addr().to_string()
            });
            tracing::info!(
                mode = "headless",
                bind = %bind_addr,
                workers,
                "Launching headless API server (Pure Rust!)"
            );
            headless_mode::run(&bind_addr, workers, data_service).await
        }
        Commands::Server { port, bind, socket } => {
            let (merged_socket, merged_port, bind_host) = resolve_server_transport(
                transport_endpoint.as_ref(),
                socket,
                port,
                global_socket,
                global_port,
                bind.as_deref(),
            );
            tracing::info!(mode = "server", tcp_port = ?merged_port, ?bind_host, socket = ?merged_socket, "Launching IPC server (no display)");
            server_mode::run(data_service, merged_port, bind_host, merged_socket).await
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
    global_socket: Option<String>,
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
        config.web.backend.clone().into_owned()
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
                global_socket,
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
        compositions: discover_compositions(),
    };
    web_mode::run(cfg, data_service).await
}

/// Resolve bind address from `--bind` (explicit), `--port` (`UniBin` standard), or config default.
pub fn resolve_bind(
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

/// Resolve server transport from `TRANSPORT_ENDPOINT` (launcher-injected) or CLI args.
///
/// `TRANSPORT_ENDPOINT` takes priority over CLI flags when set.
/// This implements the sourDough canonical transport injection standard (Wave 100).
pub fn resolve_server_transport(
    transport: Option<&petal_tongue_core::transport::TransportEndpoint>,
    cli_socket: Option<String>,
    cli_port: Option<u16>,
    global_socket: Option<String>,
    global_port: Option<u16>,
    cli_bind: Option<&str>,
) -> (Option<String>, Option<u16>, std::net::IpAddr) {
    use petal_tongue_core::transport::TransportEndpoint;

    if let Some(ep) = transport {
        match ep {
            TransportEndpoint::Uds { path } => {
                tracing::info!(transport = %ep, "using launcher-injected UDS transport");
                return (
                    Some(path.to_string_lossy().into_owned()),
                    None,
                    std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                );
            }
            TransportEndpoint::Tcp { host, port } => {
                tracing::info!(transport = %ep, "using launcher-injected TCP transport");
                let bind_host: std::net::IpAddr = host
                    .parse()
                    .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
                return (None, Some(*port), bind_host);
            }
            TransportEndpoint::MeshRelay { .. } => {
                tracing::warn!(transport = %ep, "mesh_relay transport not supported for server bind, falling back to CLI");
            }
        }
    }

    let merged_socket = cli_socket.or(global_socket);
    let merged_port = cli_port.or(global_port);
    let bind_host = parse_ipc_bind_host(cli_bind);
    (merged_socket, merged_port, bind_host)
}

/// Parse an IPC TCP bind host from the `--bind` flag or `PETALTONGUE_IPC_HOST` env.
///
/// PG-55: secure default `127.0.0.1`. Docker/network-facing deployments
/// use `--bind 0.0.0.0`. Matches Squirrel SQ-04 / coralReef `--bind` pattern.
pub fn parse_ipc_bind_host(bind: Option<&str>) -> std::net::IpAddr {
    bind.and_then(|s| s.parse().ok())
        .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
}
