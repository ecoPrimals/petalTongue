// SPDX-License-Identifier: AGPL-3.0-only
//! Text/JSON/DOT export
//!
//! Exports primal topologies in text-based formats for APIs, automation, and tools.

use crate::trait_def::{ExportFormat, UICapability, UniversalUI};
use crate::utils::{get_family_lineage, get_trust_level, health_to_color, health_to_percentage};
use anyhow::Result;
use petal_tongue_core::GraphEngine;
use std::sync::{Arc, RwLock};

/// Text UI (plain text, JSON, DOT formats)
///
/// Exports primal topology in text-based formats suitable for:
/// - Plain text: Human-readable reports
/// - JSON: API consumption, data exchange
/// - DOT: Graphviz visualization
///
/// # Examples
///
/// ```no_run
/// use petal_tongue_ui_core::{TextUI, UniversalUI, ExportFormat};
/// use petal_tongue_core::GraphEngine;
/// use std::sync::{Arc, RwLock};
/// use std::path::Path;
///
/// # fn main() -> anyhow::Result<()> {
/// let graph = Arc::new(RwLock::new(GraphEngine::new()));
/// let ui = TextUI::new(graph);
///
/// // Export as JSON
/// ui.export(Path::new("topology.json"), ExportFormat::Json)?;
///
/// // Export as DOT (for graphviz)
/// ui.export(Path::new("topology.dot"), ExportFormat::Dot)?;
/// # Ok(())
/// # }
/// ```
pub struct TextUI {
    graph: Arc<RwLock<GraphEngine>>,
    format: ExportFormat,
}

impl TextUI {
    /// Create a new Text UI
    pub fn new(graph: Arc<RwLock<GraphEngine>>) -> Self {
        Self {
            graph,
            format: ExportFormat::Text,
        }
    }

    /// Set export format
    pub fn with_format(mut self, format: ExportFormat) -> Self {
        self.format = format;
        self
    }

    /// Render as plain text
    fn render_text(&self) -> Result<String> {
        let mut output = String::new();

        output.push_str("petalTongue Topology Report\n");
        output.push_str("===========================\n\n");

        let graph = self.graph.read().unwrap();
        let nodes = graph.nodes();
        let edges = graph.edges();

        // Primals
        output.push_str("PRIMALS:\n");
        output.push_str("--------\n");
        for node in nodes.iter() {
            let health_pct = health_to_percentage(&node.info.health);
            let trust = get_trust_level(&node.info);
            let family = get_family_lineage(&node.info);
            output.push_str(&format!(
                "  • {} (health: {}%, trust: {}, family: {})\n",
                node.info.name, health_pct, trust, family
            ));
        }
        output.push('\n');

        // Connections
        output.push_str("CONNECTIONS:\n");
        output.push_str("------------\n");
        for edge in edges.iter() {
            let from_name = nodes
                .iter()
                .find(|n| n.info.id == edge.from)
                .map(|n| n.info.name.as_str())
                .unwrap_or("unknown");
            let to_name = nodes
                .iter()
                .find(|n| n.info.id == edge.to)
                .map(|n| n.info.name.as_str())
                .unwrap_or("unknown");

            output.push_str(&format!(
                "  • {} → {} ({})\n",
                from_name, to_name, edge.edge_type
            ));
        }
        output.push('\n');

        // Summary
        output.push_str("SUMMARY:\n");
        output.push_str("--------\n");
        output.push_str(&format!("  Total primals: {}\n", nodes.len()));
        output.push_str(&format!("  Total connections: {}\n", edges.len()));

        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        output.push_str(&format!("  Generated: {timestamp}\n"));

        Ok(output)
    }

    /// Render as JSON
    fn render_json(&self) -> Result<String> {
        let graph = self.graph.read().unwrap();

        // Create a simplified structure for JSON export
        let data = serde_json::json!({
            "topology": {
                "primals": graph.nodes().iter().map(|n| {
                    let health_pct = health_to_percentage(&n.info.health);
                    let trust = get_trust_level(&n.info);
                    let family = get_family_lineage(&n.info);
                    serde_json::json!({
                        "id": n.info.id,
                        "name": n.info.name,
                        "health": health_pct,
                        "health_status": n.info.health.as_str(),
                        "trust": trust,
                        "family": family,
                        "position": {
                            "x": n.position.x,
                            "y": n.position.y,
                        }
                    })
                }).collect::<Vec<_>>(),
                "connections": graph.edges().iter().map(|e| {
                    serde_json::json!({
                        "from": e.from,
                        "to": e.to,
                        "type": e.edge_type,
                    })
                }).collect::<Vec<_>>(),
            },
            "metadata": {
                "generated_at": chrono::Utc::now().to_rfc3339(),
                "primal_count": graph.nodes().len(),
                "connection_count": graph.edges().len(),
            }
        });

        Ok(serde_json::to_string_pretty(&data)?)
    }

    /// Render as DOT (Graphviz)
    fn render_dot(&self) -> Result<String> {
        let mut dot = String::new();

        dot.push_str("digraph PetalTongue {\n");
        dot.push_str("  // Graph attributes\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  bgcolor=\"#141822\";\n");
        dot.push_str("  node [style=filled, fontcolor=white, fontname=\"sans-serif\"];\n");
        dot.push_str("  edge [color=\"#6b7280\"];\n\n");

        let graph = self.graph.read().unwrap();
        let nodes = graph.nodes();
        let edges = graph.edges();

        // Nodes
        dot.push_str("  // Primals\n");
        for node in nodes.iter() {
            let color = health_to_color(&node.info.health);
            let health_pct = health_to_percentage(&node.info.health);

            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\n{}%\", fillcolor=\"{}\"];\n",
                node.info.id, node.info.name, health_pct, color
            ));
        }

        dot.push_str("\n  // Connections\n");
        for edge in edges.iter() {
            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                edge.from, edge.to, edge.edge_type
            ));
        }

        dot.push_str("}\n");

        Ok(dot)
    }
}

impl UniversalUI for TextUI {
    fn mode_name(&self) -> &str {
        match self.format {
            ExportFormat::Text => "Text",
            ExportFormat::Json => "JSON",
            ExportFormat::Dot => "DOT",
            _ => "Text",
        }
    }

    fn supports(&self, capability: UICapability) -> bool {
        matches!(
            capability,
            UICapability::RenderToString | UICapability::Export
        )
    }

    fn render_to_string(&self) -> Result<String> {
        match self.format {
            ExportFormat::Json => self.render_json(),
            ExportFormat::Dot => self.render_dot(),
            _ => self.render_text(),
        }
    }

    fn render_to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.render_to_string()?.into_bytes())
    }

    fn recommended_format(&self) -> ExportFormat {
        self.format
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_ui_creation() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = TextUI::new(graph);
        assert_eq!(ui.mode_name(), "Text");
    }

    #[test]
    fn test_text_render() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = TextUI::new(graph);

        let result = ui.render_to_string();
        assert!(result.is_ok());

        let text = result.unwrap();
        assert!(text.contains("petalTongue Topology"));
        assert!(text.contains("PRIMALS"));
        assert!(text.contains("CONNECTIONS"));
    }

    #[test]
    fn test_json_render() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = TextUI::new(graph).with_format(ExportFormat::Json);

        let result = ui.render_to_string();
        assert!(result.is_ok());

        let json = result.unwrap();
        assert!(json.contains("topology"));
        assert!(json.contains("primals"));
        assert!(json.contains("connections"));

        // Verify it's valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&json).is_ok());
    }

    #[test]
    fn test_dot_render() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = TextUI::new(graph).with_format(ExportFormat::Dot);

        let result = ui.render_to_string();
        assert!(result.is_ok());

        let dot = result.unwrap();
        assert!(dot.contains("digraph"));
        assert!(dot.contains("PetalTongue"));
    }
}
