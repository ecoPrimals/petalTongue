//! # SVG GUI Modality
//!
//! Vector export to SVG files (Tier 1: Always Available).

use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;

use petal_tongue_core::{
    engine::UniversalRenderingEngine,
    event::EngineEvent,
    modality::{AccessibilityFeatures, GUIModality, ModalityCapabilities, ModalityTier},
};

/// SVG GUI Modality
///
/// Exports topology as SVG vector graphics.
///
/// **Tier**: 1 (Always Available)
/// **Dependencies**: Zero (pure Rust SVG generation)
/// **Interactive**: No (export-only)
/// **Real-time**: No
pub struct SVGGUI {
    /// Reference to engine
    engine: Option<Arc<UniversalRenderingEngine>>,

    /// Output path
    output_path: PathBuf,

    /// Width
    width: u32,

    /// Height
    height: u32,
}

impl SVGGUI {
    /// Create new SVG GUI modality
    pub fn new(output_path: PathBuf) -> Self {
        Self {
            engine: None,
            output_path,
            width: 800,
            height: 600,
        }
    }

    /// Set canvas size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Generate SVG content
    fn generate_svg(&self) -> Result<String> {
        let mut svg = String::new();

        // SVG header
        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="{}" height="{}" viewBox="0 0 {} {}">
"#,
            self.width, self.height, self.width, self.height
        ));

        // Background
        svg.push_str("  <rect width=\"");
        svg.push_str(&self.width.to_string());
        svg.push_str("\" height=\"");
        svg.push_str(&self.height.to_string());
        svg.push_str("\" fill=\"#1e1e1e\"/>\n");

        // Title
        svg.push_str("  <text x=\"400\" y=\"50\" text-anchor=\"middle\" fill=\"#00ff88\" font-size=\"24\" font-family=\"monospace\">\n");
        svg.push_str("    🌸 petalTongue Topology\n");
        svg.push_str("  </text>\n");

        // TODO: Render actual graph data from engine
        // For now, render placeholder nodes

        // Sample node
        svg.push_str("  <g id=\"node-1\" transform=\"translate(400, 200)\">\n");
        svg.push_str(
            "    <circle r=\"40\" fill=\"#00ff88\" stroke=\"#ffffff\" stroke-width=\"2\"/>\n",
        );
        svg.push_str("    <text y=\"5\" text-anchor=\"middle\" fill=\"#000000\" font-size=\"14\" font-family=\"monospace\">Node 1</text>\n");
        svg.push_str("  </g>\n");

        // Sample edge
        svg.push_str("  <line x1=\"400\" y1=\"240\" x2=\"400\" y2=\"340\" stroke=\"#ffffff\" stroke-width=\"2\" marker-end=\"url(#arrowhead)\"/>\n");

        // Another node
        svg.push_str("  <g id=\"node-2\" transform=\"translate(400, 380)\">\n");
        svg.push_str(
            "    <circle r=\"40\" fill=\"#00ff88\" stroke=\"#ffffff\" stroke-width=\"2\"/>\n",
        );
        svg.push_str("    <text y=\"5\" text-anchor=\"middle\" fill=\"#000000\" font-size=\"14\" font-family=\"monospace\">Node 2</text>\n");
        svg.push_str("  </g>\n");

        // Arrow marker definition
        svg.push_str("  <defs>\n");
        svg.push_str("    <marker id=\"arrowhead\" markerWidth=\"10\" markerHeight=\"10\" refX=\"9\" refY=\"3\" orient=\"auto\">\n");
        svg.push_str("      <polygon points=\"0 0, 10 3, 0 6\" fill=\"#ffffff\"/>\n");
        svg.push_str("    </marker>\n");
        svg.push_str("  </defs>\n");

        // Close SVG
        svg.push_str("</svg>\n");

        Ok(svg)
    }

    /// Write SVG to file
    fn write_svg(&self) -> Result<()> {
        let svg_content = self.generate_svg()?;
        std::fs::write(&self.output_path, svg_content)?;
        tracing::info!("📊 SVG exported to: {}", self.output_path.display());
        Ok(())
    }
}

impl Default for SVGGUI {
    fn default() -> Self {
        Self::new(PathBuf::from("topology.svg"))
    }
}

#[async_trait]
impl GUIModality for SVGGUI {
    fn name(&self) -> &'static str {
        "svg"
    }

    fn is_available(&self) -> bool {
        // SVG export is always available
        true
    }

    fn tier(&self) -> ModalityTier {
        ModalityTier::AlwaysAvailable
    }

    async fn initialize(&mut self, engine: Arc<UniversalRenderingEngine>) -> Result<()> {
        tracing::info!("📊 Initializing SVG GUI");
        self.engine = Some(engine);
        Ok(())
    }

    async fn render(&mut self) -> Result<()> {
        tracing::info!("📊 Rendering to SVG: {}", self.output_path.display());
        self.write_svg()?;
        tracing::info!("✅ SVG export complete");
        Ok(())
    }

    async fn handle_event(&mut self, _event: EngineEvent) -> Result<()> {
        // SVG export doesn't handle real-time events
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("📊 Shutting down SVG GUI");
        Ok(())
    }

    fn capabilities(&self) -> ModalityCapabilities {
        ModalityCapabilities {
            interactive: false,
            realtime: false,
            export: true,
            animation: false,
            three_d: false,
            audio: false,
            haptic: false,
            max_nodes: None, // No limit for SVG
            accessibility: AccessibilityFeatures {
                screen_reader: true, // SVG has semantic markup
                keyboard_only: false,
                high_contrast: false,
                blind_users: false,
                audio_description: false,
                spatial_audio: false,
                aria_labels: true,
                semantic_markup: true,
                wcag_compliant: true,
                gesture_control: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_svg_gui_creation() {
        let gui = SVGGUI::new(PathBuf::from("test.svg"));
        assert_eq!(gui.name(), "svg");
        assert_eq!(gui.tier(), ModalityTier::AlwaysAvailable);
        assert!(gui.is_available());
    }

    #[test]
    fn test_svg_gui_with_size() {
        let gui = SVGGUI::new(PathBuf::from("test.svg")).with_size(1024, 768);
        assert_eq!(gui.width, 1024);
        assert_eq!(gui.height, 768);
    }

    #[test]
    fn test_svg_gui_capabilities() {
        let gui = SVGGUI::new(PathBuf::from("test.svg"));
        let caps = gui.capabilities();

        assert!(!caps.interactive);
        assert!(!caps.realtime);
        assert!(caps.export);
        assert_eq!(caps.max_nodes, None);
    }

    #[test]
    fn test_svg_generation() {
        let gui = SVGGUI::new(PathBuf::from("test.svg"));
        let svg = gui.generate_svg().unwrap();

        assert!(svg.contains("<?xml"));
        assert!(svg.contains("<svg"));
        assert!(svg.contains("petalTongue"));
        assert!(svg.contains("</svg>"));
    }

    #[tokio::test]
    async fn test_svg_export() {
        let temp_path = PathBuf::from("/tmp/test_topology.svg");
        let gui = SVGGUI::new(temp_path.clone());

        gui.write_svg().unwrap();

        assert!(temp_path.exists());
        let content = fs::read_to_string(&temp_path).unwrap();
        assert!(content.contains("petalTongue"));

        // Cleanup
        let _ = fs::remove_file(&temp_path);
    }
}
