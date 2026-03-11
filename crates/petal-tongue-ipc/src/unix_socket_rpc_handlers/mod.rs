// SPDX-License-Identifier: AGPL-3.0-only
//! JSON-RPC request handlers for petalTongue IPC.
//!
//! Dispatches to domain-specific submodules:
//! - `visualization` — visualization.render, stream, grammar, validate, export, dismiss, interact
//! - `interaction` — interaction.subscribe, poll, unsubscribe
//! - `motor` — motor.* (UI efferent bridge)
//! - `system` — health.check, capability.*, topology.get
//! - `ui` — ui.render, ui.display_status
//! - `graph` — visualization.render_graph

mod graph;
mod interaction;
mod motor;
mod system;
mod ui;
mod visualization;

use crate::visualization_handler::VisualizationState;
use petal_tongue_core::RenderingAwareness;
use petal_tongue_core::graph_engine::GraphEngine;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tracing::warn;

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
        }
    }

    fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().unwrap_or_default().as_secs()
    }

    /// Dispatch JSON-RPC request to the appropriate handler
    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let method = req.method.as_str();
        match method {
            "health.check" => system::handle_health_check(self, req),
            "capability.announce" => system::handle_announce_capabilities(self, req),
            "ui.render" => ui::handle_ui_render(self, req).await,
            "ui.display_status" => ui::handle_ui_display_status(self, req),
            "capability.list" => system::get_capabilities(self, req.id),
            "visualization.render_graph" => graph::render_graph(self, req.params, req.id).await,
            "health.get" => system::get_health(self, req.id),
            "topology.get" => system::get_topology(self, req.id),
            "visualization.render" => visualization::handle_render(self, req),
            "visualization.render.stream" => visualization::handle_stream(self, req),
            "visualization.render.grammar" => visualization::handle_grammar_render(self, req),
            "visualization.render.dashboard" => visualization::handle_dashboard_render(self, req),
            "visualization.validate" => visualization::handle_validate(self, req),
            "visualization.export" => visualization::handle_export(self, req),
            "visualization.dismiss" => visualization::handle_dismiss(self, req),
            "visualization.interact.apply" => visualization::handle_interact_apply(self, req),
            "visualization.interact.perspectives" => {
                visualization::handle_interact_perspectives(self, req.id)
            }
            "visualization.capabilities" => visualization::handle_capabilities(self, req.id),
            "visualization.introspect" => visualization::handle_introspect(self, req.id),
            "visualization.panels" => visualization::handle_panels(self, req.id),
            "visualization.showing" => visualization::handle_showing(self, req),
            "interaction.subscribe" | "visualization.interact.subscribe" => {
                self.handle_interaction_subscribe(req)
            }
            "interaction.poll" | "visualization.interact.poll" => self.handle_interaction_poll(req),
            "interaction.unsubscribe" | "visualization.interact.unsubscribe" => {
                self.handle_interaction_unsubscribe(req)
            }
            "interaction.sensor_stream.subscribe" => self.handle_sensor_stream_subscribe(req),
            "interaction.sensor_stream.unsubscribe" => self.handle_sensor_stream_unsubscribe(req),
            "interaction.sensor_stream.poll" => self.handle_sensor_stream_poll(req),
            "motor.set_panel" | "motor.set_zoom" | "motor.fit_to_view" | "motor.set_mode"
            | "motor.navigate" => motor::handle_motor_command(self, req),
            _ => {
                warn!("Unknown method: {}", method);
                JsonRpcResponse::error(
                    req.id,
                    error_codes::METHOD_NOT_FOUND,
                    format!("Method not found: {method}"),
                )
            }
        }
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

    /// Handle ui.display_status: update status for a primal
    pub fn handle_ui_display_status(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        ui::handle_ui_display_status(self, request)
    }

    /// Handle capability.list: return supported capabilities and protocol info
    #[must_use]
    pub fn get_capabilities(&self, id: serde_json::Value) -> JsonRpcResponse {
        system::get_capabilities(self, id)
    }

    /// Handle health.get: return health status and graph stats
    pub fn get_health(&self, id: serde_json::Value) -> JsonRpcResponse {
        system::get_health(self, id)
    }

    /// Handle topology.get: return graph nodes and edges
    pub fn get_topology(&self, id: serde_json::Value) -> JsonRpcResponse {
        system::get_topology(self, id)
    }

    /// Handle visualization.render_graph: render graph to specified format (svg, png, terminal)
    #[allow(clippy::unused_async)]
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
        let mut reg = self
            .sensor_stream_subscribers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let subscription_id = reg.subscribe();
        JsonRpcResponse::success(
            req.id,
            serde_json::json!({
                "subscription_id": subscription_id,
            }),
        )
    }

    fn handle_sensor_stream_unsubscribe(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let subscription_id = req.params["subscription_id"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if subscription_id.is_empty() {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                "subscription_id is required",
            );
        }
        let mut reg = self
            .sensor_stream_subscribers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let was_subscribed = reg.unsubscribe(&subscription_id);
        JsonRpcResponse::success(
            req.id,
            serde_json::json!({
                "unsubscribed": was_subscribed,
                "subscription_id": subscription_id,
            }),
        )
    }

    fn handle_sensor_stream_poll(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let subscription_id = req.params["subscription_id"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if subscription_id.is_empty() {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                "subscription_id is required",
            );
        }
        let mut reg = self
            .sensor_stream_subscribers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let batch = reg.poll(&subscription_id);
        JsonRpcResponse::success(req.id, serde_json::to_value(&batch).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_rpc::JsonRpcRequest;
    use serde_json::json;

    fn test_handlers() -> RpcHandlers {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let viz_state = Arc::new(RwLock::new(VisualizationState::new()));
        RpcHandlers::new(graph, "test".to_string(), viz_state)
    }

    #[test]
    fn subscribe_new_subscriber() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "interaction.subscribe",
            json!({"subscriber_id": "spring-1"}),
            json!(1),
        );
        let resp = h.handle_interaction_subscribe(req);
        assert!(resp.result.is_some());
        let r = resp.result.unwrap();
        assert_eq!(r["subscribed"], true);
        assert_eq!(r["is_new"], true);
    }

    #[test]
    fn subscribe_duplicate_subscriber() {
        let h = test_handlers();
        let req1 = JsonRpcRequest::new(
            "interaction.subscribe",
            json!({"subscriber_id": "spring-1"}),
            json!(1),
        );
        h.handle_interaction_subscribe(req1);
        let req2 = JsonRpcRequest::new(
            "interaction.subscribe",
            json!({"subscriber_id": "spring-1"}),
            json!(2),
        );
        let resp = h.handle_interaction_subscribe(req2);
        assert_eq!(resp.result.unwrap()["is_new"], false);
    }

    #[test]
    fn subscribe_missing_id_returns_error() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("interaction.subscribe", json!({}), json!(1));
        let resp = h.handle_interaction_subscribe(req);
        assert!(resp.error.is_some());
    }

    #[test]
    fn poll_empty_returns_no_events() {
        let h = test_handlers();
        let sub = JsonRpcRequest::new(
            "interaction.subscribe",
            json!({"subscriber_id": "s1"}),
            json!(1),
        );
        h.handle_interaction_subscribe(sub);
        let poll =
            JsonRpcRequest::new("interaction.poll", json!({"subscriber_id": "s1"}), json!(2));
        let resp = h.handle_interaction_poll(poll);
        let r = resp.result.unwrap();
        assert!(r["events"].as_array().unwrap().is_empty());
    }

    #[test]
    fn broadcast_and_poll_events() {
        let h = test_handlers();
        let sub = JsonRpcRequest::new(
            "interaction.subscribe",
            json!({"subscriber_id": "s1"}),
            json!(1),
        );
        h.handle_interaction_subscribe(sub);

        let ev = crate::visualization_handler::InteractionEventNotification {
            event_type: "select".to_string(),
            targets: vec!["node-1".to_string()],
            timestamp: "2026-03-09T00:00:00Z".to_string(),
            perspective_id: Some(1),
        };
        h.interaction_subscribers.write().unwrap().broadcast(&ev);

        let poll =
            JsonRpcRequest::new("interaction.poll", json!({"subscriber_id": "s1"}), json!(3));
        let resp = h.handle_interaction_poll(poll);
        let events = resp.result.unwrap()["events"].as_array().unwrap().clone();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["event_type"], "select");
    }

    #[test]
    fn unsubscribe_removes_subscriber() {
        let h = test_handlers();
        let sub = JsonRpcRequest::new(
            "interaction.subscribe",
            json!({"subscriber_id": "s1"}),
            json!(1),
        );
        h.handle_interaction_subscribe(sub);
        let unsub = JsonRpcRequest::new(
            "interaction.unsubscribe",
            json!({"subscriber_id": "s1"}),
            json!(2),
        );
        let resp = h.handle_interaction_unsubscribe(unsub);
        assert_eq!(resp.result.unwrap()["unsubscribed"], true);
    }

    #[tokio::test]
    async fn dispatch_interaction_subscribe() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "interaction.subscribe",
            json!({"subscriber_id": "test-spring"}),
            json!(1),
        );
        let resp = h.handle_request(req).await;
        assert!(resp.result.is_some());
        assert_eq!(resp.result.unwrap()["subscribed"], true);
    }
}
