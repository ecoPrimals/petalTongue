// SPDX-License-Identifier: AGPL-3.0-or-later
//! Handlers for visualization render, stream, grammar, dashboard, validate, and export RPC methods.
//!
//! Each method deserializes typed request params, delegates to `VisualizationState`, and
//! serializes the handler response back into JSON-RPC.

use super::RpcHandlers;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use crate::visualization_handler::{
    DashboardRenderRequest, ExportRequest, GrammarRenderRequest, StreamUpdateRequest, UiConfig,
    ValidateRequest, VisualizationRenderRequest,
};

/// Handle visualization.render: create or replace a visualization session.
///
/// Accepts either the canonical `VisualizationRenderRequest` format or any
/// spring-native format recognized by `SpringDataAdapter` (ludoSpring game
/// channels, ecoPrimals/time-series/v1, bare `DataBinding` arrays).
pub fn handle_render(handlers: &RpcHandlers, mut req: JsonRpcRequest) -> JsonRpcResponse {
    let has_bindings = req.params.get("bindings").is_some();
    let params = if has_bindings {
        match serde_json::from_value::<VisualizationRenderRequest>(req.params) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    req.id,
                    error_codes::INVALID_PARAMS,
                    format!("Invalid params: {e}"),
                );
            }
        }
    } else {
        let session_id = req
            .params
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("spring-session")
            .to_string();
        let title = req
            .params
            .get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Spring Data")
            .to_string();
        let domain = req
            .params
            .get("domain")
            .and_then(|v| v.as_str())
            .map(String::from);
        let ui_config = req
            .params
            .as_object_mut()
            .and_then(|m| m.remove("ui_config"))
            .and_then(|v| serde_json::from_value::<UiConfig>(v).ok());
        let thresholds = req
            .params
            .as_object_mut()
            .and_then(|m| m.remove("thresholds"))
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();

        match petal_tongue_core::spring_adapter::SpringDataAdapter::adapt(req.params) {
            Ok(bindings) if !bindings.is_empty() => VisualizationRenderRequest {
                session_id,
                title,
                bindings,
                thresholds,
                domain,
                ui_config,
            },
            _ => {
                return JsonRpcResponse::error(
                    req.id,
                    error_codes::INVALID_PARAMS,
                    "Invalid params: unrecognized spring format".to_owned(),
                );
            }
        }
    };
    let response = handlers
        .viz_state
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .handle_render(params);
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

/// Handle visualization.render.stream: incremental update to a binding
pub fn handle_stream(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<StreamUpdateRequest>(req.params) {
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
        .handle_stream_update(params);
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

/// Handle visualization.render.grammar: compile grammar expression through scene engine
pub fn handle_grammar_render(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<GrammarRenderRequest>(req.params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                format!("Invalid grammar params: {e}"),
            );
        }
    };
    let response = handlers
        .viz_state
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .handle_grammar_render(params);
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

/// Handle visualization.render.dashboard: compile bindings into a multi-panel dashboard
pub fn handle_dashboard_render(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<DashboardRenderRequest>(req.params) {
        Ok(p) => p,
        Err(e) => {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                format!("Invalid dashboard params: {e}"),
            );
        }
    };
    let response = handlers
        .viz_state
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .handle_dashboard_render(params);
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

/// Handle visualization.validate: validate grammar + data against Tufte constraints
pub fn handle_validate(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<ValidateRequest>(req.params) {
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
        .handle_validate(&params);
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

/// Handle visualization.export: export a session to the requested format
pub fn handle_export(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let params = match serde_json::from_value::<ExportRequest>(req.params) {
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
        .handle_export(params);
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
