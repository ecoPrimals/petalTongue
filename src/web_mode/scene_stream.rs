// SPDX-License-Identifier: AGPL-3.0-or-later
//! WebSocket scene streaming handler — G19 WebGL pipeline.
//!
//! Mounts at `/ws/scene` on the Axum router. Browser clients subscribe to
//! visualization sessions and receive server-push `WebGlScene` frames compiled
//! from the scene graph whenever the underlying data changes.
//!
//! # Protocol
//!
//! 1. Client connects to `/ws/scene`
//! 2. Client sends a subscribe message:
//!    ```json
//!    {"action": "subscribe", "session_id": "my-session", "modality": "webgl"}
//!    ```
//!    - `modality`: "webgl" (default) | "svg" | "description"
//! 3. Server pushes frames as JSON messages:
//!    ```json
//!    {"type": "frame", "session_id": "...", "scene": <WebGlScene or SVG string>}
//!    ```
//! 4. Client can unsubscribe or disconnect at any time.

use std::sync::Arc;

use axum::{
    extract::{State, WebSocketUpgrade, ws::Message},
    response::IntoResponse,
};
use tokio::sync::broadcast;

use crate::data_service::DataService;

/// Scene stream shared state.
#[derive(Clone)]
pub struct SceneStreamState {
    #[expect(dead_code, reason = "reserved for G19 scene data queries")]
    pub data_service: Arc<DataService>,
    pub scene_tx: broadcast::Sender<SceneFrame>,
}

/// A compiled scene frame ready for push to browser clients.
#[derive(Clone, Debug)]
pub struct SceneFrame {
    pub session_id: String,
    pub payload: String,
}

/// Axum handler: upgrades the connection to WebSocket for scene streaming.
pub async fn scene_stream_handler(
    ws: WebSocketUpgrade,
    State(state): State<SceneStreamState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_scene_stream(socket, state))
}

async fn handle_scene_stream(
    mut socket: axum::extract::ws::WebSocket,
    state: SceneStreamState,
) {
    let mut subscriptions: Vec<String> = Vec::new();
    let mut rx = state.scene_tx.subscribe();

    loop {
        tokio::select! {
            msg = socket.recv() => {
                let Some(Ok(msg)) = msg else { break };
                match msg {
                    Message::Text(text) => {
                        if let Ok(req) = serde_json::from_str::<SceneStreamRequest>(&text) {
                            match req.action.as_str() {
                                "subscribe" => {
                                    let session_id = req.session_id.unwrap_or_else(|| "default".to_owned());
                                    if !subscriptions.contains(&session_id) {
                                        subscriptions.push(session_id.clone());
                                        // Gossip: announce active scene streaming
                                        let gate = std::env::var("GATE_NAME").unwrap_or_else(|_| "unknown".to_owned());
                                        let sub_count = subscriptions.len();
                                        let sid = session_id.clone();
                                        tokio::spawn(async move {
                                            let entry = petal_tongue_core::gossip_injection::scene_stream_active(&gate, &sid, sub_count);
                                            if let Err(e) = petal_tongue_core::gossip_injection::inject_gossip(&entry).await {
                                                tracing::debug!("gossip inject (scene.stream): {e}");
                                            }
                                        });
                                    }
                                    let ack = serde_json::json!({
                                        "type": "subscribed",
                                        "session_id": session_id,
                                        "modality": req.modality.as_deref().unwrap_or("webgl"),
                                    });
                                    if socket.send(Message::Text(ack.to_string().into())).await.is_err() {
                                        break;
                                    }
                                }
                                "unsubscribe" => {
                                    if let Some(sid) = &req.session_id {
                                        subscriptions.retain(|s| s != sid);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            frame = rx.recv() => {
                let Ok(frame) = frame else { continue };
                if subscriptions.contains(&frame.session_id) || subscriptions.contains(&"*".to_owned()) {
                    let msg = serde_json::json!({
                        "type": "frame",
                        "session_id": frame.session_id,
                        "scene": serde_json::from_str::<serde_json::Value>(&frame.payload).unwrap_or(serde_json::Value::String(frame.payload.clone())),
                    });
                    if socket.send(Message::Text(msg.to_string().into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
}

/// Publish a scene frame to all connected WebSocket clients.
///
/// Called by IPC handlers (e.g., `visualization.render.scene`) to push
/// compiled scenes to browser consumers. No-op if no clients are subscribed.
pub fn publish_scene_frame(tx: &broadcast::Sender<SceneFrame>, session_id: &str, payload: String) {
    let frame = SceneFrame {
        session_id: session_id.to_owned(),
        payload,
    };
    let _ = tx.send(frame);
}

#[derive(serde::Deserialize)]
struct SceneStreamRequest {
    action: String,
    session_id: Option<String>,
    #[serde(default)]
    modality: Option<String>,
}
