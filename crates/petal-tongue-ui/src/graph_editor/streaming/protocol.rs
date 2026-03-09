// SPDX-License-Identifier: AGPL-3.0-only

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
