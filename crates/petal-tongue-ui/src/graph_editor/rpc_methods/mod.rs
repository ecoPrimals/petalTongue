// SPDX-License-Identifier: AGPL-3.0-or-later

mod types;

use crate::error::{GraphEditorError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::graph_editor::edge::GraphEdge;
use crate::graph_editor::graph::Graph;
use types::{
    AddEdgeResponse, AddNodeResponse, ApplyTemplateResponse, EditorOpenResponse,
    GetPreviewResponse, ModifyNodeResponse, RemoveNodeResponse, ResourceEstimate,
    SaveTemplateResponse, ValidationResult,
};

pub struct GraphEditorService {
    graphs: Arc<RwLock<HashMap<String, Graph>>>,
    templates: Arc<RwLock<HashMap<String, GraphTemplate>>>,
}

impl GraphEditorService {
    #[must_use]
    pub fn new() -> Self {
        Self {
            graphs: Arc::new(RwLock::new(HashMap::new())),
            templates: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// # Errors
    ///
    /// Currently always returns `Ok`; no error paths.
    pub async fn editor_open(&self, req: EditorOpenRequest) -> Result<EditorOpenResponse> {
        info!("Opening graph editor for graph '{}'", req.graph_id);

        let graph = {
            let graphs = self.graphs.read().await;
            graphs
                .get(&req.graph_id)
                .cloned()
                .unwrap_or_else(|| Graph::new(req.graph_id.clone(), "New Graph".to_string()))
        };

        let template_info = if let Some(template_id) = &graph.metadata.template_id {
            let templates = self.templates.read().await;
            let info = templates.get(template_id).cloned();
            drop(templates);
            info
        } else {
            None
        };

        Ok(EditorOpenResponse {
            graph,
            template_info,
        })
    }

    /// # Errors
    ///
    /// Currently always returns `Ok`; validation errors are embedded in the response.
    pub async fn add_node(&self, req: AddNodeRequest) -> Result<AddNodeResponse> {
        debug!("Adding node to graph '{}'", req.graph_id);

        let (node_id, validation) = {
            let mut graphs = self.graphs.write().await;
            let graph = graphs
                .entry(req.graph_id.clone())
                .or_insert_with(|| Graph::new(req.graph_id.clone(), "New Graph".to_string()));

            let node_id = req.node.id.clone();

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
            drop(graphs);
            (node_id, validation)
        };

        Ok(AddNodeResponse {
            node_id,
            validation,
        })
    }

    /// # Errors
    ///
    /// Returns an error if the graph or node is not found.
    pub async fn modify_node(&self, req: ModifyNodeRequest) -> Result<ModifyNodeResponse> {
        debug!(
            "Modifying node '{}' in graph '{}'",
            req.node_id, req.graph_id
        );

        let validation = {
            let mut graphs = self.graphs.write().await;
            let graph = graphs
                .get_mut(&req.graph_id)
                .ok_or(GraphEditorError::GraphNotFound)?;

            let mut node = graph
                .get_node(&req.node_id)
                .cloned()
                .ok_or(GraphEditorError::RpcNodeNotFound)?;

            if let serde_json::Value::Object(changes) = req.changes
                && let serde_json::Value::Object(ref mut props) = node.properties
            {
                for (key, value) in changes {
                    props.insert(key, value);
                }
            }

            let v = match graph.modify_node(&req.node_id, node) {
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
            drop(graphs);
            v
        };

        Ok(ModifyNodeResponse {
            success: validation.valid,
            validation,
        })
    }

    /// # Errors
    ///
    /// Returns an error if the graph is not found or node removal fails.
    pub async fn remove_node(&self, req: RemoveNodeRequest) -> Result<RemoveNodeResponse> {
        debug!(
            "Removing node '{}' from graph '{}'",
            req.node_id, req.graph_id
        );

        let mut graphs = self.graphs.write().await;
        let graph = graphs
            .get_mut(&req.graph_id)
            .ok_or(GraphEditorError::GraphNotFound)?;

        match graph.remove_node(&req.node_id) {
            Ok(affected_edges) => {
                drop(graphs);
                Ok(RemoveNodeResponse {
                    success: true,
                    affected_edges,
                })
            }
            Err(e) => {
                drop(graphs);
                Err(GraphEditorError::RemoveNodeFailed(e.to_string()).into())
            }
        }
    }

    /// # Errors
    ///
    /// Returns an error if the graph is not found; validation errors are embedded in the response.
    pub async fn add_edge(&self, req: AddEdgeRequest) -> Result<AddEdgeResponse> {
        debug!(
            "Adding edge from '{}' to '{}' in graph '{}'",
            req.from, req.to, req.graph_id
        );

        let (edge_id, validation) = {
            let mut graphs = self.graphs.write().await;
            let graph = graphs
                .get_mut(&req.graph_id)
                .ok_or(GraphEditorError::GraphNotFound)?;

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
            drop(graphs);
            (edge_id, validation)
        };

        Ok(AddEdgeResponse {
            edge_id,
            validation,
        })
    }

    /// # Errors
    ///
    /// Returns an error if the graph is not found.
    pub async fn save_template(&self, req: SaveTemplateRequest) -> Result<SaveTemplateResponse> {
        info!("Saving graph '{}' as template '{}'", req.graph_id, req.name);

        let graph = {
            let graphs = self.graphs.read().await;
            graphs
                .get(&req.graph_id)
                .cloned()
                .ok_or(GraphEditorError::GraphNotFound)?
        };

        let template_id = format!("template-{}", uuid::Uuid::new_v4());
        let saved_at = chrono::Utc::now();

        let template = GraphTemplate {
            id: template_id.clone(),
            name: req.name,
            description: req.description,
            graph,
            author: None,
            tags: req.tags,
            created_at: saved_at,
            usage_count: 0,
        };

        {
            let mut templates = self.templates.write().await;
            templates.insert(template_id.clone(), template);
        }

        Ok(SaveTemplateResponse {
            template_id,
            saved_at,
        })
    }

    /// # Errors
    ///
    /// Returns an error if the template is not found.
    pub async fn apply_template(&self, req: ApplyTemplateRequest) -> Result<ApplyTemplateResponse> {
        info!(
            "Applying template '{}' (merge: {})",
            req.template_id, req.merge
        );

        let graph = {
            let templates = self.templates.read().await;
            templates
                .get(&req.template_id)
                .ok_or(GraphEditorError::TemplateNotFound)?
                .graph
                .clone()
        };

        let nodes_added = graph.nodes.len();
        let edges_added = graph.edges.len();

        Ok(ApplyTemplateResponse {
            graph,
            nodes_added,
            edges_added,
        })
    }

    /// # Errors
    ///
    /// Returns an error if graph validation fails or topological sort fails (e.g., cycles).
    #[expect(clippy::unused_async, reason = "async for trait compatibility")]
    pub async fn get_preview(&self, req: GetPreviewRequest) -> Result<GetPreviewResponse> {
        debug!("Getting execution preview for graph '{}'", req.graph.id);

        req.graph.validate()?;

        let execution_order = req.graph.topological_sort()?;

        let estimated_duration = format!("{}s", execution_order.len());

        let resource_requirements = ResourceEstimate {
            cpu_cores: execution_order.len() as f32 * 0.5,
            memory_mb: execution_order.len() as u64 * 100,
            disk_gb: execution_order.len() as f32 * 0.1,
            network_mbps: execution_order.len() as f32 * 10.0,
        };

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

    pub async fn get_graph(&self, graph_id: &str) -> Option<Graph> {
        let graphs = self.graphs.read().await;
        graphs.get(graph_id).cloned()
    }

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

pub use types::{
    AddEdgeRequest, AddNodeRequest, ApplyTemplateRequest, EditorOpenRequest, GetPreviewRequest,
    GraphTemplate, ModifyNodeRequest, RemoveNodeRequest, SaveTemplateRequest,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_editor::edge::DependencyType;
    use crate::graph_editor::node::GraphNode;

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

        let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
        let add_req = AddNodeRequest {
            graph_id: "test-graph".to_string(),
            node,
        };
        service.add_node(add_req).await.unwrap();

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

        let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
        service
            .add_node(AddNodeRequest {
                graph_id: "test-graph".to_string(),
                node,
            })
            .await
            .unwrap();

        let save_req = SaveTemplateRequest {
            graph_id: "test-graph".to_string(),
            name: "Test Template".to_string(),
            description: "A test template".to_string(),
            tags: vec!["test".to_string()],
        };
        let save_resp = service.save_template(save_req).await.unwrap();

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
