// SPDX-License-Identifier: AGPL-3.0-or-later
//! WebSocket JSON-RPC handler for the web server.
//!
//! Mounts at `/ws` on the Axum router. Accepts WebSocket upgrade requests and
//! delegates JSON-RPC messages to [`EmbeddedRuntime::ipc_request()`].
//!
//! This enables footPrint and other composition consumers to reach petalTongue's
//! rendering + metrics capabilities through the same HTTP port (8080) that serves
//! TOPO-VIS, without requiring a separate WS bridge port.

use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade, ws::Message},
    response::IntoResponse,
};
use tokio::sync::RwLock;

use petal_tongue_platform::EmbeddedRuntime;

/// Shared state for the WebSocket handler.
pub type WsState = Arc<RwLock<EmbeddedRuntime>>;

/// Axum handler: upgrades the connection to WebSocket and processes JSON-RPC.
pub async fn ws_upgrade_handler(
    ws: WebSocketUpgrade,
    State(runtime): State<WsState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_connection(socket, runtime))
}

async fn handle_ws_connection(mut socket: axum::extract::ws::WebSocket, runtime: WsState) {
    while let Some(msg) = socket.recv().await {
        let Ok(msg) = msg else { break };

        let request_text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            _ => continue,
        };

        let response = {
            let mut rt = runtime.write().await;
            match rt.ipc_request(&request_text) {
                Ok(resp) => resp,
                Err(e) => {
                    let err_json = serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": { "code": -32603, "message": e.to_string() }
                    });
                    err_json.to_string()
                }
            }
        };

        if socket.send(Message::Text(response)).await.is_err() {
            break;
        }
    }
}
