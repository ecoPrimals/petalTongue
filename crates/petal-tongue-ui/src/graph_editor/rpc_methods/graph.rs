// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::error::Result;
use tracing::{debug, info};

use crate::graph_editor::graph::Graph;

use super::GraphEditorService;
use super::types::{
    EditorOpenRequest, EditorOpenResponse, GetPreviewRequest, GetPreviewResponse, ResourceEstimate,
};

impl GraphEditorService {
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
}
