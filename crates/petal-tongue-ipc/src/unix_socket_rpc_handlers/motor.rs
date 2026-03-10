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
