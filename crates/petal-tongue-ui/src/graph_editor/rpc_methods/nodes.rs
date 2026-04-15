// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::error::{GraphEditorError, Result};
use tracing::debug;

use super::GraphEditorService;
use super::types::{
    AddNodeRequest, AddNodeResponse, ModifyNodeRequest, ModifyNodeResponse, RemoveNodeRequest,
    RemoveNodeResponse, ValidationResult,
};

impl GraphEditorService {
    /// # Errors
    ///
    /// Currently always returns `Ok`; validation errors are embedded in the response.
    pub async fn add_node(&self, req: AddNodeRequest) -> Result<AddNodeResponse> {
        debug!("Adding node to graph '{}'", req.graph_id);

        let (node_id, validation) = {
            let mut graphs = self.graphs.write().await;
            let graph = graphs.entry(req.graph_id.clone()).or_insert_with(|| {
                crate::graph_editor::graph::Graph::new(
                    req.graph_id.clone(),
                    "New Graph".to_string(),
                )
            });

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
}
