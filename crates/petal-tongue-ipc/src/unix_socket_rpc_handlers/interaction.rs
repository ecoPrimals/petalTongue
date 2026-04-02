// SPDX-License-Identifier: AGPL-3.0-or-later
//! Handlers for interaction.subscribe, interaction.poll, interaction.unsubscribe.
//!
//! Manages poll-based IPC subscriptions for interaction events.

use super::RpcHandlers;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};

/// Handle interaction.subscribe: register a subscriber for interaction events
pub fn handle_subscribe(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let subscriber_id = req.params["subscriber_id"].as_str().unwrap_or("");
    if subscriber_id.is_empty() {
        return JsonRpcResponse::error(
            req.id,
            error_codes::INVALID_PARAMS,
            "subscriber_id is required",
        );
    }

    let event_filter: Vec<String> = req.params["events"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let callback_method = req.params["callback_method"].as_str().map(String::from);
    let grammar_id = req.params["grammar_id"].as_str().map(String::from);
    let callback_socket = req.params["callback_socket"].as_str().map(String::from);

    let is_new = handlers
        .interaction_subscribers
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .subscribe_with_filter(
            subscriber_id,
            event_filter,
            callback_method.clone(),
            grammar_id,
            callback_socket.clone(),
        );

    JsonRpcResponse::success(
        req.id,
        serde_json::json!({
            "subscribed": true,
            "subscriber_id": subscriber_id,
            "is_new": is_new,
            "callback_method": callback_method,
            "push_delivery": callback_socket.is_some() && callback_method.is_some(),
        }),
    )
}

/// Handle interaction.poll: retrieve pending events for a subscriber
pub fn handle_poll(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let subscriber_id = req.params["subscriber_id"].as_str().unwrap_or("");
    if subscriber_id.is_empty() {
        return JsonRpcResponse::error(
            req.id,
            error_codes::INVALID_PARAMS,
            "subscriber_id is required",
        );
    }
    let events = handlers
        .interaction_subscribers
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .poll(subscriber_id);
    JsonRpcResponse::success(
        req.id,
        serde_json::json!({
            "subscriber_id": subscriber_id,
            "events": events,
        }),
    )
}

/// Handle interaction.unsubscribe: remove a subscriber
pub fn handle_unsubscribe(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let subscriber_id = req.params["subscriber_id"].as_str().unwrap_or("");
    if subscriber_id.is_empty() {
        return JsonRpcResponse::error(
            req.id,
            error_codes::INVALID_PARAMS,
            "subscriber_id is required",
        );
    }
    let was_subscribed = handlers
        .interaction_subscribers
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .unsubscribe(subscriber_id);
    JsonRpcResponse::success(
        req.id,
        serde_json::json!({
            "unsubscribed": was_subscribed,
            "subscriber_id": subscriber_id,
        }),
    )
}
