// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::error::{GraphEditorError, Result};
use tracing::debug;

use crate::graph_editor::edge::GraphEdge;

use super::GraphEditorService;
use super::types::{AddEdgeRequest, AddEdgeResponse, ValidationResult};

impl GraphEditorService {
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
}
