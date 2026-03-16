// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC protocol definitions
//!
//! Defines the commands and responses used for inter-instance communication.

use petal_tongue_core::{InstanceId, SessionState};
use serde::{Deserialize, Serialize};

/// Commands that can be sent to an instance via IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcCommand {
    /// Ping - check if instance is responsive
    Ping,

    /// Get instance status
    GetStatus,

    /// Get full session state
    GetState,

    /// Transfer state to this instance
    TransferState {
        /// The session state to transfer
        state: Box<SessionState>,
    },

    /// Merge graph data from another session
    MergeGraph {
        /// Nodes to merge
        nodes: Vec<petal_tongue_core::PrimalInfo>,
        /// Edges to merge
        edges: Vec<petal_tongue_core::TopologyEdge>,
    },

    /// Bring window to front (present)
    Show,

    /// Hide window
    Hide,

    /// Graceful shutdown
    Shutdown,

    /// List all instances (registry query)
    ListInstances,

    // === Motor commands (efferent via IPC afferent channel) ===
    /// Set a panel's visibility
    SetPanel {
        /// Panel name (e.g. "`left_sidebar`", "audio", "`system_dashboard`", "trust", "proprioception", "`graph_stats`", "`top_menu`")
        panel: String,
        /// Whether to show or hide
        visible: bool,
    },

    /// Set the graph zoom level
    SetZoom {
        /// Zoom level (1.0 = default)
        level: f32,
    },

    /// Fit all nodes into the viewport
    FitToView,

    /// Switch to a named mode preset
    SetMode {
        /// Mode name (e.g. "clinical", "developer", "presentation")
        mode: String,
    },

    /// Center the viewport on a specific node
    Navigate {
        /// Node ID to center on
        node_id: String,
    },

    /// Reload the current scenario from disk
    ReloadScenario,
}

/// Responses from an instance via IPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    /// Success (generic)
    Success,

    /// Pong response to Ping
    Pong,

    /// Instance status information
    Status(InstanceStatus),

    /// Full session state
    State(Box<SessionState>),

    /// List of instance IDs
    InstanceList(Vec<InstanceId>),

    /// Error occurred
    Error {
        /// Error message
        message: String,
    },
}

/// Instance status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceStatus {
    /// Instance ID
    pub instance_id: InstanceId,

    /// Process ID
    pub pid: u32,

    /// Window ID (if known)
    pub window_id: Option<u64>,

    /// Instance name
    pub name: Option<String>,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Number of nodes in graph
    pub node_count: usize,

    /// Number of edges in graph
    pub edge_count: usize,

    /// Whether window is visible
    pub window_visible: bool,

    /// Custom metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl IpcCommand {
    /// Check if this command requires the instance to be running
    #[must_use]
    pub const fn requires_running(&self) -> bool {
        matches!(
            self,
            Self::GetStatus
                | Self::GetState
                | Self::TransferState { .. }
                | Self::MergeGraph { .. }
                | Self::Show
                | Self::Hide
                | Self::SetPanel { .. }
                | Self::SetZoom { .. }
                | Self::FitToView
                | Self::SetMode { .. }
                | Self::Navigate { .. }
                | Self::ReloadScenario
        )
    }

    /// Get command name for logging
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Ping => "Ping",
            Self::GetStatus => "GetStatus",
            Self::GetState => "GetState",
            Self::TransferState { .. } => "TransferState",
            Self::MergeGraph { .. } => "MergeGraph",
            Self::Show => "Show",
            Self::Hide => "Hide",
            Self::Shutdown => "Shutdown",
            Self::ListInstances => "ListInstances",
            Self::SetPanel { .. } => "SetPanel",
            Self::SetZoom { .. } => "SetZoom",
            Self::FitToView => "FitToView",
            Self::SetMode { .. } => "SetMode",
            Self::Navigate { .. } => "Navigate",
            Self::ReloadScenario => "ReloadScenario",
        }
    }
}

impl IpcCommand {
    /// Convert this IPC command to a motor command, if applicable.
    ///
    /// IPC commands that map to motor efferent signals return `Some(MotorCommand)`.
    /// Non-motor commands (Ping, `GetStatus`, etc.) return `None`.
    #[must_use]
    pub fn to_motor_command(&self) -> Option<petal_tongue_core::MotorCommand> {
        use petal_tongue_core::{MotorCommand, PanelId};
        match self {
            Self::SetPanel { panel, visible } => {
                let pid = match panel.as_str() {
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
                    visible: *visible,
                })
            }
            Self::SetZoom { level } => Some(MotorCommand::SetZoom { level: *level }),
            Self::FitToView => Some(MotorCommand::FitToView),
            Self::SetMode { mode } => Some(MotorCommand::SetMode { mode: mode.clone() }),
            Self::Navigate { node_id } => Some(MotorCommand::Navigate {
                target_node: node_id.clone(),
            }),
            _ => None,
        }
    }
}

impl IpcResponse {
    /// Create an error response
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self::Error {
            message: message.into(),
        }
    }

    /// Check if this is an error response
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Error { .. })
    }

    /// Get error message if this is an error
    #[must_use]
    pub fn error_message(&self) -> Option<&str> {
        match self {
            Self::Error { message } => Some(message),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serialization() {
        let cmd = IpcCommand::Ping;
        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: IpcCommand = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, IpcCommand::Ping));
    }

    #[test]
    fn test_response_serialization() {
        let resp = IpcResponse::Success;
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: IpcResponse = serde_json::from_str(&json).unwrap();
        assert!(matches!(deserialized, IpcResponse::Success));
    }

    #[test]
    fn test_error_response() {
        let resp = IpcResponse::error("test error");
        assert!(resp.is_error());
        assert_eq!(resp.error_message(), Some("test error"));
    }

    #[test]
    fn test_command_name() {
        assert_eq!(IpcCommand::Ping.name(), "Ping");
        assert_eq!(IpcCommand::GetStatus.name(), "GetStatus");
    }

    #[test]
    fn test_command_requires_running() {
        assert!(!IpcCommand::Ping.requires_running());
        assert!(!IpcCommand::Shutdown.requires_running());
        assert!(!IpcCommand::ListInstances.requires_running());
        assert!(IpcCommand::GetStatus.requires_running());
        assert!(
            IpcCommand::SetPanel {
                panel: "audio".to_string(),
                visible: true,
            }
            .requires_running()
        );
    }

    #[test]
    fn test_command_parse_navigate() {
        let json = r#"{"Navigate":{"node_id":"some-node"}}"#;
        let cmd: IpcCommand = serde_json::from_str(json).unwrap();
        assert!(matches!(cmd, IpcCommand::Navigate { node_id } if node_id == "some-node"));
    }

    #[test]
    fn test_command_set_zoom_roundtrip() {
        let cmd = IpcCommand::SetZoom { level: 1.5 };
        let json = serde_json::to_string(&cmd).unwrap();
        let parsed: IpcCommand = serde_json::from_str(&json).unwrap();
        if let IpcCommand::SetZoom { level } = parsed {
            assert!((level - 1.5).abs() < f32::EPSILON);
        } else {
            panic!("Expected SetZoom");
        }
    }

    #[test]
    fn test_response_status_roundtrip() {
        let status = InstanceStatus {
            instance_id: petal_tongue_core::InstanceId::new(),
            pid: 1234,
            window_id: Some(1),
            name: Some("test".to_string()),
            uptime_seconds: 60,
            node_count: 5,
            edge_count: 3,
            window_visible: true,
            metadata: std::collections::HashMap::new(),
        };
        let resp = IpcResponse::Status(status);
        let json = serde_json::to_string(&resp).unwrap();
        let parsed: IpcResponse = serde_json::from_str(&json).unwrap();
        assert!(!parsed.is_error());
    }

    #[test]
    fn test_command_to_motor_command() {
        let cmd = IpcCommand::SetZoom { level: 2.0 };
        let motor = cmd.to_motor_command().unwrap();
        if let petal_tongue_core::MotorCommand::SetZoom { level } = motor {
            assert!((level - 2.0).abs() < f32::EPSILON);
        } else {
            panic!("Expected SetZoom");
        }

        let cmd = IpcCommand::Ping;
        assert!(cmd.to_motor_command().is_none());
    }

    #[test]
    fn test_command_set_panel_to_motor() {
        let cmd = IpcCommand::SetPanel {
            panel: "left_sidebar".to_string(),
            visible: true,
        };
        let motor = cmd.to_motor_command().unwrap();
        assert!(matches!(
            motor,
            petal_tongue_core::MotorCommand::SetPanelVisibility {
                panel: petal_tongue_core::PanelId::LeftSidebar,
                visible: true
            }
        ));
    }

    #[test]
    fn test_all_command_names() {
        assert_eq!(IpcCommand::Ping.name(), "Ping");
        assert_eq!(IpcCommand::Shutdown.name(), "Shutdown");
        assert_eq!(
            IpcCommand::MergeGraph {
                nodes: vec![],
                edges: vec![]
            }
            .name(),
            "MergeGraph"
        );
        assert_eq!(IpcCommand::Show.name(), "Show");
        assert_eq!(IpcCommand::Hide.name(), "Hide");
        assert_eq!(IpcCommand::ReloadScenario.name(), "ReloadScenario");
    }

    #[test]
    fn test_command_to_motor_all_panel_aliases() {
        for (panel, expected_id) in [
            ("controls", petal_tongue_core::PanelId::LeftSidebar),
            ("right_sidebar", petal_tongue_core::PanelId::RightSidebar),
            ("top_menu", petal_tongue_core::PanelId::TopMenu),
            ("dashboard", petal_tongue_core::PanelId::SystemDashboard),
            ("audio_panel", petal_tongue_core::PanelId::AudioPanel),
            (
                "trust_dashboard",
                petal_tongue_core::PanelId::TrustDashboard,
            ),
            ("proprioception", petal_tongue_core::PanelId::Proprioception),
            ("graph_stats", petal_tongue_core::PanelId::GraphStats),
        ] {
            let cmd = IpcCommand::SetPanel {
                panel: panel.to_string(),
                visible: true,
            };
            let motor = cmd.to_motor_command().expect("motor");
            assert!(
                matches!(motor, petal_tongue_core::MotorCommand::SetPanelVisibility { panel: p, .. } if p == expected_id),
                "panel {panel} -> {expected_id:?}"
            );
        }
    }

    #[test]
    fn test_command_to_motor_custom_panel() {
        let cmd = IpcCommand::SetPanel {
            panel: "custom_panel".to_string(),
            visible: false,
        };
        let motor = cmd.to_motor_command().expect("motor");
        assert!(matches!(
            motor,
            petal_tongue_core::MotorCommand::SetPanelVisibility {
                panel: petal_tongue_core::PanelId::Custom(s),
                visible: false
            } if s == "custom_panel"
        ));
    }

    #[test]
    fn test_response_success_not_error() {
        assert!(!IpcResponse::Success.is_error());
        assert!(IpcResponse::error("e").is_error());
    }

    #[test]
    fn test_response_error_message_none_for_success() {
        assert!(IpcResponse::Success.error_message().is_none());
        assert!(IpcResponse::Pong.error_message().is_none());
    }

    #[test]
    fn test_response_error_into_string() {
        let resp = IpcResponse::error("failed");
        assert_eq!(resp.error_message(), Some("failed"));
    }

    #[test]
    fn test_command_serialization_all_variants() {
        let variants = [
            IpcCommand::Ping,
            IpcCommand::GetStatus,
            IpcCommand::GetState,
            IpcCommand::Show,
            IpcCommand::Hide,
            IpcCommand::Shutdown,
            IpcCommand::ListInstances,
            IpcCommand::FitToView,
        ];
        for cmd in variants {
            let json = serde_json::to_string(&cmd).expect("serialize");
            let _: IpcCommand = serde_json::from_str(&json).expect("deserialize");
        }
    }
}
