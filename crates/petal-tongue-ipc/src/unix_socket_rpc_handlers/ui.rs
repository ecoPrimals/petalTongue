// SPDX-License-Identifier: AGPL-3.0-only
//! Handlers for ui.render and ui.display_status JSON-RPC methods.

use super::RpcHandlers;
use super::graph;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use serde_json::json;

/// Handle ui.render: render content by type (graph, etc.)
pub async fn handle_ui_render(handlers: &RpcHandlers, request: JsonRpcRequest) -> JsonRpcResponse {
    let Some(params) = request.params.as_object() else {
        return JsonRpcResponse::error(
            request.id,
            error_codes::INVALID_PARAMS,
            "params must be an object",
        );
    };

    let content_type = params
        .get("content_type")
        .and_then(|v| v.as_str())
        .unwrap_or("graph");

    let data = params.get("data").cloned().unwrap_or(json!({}));

    match content_type {
        "graph" => {
            let result = graph::render_graph_data(handlers, data).await;

            match result {
                Ok(()) => JsonRpcResponse::success(
                    request.id,
                    json!({
                        "rendered": true,
                        "modality": "visual",
                        "window_id": "main"
                    }),
                ),
                Err(e) => JsonRpcResponse::error(
                    request.id,
                    error_codes::INTERNAL_ERROR,
                    format!("Render error: {e}"),
                ),
            }
        }
        _ => JsonRpcResponse::error(
            request.id,
            error_codes::INVALID_PARAMS,
            format!("Unsupported content_type: {content_type}"),
        ),
    }
}

/// Handle ui.display_status: update status for a primal
pub fn handle_ui_display_status(
    _handlers: &RpcHandlers,
    request: JsonRpcRequest,
) -> JsonRpcResponse {
    let Some(params) = request.params.as_object() else {
        return JsonRpcResponse::error(
            request.id,
            error_codes::INVALID_PARAMS,
            "params must be an object",
        );
    };

    let primal_name = params
        .get("primal_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    let status = params.get("status").cloned().unwrap_or(json!({}));

    tracing::debug!("Status update for {}: {:?}", primal_name, status);

    JsonRpcResponse::success(
        request.id,
        json!({
            "updated": true,
            "primal": primal_name,
        }),
    )
}
