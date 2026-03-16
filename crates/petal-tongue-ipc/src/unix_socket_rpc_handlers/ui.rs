// SPDX-License-Identifier: AGPL-3.0-or-later
//! Handlers for ui.render and `ui.display_status` JSON-RPC methods.

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

    let data = params.get("data").cloned().unwrap_or_else(|| json!({}));

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

/// Handle `ui.display_status`: update status for a primal
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

    let status = params.get("status").cloned().unwrap_or_else(|| json!({}));

    tracing::debug!("Status update for {}: {:?}", primal_name, status);

    JsonRpcResponse::success(
        request.id,
        json!({
            "updated": true,
            "primal": primal_name,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unix_socket_rpc_handlers::RpcHandlers;
    use crate::visualization_handler::VisualizationState;
    use petal_tongue_core::graph_engine::GraphEngine;
    use serde_json::json;
    use std::sync::{Arc, RwLock};

    fn test_handlers() -> RpcHandlers {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let viz_state = Arc::new(RwLock::new(VisualizationState::new()));
        RpcHandlers::new(graph, "test".to_string(), viz_state)
    }

    #[tokio::test]
    async fn render_params_not_object_returns_error() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("ui.render", json!([]), json!(1));
        let resp = handle_ui_render(&h, req).await;
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("err").code,
            error_codes::INVALID_PARAMS
        );
    }

    #[tokio::test]
    async fn render_graph_content_type_succeeds() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "ui.render",
            json!({"content_type": "graph", "data": {}}),
            json!(1),
        );
        let resp = handle_ui_render(&h, req).await;
        assert!(resp.error.is_none());
        let r = resp.result.unwrap();
        assert_eq!(r["rendered"], true);
        assert_eq!(r["modality"], "visual");
    }

    #[tokio::test]
    async fn render_unsupported_content_type_returns_error() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("ui.render", json!({"content_type": "unknown"}), json!(1));
        let resp = handle_ui_render(&h, req).await;
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("err").code,
            error_codes::INVALID_PARAMS
        );
    }

    #[tokio::test]
    async fn render_default_content_type_is_graph() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("ui.render", json!({}), json!(1));
        let resp = handle_ui_render(&h, req).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["rendered"], true);
    }

    #[test]
    fn display_status_params_not_object_returns_error() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("ui.display_status", json!([]), json!(1));
        let resp = handle_ui_display_status(&h, req);
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("err").code,
            error_codes::INVALID_PARAMS
        );
    }

    #[test]
    fn display_status_with_primal_name() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "ui.display_status",
            json!({"primal_name": "spring-1", "status": {"health": "ok"}}),
            json!(1),
        );
        let resp = handle_ui_display_status(&h, req);
        assert!(resp.error.is_none());
        let r = resp.result.unwrap();
        assert_eq!(r["updated"], true);
        assert_eq!(r["primal"], "spring-1");
    }

    #[test]
    fn display_status_default_primal_unknown() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("ui.display_status", json!({}), json!(1));
        let resp = handle_ui_display_status(&h, req);
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["primal"], "unknown");
    }
}
