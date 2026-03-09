// SPDX-License-Identifier: AGPL-3.0-only
//! RPC Methods for Graph Editor
//!
//! Implements the 8 JSON-RPC methods for collaborative intelligence.
//!
//! # Methods
//!
//! 1. `ui.graph.editor_open` - Open graph editor
//! 2. `ui.graph.add_node` - Add node to graph
//! 3. `ui.graph.modify_node` - Modify existing node
//! 4. `ui.graph.remove_node` - Remove node from graph
//! 5. `ui.graph.add_edge` - Add edge between nodes
//! 6. `ui.graph.save_template` - Save graph as template
//! 7. `ui.graph.apply_template` - Load template into graph
//! 8. `ui.graph.get_preview` - Preview execution plan

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use super::edge::{DependencyType, GraphEdge};
use super::graph::Graph;
use super::node::GraphNode;

/// Graph editor service
///
/// Manages graphs and provides RPC methods for manipulation.
pub struct GraphEditorService {
    /// Active graphs (`graph_id` -> Graph)
    graphs: Arc<RwLock<HashMap<String, Graph>>>,

    /// Templates (`template_id` -> Graph)
    templates: Arc<RwLock<HashMap<String, GraphTemplate>>>,
}

/// Graph template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphTemplate {
    /// Template identifier
    pub id: String,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Graph definition
    pub graph: Graph,
    /// Template author
    pub author: Option<String>,
    /// Classification tags
    pub tags: Vec<String>,
    /// When template was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Number of times template has been used
    pub usage_count: u64,
}

/// Request: Open graph editor
#[derive(Debug, Deserialize)]
pub struct EditorOpenRequest {
    /// Graph to open
    pub graph_id: String,
}

/// Response: Open graph editor
#[derive(Debug, Serialize)]
pub struct EditorOpenResponse {
    /// Opened graph
    pub graph: Graph,
    /// Template information if graph is from template
    pub template_info: Option<GraphTemplate>,
}

/// Request: Add node
#[derive(Debug, Deserialize)]
pub struct AddNodeRequest {
    /// Graph to add node to
    pub graph_id: String,
    /// Node to add
    pub node: GraphNode,
}

/// Response: Add node
#[derive(Debug, Serialize)]
pub struct AddNodeResponse {
    /// ID of added node
    pub node_id: String,
    /// Validation result
    pub validation: ValidationResult,
}

/// Request: Modify node
#[derive(Debug, Deserialize)]
pub struct ModifyNodeRequest {
    /// Graph containing node
    pub graph_id: String,
    /// Node to modify
    pub node_id: String,
    /// Changes to apply
    pub changes: serde_json::Value,
}

/// Response: Modify node
#[derive(Debug, Serialize)]
pub struct ModifyNodeResponse {
    /// Whether modification succeeded
    pub success: bool,
    /// Validation result
    pub validation: ValidationResult,
}

/// Request: Remove node
#[derive(Debug, Deserialize)]
pub struct RemoveNodeRequest {
    /// Graph containing node
    pub graph_id: String,
    /// Node to remove
    pub node_id: String,
}

/// Response: Remove node
#[derive(Debug, Serialize)]
pub struct RemoveNodeResponse {
    /// Whether removal succeeded
    pub success: bool,
    /// Edges that were removed due to node removal
    pub affected_edges: Vec<String>,
}

/// Request: Add edge
#[derive(Debug, Deserialize)]
pub struct AddEdgeRequest {
    /// Graph to add edge to
    pub graph_id: String,
    /// Source node
    pub from: String,
    /// Target node
    pub to: String,
    /// Type of edge (dependency type)
    pub edge_type: DependencyType,
}

/// Response: Add edge
#[derive(Debug, Serialize)]
pub struct AddEdgeResponse {
    /// ID of added edge
    pub edge_id: String,
    /// Validation result
    pub validation: ValidationResult,
}

/// Request: Save template
#[derive(Debug, Deserialize)]
pub struct SaveTemplateRequest {
    /// Graph to save as template
    pub graph_id: String,
    /// Template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Classification tags
    pub tags: Vec<String>,
}

/// Response: Save template
#[derive(Debug, Serialize)]
pub struct SaveTemplateResponse {
    /// ID of saved template
    pub template_id: String,
    /// When template was saved
    pub saved_at: chrono::DateTime<chrono::Utc>,
}

/// Request: Apply template
#[derive(Debug, Deserialize)]
pub struct ApplyTemplateRequest {
    /// Template to apply
    pub template_id: String,
    /// Whether to merge with existing graph (true) or replace (false)
    pub merge: bool,
}

/// Response: Apply template
#[derive(Debug, Serialize)]
pub struct ApplyTemplateResponse {
    /// Resulting graph
    pub graph: Graph,
    /// Number of nodes added
    pub nodes_added: usize,
    /// Number of edges added
    pub edges_added: usize,
}

/// Request: Get preview
#[derive(Debug, Deserialize)]
pub struct GetPreviewRequest {
    /// Graph to preview
    pub graph: Graph,
}

/// Response: Get preview
#[derive(Debug, Serialize)]
pub struct GetPreviewResponse {
    /// Execution order (node IDs)
    pub execution_order: Vec<String>,
    /// Estimated execution duration
    pub estimated_duration: String,
    /// Resource requirements estimate
    pub resource_requirements: ResourceEstimate,
    /// Validation warnings
    pub validation_warnings: Vec<String>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

/// Resource estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEstimate {
    /// Estimated CPU cores needed
    pub cpu_cores: f32,
    /// Estimated memory in megabytes
    pub memory_mb: u64,
    /// Estimated disk space in gigabytes
    pub disk_gb: f32,
    /// Estimated network bandwidth in MB/s
    pub network_mbps: f32,
}

impl GraphEditorService {
    /// Create a new graph editor service
    #[must_use]
    pub fn new() -> Self {
        Self {
            graphs: Arc::new(RwLock::new(HashMap::new())),
            templates: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 1. Open graph editor
    pub async fn editor_open(&self, req: EditorOpenRequest) -> Result<EditorOpenResponse> {
        info!("Opening graph editor for graph '{}'", req.graph_id);

        let graphs = self.graphs.read().await;
        let graph = graphs
            .get(&req.graph_id)
            .cloned()
            .unwrap_or_else(|| Graph::new(req.graph_id.clone(), "New Graph".to_string()));

        // Check if graph was loaded from template
        let template_info = if let Some(template_id) = &graph.metadata.template_id {
            let templates = self.templates.read().await;
            templates.get(template_id).cloned()
        } else {
            None
        };

        Ok(EditorOpenResponse {
            graph,
            template_info,
        })
    }

    /// 2. Add node to graph
    pub async fn add_node(&self, req: AddNodeRequest) -> Result<AddNodeResponse> {
        debug!("Adding node to graph '{}'", req.graph_id);

        let mut graphs = self.graphs.write().await;
        let graph = graphs
            .entry(req.graph_id.clone())
            .or_insert_with(|| Graph::new(req.graph_id.clone(), "New Graph".to_string()));

        let node_id = req.node.id.clone();

        // Validate and add node
        let validation = match graph.add_node(req.node) {
            Ok(()) => ValidationResult {
                valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
            },
            Err(e) => ValidationResult {
                valid: false,
                errors: vec![e.to_string()],
                warnings: Vec::new(),
            },
        };

        Ok(AddNodeResponse {
            node_id,
            validation,
        })
    }

    /// 3. Modify node in graph
    pub async fn modify_node(&self, req: ModifyNodeRequest) -> Result<ModifyNodeResponse> {
        debug!(
            "Modifying node '{}' in graph '{}'",
            req.node_id, req.graph_id
        );

        let mut graphs = self.graphs.write().await;
        let graph = graphs.get_mut(&req.graph_id).context("Graph not found")?;

        // Get existing node
        let mut node = graph
            .get_node(&req.node_id)
            .cloned()
            .context("Node not found")?;

        // Apply changes (merge JSON)
        if let serde_json::Value::Object(changes) = req.changes
            && let serde_json::Value::Object(ref mut props) = node.properties
        {
            for (key, value) in changes {
                props.insert(key, value);
            }
        }

        // Validate and update
        let validation = match graph.modify_node(&req.node_id, node) {
            Ok(()) => ValidationResult {
                valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
            },
            Err(e) => ValidationResult {
                valid: false,
                errors: vec![e.to_string()],
                warnings: Vec::new(),
            },
        };

        Ok(ModifyNodeResponse {
            success: validation.valid,
            validation,
        })
    }

    /// 4. Remove node from graph
    pub async fn remove_node(&self, req: RemoveNodeRequest) -> Result<RemoveNodeResponse> {
        debug!(
            "Removing node '{}' from graph '{}'",
            req.node_id, req.graph_id
        );

        let mut graphs = self.graphs.write().await;
        let graph = graphs.get_mut(&req.graph_id).context("Graph not found")?;

        match graph.remove_node(&req.node_id) {
            Ok(affected_edges) => Ok(RemoveNodeResponse {
                success: true,
                affected_edges,
            }),
            Err(e) => anyhow::bail!("Failed to remove node: {e}"),
        }
    }

    /// 5. Add edge between nodes
    pub async fn add_edge(&self, req: AddEdgeRequest) -> Result<AddEdgeResponse> {
        debug!(
            "Adding edge from '{}' to '{}' in graph '{}'",
            req.from, req.to, req.graph_id
        );

        let mut graphs = self.graphs.write().await;
        let graph = graphs.get_mut(&req.graph_id).context("Graph not found")?;

        let edge_id = format!("edge-{}", uuid::Uuid::new_v4());
        let edge = GraphEdge::new(edge_id.clone(), req.from, req.to, req.edge_type);

        let validation = match graph.add_edge(edge) {
            Ok(()) => ValidationResult {
                valid: true,
                errors: Vec::new(),
                warnings: Vec::new(),
            },
            Err(e) => ValidationResult {
                valid: false,
                errors: vec![e.to_string()],
                warnings: Vec::new(),
            },
        };

        Ok(AddEdgeResponse {
            edge_id,
            validation,
        })
    }

    /// 6. Save graph as template
    pub async fn save_template(&self, req: SaveTemplateRequest) -> Result<SaveTemplateResponse> {
        info!("Saving graph '{}' as template '{}'", req.graph_id, req.name);

        let graphs = self.graphs.read().await;
        let graph = graphs
            .get(&req.graph_id)
            .cloned()
            .context("Graph not found")?;

        let template_id = format!("template-{}", uuid::Uuid::new_v4());
        let saved_at = chrono::Utc::now();

        let template = GraphTemplate {
            id: template_id.clone(),
            name: req.name,
            description: req.description,
            graph,
            author: None, // TODO: Get from auth context
            tags: req.tags,
            created_at: saved_at,
            usage_count: 0,
        };

        let mut templates = self.templates.write().await;
        templates.insert(template_id.clone(), template);

        Ok(SaveTemplateResponse {
            template_id,
            saved_at,
        })
    }

    /// 7. Apply template to graph
    pub async fn apply_template(&self, req: ApplyTemplateRequest) -> Result<ApplyTemplateResponse> {
        info!(
            "Applying template '{}' (merge: {})",
            req.template_id, req.merge
        );

        let templates = self.templates.read().await;
        let template = templates
            .get(&req.template_id)
            .context("Template not found")?;

        let graph = template.graph.clone();

        if req.merge {
            // TODO: Implement merge logic
            // For now, just return the template graph
        }

        let nodes_added = graph.nodes.len();
        let edges_added = graph.edges.len();

        Ok(ApplyTemplateResponse {
            graph,
            nodes_added,
            edges_added,
        })
    }

    /// 8. Get execution preview
    pub async fn get_preview(&self, req: GetPreviewRequest) -> Result<GetPreviewResponse> {
        debug!("Getting execution preview for graph '{}'", req.graph.id);

        // Validate graph
        req.graph.validate().context("Graph validation failed")?;

        // Get topological sort (execution order)
        let execution_order = req.graph.topological_sort()?;

        // Estimate duration (simple heuristic: 1 second per node)
        let estimated_duration = format!("{}s", execution_order.len());

        // Estimate resources (simple heuristic)
        let resource_requirements = ResourceEstimate {
            cpu_cores: execution_order.len() as f32 * 0.5,
            memory_mb: execution_order.len() as u64 * 100,
            disk_gb: execution_order.len() as f32 * 0.1,
            network_mbps: execution_order.len() as f32 * 10.0,
        };

        // Collect warnings
        let mut warnings = Vec::new();
        if execution_order.len() > 100 {
            warnings
                .push("Large graph (>100 nodes) may take significant time to execute".to_string());
        }

        Ok(GetPreviewResponse {
            execution_order,
            estimated_duration,
            resource_requirements,
            validation_warnings: warnings,
        })
    }

    /// Get graph by ID
    pub async fn get_graph(&self, graph_id: &str) -> Option<Graph> {
        let graphs = self.graphs.read().await;
        graphs.get(graph_id).cloned()
    }

    /// List all templates
    pub async fn list_templates(&self) -> Vec<GraphTemplate> {
        let templates = self.templates.read().await;
        templates.values().cloned().collect()
    }
}

impl Default for GraphEditorService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_editor_open() {
        let service = GraphEditorService::new();
        let req = EditorOpenRequest {
            graph_id: "test-graph".to_string(),
        };

        let resp = service.editor_open(req).await.unwrap();
        assert_eq!(resp.graph.id, "test-graph");
        assert!(resp.template_info.is_none());
    }

    #[tokio::test]
    async fn test_add_node() {
        let service = GraphEditorService::new();
        let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
        let req = AddNodeRequest {
            graph_id: "test-graph".to_string(),
            node,
        };

        let resp = service.add_node(req).await.unwrap();
        assert_eq!(resp.node_id, "node-1");
        assert!(resp.validation.valid);
    }

    #[tokio::test]
    async fn test_remove_node() {
        let service = GraphEditorService::new();

        // First add a node
        let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
        let add_req = AddNodeRequest {
            graph_id: "test-graph".to_string(),
            node,
        };
        service.add_node(add_req).await.unwrap();

        // Then remove it
        let remove_req = RemoveNodeRequest {
            graph_id: "test-graph".to_string(),
            node_id: "node-1".to_string(),
        };
        let resp = service.remove_node(remove_req).await.unwrap();
        assert!(resp.success);
    }

    #[tokio::test]
    async fn test_add_edge() {
        let service = GraphEditorService::new();

        // Add two nodes
        let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
        let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());

        service
            .add_node(AddNodeRequest {
                graph_id: "test-graph".to_string(),
                node: node1,
            })
            .await
            .unwrap();

        service
            .add_node(AddNodeRequest {
                graph_id: "test-graph".to_string(),
                node: node2,
            })
            .await
            .unwrap();

        // Add edge
        let req = AddEdgeRequest {
            graph_id: "test-graph".to_string(),
            from: "node-1".to_string(),
            to: "node-2".to_string(),
            edge_type: DependencyType::Sequential,
        };

        let resp = service.add_edge(req).await.unwrap();
        assert!(resp.validation.valid);
    }

    #[tokio::test]
    async fn test_save_and_apply_template() {
        let service = GraphEditorService::new();

        // Add a node
        let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
        service
            .add_node(AddNodeRequest {
                graph_id: "test-graph".to_string(),
                node,
            })
            .await
            .unwrap();

        // Save as template
        let save_req = SaveTemplateRequest {
            graph_id: "test-graph".to_string(),
            name: "Test Template".to_string(),
            description: "A test template".to_string(),
            tags: vec!["test".to_string()],
        };
        let save_resp = service.save_template(save_req).await.unwrap();

        // Apply template
        let apply_req = ApplyTemplateRequest {
            template_id: save_resp.template_id,
            merge: false,
        };
        let apply_resp = service.apply_template(apply_req).await.unwrap();
        assert_eq!(apply_resp.nodes_added, 1);
    }

    #[tokio::test]
    async fn test_get_preview() {
        let service = GraphEditorService::new();

        let mut graph = Graph::new("test-graph".to_string(), "Test".to_string());
        let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
        let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());

        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();

        let req = GetPreviewRequest { graph };
        let resp = service.get_preview(req).await.unwrap();

        assert_eq!(resp.execution_order.len(), 2);
        assert!(resp.validation_warnings.is_empty());
    }

    #[tokio::test]
    async fn test_modify_node() {
        let service = GraphEditorService::new();

        let node = GraphNode::new("node-1".to_string(), "test-type".to_string())
            .with_properties(serde_json::json!({"key": "original"}));
        service
            .add_node(AddNodeRequest {
                graph_id: "test-graph".to_string(),
                node,
            })
            .await
            .unwrap();

        let changes = serde_json::json!({"key": "modified", "new_key": "value"});
        let req = ModifyNodeRequest {
            graph_id: "test-graph".to_string(),
            node_id: "node-1".to_string(),
            changes,
        };
        let resp = service.modify_node(req).await.unwrap();
        assert!(resp.success);
        assert!(resp.validation.valid);
    }

    #[tokio::test]
    async fn test_list_templates() {
        let service = GraphEditorService::new();
        let templates = service.list_templates().await;
        assert!(templates.is_empty());
    }
}
