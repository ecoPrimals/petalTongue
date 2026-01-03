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

    /// Bring window to front (show)
    Show,

    /// Hide window
    Hide,

    /// Graceful shutdown
    Shutdown,

    /// List all instances (registry query)
    ListInstances,
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
    pub fn requires_running(&self) -> bool {
        matches!(
            self,
            Self::GetStatus
                | Self::GetState
                | Self::TransferState { .. }
                | Self::MergeGraph { .. }
                | Self::Show
                | Self::Hide
        )
    }

    /// Get command name for logging
    #[must_use]
    pub fn name(&self) -> &'static str {
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
    pub fn is_error(&self) -> bool {
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
}

