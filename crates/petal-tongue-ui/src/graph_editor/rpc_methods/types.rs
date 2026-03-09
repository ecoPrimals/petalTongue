// SPDX-License-Identifier: AGPL-3.0-only

use serde::{Deserialize, Serialize};

use crate::graph_editor::edge::DependencyType;
use crate::graph_editor::graph::Graph;
use crate::graph_editor::node::GraphNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub graph: Graph,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub usage_count: u64,
}

#[derive(Debug, Deserialize)]
pub struct EditorOpenRequest {
    pub graph_id: String,
}

#[derive(Debug, Serialize)]
pub struct EditorOpenResponse {
    pub graph: Graph,
    pub template_info: Option<GraphTemplate>,
}

#[derive(Debug, Deserialize)]
pub struct AddNodeRequest {
    pub graph_id: String,
    pub node: GraphNode,
}

#[derive(Debug, Serialize)]
pub struct AddNodeResponse {
    pub node_id: String,
    pub validation: ValidationResult,
}

#[derive(Debug, Deserialize)]
pub struct ModifyNodeRequest {
    pub graph_id: String,
    pub node_id: String,
    pub changes: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ModifyNodeResponse {
    pub success: bool,
    pub validation: ValidationResult,
}

#[derive(Debug, Deserialize)]
pub struct RemoveNodeRequest {
    pub graph_id: String,
    pub node_id: String,
}

#[derive(Debug, Serialize)]
pub struct RemoveNodeResponse {
    pub success: bool,
    pub affected_edges: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddEdgeRequest {
    pub graph_id: String,
    pub from: String,
    pub to: String,
    pub edge_type: DependencyType,
}

#[derive(Debug, Serialize)]
pub struct AddEdgeResponse {
    pub edge_id: String,
    pub validation: ValidationResult,
}

#[derive(Debug, Deserialize)]
pub struct SaveTemplateRequest {
    pub graph_id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SaveTemplateResponse {
    pub template_id: String,
    pub saved_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ApplyTemplateRequest {
    pub template_id: String,
    pub merge: bool,
}

#[derive(Debug, Serialize)]
pub struct ApplyTemplateResponse {
    pub graph: Graph,
    pub nodes_added: usize,
    pub edges_added: usize,
}

#[derive(Debug, Deserialize)]
pub struct GetPreviewRequest {
    pub graph: Graph,
}

#[derive(Debug, Serialize)]
pub struct GetPreviewResponse {
    pub execution_order: Vec<String>,
    pub estimated_duration: String,
    pub resource_requirements: ResourceEstimate,
    pub validation_warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEstimate {
    pub cpu_cores: f32,
    pub memory_mb: u64,
    pub disk_gb: f32,
    pub network_mbps: f32,
}
