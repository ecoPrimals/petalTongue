// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC request handlers for petalTongue IPC.
//!
//! Dispatches to domain-specific submodules:
//! - `system` — health.*, identity.get, lifecycle.status, capabilities.list, topology.get
//! - `visualization` — visualization.render, stream, grammar, validate, export, dismiss, interact
//! - `interaction` — interaction.subscribe, poll, unsubscribe
//! - `motor` — motor.* (UI efferent bridge)
//! - `ui` — ui.render, ui.display_status
//! - `graph` — visualization.render.graph

mod audio;
mod dispatch;
mod graph;
mod interaction;
mod motor;
mod system;
mod ui;
mod visualization;
mod visualization_session;

use crate::visualization_handler::VisualizationState;
use petal_tongue_core::RenderingAwareness;
use petal_tongue_core::graph_engine::GraphEngine;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};

/// JSON-RPC request handlers for petalTongue IPC
pub struct RpcHandlers {
    /// Shared graph engine state
    pub graph: Arc<RwLock<GraphEngine>>,
    /// Family ID for this instance
    pub family_id: String,
    /// Server start time for uptime
    pub start_time: SystemTime,
    /// Shared visualization state from springs/primals
    pub viz_state: Arc<RwLock<VisualizationState>>,
    /// Motor command sender for UI-controlling IPC commands (efferent bridge)
    pub motor_tx: Option<std::sync::mpsc::Sender<petal_tongue_core::MotorCommand>>,
    /// Interaction event subscriber registry (poll-based IPC subscriptions)
    pub interaction_subscribers:
        Arc<RwLock<crate::visualization_handler::InteractionSubscriberRegistry>>,
    /// Sensor event stream subscriber registry
    pub sensor_stream_subscribers: Arc<RwLock<crate::visualization_handler::SensorStreamRegistry>>,
    /// Rendering awareness (content-level introspection for IPC queries)
    pub rendering_awareness: Option<Arc<RwLock<RenderingAwareness>>>,
    /// Whether TCP JSON-RPC is active (for dynamic `capabilities.list` transport)
    pub tcp_enabled: bool,
    /// Push delivery channel for callback dispatches (PT-06).
    /// When set, `handle_interact_apply` sends dispatches here instead of dropping them.
    pub callback_tx:
        Option<tokio::sync::mpsc::UnboundedSender<crate::visualization_handler::CallbackDispatch>>,
}

impl RpcHandlers {
    /// Create new RPC handlers with graph and visualization state
    pub fn new(
        graph: Arc<RwLock<GraphEngine>>,
        family_id: String,
        viz_state: Arc<RwLock<VisualizationState>>,
    ) -> Self {
        Self {
            graph,
            family_id,
            start_time: SystemTime::now(),
            viz_state,
            motor_tx: None,
            interaction_subscribers: Arc::new(RwLock::new(
                crate::visualization_handler::InteractionSubscriberRegistry::new(),
            )),
            sensor_stream_subscribers: Arc::new(RwLock::new(
                crate::visualization_handler::SensorStreamRegistry::new(),
            )),
            rendering_awareness: None,
            tcp_enabled: false,
            callback_tx: None,
        }
    }

    fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().unwrap_or_default().as_secs()
    }

    // --- Delegation methods for tests and external callers ---

    /// Handle health.check: return status, version, uptime, and modalities
    #[must_use]
    pub fn handle_health_check(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        system::handle_health_check(self, request)
    }

    /// Handle capability.announce: return detected capabilities
    #[must_use]
    pub fn handle_announce_capabilities(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        system::handle_announce_capabilities(self, request)
    }

    /// Handle ui.render: render content by type (graph, etc.)
    pub async fn handle_ui_render(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        ui::handle_ui_render(self, request).await
    }

    /// Handle `ui.display_status`: update status for a primal
    #[must_use]
    pub fn handle_ui_display_status(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        ui::handle_ui_display_status(self, request)
    }

    /// Handle capability.list: return supported capabilities and protocol info
    #[must_use]
    pub fn get_capabilities(&self, id: serde_json::Value) -> JsonRpcResponse {
        system::get_capabilities(self, id)
    }

    /// Handle health.get: return health status and graph stats
    #[must_use]
    pub fn get_health(&self, id: serde_json::Value) -> JsonRpcResponse {
        system::get_health(self, id)
    }

    /// Handle topology.get: return graph nodes and edges
    #[must_use]
    pub fn get_topology(&self, id: serde_json::Value) -> JsonRpcResponse {
        system::get_topology(self, id)
    }

    /// Handle `visualization.render.graph`: render graph to specified format (svg, png, terminal)
    pub async fn render_graph(
        &self,
        params: serde_json::Value,
        id: serde_json::Value,
    ) -> JsonRpcResponse {
        graph::render_graph(self, params, id).await
    }

    fn handle_interaction_subscribe(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        interaction::handle_subscribe(self, req)
    }

    fn handle_interaction_poll(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        interaction::handle_poll(self, req)
    }

    fn handle_interaction_unsubscribe(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        interaction::handle_unsubscribe(self, req)
    }

    fn handle_sensor_stream_subscribe(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let subscription_id = self
            .sensor_stream_subscribers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .subscribe();
        JsonRpcResponse::success(
            req.id,
            serde_json::json!({
                "subscription_id": subscription_id,
            }),
        )
    }

    fn handle_sensor_stream_unsubscribe(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let subscription_id = req.params["subscription_id"].as_str().unwrap_or("");
        if subscription_id.is_empty() {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                "subscription_id is required",
            );
        }
        let was_subscribed = self
            .sensor_stream_subscribers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .unsubscribe(subscription_id);
        JsonRpcResponse::success(
            req.id,
            serde_json::json!({
                "unsubscribed": was_subscribed,
                "subscription_id": subscription_id,
            }),
        )
    }

    fn handle_sensor_stream_poll(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let subscription_id = req.params["subscription_id"].as_str().unwrap_or("");
        if subscription_id.is_empty() {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                "subscription_id is required",
            );
        }
        let batch = self
            .sensor_stream_subscribers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .poll(subscription_id);
        let value = match serde_json::to_value(&batch) {
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
}

#[cfg(test)]
mod tests;
