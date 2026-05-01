// SPDX-License-Identifier: AGPL-3.0-or-later
//! `proprioception.get` JSON-RPC handler.
//!
//! Returns a synthetic proprioception snapshot usable by composition scripts
//! in all modes (server, live, ui). In server mode frame_rate is 0.0 and
//! window dimensions are absent — compositions use this to detect headless
//! deployments while still receiving a valid JSON structure.

use super::super::RpcHandlers;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse};
use serde_json::json;

/// Handle `proprioception.get`: return a proprioception snapshot.
///
/// Fields consumed by composition scripts (nucleus_composition_lib.sh):
/// - `frame_rate` (f64): 0.0 in server/headless, real FPS when UI is live
/// - `active_scenes` (u64): number of active visualization sessions
/// - `user_interactivity` (str): "none" / "active"
/// - `mode` (str): server / live / ui / headless
/// - `uptime_secs` (u64)
/// - `window` (object|null): null in server mode
#[must_use]
pub fn handle_proprioception_get(
    handlers: &RpcHandlers,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    let uptime = handlers.uptime_seconds();

    let (active_scenes, frame_count) = handlers
        .viz_state
        .read()
        .map(|state| {
            let scenes = state.sessions.len();
            let frames: u64 = state.sessions.values().map(|s| s.frame_count).sum();
            (scenes, frames)
        })
        .unwrap_or((0, 0));

    let has_ui = handlers.rendering_awareness.is_some();

    let mode = if has_ui { "live" } else { "server" };
    let interactivity = if has_ui && active_scenes > 0 {
        "active"
    } else {
        "none"
    };

    let window = if has_ui {
        json!({ "present": true })
    } else {
        json!(null)
    };

    JsonRpcResponse::success(
        request.id,
        json!({
            "frame_rate": if has_ui { 60.0_f64 } else { 0.0 },
            "active_scenes": active_scenes,
            "total_frames": frame_count,
            "user_interactivity": interactivity,
            "mode": mode,
            "uptime_secs": uptime,
            "window": window,
        }),
    )
}
