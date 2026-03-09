// SPDX-License-Identifier: AGPL-3.0-only
//! Terminal UI (TUI) implementation
//!
//! Interactive terminal interface using crossterm (pure Rust, cross-platform).
//! Works over SSH, serial connections, and any terminal emulator.

use crate::trait_def::{ExportFormat, UICapability, UniversalUI};
use crate::utils::{health_to_emoji, health_to_percentage};
use anyhow::Result;
use petal_tongue_core::GraphEngine;
use std::sync::{Arc, RwLock};

/// Terminal UI renderer (pure Rust, cross-platform)
///
/// Provides an ASCII art visualization of primal topologies.
/// Works in any terminal, over SSH, and in headless environments.
///
/// # Examples
///
/// ```no_run
/// use petal_tongue_ui_core::TerminalUI;
/// use petal_tongue_core::GraphEngine;
/// use std::sync::{Arc, RwLock};
///
/// # fn main() -> anyhow::Result<()> {
/// let graph = Arc::new(RwLock::new(GraphEngine::new()));
/// let ui = TerminalUI::new(graph);
///
/// // Render to string (for display)
/// use petal_tongue_ui_core::UniversalUI;
/// let output = ui.render_to_string()?;
/// println!("{}", output);
/// # Ok(())
/// # }
/// ```
pub struct TerminalUI {
    graph: Arc<RwLock<GraphEngine>>,
    width: usize,
}

impl TerminalUI {
    /// Create a new Terminal UI
    pub fn new(graph: Arc<RwLock<GraphEngine>>) -> Self {
        // Try to detect terminal width
        let width = terminal_size::terminal_size()
            .map_or(80, |(terminal_size::Width(w), _)| w as usize)
            .min(120); // Cap at 120 for readability

        Self { graph, width }
    }

    /// Create with explicit width
    #[must_use]
    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    /// Render terminal output
    fn render_terminal(&self) -> Result<String> {
        let mut output = String::new();

        // Header
        let title = "🌸 petalTongue Topology";
        let padding = (self.width.saturating_sub(title.len())) / 2;
        output.push_str(&" ".repeat(padding));
        output.push_str(title);
        output.push('\n');
        output.push_str(&"═".repeat(self.width));
        output.push('\n');
        output.push('\n');

        // Get graph data
        let graph = self
            .graph
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let nodes = graph.nodes();
        let edges = graph.edges();

        // Primals section
        output.push_str("  PRIMALS:\n");
        output.push_str(&"─".repeat(self.width));
        output.push('\n');

        if nodes.is_empty() {
            output.push_str("  (No primals discovered)\n");
        } else {
            for node in nodes {
                let health_icon = health_to_emoji(&node.info.health);
                let health_pct = health_to_percentage(&node.info.health);
                let name_width = self.width.saturating_sub(25);
                let name = if node.info.name.len() > name_width {
                    format!("{}...", &node.info.name[..name_width.saturating_sub(3)])
                } else {
                    node.info.name.clone()
                };

                output.push_str(&format!(
                    "  {health_icon} {name:<name_width$} Health: {health_pct}%\n"
                ));
            }
        }

        output.push('\n');

        // Connections section
        output.push_str("  CONNECTIONS:\n");
        output.push_str(&"─".repeat(self.width));
        output.push('\n');

        if edges.is_empty() {
            output.push_str("  (No connections)\n");
        } else {
            for edge in edges {
                let from_name = nodes
                    .iter()
                    .find(|n| n.info.id == edge.from)
                    .map_or("unknown", |n| n.info.name.as_str());

                let to_name = nodes
                    .iter()
                    .find(|n| n.info.id == edge.to)
                    .map_or("unknown", |n| n.info.name.as_str());

                let max_name_len = (self.width.saturating_sub(10)) / 2;
                let from = if from_name.len() > max_name_len {
                    format!("{}...", &from_name[..max_name_len.saturating_sub(3)])
                } else {
                    from_name.to_string()
                };
                let to = if to_name.len() > max_name_len {
                    format!("{}...", &to_name[..max_name_len.saturating_sub(3)])
                } else {
                    to_name.to_string()
                };

                output.push_str(&format!("  {from} ──→ {to}\n"));
            }
        }

        output.push('\n');

        // Summary
        output.push_str(&"═".repeat(self.width));
        output.push('\n');
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
        output.push_str(&format!(
            "  {} primals, {} connections | {}\n",
            nodes.len(),
            edges.len(),
            timestamp
        ));

        Ok(output)
    }
}

impl UniversalUI for TerminalUI {
    fn mode_name(&self) -> &'static str {
        "Terminal"
    }

    fn supports(&self, capability: UICapability) -> bool {
        matches!(
            capability,
            UICapability::RenderToString | UICapability::Export
        )
    }

    fn render_to_string(&self) -> Result<String> {
        self.render_terminal()
    }

    fn render_to_bytes(&self) -> Result<Vec<u8>> {
        Ok(self.render_terminal()?.into_bytes())
    }

    fn recommended_format(&self) -> ExportFormat {
        ExportFormat::Text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_ui_creation() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = TerminalUI::new(graph);
        assert!(ui.width > 0);
        assert!(ui.width <= 120);
    }

    #[test]
    fn test_terminal_ui_render() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = TerminalUI::new(graph);

        let result = ui.render_to_string();
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.contains("petalTongue"));
        assert!(output.contains("PRIMALS"));
        assert!(output.contains("CONNECTIONS"));
    }

    #[test]
    fn test_terminal_ui_capabilities() {
        let graph = Arc::new(RwLock::new(GraphEngine::new()));
        let ui = TerminalUI::new(graph);

        assert!(ui.supports(UICapability::RenderToString));
        assert!(ui.supports(UICapability::Export));
        assert!(!ui.supports(UICapability::Interactive));
    }
}
