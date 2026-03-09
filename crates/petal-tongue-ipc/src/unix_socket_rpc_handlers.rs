// SPDX-License-Identifier: AGPL-3.0-only

use crate::capability_detection;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use crate::visualization_handler::{
    DismissRequest, ExportRequest, GrammarRenderRequest, InteractionApplyRequest,
    StreamUpdateRequest, ValidateRequest, VisualizationRenderRequest, VisualizationState,
};
use petal_tongue_core::graph_engine::GraphEngine;
use serde_json::{Value, json};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tracing::{debug, warn};

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
        }
    }

    fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().unwrap_or_default().as_secs()
    }

    /// Dispatch JSON-RPC request to the appropriate handler
    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let method = req.method.as_str();
        match method {
            "health.check" => self.handle_health_check(req),
            "capability.announce" => self.handle_announce_capabilities(req),
            "ui.render" => self.handle_ui_render(req).await,
            "ui.display_status" => self.handle_ui_display_status(req),
            "capability.list" => self.get_capabilities(req.id),
            "visualization.render_graph" => self.render_graph(req.params, req.id).await,
            "health.get" => self.get_health(req.id),
            "topology.get" => self.get_topology(req.id),
            "visualization.render" => self.handle_visualization_render(req),
            "visualization.render.stream" => self.handle_visualization_stream(req),
            "visualization.render.grammar" => self.handle_grammar_render(req),
            "visualization.validate" => self.handle_validate_grammar(req),
            "visualization.export" => self.handle_export(req),
            "visualization.dismiss" => self.handle_dismiss(req),
            "visualization.interact.apply" => self.handle_interact_apply(req),
            "visualization.interact.perspectives" => self.handle_interact_perspectives(req.id),
            "visualization.capabilities" => self.handle_visualization_capabilities(req.id),
            "interaction.subscribe" => self.handle_interaction_subscribe(req),
            "interaction.poll" => self.handle_interaction_poll(req),
            "interaction.unsubscribe" => self.handle_interaction_unsubscribe(req),

            // Motor commands (IPC afferent → UI efferent bridge)
            "motor.set_panel" | "motor.set_zoom" | "motor.fit_to_view" | "motor.set_mode"
            | "motor.navigate" => self.handle_motor_command(req),
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

    /// Handle visualization.render: create or replace a visualization session
    fn handle_visualization_render(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let params = match serde_json::from_value::<VisualizationRenderRequest>(req.params) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    req.id,
                    error_codes::INVALID_PARAMS,
                    format!("Invalid params: {e}"),
                );
            }
        };
        let response = self
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
    fn handle_visualization_stream(&self, req: JsonRpcRequest) -> JsonRpcResponse {
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
        let response = self
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

    /// Handle `visualization.render.grammar`: compile grammar expression through scene engine
    fn handle_grammar_render(&self, req: JsonRpcRequest) -> JsonRpcResponse {
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
        let response = self
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

    /// Handle `visualization.validate`: validate grammar + data against Tufte constraints
    fn handle_validate_grammar(&self, req: JsonRpcRequest) -> JsonRpcResponse {
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
        let response = self
            .viz_state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .handle_validate(params);
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

    /// Handle `visualization.export`: export a session to the requested format
    fn handle_export(&self, req: JsonRpcRequest) -> JsonRpcResponse {
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
        let response = self
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

    /// Handle `visualization.dismiss`: remove a session
    fn handle_dismiss(&self, req: JsonRpcRequest) -> JsonRpcResponse {
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
        let response = self
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

    /// Handle `visualization.interact.apply`: apply interaction intent and broadcast
    fn handle_interact_apply(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let params = match serde_json::from_value::<InteractionApplyRequest>(req.params) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    req.id,
                    error_codes::INVALID_PARAMS,
                    format!("Invalid params: {e}"),
                );
            }
        };
        let response = self
            .interaction_subscribers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .apply_interaction(&params);
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

    /// Handle `visualization.interact.perspectives`: return available perspectives
    fn handle_interact_perspectives(&self, id: Value) -> JsonRpcResponse {
        let perspectives = self
            .interaction_subscribers
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .perspectives();
        JsonRpcResponse::success(id, json!({ "perspectives": perspectives }))
    }

    /// Handle visualization.capabilities: return supported `DataBinding` variant names
    #[allow(clippy::unused_self)]
    fn handle_visualization_capabilities(&self, id: Value) -> JsonRpcResponse {
        let variants = [
            "TimeSeries",
            "Distribution",
            "Bar",
            "Gauge",
            "Heatmap",
            "Scatter3D",
            "FieldMap",
            "Spectrum",
        ];
        let grammar_geometry = [
            "Point", "Line", "Bar", "Area", "Ribbon", "Tile", "Arc", "ErrorBar", "Mesh3D",
            "Sphere", "Cylinder", "Text",
        ];
        let output_modalities = ["svg", "audio", "description"];
        let tufte_constraints = ["DataInkRatio", "ChartjunkDetection"];
        JsonRpcResponse::success(
            id,
            json!({
                "data_binding_variants": variants,
                "grammar_geometry_types": grammar_geometry,
                "output_modalities": output_modalities,
                "tufte_constraints": tufte_constraints,
                "scene_engine": true,
            }),
        )
    }

    /// Handle `health_check`: return status, version, uptime, and modalities
    #[must_use]
    pub fn handle_health_check(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let modalities = capability_detection::detect_active_modalities();

        JsonRpcResponse::success(
            request.id,
            json!({
                "status": "healthy",
                "version": env!("CARGO_PKG_VERSION"),
                "uptime_seconds": self.uptime_seconds(),
                "display_available": modalities.contains(&"visual"),
                "modalities_active": modalities,
            }),
        )
    }

    /// Handle `announce_capabilities`: return detected capabilities
    #[must_use]
    pub fn handle_announce_capabilities(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let capabilities = capability_detection::detect_capabilities();

        JsonRpcResponse::success(
            request.id,
            json!({
                "capabilities": capabilities,
            }),
        )
    }

    /// Handle ui.render: render content by type (graph, etc.)
    pub async fn handle_ui_render(&self, request: JsonRpcRequest) -> JsonRpcResponse {
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
                let result = self.render_graph_data(data).await;

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
    pub fn handle_ui_display_status(&self, request: JsonRpcRequest) -> JsonRpcResponse {
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

        debug!("Status update for {}: {:?}", primal_name, status);

        JsonRpcResponse::success(
            request.id,
            json!({
                "updated": true,
                "primal": primal_name,
            }),
        )
    }

    /// Handle `get_capabilities`: return supported capabilities and protocol info
    #[must_use]
    pub fn get_capabilities(&self, id: Value) -> JsonRpcResponse {
        JsonRpcResponse::success(
            id,
            json!({
                "capabilities": [
                    "interaction.subscribe",
                    "ui.desktop-interface",
                    "ui.primal-interaction",
                    "visualization.graph-rendering",
                    "visualization.real-time-topology",
                    "visualization.flow-animation",
                    "ui.multi-modal",
                    "ui.awakening-experience",
                    "visualization.terminal",
                    "visualization.svg",
                    "visualization.png",
                    "visualization.egui"
                ],
                "version": env!("CARGO_PKG_VERSION"),
                "family_id": &self.family_id,
                "protocol": "json-rpc-2.0",
                "transport": "unix-socket"
            }),
        )
    }

    /// Handle `get_health`: return health status and graph stats
    pub fn get_health(&self, id: Value) -> JsonRpcResponse {
        let graph = self
            .graph
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let node_count = graph.nodes().len();
        let edge_count = graph.edges().len();

        JsonRpcResponse::success(
            id,
            json!({
                "status": "healthy",
                "family_id": &self.family_id,
                "graph": {
                    "nodes": node_count,
                    "edges": edge_count
                },
                "protocol": "json-rpc-2.0"
            }),
        )
    }

    /// Handle `get_topology`: return graph nodes and edges
    pub fn get_topology(&self, id: Value) -> JsonRpcResponse {
        let graph = self
            .graph
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let topology = json!({
            "nodes": graph.nodes().iter().map(|node| {
                json!({
                    "id": node.info.id,
                    "name": node.info.name,
                    "type": node.info.primal_type,
                    "capabilities": node.info.capabilities,
                    "health": format!("{:?}", node.info.health),
                    "position": {
                        "x": node.position.x,
                        "y": node.position.y,
                        "z": node.position.z
                    }
                })
            }).collect::<Vec<_>>(),
            "edges": graph.edges().iter().map(|edge| {
                json!({
                    "from": edge.from,
                    "to": edge.to,
                    "type": edge.edge_type
                })
            }).collect::<Vec<_>>()
        });

        JsonRpcResponse::success(id, topology)
    }

    #[allow(clippy::unused_async)]
    async fn render_graph_data(&self, data: Value) -> anyhow::Result<()> {
        debug!("Rendering graph data: {:?}", data);
        Ok(())
    }

    /// Handle `render_graph`: render graph to specified format (svg, png, terminal)
    #[allow(clippy::unused_async)]
    pub async fn render_graph(&self, params: Value, id: Value) -> JsonRpcResponse {
        let format = params["format"].as_str().unwrap_or("svg");

        match format {
            "svg" => JsonRpcResponse::success(
                id,
                json!({
                    "format": "svg",
                    "data": "<svg><!-- TODO: Implement SVG rendering --></svg>",
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
                    "data": "TODO: Terminal rendering"
                }),
            ),
            _ => JsonRpcResponse::error(
                id,
                error_codes::INVALID_PARAMS,
                format!("Unsupported format: {format}"),
            ),
        }
    }

    fn handle_interaction_subscribe(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let subscriber_id = req.params["subscriber_id"]
            .as_str()
            .unwrap_or("")
            .to_string();
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

        let is_new = self
            .interaction_subscribers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .subscribe_with_filter(
                &subscriber_id,
                event_filter,
                callback_method.clone(),
                grammar_id,
            );

        JsonRpcResponse::success(
            req.id,
            json!({
                "subscribed": true,
                "subscriber_id": subscriber_id,
                "is_new": is_new,
                "callback_method": callback_method,
            }),
        )
    }

    fn handle_interaction_poll(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let subscriber_id = req.params["subscriber_id"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if subscriber_id.is_empty() {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                "subscriber_id is required",
            );
        }
        let events = self
            .interaction_subscribers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .poll(&subscriber_id);
        JsonRpcResponse::success(
            req.id,
            json!({
                "subscriber_id": subscriber_id,
                "events": events,
            }),
        )
    }

    fn handle_interaction_unsubscribe(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        let subscriber_id = req.params["subscriber_id"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if subscriber_id.is_empty() {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INVALID_PARAMS,
                "subscriber_id is required",
            );
        }
        let was_subscribed = self
            .interaction_subscribers
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .unsubscribe(&subscriber_id);
        JsonRpcResponse::success(
            req.id,
            json!({
                "unsubscribed": was_subscribed,
                "subscriber_id": subscriber_id,
            }),
        )
    }

    /// Bridge a JSON-RPC motor command to the UI efferent channel.
    fn handle_motor_command(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        use petal_tongue_core::{MotorCommand, PanelId};

        let Some(ref tx) = self.motor_tx else {
            return JsonRpcResponse::error(
                req.id,
                error_codes::INTERNAL_ERROR,
                "Motor channel not connected",
            );
        };

        let cmd: Option<MotorCommand> = match req.method.as_str() {
            "motor.set_panel" => {
                let panel_name = req.params["panel"].as_str().unwrap_or("");
                let visible = req.params["visible"].as_bool().unwrap_or(true);
                let pid = match panel_name {
                    "left_sidebar" | "controls" => PanelId::LeftSidebar,
                    "right_sidebar" => PanelId::RightSidebar,
                    "top_menu" => PanelId::TopMenu,
                    "system_dashboard" | "dashboard" => PanelId::SystemDashboard,
                    "audio" | "audio_panel" => PanelId::AudioPanel,
                    "trust" | "trust_dashboard" => PanelId::TrustDashboard,
                    "proprioception" => PanelId::Proprioception,
                    "graph_stats" => PanelId::GraphStats,
                    other => PanelId::Custom(other.to_string()),
                };
                Some(MotorCommand::SetPanelVisibility {
                    panel: pid,
                    visible,
                })
            }
            "motor.set_zoom" => {
                let level = req.params["level"].as_f64().unwrap_or(1.0) as f32;
                Some(MotorCommand::SetZoom { level })
            }
            "motor.fit_to_view" => Some(MotorCommand::FitToView),
            "motor.set_mode" => {
                let mode = req.params["mode"].as_str().unwrap_or("").to_string();
                Some(MotorCommand::SetMode { mode })
            }
            "motor.navigate" => {
                let node_id = req.params["node_id"].as_str().unwrap_or("").to_string();
                Some(MotorCommand::Navigate {
                    target_node: node_id,
                })
            }
            _ => None,
        };

        match cmd {
            Some(motor_cmd) => {
                if tx.send(motor_cmd).is_ok() {
                    JsonRpcResponse::success(req.id, json!({ "ok": true }))
                } else {
                    JsonRpcResponse::error(
                        req.id,
                        error_codes::INTERNAL_ERROR,
                        "Motor channel disconnected",
                    )
                }
            }
            None => JsonRpcResponse::error(
                req.id,
                error_codes::METHOD_NOT_FOUND,
                format!("Unknown motor method: {}", req.method),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json_rpc::JsonRpcRequest;

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

        h.interaction_subscribers.write().unwrap().broadcast(
            crate::visualization_handler::InteractionEventNotification {
                event_type: "select".to_string(),
                targets: vec!["node-1".to_string()],
                timestamp: "2026-03-09T00:00:00Z".to_string(),
                perspective_id: Some(1),
            },
        );

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
