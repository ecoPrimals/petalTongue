// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::error::{GraphEditorError, Result};
use tracing::info;

use super::GraphEditorService;
use super::types::{
    ApplyTemplateRequest, ApplyTemplateResponse, GraphTemplate, SaveTemplateRequest,
    SaveTemplateResponse,
};

impl GraphEditorService {
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

    pub async fn list_templates(&self) -> Vec<GraphTemplate> {
        let templates = self.templates.read().await;
        templates.values().cloned().collect()
    }
}
