// SPDX-License-Identifier: AGPL-3.0-only
//! Canvas/PNG rendering (pure Rust)
//!
//! Pixel-perfect rendering using tiny-skia (no native dependencies).
//! Exports to PNG for reports, embedding, and automation.

use crate::trait_def::{ExportFormat, UICapability, UniversalUI};
use anyhow::Result;
use petal_tongue_core::GraphEngine;
use std::sync::{Arc, RwLock};

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
#[allow(dead_code)]
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
    /// TODO: Implement full rendering with tiny-skia
    /// For now, returns a placeholder
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn render_png(&self) -> Result<Vec<u8>> {
        tracing::warn!("Canvas rendering not fully implemented yet - generating placeholder");

        // For now, create a simple placeholder PNG
        // Full implementation will use tiny-skia for proper rendering

        // Minimal valid PNG (1x1 transparent pixel)
        Ok(vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
            0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, // 1x1
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49,
            0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D,
            0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60,
            0x82,
        ])
    }
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
    fn test_canvas_render_placeholder() {
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
    }
}
