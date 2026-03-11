// SPDX-License-Identifier: AGPL-3.0-only
//! Handlers for `visualization.render_graph` and graph rendering.

use super::RpcHandlers;
use crate::json_rpc::{JsonRpcResponse, error_codes};
use serde_json::{Value, json};

/// Internal: render graph data (used by ui.render for `content_type` "graph")
#[expect(
    clippy::unused_async,
    reason = "async trait requirement for RPC handler"
)]
pub async fn render_graph_data(_handlers: &RpcHandlers, data: Value) -> anyhow::Result<()> {
    tracing::debug!("Rendering graph data: {:?}", data);
    Ok(())
}

/// Handle `visualization.render_graph`: render graph to specified format (svg, png, terminal)
///
/// **Fallback**: When graph data is unavailable or rendering is not implemented,
/// returns empty/placeholder with clear logging. Callers should handle empty responses.
#[expect(
    clippy::unused_async,
    reason = "async trait requirement for RPC handler"
)]
pub async fn render_graph(_handlers: &RpcHandlers, params: Value, id: Value) -> JsonRpcResponse {
    let format = params["format"].as_str().unwrap_or("svg");

    // Placeholder fallback: real rendering delegated to scene engine / visualization primals.
    // Log so callers know this is not live data.
    tracing::debug!(
        "render_graph fallback: format={} (actual rendering delegated to visualization primals)",
        format
    );

    match format {
        "svg" => JsonRpcResponse::success(
            id,
            json!({
                "format": "svg",
                "data": "<svg><!-- placeholder: rendering delegated to visualization primals --></svg>",
                "metadata": {
                    "nodes": 0,
                    "edges": 0
                }
            }),
        ),
        "png" => JsonRpcResponse::success(
            id,
            json!({
                "format": "png",
                "data": "",
                "metadata": {
                    "nodes": 0,
                    "edges": 0
                }
            }),
        ),
        "terminal" => JsonRpcResponse::success(
            id,
            json!({
                "format": "terminal",
                "data": "(placeholder: rendering delegated to visualization primals)"
            }),
        ),
        _ => JsonRpcResponse::error(
            id,
            error_codes::INVALID_PARAMS,
            format!("Unsupported format: {format}"),
        ),
    }
}
