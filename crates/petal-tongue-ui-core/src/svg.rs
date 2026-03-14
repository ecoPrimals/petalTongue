// SPDX-License-Identifier: AGPL-3.0-only
//! Pure Rust SVG generation
//!
//! Generates SVG visualizations of primal topologies without any native dependencies.
//! SVG files can be viewed in any browser and embedded in web pages.

use crate::trait_def::{ExportFormat, UICapability, UniversalUI};
/// Utility functions for health visualization
use crate::utils::health_to_color;
use anyhow::Result;
use petal_tongue_core::GraphEngine;
use std::sync::{Arc, RwLock};

/// SVG UI renderer (pure Rust, zero native dependencies)
///
/// Generates Scalable Vector Graphics visualizations of primal topologies.
/// SVG files are universally supported and can be viewed in any browser.
///
/// # Examples
///
/// ```no_run
/// use petal_tongue_ui_core::SvgUI;
/// use petal_tongue_core::GraphEngine;
/// use std::sync::{Arc, RwLock};
/// use std::path::Path;
///
/// # fn main() -> anyhow::Result<()> {
/// let graph = Arc::new(RwLock::new(GraphEngine::new()));
/// let ui = SvgUI::new(graph, 1920, 1080);
///
/// // Export to file
/// use petal_tongue_ui_core::{UniversalUI, ExportFormat};
/// ui.export(Path::new("topology.svg"), ExportFormat::Svg)?;
/// # Ok(())
/// # }
/// ```
pub struct SvgUI {
    graph: Arc<RwLock<GraphEngine>>,
    width: u32,
    height: u32,
    background_color: String,
    node_radius: f32,
}

impl SvgUI {
    /// Create a new SVG UI renderer
    pub fn new(graph: Arc<RwLock<GraphEngine>>, width: u32, height: u32) -> Self {
        Self {
            graph,
            width,
            height,
            background_color: "#141822".to_string(), // Dark mode
            node_radius: 20.0,
        }
    }

    /// Set background color (hex format)
    #[must_use]
    pub fn with_background(mut self, color: &str) -> Self {
        self.background_color = color.to_string();
        self
    }

    /// Set node radius
    #[must_use]
    pub const fn with_node_radius(mut self, radius: f32) -> Self {
        self.node_radius = radius;
        self
    }

    /// Render SVG content
    fn render_svg(&self) -> Result<String> {
        let mut svg = String::new();

        // SVG header
        svg.push_str(&format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            self.width, self.height
        ));
        svg.push('\n');

        // Background
        svg.push_str(&format!(
            r#"<rect width="100%" height="100%" fill="{}"/>"#,
            self.background_color
        ));
        svg.push('\n');

        // Styles
        svg.push_str(
            r"<style>
            text { font-family: system-ui, sans-serif; fill: #f3f4f6; }
            .node-label { font-size: 12px; text-anchor: middle; }
            .edge { stroke: #6b7280; stroke-width: 2; }
        </style>",
        );
        svg.push('\n');

        // Get graph data
        let graph = match self.graph.read() {
            Ok(guard) => guard,
            Err(e) => return Err(anyhow::anyhow!("graph lock poisoned: {e}")),
        };
        let nodes = graph.nodes();
        let edges = graph.edges();

        // Render edges first (so they appear behind nodes)
        for edge in edges {
            let from_pos = nodes
                .iter()
                .find(|n| n.info.id == edge.from)
                .map_or((self.width as f32 / 2.0, self.height as f32 / 2.0), |n| {
                    (n.position.x, n.position.y)
                });

            let to_pos = nodes
                .iter()
                .find(|n| n.info.id == edge.to)
                .map_or((self.width as f32 / 2.0, self.height as f32 / 2.0), |n| {
                    (n.position.x, n.position.y)
                });

            svg.push_str(&format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" class="edge"/>"#,
                from_pos.0, from_pos.1, to_pos.0, to_pos.1
            ));
            svg.push('\n');
        }

        // Render nodes
        for node in nodes {
            let (x, y) = (node.position.x, node.position.y);
            let color = health_to_color(&node.info.health);

            // Node circle
            svg.push_str(&format!(
                r##"<circle cx="{}" cy="{}" r="{}" fill="{}" stroke="#f3f4f6" stroke-width="2"/>"##,
                x, y, self.node_radius, color
            ));
            svg.push('\n');

            // Node label
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" class="node-label">{}</text>"#,
                x,
                y + self.node_radius + 15.0,
                node.info.name
            ));
            svg.push('\n');
        }

        // Title
        svg.push_str(&format!(
            r#"<text x="20" y="30" style="font-size: 18px; font-weight: bold;">🌸 petalTongue Topology - {} primals</text>"#,
            nodes.len()
        ));
        svg.push('\n');

        // Footer
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        svg.push_str(&format!(
            r#"<text x="20" y="{}" style="font-size: 10px; fill: #9ca3af;">Generated: {}</text>"#,
            self.height - 10,
            timestamp
        ));
        svg.push('\n');

        svg.push_str("</svg>");

        Ok(svg)
    }
}

impl UniversalUI for SvgUI {
    fn mode_name(&self) -> &'static str {
        "SVG"
    }

    fn supports(&self, capability: UICapability) -> bool {
        matches!(
            capability,
            UICapability::RenderToString | UICapability::Export
        )
    }

    fn render_to_string(&self) -> Result<String> {
        self.render_svg()
    }

    fn render_to_bytes(&self) -> Result<bytes::Bytes> {
        Ok(bytes::Bytes::from(self.render_svg()?.into_bytes()))
    }

    fn recommended_format(&self) -> ExportFormat {
        ExportFormat::Svg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_svg_ui_creation() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = SvgUI::new(graph, 800, 600);
        assert_eq!(ui.width, 800);
        assert_eq!(ui.height, 600);
    }

    #[test]
    fn test_svg_ui_render() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = SvgUI::new(graph, 800, 600);

        let result = ui.render_to_string();
        assert!(result.is_ok());

        let svg = result.unwrap();
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("800"));
        assert!(svg.contains("600"));
    }

    #[test]
    fn test_svg_ui_capabilities() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = SvgUI::new(graph, 800, 600);

        assert!(ui.supports(UICapability::RenderToString));
        assert!(ui.supports(UICapability::Export));
        assert!(!ui.supports(UICapability::Interactive));
    }
}
