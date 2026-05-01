// SPDX-License-Identifier: AGPL-3.0-or-later
//! Types for `visualization.render.dashboard`.
//!
//! # Wire-level JSON-RPC schema
//!
//! ## Request
//!
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "method": "visualization.render.dashboard",
//!   "params": {
//!     "session_id": "dash-abc123",
//!     "title": "Ecosystem Health",
//!     "bindings": [
//!       {
//!         "channel_type": "timeseries",
//!         "id": "cpu",
//!         "label": "CPU Usage",
//!         "x_label": "Time",
//!         "y_label": "Percent",
//!         "unit": "%",
//!         "x_values": [0.0, 1.0, 2.0],
//!         "y_values": [45.0, 62.0, 58.0]
//!       },
//!       {
//!         "channel_type": "gauge",
//!         "id": "mem",
//!         "label": "Memory",
//!         "value": 4.2,
//!         "min": 0.0,
//!         "max": 16.0,
//!         "unit": "GiB",
//!         "normal_range": [0.0, 12.0],
//!         "warning_range": [12.0, 14.0]
//!       }
//!     ],
//!     "domain": "health",
//!     "modality": "svg",
//!     "max_columns": 3
//!   },
//!   "id": 1
//! }
//! ```
//!
//! ### Required fields
//!
//! | Field        | Type               | Description                                         |
//! |--------------|--------------------|-----------------------------------------------------|
//! | `session_id` | `string`           | Unique session identifier (caller-assigned)          |
//! | `title`      | `string`           | Dashboard title rendered above the grid              |
//! | `bindings`   | `DataBinding[]`    | Array of data bindings — each becomes one panel      |
//!
//! ### Optional fields
//!
//! | Field         | Type     | Default  | Description                                  |
//! |---------------|----------|----------|----------------------------------------------|
//! | `domain`      | `string` | `null`   | Domain hint for theming (`"health"`, etc.)   |
//! | `modality`    | `string` | `"svg"`  | Output modality: `"svg"` or `"description"` |
//! | `max_columns` | `usize`  | `3`      | Maximum grid columns for panel layout        |
//!
//! ### `DataBinding` variants
//!
//! Each element in `bindings` is tagged via `channel_type`:
//! `"timeseries"`, `"distribution"`, `"bar"`, `"gauge"`, `"heatmap"`,
//! `"scatter3d"`, `"scatter"`, `"fieldmap"`, `"game_scene"`, `"soundscape"`,
//! `"spectrum"`. See [`DataBinding`] for per-variant fields.
//!
//! ## Response
//!
//! ```json
//! {
//!   "jsonrpc": "2.0",
//!   "result": {
//!     "session_id": "dash-abc123",
//!     "output": "<svg>...</svg>",
//!     "modality": "svg",
//!     "panel_count": 2,
//!     "columns": 2,
//!     "rows": 1,
//!     "scene_nodes": 12,
//!     "total_primitives": 48
//!   },
//!   "id": 1
//! }
//! ```
//!
//! ## Error codes
//!
//! | Code   | Meaning                                           |
//! |--------|---------------------------------------------------|
//! | `-32602` | Invalid params — missing `session_id`, `title`, or `bindings`, or malformed binding |

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
