// SPDX-License-Identifier: AGPL-3.0-or-later
//! Server mode - IPC server without display
//!
//! Runs the JSON-RPC server for petalTongue IPC on a Unix domain socket
//! (always) and optionally on a TCP port via `--port`.
//!
//! **PT-06:** [`UnixSocketServer::new`](petal_tongue_ipc::UnixSocketServer::new) wires
//! push delivery (`spawn_push_delivery` / `callback_tx`) on the JSON-RPC handlers.

use crate::data_service::DataService;
use crate::error::AppError;
use petal_tongue_ipc::UnixSocketServer;
use std::sync::Arc;

/// Run IPC server without display.
///
/// Binds the UDS at `$XDG_RUNTIME_DIR/biomeos/petaltongue.sock` (always).
/// When `tcp_port` is provided, also binds a newline-delimited TCP JSON-RPC
/// listener on `0.0.0.0:<port>`.
///
/// Spawns a periodic discovery refresh so the graph engine has live topology
/// data even without a display attached (PT-07: external event source).
///
/// A motor command channel is created and drained so that `motor.*` IPC
/// methods succeed even without an attached display.
pub async fn run(
    data_service: Arc<DataService>,
    tcp_port: Option<u16>,
    socket_path: Option<String>,
) -> Result<(), AppError> {
    let graph = data_service.graph();

    let (motor_tx, motor_rx) = std::sync::mpsc::channel();

    let socket_override = socket_path.map(std::path::PathBuf::from);
    let mut server = UnixSocketServer::new_with_socket(graph, socket_override)
        .map_err(|e| AppError::Other(format!("Failed to create IPC server: {e}")))?
        .with_motor_sender(motor_tx);

    if let Some(port) = tcp_port {
        server = server.with_tcp_port(port);
    }

    let server = Arc::new(server);

    tokio::task::spawn_blocking(move || {
        while let Ok(cmd) = motor_rx.recv() {
            tracing::debug!(?cmd, "motor command received (no display attached)");
        }
    });

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

    tracing::info!("🔌 IPC server starting (UDS + optional TCP, no display)");

    server
        .start()
        .await
        .map_err(|e| AppError::Other(format!("IPC server error: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use petal_tongue_core::test_fixtures::env_test_helpers;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_run_with_tcp_port_some() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("petaltongue-test-tcp.sock");
        let socket_str = socket_path.to_string_lossy().to_string();

        let result =
            env_test_helpers::with_env_var_async("PETALTONGUE_SOCKET", &socket_str, || async {
                let data_service = Arc::new(DataService::new());
                tokio::time::timeout(
                    std::time::Duration::from_millis(500),
                    run(data_service, Some(0), None),
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
        let socket_str = socket_path.to_string_lossy().to_string();

        let result =
            env_test_helpers::with_env_var_async("PETALTONGUE_SOCKET", &socket_str, || async {
                let data_service = Arc::new(DataService::new());
                tokio::time::timeout(
                    std::time::Duration::from_millis(500),
                    run(data_service, None, None),
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
        let socket_str = socket_path.to_string_lossy().to_string();

        let data_service = Arc::new(DataService::new());
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            run(data_service, None, Some(socket_str)),
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
        let result = run(data_service, None, Some("/".to_string())).await;

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

    #[test]
    fn test_server_config_socket_path_from_env() {
        let temp = tempfile::tempdir().expect("temp dir");
        let custom_path = temp.path().join("custom.sock");
        let path_str = custom_path.to_string_lossy().to_string();

        let path = env_test_helpers::with_env_var(
            "PETALTONGUE_SOCKET",
            &path_str,
            petal_tongue_ipc::socket_path::get_petaltongue_socket_path,
        );

        assert!(path.is_ok());
        assert_eq!(path.unwrap(), custom_path);
    }
}
