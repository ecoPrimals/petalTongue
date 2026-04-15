// SPDX-License-Identifier: AGPL-3.0-or-later
//! Types for `visualization.render.dashboard`.

use petal_tongue_core::DataBinding;
use serde::{Deserialize, Serialize};

/// Request for `visualization.render.dashboard`: compile all bindings into a multi-panel layout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardRenderRequest {
    /// Unique session identifier.
    pub session_id: String,
    /// Dashboard title.
    pub title: String,
    /// Data bindings — each becomes a panel.
    pub bindings: Vec<DataBinding>,
    /// Domain hint (e.g. "health", "physics") for theming.
    #[serde(default)]
    pub domain: Option<String>,
    /// Requested output modality: "svg" (default), "description".
    #[serde(default = "super::defaults::default_modality")]
    pub modality: String,
    /// Maximum columns in grid layout.
    #[serde(default = "super::defaults::default_dashboard_columns")]
    pub max_columns: usize,
}

/// Response for `visualization.render.dashboard`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardRenderResponse {
    /// Session ID (echoed back).
    pub session_id: String,
    /// The compiled output (SVG string or description text).
    pub output: serde_json::Value,
    /// Output modality that was used.
    pub modality: String,
    /// Number of panels in the dashboard.
    pub panel_count: usize,
    /// Grid columns.
    pub columns: usize,
    /// Grid rows.
    pub rows: usize,
    /// Total scene graph nodes.
    pub scene_nodes: usize,
    /// Total rendering primitives.
    pub total_primitives: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dashboard_render_request_default_columns() {
        let req = DashboardRenderRequest {
            session_id: "s1".into(),
            title: "Dashboard".into(),
            bindings: vec![],
            domain: None,
            modality: "svg".into(),
            max_columns: 3,
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let restored: DashboardRenderRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.max_columns, 3);
    }

    #[test]
    fn dashboard_render_response_roundtrip() {
        let resp = DashboardRenderResponse {
            session_id: "s1".into(),
            output: json!("<svg></svg>"),
            modality: "svg".into(),
            panel_count: 2,
            columns: 2,
            rows: 1,
            scene_nodes: 4,
            total_primitives: 8,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let restored: DashboardRenderResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.columns, 2);
    }
}
