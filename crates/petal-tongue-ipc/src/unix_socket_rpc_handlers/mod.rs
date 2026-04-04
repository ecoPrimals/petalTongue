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

    #[tokio::test]
    async fn dispatch_unknown_method_returns_method_not_found() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("unknown.method", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("err").code,
            error_codes::METHOD_NOT_FOUND
        );
        assert!(resp.result.is_none());
    }

    #[tokio::test]
    async fn dispatch_visualization_introspect() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("visualization.introspect", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_some());
    }

    #[tokio::test]
    async fn dispatch_topology_get() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("topology.get", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        let r = resp.result.unwrap();
        assert!(r["nodes"].is_array());
        assert!(r["edges"].is_array());
    }

    #[tokio::test]
    async fn dispatch_health_check() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("health.check", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["status"], "healthy");
    }

    #[tokio::test]
    async fn dispatch_health_liveness() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("health.liveness", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        let r = resp.result.unwrap();
        assert_eq!(r["status"], "alive");
        assert_eq!(r["alive"], true);
    }

    #[tokio::test]
    async fn dispatch_health_readiness() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("health.readiness", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        let r = resp.result.unwrap();
        assert_eq!(r["status"], "ready");
        assert_eq!(r["ready"], true);
    }

    #[tokio::test]
    async fn dispatch_ping_alias_routes_to_liveness() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("ping", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["status"], "alive");
    }

    #[tokio::test]
    async fn dispatch_status_alias_routes_to_health_check() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("status", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["status"], "healthy");
    }

    #[tokio::test]
    async fn dispatch_identity_get() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("identity.get", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["primal"], "petaltongue");
    }

    #[tokio::test]
    async fn dispatch_lifecycle_status() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("lifecycle.status", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        let r = resp.result.unwrap();
        assert_eq!(r["state"], "running");
        assert_eq!(r["healthy"], true);
    }

    #[tokio::test]
    async fn dispatch_primal_capabilities_alias() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("primal.capabilities", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert!(resp.result.unwrap()["capabilities"].as_array().is_some());
    }

    #[tokio::test]
    async fn dispatch_capability_list() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("capability.list", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert!(resp.result.unwrap()["capabilities"].as_array().is_some());
    }

    #[tokio::test]
    async fn dispatch_visualization_capabilities() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("visualization.capabilities", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert!(
            resp.result.unwrap()["data_binding_variants"]
                .as_array()
                .is_some()
        );
    }

    #[tokio::test]
    async fn dispatch_sensor_stream_subscribe() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("interaction.sensor_stream.subscribe", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert!(resp.result.unwrap()["subscription_id"].as_str().is_some());
    }

    #[tokio::test]
    async fn dispatch_sensor_stream_unsubscribe_empty_id_returns_error() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "interaction.sensor_stream.unsubscribe",
            json!({"subscription_id": ""}),
            json!(1),
        );
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("err").code,
            error_codes::INVALID_PARAMS
        );
    }

    #[tokio::test]
    async fn dispatch_sensor_stream_poll_empty_id_returns_error() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "interaction.sensor_stream.poll",
            json!({"subscription_id": ""}),
            json!(1),
        );
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("err").code,
            error_codes::INVALID_PARAMS
        );
    }

    #[tokio::test]
    async fn dispatch_sensor_stream_subscribe_and_unsubscribe() {
        let h = test_handlers();
        let sub_req =
            JsonRpcRequest::new("interaction.sensor_stream.subscribe", json!({}), json!(1));
        let sub_resp = h.handle_request(sub_req).await;
        let sub_id = sub_resp.result.unwrap()["subscription_id"]
            .as_str()
            .unwrap()
            .to_string();
        let unsub_req = JsonRpcRequest::new(
            "interaction.sensor_stream.unsubscribe",
            json!({"subscription_id": sub_id}),
            json!(2),
        );
        let unsub_resp = h.handle_request(unsub_req).await;
        assert!(unsub_resp.result.is_some());
        assert_eq!(unsub_resp.result.unwrap()["unsubscribed"], true);
    }

    #[tokio::test]
    async fn dispatch_sensor_stream_poll_with_valid_id() {
        let h = test_handlers();
        let sub_req =
            JsonRpcRequest::new("interaction.sensor_stream.subscribe", json!({}), json!(1));
        let sub_resp = h.handle_request(sub_req).await;
        let sub_id = sub_resp.result.unwrap()["subscription_id"]
            .as_str()
            .unwrap()
            .to_string();
        let poll_req = JsonRpcRequest::new(
            "interaction.sensor_stream.poll",
            json!({"subscription_id": sub_id}),
            json!(2),
        );
        let poll_resp = h.handle_request(poll_req).await;
        assert!(poll_resp.result.is_some());
    }

    #[tokio::test]
    async fn dispatch_ui_render() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("ui.render", json!({"content_type": "graph"}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["rendered"], true);
    }

    #[tokio::test]
    async fn dispatch_ui_display_status() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "ui.display_status",
            json!({"primal_name": "test-primal"}),
            json!(1),
        );
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["primal"], "test-primal");
    }

    #[tokio::test]
    async fn dispatch_health_get() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("health.get", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["status"], "healthy");
    }

    #[tokio::test]
    async fn dispatch_provider_register_capability() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "provider.register_capability",
            json!({
                "capability": "test.cap",
                "provider_name": "test-provider",
                "socket_path": "/tmp/test.sock"
            }),
            json!(1),
        );
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["registered"], true);
    }

    #[tokio::test]
    async fn dispatch_visualization_session_list() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("visualization.session.list", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        let r = resp.result.unwrap();
        assert!(r["sessions"].as_array().is_some());
    }

    #[tokio::test]
    async fn dispatch_visualization_interact_perspectives() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("visualization.interact.perspectives", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert!(resp.result.unwrap()["perspectives"].as_array().is_some());
    }

    #[tokio::test]
    async fn dispatch_interaction_poll_missing_subscriber_id() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("interaction.poll", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("err").code,
            error_codes::INVALID_PARAMS
        );
    }

    #[tokio::test]
    async fn dispatch_interaction_unsubscribe_missing_subscriber_id() {
        let h = test_handlers();
        let req = JsonRpcRequest::new("interaction.unsubscribe", json!({}), json!(1));
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("err").code,
            error_codes::INVALID_PARAMS
        );
    }

    #[tokio::test]
    async fn dispatch_visualization_interact_subscribe_alias() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "visualization.interact.subscribe",
            json!({"subscriber_id": "viz-sub"}),
            json!(1),
        );
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["subscribed"], true);
    }

    #[tokio::test]
    async fn dispatch_interaction_subscribe_with_event_filter() {
        let h = test_handlers();
        let req = JsonRpcRequest::new(
            "interaction.subscribe",
            json!({
                "subscriber_id": "filtered-sub",
                "events": ["select", "hover"]
            }),
            json!(1),
        );
        let resp = h.handle_request(req).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["subscribed"], true);
    }

    #[tokio::test]
    async fn dispatch_interaction_unsubscribe_alias() {
        let h = test_handlers();
        let sub_req = JsonRpcRequest::new(
            "interaction.subscribe",
            json!({"subscriber_id": "unsub-alias"}),
            json!(1),
        );
        h.handle_request(sub_req).await;
        let unsub_req = JsonRpcRequest::new(
            "visualization.interact.unsubscribe",
            json!({"subscriber_id": "unsub-alias"}),
            json!(2),
        );
        let resp = h.handle_request(unsub_req).await;
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["unsubscribed"], true);
    }
}
