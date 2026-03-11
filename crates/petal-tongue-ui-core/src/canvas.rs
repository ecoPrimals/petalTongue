// SPDX-License-Identifier: AGPL-3.0-only
//! Canvas/PNG rendering (pure Rust)
//!
//! Pixel-perfect rendering using tiny-skia (no native dependencies).
//! Exports to PNG for reports, embedding, and automation.

use crate::trait_def::{ExportFormat, UICapability, UniversalUI};
use crate::utils::health_to_color;
use anyhow::Result;
use petal_tongue_core::GraphEngine;
use std::sync::{Arc, RwLock};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

/// Canvas UI renderer (pure Rust, zero native dependencies)
///
/// Renders primal topologies to pixel buffers using tiny-skia.
/// Can export to PNG format for reports and automation.
///
/// # Examples
///
/// ```no_run
/// use petal_tongue_ui_core::{CanvasUI, UniversalUI, ExportFormat};
/// use petal_tongue_core::GraphEngine;
/// use std::sync::{Arc, RwLock};
/// use std::path::Path;
///
/// # fn main() -> anyhow::Result<()> {
/// let graph = Arc::new(RwLock::new(GraphEngine::new()));
/// let ui = CanvasUI::new(graph, 1920, 1080);
///
/// // Export to PNG
/// ui.export(Path::new("topology.png"), ExportFormat::Png)?;
/// # Ok(())
/// # }
/// ```
pub struct CanvasUI {
    graph: Arc<RwLock<GraphEngine>>,
    width: u32,
    height: u32,
}

impl CanvasUI {
    /// Create a new Canvas UI renderer
    pub fn new(graph: Arc<RwLock<GraphEngine>>, width: u32, height: u32) -> Self {
        Self {
            graph,
            width,
            height,
        }
    }

    /// Render to pixel buffer (PNG format)
    ///
    /// Uses tiny-skia to render the graph topology: nodes as filled circles,
    /// edges as lines. Transforms graph coordinates to fit the canvas with padding.
    fn render_png(&self) -> Result<Vec<u8>> {
        let graph = match self.graph.read() {
            Ok(guard) => guard,
            Err(e) => return Err(anyhow::anyhow!("graph lock poisoned: {e}")),
        };

        let nodes = graph.nodes();
        let edges = graph.edges();

        let mut pixmap = Pixmap::new(self.width, self.height).ok_or_else(|| {
            anyhow::anyhow!("Failed to create pixmap {}x{}", self.width, self.height)
        })?;

        // Dark background (match SVG style)
        let bg = Color::from_rgba8(20, 24, 34, 255);
        pixmap.fill(bg);

        if nodes.is_empty() {
            return pixmap
                .encode_png()
                .map_err(|e| anyhow::anyhow!("PNG encode failed: {e}"));
        }

        // Compute bounds and scale to fit canvas with padding
        let padding = 60.0;
        let (min_x, max_x, min_y, max_y) = nodes.iter().fold(
            (f32::MAX, f32::MIN, f32::MAX, f32::MIN),
            |(min_x, max_x, min_y, max_y), n| {
                (
                    min_x.min(n.position.x),
                    max_x.max(n.position.x),
                    min_y.min(n.position.y),
                    max_y.max(n.position.y),
                )
            },
        );
        let range_x = (max_x - min_x).max(1.0);
        let range_y = (max_y - min_y).max(1.0);
        let avail_w = (self.width as f32) - 2.0 * padding;
        let avail_h = (self.height as f32) - 2.0 * padding;
        let scale = (avail_w / range_x).min(avail_h / range_y);
        let offset_x = padding - min_x * scale;
        let offset_y = padding - min_y * scale;

        let to_screen = |x: f32, y: f32| (x * scale + offset_x, y * scale + offset_y);

        let transform = Transform::default();
        let node_radius = 12.0f32.max(scale * 0.05);

        // Draw edges first (behind nodes)
        let mut edge_paint = Paint::default();
        edge_paint.set_color_rgba8(107, 114, 128, 255);
        let stroke = Stroke {
            width: 2.0,
            ..Stroke::default()
        };

        let default_center = (self.width as f32 / 2.0, self.height as f32 / 2.0);
        for edge in edges {
            let from_node = nodes
                .iter()
                .find(|n| n.info.id.as_str() == edge.from.as_str());
            let to_node = nodes
                .iter()
                .find(|n| n.info.id.as_str() == edge.to.as_str());
            let (x1, y1) =
                from_node.map_or(default_center, |n| to_screen(n.position.x, n.position.y));
            let (x2, y2) =
                to_node.map_or(default_center, |n| to_screen(n.position.x, n.position.y));

            let path = {
                let mut pb = tiny_skia::PathBuilder::new();
                pb.move_to(x1, y1);
                pb.line_to(x2, y2);
                pb.finish()
            };
            if let Some(path) = path {
                pixmap.stroke_path(&path, &edge_paint, &stroke, transform, None);
            }
        }

        // Draw nodes as filled circles
        for node in nodes {
            let (cx, cy) = to_screen(node.position.x, node.position.y);
            let hex = health_to_color(&node.info.health);
            let color = parse_hex_color(hex).unwrap_or(Color::from_rgba8(156, 163, 175, 255));

            let path = PathBuilder::from_circle(cx, cy, node_radius);
            if let Some(path) = path {
                let mut paint = Paint::default();
                paint.set_color(color);
                pixmap.fill_path(&path, &paint, FillRule::Winding, transform, None);
            }
        }

        pixmap
            .encode_png()
            .map_err(|e| anyhow::anyhow!("PNG encode failed: {e}"))
    }
}

/// Parse hex color (#RRGGBB) to tiny_skia Color
fn parse_hex_color(hex: &str) -> Option<Color> {
    let s = hex.strip_prefix('#')?.trim();
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::from_rgba8(r, g, b, 255))
}

impl UniversalUI for CanvasUI {
    fn mode_name(&self) -> &'static str {
        "Canvas/PNG"
    }

    fn supports(&self, capability: UICapability) -> bool {
        matches!(
            capability,
            UICapability::RenderToBytes | UICapability::Export
        )
    }

    fn render_to_string(&self) -> Result<String> {
        anyhow::bail!("Canvas UI only supports binary export (PNG)")
    }

    fn render_to_bytes(&self) -> Result<Vec<u8>> {
        self.render_png()
    }

    fn recommended_format(&self) -> ExportFormat {
        ExportFormat::Png
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_core::test_fixtures::primals;
    use petal_tongue_core::{PrimalId, TopologyEdge};

    #[test]
    fn test_canvas_ui_creation() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = CanvasUI::new(graph, 800, 600);
        assert_eq!(ui.width, 800);
        assert_eq!(ui.height, 600);
    }

    #[test]
    fn test_canvas_ui_capabilities() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = CanvasUI::new(graph, 800, 600);

        assert!(ui.supports(UICapability::RenderToBytes));
        assert!(ui.supports(UICapability::Export));
        assert!(!ui.supports(UICapability::RenderToString));
    }

    #[test]
    fn test_canvas_render_png() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = CanvasUI::new(graph, 800, 600);

        let result = ui.render_to_bytes();
        assert!(result.is_ok());

        let png_data = result.unwrap();
        assert!(!png_data.is_empty());
        // PNG signature
        assert_eq!(
            &png_data[0..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );
        // Actual rendering produces much larger PNG than 1x1 placeholder (~68 bytes)
        assert!(
            png_data.len() > 100,
            "PNG should be larger than placeholder (got {} bytes)",
            png_data.len()
        );
    }

    #[test]
    fn test_canvas_render_to_string_returns_error() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = CanvasUI::new(graph, 800, 600);
        let result = ui.render_to_string();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("binary export"));
    }

    #[test]
    fn test_canvas_recommended_format() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = CanvasUI::new(graph, 800, 600);
        assert_eq!(ui.recommended_format(), ExportFormat::Png);
    }

    #[test]
    fn test_canvas_mode_name() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = CanvasUI::new(graph, 800, 600);
        assert_eq!(ui.mode_name(), "Canvas/PNG");
    }

    #[test]
    fn test_canvas_render_with_nodes() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        {
            let mut g = graph.write().expect("lock");
            g.add_node(primals::test_primal("n1"));
            g.add_node(primals::test_primal("n2"));
            g.add_edge(TopologyEdge {
                from: PrimalId::from("n1"),
                to: PrimalId::from("n2"),
                edge_type: "test".to_string(),
                label: None,
                capability: None,
                metrics: None,
            });
        }
        let ui = CanvasUI::new(graph, 400, 300);
        let result = ui.render_to_bytes();
        assert!(result.is_ok());
        let png_data = result.expect("png");
        assert!(!png_data.is_empty());
        assert_eq!(
            &png_data[0..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );
    }
}
