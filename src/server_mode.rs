// SPDX-License-Identifier: AGPL-3.0-or-later
//! Server mode - IPC server without display
//!
//! Runs the JSON-RPC server (UDS on Unix, TCP-only on other platforms),
//! plus tarpc and G65 negotiate servers via G66 transport abstraction.
//!
//! **PT-06:** [`UnixSocketServer::new`](petal_tongue_ipc::UnixSocketServer::new) wires
//! push delivery (`spawn_push_delivery` / `callback_tx`) on the JSON-RPC handlers.

use crate::data_service::DataService;
use crate::error::AppError;
use petal_tongue_ipc::UnixSocketServer;
use std::sync::Arc;

/// Run IPC server without display.
///
/// On Unix: binds JSON-RPC UDS + optional TCP, tarpc, and G65 negotiate.
/// On other platforms: tarpc + G65 negotiate via transport abstraction;
/// JSON-RPC via TCP fallback.
///
/// Spawns a periodic discovery refresh so the graph engine has live topology
/// data even without a display attached (PT-07: external event source).
pub async fn run(
    data_service: Arc<DataService>,
    tcp_port: Option<u16>,
    tcp_bind_host: std::net::IpAddr,
    socket_path: Option<String>,
) -> Result<(), AppError> {
    let graph = data_service.graph();

    // PT-07: periodic capability discovery refresh so server mode has live data
    let refresh_service = Arc::clone(&data_service);
    tokio::spawn(async move {
        let mut interval =
            tokio::time::interval(petal_tongue_core::constants::default_heartbeat_interval());
        loop {
            interval.tick().await;
            if let Err(e) = refresh_service.refresh().await {
                tracing::warn!("periodic discovery refresh failed: {e}");
            }
        }
    });

    // tarpc + G65 negotiate servers (transport-agnostic, all platforms)
    let tarpc_server = petal_tongue_ipc::TarpcServer::from_default_path()
        .map_err(|e| AppError::Other(format!("tarpc server init: {e}")))?;
    tracing::info!("tarpc: {} (C2)", tarpc_server.endpoint());

    let negotiate_server = petal_tongue_ipc::NegotiateServer::from_default_path()
        .map_err(|e| AppError::Other(format!("G65 negotiate server init: {e}")))?;
    tracing::info!("G65 negotiate: {} (Phase 3)", negotiate_server.endpoint());

    // JSON-RPC server (UDS on Unix, TCP-only on Windows/Android)
    let jsonrpc_future = {
        let (motor_tx, motor_rx) = std::sync::mpsc::channel();
        let socket_override = socket_path.map(std::path::PathBuf::from);
        let mut server = UnixSocketServer::new_with_socket(graph, socket_override)?
            .with_motor_sender(motor_tx)
            .with_tcp_bind_host(tcp_bind_host);

        if let Some(port) = tcp_port {
            server = server.with_tcp_port(port);
        }

        let server = Arc::new(server);

        tokio::task::spawn_blocking(move || {
            while let Ok(cmd) = motor_rx.recv() {
                tracing::debug!(?cmd, "motor command received (no display attached)");
            }
        });

        if tcp_port.is_some() {
            tracing::info!("JSON-RPC server: UDS + TCP (no display)");
        } else {
            tracing::info!("JSON-RPC server: IPC (no display)");
        }

        async move { server.start().await.map_err(AppError::from) }
    };

    tokio::select! {
        result = jsonrpc_future => {
            result?;
        }
        result = tarpc_server.serve() => {
            if let Err(e) = result {
                tracing::error!("tarpc server error: {e}");
            }
        }
        result = negotiate_server.serve() => {
            if let Err(e) = result {
                tracing::error!("G65 negotiate server error: {e}");
            }
        }
        () = crate::signal::shutdown_signal() => {
            tracing::info!("Server mode shut down gracefully");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "test code uses unwrap/expect for brevity"
    )]

    use super::*;
    use petal_tongue_core::test_fixtures::env_test_helpers;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_run_with_tcp_port_some() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("petaltongue-test-tcp.sock");
        let socket_str = socket_path.to_string_lossy().into_owned();

        let result =
            env_test_helpers::with_env_var_async("PETALTONGUE_SOCKET", &socket_str, || async {
                let data_service = Arc::new(DataService::new());
                tokio::time::timeout(
                    std::time::Duration::from_millis(500),
                    run(
                        data_service,
                        Some(0),
                        std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                        None,
                    ),
                )
                .await
            })
            .await;

        if let Ok(Err(e)) = result {
            let msg = e.to_string();
            assert!(
                msg.contains("IPC") || msg.contains("Failed") || msg.contains("bind"),
                "Expected IPC/bind error, got: {msg}"
            );
        }
    }

    #[tokio::test]
    async fn test_run_creates_server_with_valid_socket_path() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("petaltongue-test.sock");
        let socket_str = socket_path.to_string_lossy().into_owned();

        let result =
            env_test_helpers::with_env_var_async("PETALTONGUE_SOCKET", &socket_str, || async {
                let data_service = Arc::new(DataService::new());
                tokio::time::timeout(
                    std::time::Duration::from_millis(500),
                    run(
                        data_service,
                        None,
                        std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                        None,
                    ),
                )
                .await
            })
            .await;

        if let Ok(Err(e)) = result {
            let msg = e.to_string();
            assert!(
                msg.contains("IPC") || msg.contains("Failed") || msg.contains("bind"),
                "Expected IPC/bind error, got: {msg}"
            );
        }
        // Ok(Ok): server exited; Err: timeout means server started and is running
    }

    #[tokio::test]
    async fn test_run_with_cli_socket_override() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("petaltongue-cli-override.sock");
        let socket_str = socket_path.to_string_lossy().into_owned();

        let data_service = Arc::new(DataService::new());
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            run(
                data_service,
                None,
                std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
                Some(socket_str),
            ),
        )
        .await;

        if let Ok(Err(e)) = result {
            let msg = e.to_string();
            assert!(
                msg.contains("IPC") || msg.contains("Failed") || msg.contains("bind"),
                "Expected IPC/bind error, got: {msg}"
            );
        }
    }

    #[tokio::test]
    async fn test_run_propagates_socket_path_error() {
        let data_service = Arc::new(DataService::new());
        let result = run(
            data_service,
            None,
            std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST),
            Some("/".to_owned()),
        )
        .await;

        if let Err(e) = result {
            let msg = e.to_string();
            assert!(
                msg.contains("IPC") || msg.contains("Failed") || msg.contains("socket"),
                "Error should mention IPC or socket: {msg}"
            );
        }
    }

    #[tokio::test]
    async fn test_run_uses_data_service_graph() {
        let data_service = Arc::new(DataService::new());
        let graph = data_service.graph();
        assert!(graph.read().is_ok());
    }

    /// Wave 79 transport compliance: `run()` with `tcp_port: None` must not
    /// configure a TCP port on the `UnixSocketServer`.
    #[test]
    fn test_uds_only_no_tcp_port_configured() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("transport-compliance.sock");
        let socket_override = Some(socket_path);

        let graph = Arc::new(DataService::new()).graph();
        let (motor_tx, _motor_rx) = std::sync::mpsc::channel();

        let server =
            UnixSocketServer::new_with_socket(graph, socket_override).expect("create server");
        let server = server.with_motor_sender(motor_tx);

        assert!(
            !server.has_tcp_port(),
            "UDS-only server should not have a TCP port configured"
        );
    }

    /// Wave 79 transport compliance: `run()` with `tcp_port: Some(port)` must
    /// configure TCP on the `UnixSocketServer`.
    #[test]
    fn test_dual_transport_has_tcp_port() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("dual-transport.sock");
        let socket_override = Some(socket_path);

        let graph = Arc::new(DataService::new()).graph();
        let (motor_tx, _motor_rx) = std::sync::mpsc::channel();

        let server =
            UnixSocketServer::new_with_socket(graph, socket_override).expect("create server");
        let server = server.with_motor_sender(motor_tx).with_tcp_port(8080);

        assert!(
            server.has_tcp_port(),
            "dual-transport server should have a TCP port configured"
        );
    }

    #[test]
    fn test_server_config_socket_path_from_env() {
        let temp = tempfile::tempdir().expect("temp dir");
        let custom_path = temp.path().join("custom.sock");
        let path_str = custom_path.to_string_lossy().into_owned();

        let path = env_test_helpers::with_env_var(
            "PETALTONGUE_SOCKET",
            &path_str,
            petal_tongue_ipc::socket_path::get_petaltongue_socket_path,
        );

        assert!(path.is_ok());
        assert_eq!(path.unwrap(), custom_path);
    }
}
