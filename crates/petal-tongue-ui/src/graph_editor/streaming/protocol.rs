// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamMessage {
    NodeStatus {
        graph_id: String,
        node_id: String,
        status: NodeStatus,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    Progress {
        graph_id: String,
        node_id: String,
        progress: f32,
        message: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    Reasoning {
        graph_id: String,
        reasoning: AIReasoning,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    ResourceUsage {
        graph_id: String,
        node_id: String,
        resources: ResourceUsage,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    Error {
        graph_id: String,
        node_id: Option<String>,
        error: ErrorInfo,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    GraphModification {
        graph_id: String,
        modification: GraphModification,
        user_id: Option<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },

    Heartbeat {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Pending,
    Running { progress: u8 },
    Completed,
    Failed { error: String },
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIReasoning {
    pub decision: String,
    pub confidence: f32,
    pub rationale: Vec<String>,
    pub alternatives: Vec<Alternative>,
    pub data_sources: Vec<String>,
    pub patterns: Vec<Pattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub description: String,
    pub confidence: f32,
    pub reason_not_chosen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub description: String,
    pub source: String,
    pub relevance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_percent: f32,
    pub memory_mb: u64,
    pub disk_io_mbps: f32,
    pub network_mbps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub error_type: String,
    pub message: String,
    pub details: Option<String>,
    pub recoverable: bool,
    pub suggested_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum GraphModification {
    AddNode {
        node: serde_json::Value,
    },
    RemoveNode {
        node_id: String,
    },
    ModifyNode {
        node_id: String,
        changes: serde_json::Value,
    },
    AddEdge {
        from: String,
        to: String,
    },
    RemoveEdge {
        edge_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc_now() -> chrono::DateTime<chrono::Utc> {
        chrono::Utc::now()
    }

    #[test]
    fn node_status_serialization() {
        let status = NodeStatus::Pending;
        let json = serde_json::to_string(&status).unwrap();
        let parsed: NodeStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);

        let status = NodeStatus::Running { progress: 50 };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: NodeStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);

        let status = NodeStatus::Failed {
            error: "oops".to_string(),
        };
        let json = serde_json::to_string(&status).unwrap();
        let parsed: NodeStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, parsed);
    }

    #[test]
    fn stream_message_node_status_roundtrip() {
        let msg = StreamMessage::NodeStatus {
            graph_id: "g1".to_string(),
            node_id: "n1".to_string(),
            status: NodeStatus::Completed,
            timestamp: utc_now(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: StreamMessage = serde_json::from_str(&json).unwrap();
        match (&msg, &parsed) {
            (
                StreamMessage::NodeStatus {
                    graph_id,
                    node_id,
                    status,
                    ..
                },
                StreamMessage::NodeStatus {
                    graph_id: pg,
                    node_id: pn,
                    status: ps,
                    ..
                },
            ) => {
                assert_eq!(graph_id, pg);
                assert_eq!(node_id, pn);
                assert_eq!(status, ps);
            }
            _ => panic!("expected NodeStatus"),
        }
    }

    #[test]
    fn stream_message_heartbeat_roundtrip() {
        let msg = StreamMessage::Heartbeat {
            timestamp: utc_now(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: StreamMessage = serde_json::from_str(&json).unwrap();
        assert!(matches!(parsed, StreamMessage::Heartbeat { .. }));
    }

    #[test]
    fn stream_message_progress_roundtrip() {
        let msg = StreamMessage::Progress {
            graph_id: "g1".to_string(),
            node_id: "n1".to_string(),
            progress: 0.75,
            message: "processing".to_string(),
            timestamp: utc_now(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: StreamMessage = serde_json::from_str(&json).unwrap();
        match parsed {
            StreamMessage::Progress {
                graph_id,
                node_id,
                progress,
                message,
                ..
            } => {
                assert_eq!(graph_id, "g1");
                assert_eq!(node_id, "n1");
                assert!((progress - 0.75).abs() < 1e-6);
                assert_eq!(message, "processing");
            }
            _ => panic!("expected Progress"),
        }
    }

    #[test]
    fn graph_modification_add_node_roundtrip() {
        let modif = GraphModification::AddNode {
            node: serde_json::json!({"id": "n1", "type": "start"}),
        };
        let json = serde_json::to_string(&modif).unwrap();
        let parsed: GraphModification = serde_json::from_str(&json).unwrap();
        match parsed {
            GraphModification::AddNode { node } => {
                assert_eq!(node["id"], "n1");
            }
            _ => panic!("expected AddNode"),
        }
    }

    #[test]
    fn graph_modification_remove_edge_roundtrip() {
        let modif = GraphModification::RemoveEdge {
            edge_id: "e1".to_string(),
        };
        let json = serde_json::to_string(&modif).unwrap();
        let parsed: GraphModification = serde_json::from_str(&json).unwrap();
        match parsed {
            GraphModification::RemoveEdge { edge_id } => assert_eq!(edge_id, "e1"),
            _ => panic!("expected RemoveEdge"),
        }
    }

    #[test]
    fn stream_message_parse_by_type_tag() {
        let json = r#"{"type":"node_status","graph_id":"g1","node_id":"n1","status":"completed","timestamp":"2024-01-01T00:00:00Z"}"#;
        let parsed: StreamMessage = serde_json::from_str(json).unwrap();
        match parsed {
            StreamMessage::NodeStatus {
                graph_id,
                node_id,
                status,
                ..
            } => {
                assert_eq!(graph_id, "g1");
                assert_eq!(node_id, "n1");
                assert_eq!(status, NodeStatus::Completed);
            }
            _ => panic!("expected NodeStatus"),
        }
    }
}
