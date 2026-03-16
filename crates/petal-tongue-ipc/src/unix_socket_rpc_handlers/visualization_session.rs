// SPDX-License-Identifier: AGPL-3.0-or-later
//! Session-related visualization RPC handlers: status, list, dismiss.

use super::RpcHandlers;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use crate::visualization_handler::{DismissRequest, SessionStatusRequest};
use serde_json::Value;

/// Handle visualization.session.status: return session health metrics
pub fn handle_session_status(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<SessionStatusRequest>(req.params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                format!("Invalid params: {e}"),
            );
        }
    };
    let response = handlers
        .viz_state
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .handle_session_status(&params);
    let value = match serde_json::to_value(&response) {
        Ok(v) => v,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INTERNAL_ERROR,
                format!("Serialization failed: {e}"),
            );
        }
    };
    JsonRpcResponse::success(req.id, value)
}

/// Handle visualization.session.list: return active session metadata.
pub fn handle_session_list(handlers: &RpcHandlers, id: Value) -> JsonRpcResponse {
    let (sessions, scene_count) = {
        let state = handlers
            .viz_state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let sessions: Vec<Value> = state
            .sessions
            .iter()
            .map(|(session_id, session)| {
                serde_json::json!({
                    "session_id": session_id,
                    "title": session.title,
                    "domain": session.domain,
                    "binding_count": session.bindings.len(),
                    "frame_count": session.frame_count,
                })
            })
            .collect();
        let scene_count = state.grammar_scenes.len();
        drop(state);
        (sessions, scene_count)
    };

    JsonRpcResponse::success(
        id,
        serde_json::json!({
            "sessions": sessions,
            "scene_count": scene_count,
        }),
    )
}

/// Handle visualization.dismiss: remove a session
pub fn handle_dismiss(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<DismissRequest>(req.params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                format!("Invalid params: {e}"),
            );
        }
    };
    let response = handlers
        .viz_state
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .handle_dismiss(params);
    let value = match serde_json::to_value(&response) {
        Ok(v) => v,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INTERNAL_ERROR,
                format!("Serialization failed: {e}"),
            );
        }
    };
    JsonRpcResponse::success(req.id, value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_rpc::JsonRpcRequest;
    use crate::unix_socket_rpc_handlers::RpcHandlers;
    use petal_tongue_core::graph_engine::GraphEngine;
    use serde_json::json;
    use std::sync::{Arc, RwLock};

    fn test_handlers() -> RpcHandlers {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let viz_state = Arc::new(RwLock::new(
            crate::visualization_handler::VisualizationState::new(),
        ));
        let mut h = RpcHandlers::new(graph, String::new(), viz_state);
        h.interaction_subscribers = Arc::new(RwLock::new(
            crate::visualization_handler::InteractionSubscriberRegistry::new(),
        ));
        h
    }

    #[test]
    fn handle_session_list_empty() {
        let h = test_handlers();
        let resp = handle_session_list(&h, json!(1));
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert!(r["sessions"].as_array().expect("arr").is_empty());
        assert_eq!(r["scene_count"], 0);
    }

    #[test]
    fn handle_dismiss_valid_params_succeeds() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "visualization.dismiss",
            json!({"session_id": "s1"}),
            json!(1),
        );
        let resp = handle_dismiss(&h, req);
        assert!(resp.error.is_none());
    }

    #[test]
    fn handle_session_status_invalid_params_returns_error() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("visualization.session.status", json!({}), json!(1));
        let resp = handle_session_status(&h, req);
        assert!(resp.error.is_some());
    }

    #[test]
    fn handle_session_status_valid_params_succeeds() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "visualization.session.status",
            json!({"session_id": "s1"}),
            json!(1),
        );
        let resp = handle_session_status(&h, req);
        assert!(resp.error.is_none());
        let r = resp.result.expect("result");
        assert_eq!(r["session_id"], "s1");
    }
}
