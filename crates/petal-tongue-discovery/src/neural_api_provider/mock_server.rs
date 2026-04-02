// SPDX-License-Identifier: AGPL-3.0-or-later
//! In-process mock Neural API Unix socket server for unit tests.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// Create a mock Neural API Unix socket server for testing
pub(super) async fn create_mock_neural_api_server(
    socket_path: &std::path::Path,
) -> anyhow::Result<tokio::task::JoinHandle<()>> {
    let _ = std::fs::remove_file(socket_path);
    let listener = tokio::net::UnixListener::bind(socket_path)?;
    let handle = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(handle_neural_api_connection(stream));
        }
    });
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    Ok(handle)
}

async fn handle_neural_api_connection(mut stream: tokio::net::UnixStream) {
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    while reader.read_line(&mut line).await.is_ok() && !line.is_empty() {
        if let Ok(request) = serde_json::from_str::<serde_json::Value>(&line) {
            let method = request["method"].as_str().unwrap_or("");
            let id = request["id"].clone();
            let result = match method {
                "primal.list" => serde_json::json!({
                    "primals": [{"id": "p1", "primal_type": "test", "socket_path": "/tmp/p1.sock",
                        "capabilities": ["viz"], "health": "healthy"}]
                }),
                "neural_api.get_proprioception" => {
                    let now = chrono::Utc::now();
                    serde_json::json!({
                        "timestamp": now.to_rfc3339(),
                        "family_id": "test",
                        "health": {"percentage": 100.0, "status": "healthy"},
                        "confidence": 90.0,
                        "sensory": {"active_sockets": 2, "last_scan": now.to_rfc3339()},
                        "self_awareness": {"knows_about": 1, "can_coordinate": true,
                            "has_security": false, "has_discovery": true, "has_compute": false},
                        "motor": {"can_deploy": false, "can_execute_graphs": true,
                            "can_coordinate_primals": true},
                        "afferent_channels": [],
                        "efferent_channels": []
                    })
                }
                "neural_api.get_metrics" => {
                    serde_json::json!({"cpu_percent": 10, "memory_mb": 128})
                }
                "neural_api.get_topology" => serde_json::json!({
                    "connections": [{"from": "p1", "to": "p2", "connection_type": "trust"}]
                }),
                "neural_api.save_graph" => serde_json::json!({"graph_id": "g-saved-123"}),
                "neural_api.load_graph" => serde_json::json!({"graph": {"nodes": [], "edges": []}}),
                "neural_api.list_graphs" => serde_json::json!({
                    "graphs": [{"id": "g1", "name": "Graph 1", "description": null,
                        "created_at": "2026-01-01", "modified_at": "2026-01-02",
                        "node_count": 2, "edge_count": 1}]
                }),
                "neural_api.execute_graph" => serde_json::json!({"execution_id": "exec-456"}),
                "neural_api.get_execution_status" => serde_json::json!({
                    "execution_id": "exec-456", "graph_id": "g1", "status": "completed",
                    "started_at": "2026-01-01T00:00:00Z", "completed_at": "2026-01-01T00:01:00Z",
                    "error": null, "output": {"result": "ok"}
                }),
                "neural_api.cancel_execution"
                | "neural_api.delete_graph"
                | "neural_api.update_graph_metadata" => serde_json::json!({}),
                _ => serde_json::json!({"error": "Method not found"}),
            };
            let response = serde_json::json!({"jsonrpc": "2.0", "result": result, "id": id});
            let response_str = serde_json::to_string(&response).unwrap() + "\n";
            let _ = writer.write_all(response_str.as_bytes()).await;
            let _ = writer.flush().await;
        }
        line.clear();
    }
}
