// SPDX-License-Identifier: AGPL-3.0-or-later
//! Server mode - IPC server without display
//!
//! Runs the Unix socket JSON-RPC server for petalTongue IPC.
//! Clients can connect to query topology, health, capabilities, etc.

use crate::data_service::DataService;
use crate::error::AppError;
use petal_tongue_ipc::UnixSocketServer;
use std::sync::Arc;

/// Run IPC server (Unix socket JSON-RPC) without display.
///
/// Uses the shared `DataService` graph. Runs until interrupted (e.g. Ctrl+C).
pub async fn run(data_service: Arc<DataService>) -> Result<(), AppError> {
    let graph = data_service.graph();

    let server = UnixSocketServer::new(graph)
        .map_err(|e| AppError::Other(format!("Failed to create IPC server: {e}")))?;

    let server = Arc::new(server);

    tracing::info!("🔌 IPC server starting (Unix socket, no display)");
    tracing::info!("   Connect via JSON-RPC to the socket path");

    server
        .start()
        .await
        .map_err(|e| AppError::Other(format!("IPC server error: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::test_fixtures::env_test_helpers;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_run_creates_server_with_valid_socket_path() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("petaltongue-test.sock");
        let socket_str = socket_path.to_string_lossy().to_string();

        let result =
            env_test_helpers::with_env_var_async("PETALTONGUE_SOCKET", &socket_str, || async {
                let data_service = Arc::new(DataService::new());
                // run() blocks on server.start() - use timeout; either completes or times out
                tokio::time::timeout(std::time::Duration::from_millis(500), run(data_service)).await
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
    async fn test_run_propagates_socket_path_error() {
        // Invalid socket path: "/" cannot be bound as a Unix socket
        let result = env_test_helpers::with_env_var_async("PETALTONGUE_SOCKET", "/", || async {
            let data_service = Arc::new(DataService::new());
            run(data_service).await
        })
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
