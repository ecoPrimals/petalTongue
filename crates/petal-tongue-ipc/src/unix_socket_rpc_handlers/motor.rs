// SPDX-License-Identifier: AGPL-3.0-only
//! Handlers for motor.* JSON-RPC methods.
//!
//! Bridges IPC afferent commands to the UI efferent channel (motor_tx).

use super::RpcHandlers;
use crate::json_rpc::{JsonRpcRequest, JsonRpcResponse, error_codes};
use petal_tongue_core::{MotorCommand, PanelId};

/// Bridge a JSON-RPC motor command to the UI efferent channel.
pub fn handle_motor_command(handlers: &RpcHandlers, req: JsonRpcRequest) -> JsonRpcResponse {
    let Some(ref tx) = handlers.motor_tx else {
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
            #[expect(
                clippy::cast_possible_truncation,
                reason = "zoom level is typically in reasonable range, f32 sufficient"
            )]
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
                JsonRpcResponse::success(req.id, serde_json::json!({ "ok": true }))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visualization_handler::VisualizationState;
    use petal_tongue_core::graph_engine::GraphEngine;
    use serde_json::json;
    use std::sync::mpsc;
    use std::sync::{Arc, RwLock};

    fn handlers_with_motor() -> (RpcHandlers, mpsc::Receiver<MotorCommand>) {
        let (tx, rx) = mpsc::channel();
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let viz_state = Arc::new(RwLock::new(VisualizationState::new()));
        let mut h = RpcHandlers::new(graph, "test".to_string(), viz_state);
        h.motor_tx = Some(tx);
        (h, rx)
    }

    fn handlers_without_motor() -> RpcHandlers {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let viz_state = Arc::new(RwLock::new(VisualizationState::new()));
        RpcHandlers::new(graph, "test".to_string(), viz_state)
    }

    #[test]
    fn motor_no_channel_returns_error() {
        let h = handlers_without_motor();
        let req = JsonRpcRequest::new(
            "motor.set_panel",
            json!({"panel": "left_sidebar", "visible": true}),
            json!(1),
        );
        let resp = handle_motor_command(&h, req);
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("error").code,
            error_codes::INTERNAL_ERROR
        );
        assert!(
            resp.error
                .as_ref()
                .expect("error")
                .message
                .contains("Motor channel not connected")
        );
    }

    #[test]
    fn motor_set_panel_left_sidebar() {
        let (h, rx) = handlers_with_motor();
        let req = JsonRpcRequest::new(
            "motor.set_panel",
            json!({"panel": "left_sidebar", "visible": true}),
            json!(1),
        );
        let resp = handle_motor_command(&h, req);
        assert!(resp.result.is_some());
        assert_eq!(resp.result.as_ref().expect("result")["ok"], true);
        let cmd = rx.recv().expect("command");
        assert!(matches!(
            cmd,
            MotorCommand::SetPanelVisibility {
                panel: PanelId::LeftSidebar,
                visible: true
            }
        ));
    }

    #[test]
    fn motor_set_panel_controls_alias() {
        let (h, rx) = handlers_with_motor();
        let req = JsonRpcRequest::new(
            "motor.set_panel",
            json!({"panel": "controls", "visible": false}),
            json!(2),
        );
        let resp = handle_motor_command(&h, req);
        assert!(resp.result.is_some());
        let cmd = rx.recv().expect("command");
        assert!(matches!(
            cmd,
            MotorCommand::SetPanelVisibility {
                panel: PanelId::LeftSidebar,
                visible: false
            }
        ));
    }

    #[test]
    fn motor_set_panel_all_known_panels() {
        let panels = [
            ("right_sidebar", PanelId::RightSidebar),
            ("top_menu", PanelId::TopMenu),
            ("system_dashboard", PanelId::SystemDashboard),
            ("dashboard", PanelId::SystemDashboard),
            ("audio", PanelId::AudioPanel),
            ("audio_panel", PanelId::AudioPanel),
            ("trust", PanelId::TrustDashboard),
            ("trust_dashboard", PanelId::TrustDashboard),
            ("proprioception", PanelId::Proprioception),
            ("graph_stats", PanelId::GraphStats),
        ];
        for (name, expected) in panels {
            let (h, rx) = handlers_with_motor();
            let req = JsonRpcRequest::new(
                "motor.set_panel",
                json!({"panel": name, "visible": true}),
                json!(1),
            );
            let resp = handle_motor_command(&h, req);
            assert!(resp.result.is_some(), "panel {name} should succeed");
            let cmd = rx.recv().expect("command");
            match &cmd {
                MotorCommand::SetPanelVisibility { panel, .. } => {
                    assert_eq!(format!("{panel:?}"), format!("{expected:?}"));
                }
                _ => panic!("expected SetPanelVisibility"),
            }
        }
    }

    #[test]
    fn motor_set_panel_custom() {
        let (h, rx) = handlers_with_motor();
        let req = JsonRpcRequest::new(
            "motor.set_panel",
            json!({"panel": "my_custom_panel", "visible": true}),
            json!(1),
        );
        let resp = handle_motor_command(&h, req);
        assert!(resp.result.is_some());
        let cmd = rx.recv().expect("command");
        assert!(
            matches!(cmd, MotorCommand::SetPanelVisibility { panel: PanelId::Custom(ref s), .. } if s == "my_custom_panel")
        );
    }

    #[test]
    fn motor_set_panel_default_visible() {
        let (h, rx) = handlers_with_motor();
        let req = JsonRpcRequest::new(
            "motor.set_panel",
            json!({"panel": "left_sidebar"}),
            json!(1),
        );
        let resp = handle_motor_command(&h, req);
        assert!(resp.result.is_some());
        let cmd = rx.recv().expect("command");
        assert!(matches!(
            cmd,
            MotorCommand::SetPanelVisibility { visible: true, .. }
        ));
    }

    #[test]
    fn motor_set_zoom() {
        let (h, rx) = handlers_with_motor();
        let req = JsonRpcRequest::new("motor.set_zoom", json!({"level": 2.5}), json!(1));
        let resp = handle_motor_command(&h, req);
        assert!(resp.result.is_some());
        let cmd = rx.recv().expect("command");
        assert!(
            matches!(cmd, MotorCommand::SetZoom { level } if (level - 2.5).abs() < f32::EPSILON)
        );
    }

    #[test]
    fn motor_set_zoom_default() {
        let (h, rx) = handlers_with_motor();
        let req = JsonRpcRequest::new("motor.set_zoom", json!({}), json!(1));
        let resp = handle_motor_command(&h, req);
        assert!(resp.result.is_some());
        let cmd = rx.recv().expect("command");
        assert!(
            matches!(cmd, MotorCommand::SetZoom { level } if (level - 1.0).abs() < f32::EPSILON)
        );
    }

    #[test]
    fn motor_fit_to_view() {
        let (h, rx) = handlers_with_motor();
        let req = JsonRpcRequest::new("motor.fit_to_view", json!({}), json!(1));
        let resp = handle_motor_command(&h, req);
        assert!(resp.result.is_some());
        let cmd = rx.recv().expect("command");
        assert!(matches!(cmd, MotorCommand::FitToView));
    }

    #[test]
    fn motor_set_mode() {
        let (h, rx) = handlers_with_motor();
        let req = JsonRpcRequest::new("motor.set_mode", json!({"mode": "clinical"}), json!(1));
        let resp = handle_motor_command(&h, req);
        assert!(resp.result.is_some());
        let cmd = rx.recv().expect("command");
        assert!(matches!(cmd, MotorCommand::SetMode { ref mode } if mode == "clinical"));
    }

    #[test]
    fn motor_set_mode_empty_default() {
        let (h, rx) = handlers_with_motor();
        let req = JsonRpcRequest::new("motor.set_mode", json!({}), json!(1));
        let resp = handle_motor_command(&h, req);
        assert!(resp.result.is_some());
        let cmd = rx.recv().expect("command");
        assert!(matches!(cmd, MotorCommand::SetMode { ref mode } if mode.is_empty()));
    }

    #[test]
    fn motor_navigate() {
        let (h, rx) = handlers_with_motor();
        let req = JsonRpcRequest::new("motor.navigate", json!({"node_id": "node-42"}), json!(1));
        let resp = handle_motor_command(&h, req);
        assert!(resp.result.is_some());
        let cmd = rx.recv().expect("command");
        assert!(
            matches!(cmd, MotorCommand::Navigate { ref target_node } if target_node == "node-42")
        );
    }

    #[test]
    fn motor_unknown_method_returns_error() {
        let (h, _rx) = handlers_with_motor();
        let req = JsonRpcRequest::new("motor.unknown", json!({}), json!(1));
        let resp = handle_motor_command(&h, req);
        assert!(resp.error.is_some());
        assert_eq!(
            resp.error.as_ref().expect("error").code,
            error_codes::METHOD_NOT_FOUND
        );
        assert!(
            resp.error
                .as_ref()
                .expect("error")
                .message
                .contains("Unknown motor method")
        );
    }
}
