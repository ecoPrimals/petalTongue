//! Ratatui Tree Renderer
//!
//! Renders trees in a terminal UI (TUI) context.
//!
//! ## Features
//!
//! - Interactive expansion/collapse (keyboard)
//! - ASCII/Unicode icons
//! - Color support (terminal colors)
//! - Keyboard navigation
//! - Selection callbacks
//!
//! ## Example
//!
//! ```rust,ignore
//! let mut renderer = RatatuiTreeRenderer::new();
//! renderer.render_tree(&tree).await?;
//! ```

use crate::renderer::{Modality, RendererCapabilities, TreeRenderer};
use crate::tree::TreeNode;
use anyhow::Result;
use async_trait::async_trait;
use std::marker::PhantomData;

/// Ratatui-based tree renderer
pub struct RatatuiTreeRenderer<T> {
    /// Phantom data (renderer is generic over T)
    _phantom: PhantomData<T>,

    /// Selection callback (if registered)
    selection_callback: Option<Box<dyn Fn(&T) + Send + Sync>>,

    /// Expansion state (node path -> expanded)
    expansion_state: std::collections::HashMap<Vec<usize>, bool>,

    /// Selected node path
    selected_path: Vec<usize>,
}

impl<T> RatatuiTreeRenderer<T> {
    /// Create a new ratatui tree renderer
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
            selection_callback: None,
            expansion_state: std::collections::HashMap::new(),
            selected_path: Vec::new(),
        }
    }

    /// Render tree as text lines (for TUI frameworks)
    ///
    /// Returns a Vec of (line_text, is_selected, node_path) tuples
    fn render_to_lines(
        &self,
        node: &TreeNode<T>,
        path: &mut Vec<usize>,
        depth: usize,
    ) -> Vec<(String, bool, Vec<usize>)>
    where
        T: std::fmt::Display,
    {
        let mut lines = Vec::new();

        // Build the line for this node
        let mut line = String::new();

        // Indent
        for _ in 0..depth {
            line.push_str("  ");
        }

        // Expansion indicator (if node has children)
        if !node.children().is_empty() {
            let expanded = self
                .expansion_state
                .get(path)
                .copied()
                .unwrap_or(node.is_expanded());
            line.push_str(if expanded { "▼ " } else { "▶ " });
        } else {
            line.push_str("  ");
        }

        // Icon (if present)
        if let Some(icon) = node.icon() {
            line.push_str(&icon.to_string());
            line.push(' ');
        }

        // Label
        line.push_str(&node.data().to_string());

        // Is this line selected?
        let is_selected = path == &self.selected_path;

        lines.push((line, is_selected, path.clone()));

        // Render children (if expanded)
        let expanded = self
            .expansion_state
            .get(path)
            .copied()
            .unwrap_or(node.is_expanded());
        if expanded {
            for (i, child) in node.children().iter().enumerate() {
                path.push(i);
                lines.extend(self.render_to_lines(child, path, depth + 1));
                path.pop();
            }
        }

        lines
    }

    /// Move selection up
    pub fn select_previous(&mut self) {
        // Simple implementation: decrement last path component
        if let Some(last) = self.selected_path.last_mut() {
            if *last > 0 {
                *last -= 1;
            }
        }
    }

    /// Move selection down
    pub fn select_next(&mut self) {
        // Simple implementation: increment last path component
        if let Some(last) = self.selected_path.last_mut() {
            *last += 1;
        } else if self.selected_path.is_empty() {
            self.selected_path.push(0);
        }
    }

    /// Get selected path
    pub fn selected_path(&self) -> &[usize] {
        &self.selected_path
    }
}

impl<T> Default for RatatuiTreeRenderer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T> TreeRenderer<T> for RatatuiTreeRenderer<T>
where
    T: std::fmt::Display + Send + Sync,
{
    async fn render_tree(&mut self, root: &TreeNode<T>) -> Result<()> {
        // NOTE: This is a simplified implementation
        // In practice, this would be integrated with a ratatui Frame
        // and would use the render_to_lines() method to build the display

        tracing::debug!(
            "RatatuiTreeRenderer: render_tree called for tree with {} nodes",
            root.count_nodes()
        );

        Ok(())
    }

    async fn on_select<F>(&mut self, callback: F) -> Result<()>
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.selection_callback = Some(Box::new(callback));
        Ok(())
    }

    async fn on_toggle(&mut self, node_path: &[usize]) -> Result<()> {
        // Toggle expansion state
        let expanded = self
            .expansion_state
            .get(node_path)
            .copied()
            .unwrap_or(false);
        self.expansion_state.insert(node_path.to_vec(), !expanded);
        Ok(())
    }

    fn capabilities(&self) -> RendererCapabilities {
        RendererCapabilities {
            modality: Modality::TerminalTUI,
            supports_expansion: true,
            supports_selection: true,
            supports_icons: true,      // ASCII/Unicode icons
            supports_colors: true,     // Terminal colors
            supports_filtering: false, // Not yet implemented
            is_interactive: true,
        }
    }

    fn name(&self) -> &str {
        "RatatuiTreeRenderer"
    }
}

// NOTE: In a real integration, this renderer would be used like this:
//
// ```rust
// fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
//     let mut lines = Vec::new();
//     let mut path = Vec::new();
//
//     let rendered = app.tree_renderer.render_to_lines(&app.tree, &mut path, 0);
//
//     for (line, is_selected, _path) in rendered {
//         let style = if is_selected {
//             Style::default().bg(Color::Blue).fg(Color::White)
//         } else {
//             Style::default()
//         };
//         lines.push(Line::from(Span::styled(line, style)));
//     }
//
//     let paragraph = Paragraph::new(lines)
//         .block(Block::default().borders(Borders::ALL).title("Tree"));
//     f.render_widget(paragraph, f.size());
// }
// ```

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Icon;

    #[tokio::test]
    async fn test_ratatui_renderer_creation() {
        let renderer: RatatuiTreeRenderer<String> = RatatuiTreeRenderer::new();
        assert_eq!(renderer.name(), "RatatuiTreeRenderer");
    }

    #[tokio::test]
    async fn test_ratatui_renderer_capabilities() {
        let renderer: RatatuiTreeRenderer<String> = RatatuiTreeRenderer::new();
        let caps = renderer.capabilities();

        assert_eq!(caps.modality, Modality::TerminalTUI);
        assert!(caps.supports_expansion);
        assert!(caps.supports_selection);
        assert!(caps.supports_icons);
        assert!(caps.is_interactive);
    }

    #[tokio::test]
    async fn test_ratatui_renderer_basic() {
        let mut renderer: RatatuiTreeRenderer<String> = RatatuiTreeRenderer::new();

        let tree = TreeNode::new("root".to_string())
            .with_icon(Icon::Emoji("📁".to_string()))
            .with_child(
                TreeNode::new("child".to_string()).with_icon(Icon::Emoji("📄".to_string())),
            );

        // Should not fail
        assert!(renderer.render_tree(&tree).await.is_ok());
    }

    #[tokio::test]
    async fn test_ratatui_renderer_selection() {
        let mut renderer: RatatuiTreeRenderer<String> = RatatuiTreeRenderer::new();

        let selected = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let selected_clone = selected.clone();

        renderer
            .on_select(move |data: &String| {
                *selected_clone.lock().unwrap() = data.clone();
            })
            .await
            .unwrap();

        // Simulate selection
        if let Some(ref callback) = renderer.selection_callback {
            callback(&"test".to_string());
        }

        assert_eq!(*selected.lock().unwrap(), "test");
    }

    #[tokio::test]
    async fn test_ratatui_renderer_expansion() {
        let mut renderer: RatatuiTreeRenderer<String> = RatatuiTreeRenderer::new();

        let path = vec![0, 1];

        // Initially not expanded
        assert!(!renderer.expansion_state.contains_key(&path));

        // Toggle expansion
        renderer.on_toggle(&path).await.unwrap();
        assert_eq!(renderer.expansion_state.get(&path), Some(&true));

        // Toggle again
        renderer.on_toggle(&path).await.unwrap();
        assert_eq!(renderer.expansion_state.get(&path), Some(&false));
    }

    #[tokio::test]
    async fn test_ratatui_renderer_navigation() {
        let mut renderer: RatatuiTreeRenderer<String> = RatatuiTreeRenderer::new();

        // Initially no selection
        assert_eq!(renderer.selected_path(), &[] as &[usize]);

        // Select next (should create first selection)
        renderer.select_next();
        assert_eq!(renderer.selected_path(), &[0]);

        // Select next again
        renderer.select_next();
        assert_eq!(renderer.selected_path(), &[1]);

        // Select previous
        renderer.select_previous();
        assert_eq!(renderer.selected_path(), &[0]);

        // Select previous again (should stay at 0)
        renderer.select_previous();
        assert_eq!(renderer.selected_path(), &[0]);
    }

    #[tokio::test]
    async fn test_ratatui_render_to_lines() {
        let renderer: RatatuiTreeRenderer<String> = RatatuiTreeRenderer::new();

        let tree = TreeNode::new("root".to_string())
            .with_icon(Icon::Emoji("📁".to_string()))
            .expanded(true)
            .with_child(
                TreeNode::new("child1".to_string()).with_icon(Icon::Emoji("📄".to_string())),
            )
            .with_child(
                TreeNode::new("child2".to_string()).with_icon(Icon::Emoji("📄".to_string())),
            );

        let mut path = Vec::new();
        let lines = renderer.render_to_lines(&tree, &mut path, 0);

        // Should have 3 lines (root + 2 children)
        assert_eq!(lines.len(), 3);

        // Check line contents
        assert!(lines[0].0.contains("root"));
        assert!(lines[1].0.contains("child1"));
        assert!(lines[2].0.contains("child2"));

        // Check icons are present
        assert!(lines[0].0.contains("📁"));
        assert!(lines[1].0.contains("📄"));
        assert!(lines[2].0.contains("📄"));
    }
}
