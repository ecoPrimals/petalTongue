// SPDX-License-Identifier: AGPL-3.0-only

use crate::capability_detection;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use crate::visualization_handler::{
    StreamUpdateRequest, VisualizationRenderRequest, VisualizationState,
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
        }
    }

    fn uptime_seconds(&self) -> u64 {
        self.start_time.elapsed().unwrap_or_default().as_secs()
    }

    /// Dispatch JSON-RPC request to the appropriate handler
    pub async fn handle_request(&self, req: JsonRpcRequest) -> JsonRpcResponse {
        match req.method.as_str() {
            "health_check" => self.handle_health_check(&req),
            "announce_capabilities" => self.handle_announce_capabilities(&req),
            "ui.render" => self.handle_ui_render(&req).await,
            "ui.display_status" => self.handle_ui_display_status(&req),
            "get_capabilities" => self.get_capabilities(req.id),
            "render_graph" => self.render_graph(req.params, req.id).await,
            "get_health" => self.get_health(req.id),
            "get_topology" => self.get_topology(req.id),
            "visualization.render" => self.handle_visualization_render(&req),
            "visualization.render.stream" => self.handle_visualization_stream(&req),
            "visualization.capabilities" => self.handle_visualization_capabilities(req.id),

            // Motor commands (IPC afferent → UI efferent bridge)
            "motor.set_panel" | "motor.set_zoom" | "motor.fit_to_view" | "motor.set_mode"
            | "motor.navigate" => self.handle_motor_command(&req),
            _ => {
                warn!("Unknown method: {}", req.method);
                JsonRpcResponse::error(
                    req.id,
                    error_codes::METHOD_NOT_FOUND,
                    format!("Method not found: {}", req.method),
                )
            }
        }
    }

    /// Handle visualization.render: create or replace a visualization session
    fn handle_visualization_render(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        let params = match serde_json::from_value::<VisualizationRenderRequest>(req.params.clone())
        {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    req.id.clone(),
                    error_codes::INVALID_PARAMS,
                    format!("Invalid params: {e}"),
                );
            }
        };
        let response = self
            .viz_state
            .write()
            .expect("SAFETY: Viz lock poisoned")
            .handle_render(params);
        JsonRpcResponse::success(
            req.id.clone(),
            serde_json::to_value(&response).expect("response serialization"),
        )
    }

    /// Handle visualization.render.stream: incremental update to a binding
    fn handle_visualization_stream(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        let params = match serde_json::from_value::<StreamUpdateRequest>(req.params.clone()) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    req.id.clone(),
                    error_codes::INVALID_PARAMS,
                    format!("Invalid params: {e}"),
                );
            }
        };
        let response = self
            .viz_state
            .write()
            .expect("SAFETY: Viz lock poisoned")
            .handle_stream_update(params);
        JsonRpcResponse::success(
            req.id.clone(),
            serde_json::to_value(&response).expect("response serialization"),
        )
    }

    /// Handle visualization.capabilities: return supported DataBinding variant names
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
        JsonRpcResponse::success(
            id,
            json!({
                "data_binding_variants": variants,
            }),
        )
    }

    /// Handle health_check: return status, version, uptime, and modalities
    pub fn handle_health_check(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let modalities = capability_detection::detect_active_modalities();

        JsonRpcResponse::success(
            request.id.clone(),
            json!({
                "status": "healthy",
                "version": env!("CARGO_PKG_VERSION"),
                "uptime_seconds": self.uptime_seconds(),
                "display_available": modalities.contains(&"visual"),
                "modalities_active": modalities,
            }),
        )
    }

    /// Handle announce_capabilities: return detected capabilities
    pub fn handle_announce_capabilities(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let capabilities = capability_detection::detect_capabilities();

        JsonRpcResponse::success(
            request.id.clone(),
            json!({
                "capabilities": capabilities,
            }),
        )
    }

    /// Handle ui.render: render content by type (graph, etc.)
    pub async fn handle_ui_render(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let params = match request.params.as_object() {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(
                    request.id.clone(),
                    error_codes::INVALID_PARAMS,
                    "params must be an object",
                );
            }
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
                    Ok(_) => JsonRpcResponse::success(
                        request.id.clone(),
                        json!({
                            "rendered": true,
                            "modality": "visual",
                            "window_id": "main"
                        }),
                    ),
                    Err(e) => JsonRpcResponse::error(
                        request.id.clone(),
                        error_codes::INTERNAL_ERROR,
                        format!("Render error: {e}"),
                    ),
                }
            }
            _ => JsonRpcResponse::error(
                request.id.clone(),
                error_codes::INVALID_PARAMS,
                format!("Unsupported content_type: {content_type}"),
            ),
        }
    }

    /// Handle ui.display_status: update status for a primal
    pub fn handle_ui_display_status(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let params = match request.params.as_object() {
            Some(p) => p,
            None => {
                return JsonRpcResponse::error(
                    request.id.clone(),
                    error_codes::INVALID_PARAMS,
                    "params must be an object",
                );
            }
        };

        let primal_name = params
            .get("primal_name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let status = params.get("status").cloned().unwrap_or(json!({}));

        debug!("Status update for {}: {:?}", primal_name, status);

        JsonRpcResponse::success(
            request.id.clone(),
            json!({
                "updated": true,
                "primal": primal_name,
            }),
        )
    }

    /// Handle get_capabilities: return supported capabilities and protocol info
    pub fn get_capabilities(&self, id: Value) -> JsonRpcResponse {
        JsonRpcResponse::success(
            id,
            json!({
                "capabilities": [
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

    /// Handle get_health: return health status and graph stats
    pub fn get_health(&self, id: Value) -> JsonRpcResponse {
        let graph = self
            .graph
            .read()
            .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");

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

    /// Handle get_topology: return graph nodes and edges
    pub fn get_topology(&self, id: Value) -> JsonRpcResponse {
        let graph = self
            .graph
            .read()
            .expect("SAFETY: Graph lock poisoned - indicates panic in graph thread");

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

    async fn render_graph_data(&self, data: Value) -> anyhow::Result<()> {
        debug!("Rendering graph data: {:?}", data);
        Ok(())
    }

    /// Handle render_graph: render graph to specified format (svg, png, terminal)
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

    /// Bridge a JSON-RPC motor command to the UI efferent channel.
    fn handle_motor_command(&self, req: &JsonRpcRequest) -> JsonRpcResponse {
        use petal_tongue_core::{MotorCommand, PanelId};

        let Some(ref tx) = self.motor_tx else {
            return JsonRpcResponse::error(
                req.id.clone(),
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
                    JsonRpcResponse::success(req.id.clone(), json!({ "ok": true }))
                } else {
                    JsonRpcResponse::error(
                        req.id.clone(),
                        error_codes::INTERNAL_ERROR,
                        "Motor channel disconnected",
                    )
                }
            }
            None => JsonRpcResponse::error(
                req.id.clone(),
                error_codes::METHOD_NOT_FOUND,
                format!("Unknown motor method: {}", req.method),
            ),
        }
    }
}
