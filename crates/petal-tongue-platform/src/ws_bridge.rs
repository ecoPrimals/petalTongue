// SPDX-License-Identifier: AGPL-3.0-or-later
//! WebSocket bridge for embedded platform runtime.
//!
//! Exposes the [`EmbeddedRuntime`] JSON-RPC interface over WebSocket so that
//! footPrint compositions and other network clients can communicate with an
//! embedded petalTongue instance without going through C-FFI.
//!
//! # Architecture
//!
//! ```text
//! footPrint / browser ──► WebSocket ──► ws_bridge ──► EmbeddedRuntime.ipc_request()
//! ```
//!
//! The bridge listens on a configurable TCP port (default: env `PETALTONGUE_WS_PORT`
//! or 8765) and relays JSON-RPC messages bidirectionally.

use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::runtime::EmbeddedRuntime;

/// Default WebSocket port for the embedded bridge.
const DEFAULT_WS_PORT: u16 = 8765;

/// Configuration for the WebSocket bridge.
#[derive(Debug, Clone)]
pub struct WsBridgeConfig {
    /// Bind address (default: `127.0.0.1`).
    pub bind_host: String,
    /// Port to listen on (default: 8765, or `PETALTONGUE_WS_PORT` env var).
    pub port: u16,
}

impl Default for WsBridgeConfig {
    fn default() -> Self {
        let port = std::env::var("PETALTONGUE_WS_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_WS_PORT);

        let bind_host = std::env::var("PETALTONGUE_WS_BIND_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_owned());

        Self { bind_host, port }
    }
}

impl WsBridgeConfig {
    /// Socket address to bind to.
    fn addr(&self) -> String {
        format!("{}:{}", self.bind_host, self.port)
    }
}

/// Handle returned when the bridge is spawned — allows graceful shutdown.
pub struct WsBridgeHandle {
    shutdown_tx: tokio::sync::watch::Sender<bool>,
    addr: SocketAddr,
}

impl WsBridgeHandle {
    /// The local address the bridge is listening on.
    #[must_use]
    pub const fn local_addr(&self) -> SocketAddr {
        self.addr
    }

    /// Signal the bridge to shut down.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

/// Spawn the WebSocket bridge on the runtime's tokio executor.
///
/// Returns a handle for querying the bound address and triggering shutdown.
///
/// # Errors
/// Returns error if the TCP listener cannot bind to the configured address.
pub async fn spawn_ws_bridge(
    runtime: Arc<RwLock<EmbeddedRuntime>>,
    config: WsBridgeConfig,
) -> Result<WsBridgeHandle, std::io::Error> {
    let listener = TcpListener::bind(config.addr()).await?;
    let addr = listener.local_addr()?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    info!(%addr, "WebSocket bridge listening");

    tokio::spawn(accept_loop(listener, runtime, shutdown_rx));

    Ok(WsBridgeHandle { shutdown_tx, addr })
}

async fn accept_loop(
    listener: TcpListener,
    runtime: Arc<RwLock<EmbeddedRuntime>>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, peer)) => {
                        info!(%peer, "WebSocket client connected");
                        let rt = Arc::clone(&runtime);
                        tokio::spawn(handle_connection(stream, rt, peer));
                    }
                    Err(e) => {
                        error!("WebSocket accept error: {e}");
                    }
                }
            }
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    info!("WebSocket bridge shutting down");
                    return;
                }
            }
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    runtime: Arc<RwLock<EmbeddedRuntime>>,
    peer: SocketAddr,
) {
    let ws_stream = match tokio_tungstenite::accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            warn!(%peer, "WebSocket handshake failed: {e}");
            return;
        }
    };

    let (mut sink, mut stream) = ws_stream.split();

    while let Some(msg) = stream.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                warn!(%peer, "WebSocket read error: {e}");
                break;
            }
        };

        if msg.is_close() {
            info!(%peer, "WebSocket client disconnected");
            break;
        }

        if !msg.is_text() {
            continue;
        }

        let request_text = msg.into_text().unwrap_or_default();
        let response = {
            let mut rt = runtime.write().await;
            match rt.ipc_request(&request_text) {
                Ok(r) => r,
                Err(e) => {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": { "code": -32603, "message": e.to_string() }
                    })
                    .to_string()
                }
            }
        };

        if let Err(e) = sink
            .send(tokio_tungstenite::tungstenite::Message::Text(response))
            .await
        {
            warn!(%peer, "WebSocket write error: {e}");
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{EmbedConfig, Platform};
    use futures_util::{SinkExt, StreamExt};

    fn create_test_runtime() -> Arc<RwLock<EmbeddedRuntime>> {
        let config = EmbedConfig::new(Platform::Desktop);
        let mut rt = EmbeddedRuntime::new(config).expect("runtime should create");
        rt.start().expect("runtime should start");
        Arc::new(RwLock::new(rt))
    }

    #[tokio::test]
    async fn ws_bridge_health_check_e2e() {
        let runtime = create_test_runtime();
        let config = WsBridgeConfig {
            bind_host: "127.0.0.1".to_owned(),
            port: 0, // OS assigns a free port
        };

        let handle = spawn_ws_bridge(Arc::clone(&runtime), config)
            .await
            .expect("bridge should bind");

        let url = format!("ws://{}", handle.local_addr());
        let (mut ws, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("WS connect should succeed");

        let req = r#"{"jsonrpc":"2.0","id":1,"method":"health.check","params":{}}"#;
        ws.send(tokio_tungstenite::tungstenite::Message::Text(req.to_owned()))
            .await
            .expect("send should succeed");

        let resp = ws.next().await.expect("should get response").expect("no error");
        let v: serde_json::Value = serde_json::from_str(resp.to_text().expect("text frame"))
            .expect("valid JSON");

        assert_eq!(v["id"], 1);
        assert_eq!(v["result"]["status"], "ok");

        handle.shutdown();
    }

    #[tokio::test]
    async fn ws_bridge_capabilities_e2e() {
        let runtime = create_test_runtime();
        let config = WsBridgeConfig {
            bind_host: "127.0.0.1".to_owned(),
            port: 0,
        };

        let handle = spawn_ws_bridge(Arc::clone(&runtime), config)
            .await
            .expect("bridge should bind");

        let url = format!("ws://{}", handle.local_addr());
        let (mut ws, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("WS connect");

        let req = r#"{"jsonrpc":"2.0","id":2,"method":"capabilities.list","params":{}}"#;
        ws.send(tokio_tungstenite::tungstenite::Message::Text(req.to_owned()))
            .await
            .expect("send");

        let resp = ws.next().await.expect("response").expect("no error");
        let v: serde_json::Value = serde_json::from_str(resp.to_text().expect("text"))
            .expect("valid JSON");

        assert_eq!(v["id"], 2);
        let caps = v["result"]["capabilities"].as_array().expect("array");
        assert!(caps.iter().any(|c| c == "health.check"));
        assert!(caps.iter().any(|c| c == "pt.render_svg"));

        handle.shutdown();
    }

    #[tokio::test]
    async fn ws_bridge_unknown_method_e2e() {
        let runtime = create_test_runtime();
        let config = WsBridgeConfig {
            bind_host: "127.0.0.1".to_owned(),
            port: 0,
        };

        let handle = spawn_ws_bridge(Arc::clone(&runtime), config)
            .await
            .expect("bridge should bind");

        let url = format!("ws://{}", handle.local_addr());
        let (mut ws, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("WS connect");

        let req = r#"{"jsonrpc":"2.0","id":99,"method":"bogus.method","params":{}}"#;
        ws.send(tokio_tungstenite::tungstenite::Message::Text(req.to_owned()))
            .await
            .expect("send");

        let resp = ws.next().await.expect("response").expect("no error");
        let v: serde_json::Value = serde_json::from_str(resp.to_text().expect("text"))
            .expect("valid JSON");

        assert_eq!(v["id"], 99);
        assert_eq!(v["error"]["code"], -32601);

        handle.shutdown();
    }

    #[tokio::test]
    async fn ws_bridge_metrics_e2e() {
        let runtime = create_test_runtime();
        let config = WsBridgeConfig {
            bind_host: "127.0.0.1".to_owned(),
            port: 0,
        };

        let handle = spawn_ws_bridge(Arc::clone(&runtime), config)
            .await
            .expect("bridge should bind");

        let url = format!("ws://{}", handle.local_addr());
        let (mut ws, _) = tokio_tungstenite::connect_async(&url)
            .await
            .expect("WS connect");

        let req = r#"{"jsonrpc":"2.0","id":7,"method":"pt.metrics","params":{}}"#;
        ws.send(tokio_tungstenite::tungstenite::Message::Text(req.to_owned()))
            .await
            .expect("send");

        let resp = ws.next().await.expect("response").expect("no error");
        let v: serde_json::Value = serde_json::from_str(resp.to_text().expect("text"))
            .expect("valid JSON");

        assert_eq!(v["id"], 7);
        assert!(v["result"]["cpu_count"].as_u64().unwrap_or(0) >= 1);
        assert!(v["result"]["source"].is_string());
        assert!(v["result"]["memory_percent"].is_number());

        handle.shutdown();
    }
}
