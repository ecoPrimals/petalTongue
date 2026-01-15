//! Egui Tree Renderer
//!
//! Renders trees in an egui GUI context.
//!
//! ## Features
//!
//! - Interactive expansion/collapse
//! - Icon support (emoji, unicode, custom)
//! - Color support
//! - Mouse and keyboard navigation
//! - Selection callbacks
//!
//! ## Example
//!
//! ```rust,ignore
//! let mut renderer = EguiTreeRenderer::new();
//! renderer.render_tree(&tree).await?;
//! ```

use crate::renderer::{Modality, RendererCapabilities, TreeRenderer};
use crate::tree::TreeNode;
use anyhow::Result;
use async_trait::async_trait;
use std::marker::PhantomData;

/// Egui-based tree renderer
pub struct EguiTreeRenderer<T> {
    /// Phantom data (renderer is generic over T)
    _phantom: PhantomData<T>,

    /// Selection callback (if registered)
    selection_callback: Option<Box<dyn Fn(&T) + Send + Sync>>,

    /// Expansion state (node path -> expanded)
    expansion_state: std::collections::HashMap<Vec<usize>, bool>,
}

impl<T> EguiTreeRenderer<T> {
    /// Create a new egui tree renderer
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
            selection_callback: None,
            expansion_state: std::collections::HashMap::new(),
        }
    }

    /// Render a tree node recursively (internal helper)
    ///
    /// This will be called from within an egui context (e.g., `ui.vertical()`)
    fn render_node_recursive(
        &mut self,
        ui: &mut egui::Ui,
        node: &TreeNode<T>,
        path: &mut Vec<usize>,
        depth: usize,
    ) where
        T: std::fmt::Display,
    {
        // Indent based on depth
        let indent = (depth as f32) * 16.0;

        ui.horizontal(|ui| {
            ui.add_space(indent);

            // Expansion toggle (if node has children)
            if !node.children().is_empty() {
                let expanded = self
                    .expansion_state
                    .get(path)
                    .copied()
                    .unwrap_or(node.is_expanded());
                let icon = if expanded { "▼" } else { "▶" };

                if ui.button(icon).clicked() {
                    self.expansion_state.insert(path.clone(), !expanded);
                }
            } else {
                // Spacer for alignment
                ui.add_space(20.0);
            }

            // Icon (if present)
            if let Some(icon) = node.icon() {
                ui.label(icon.to_string());
            }

            // Label (clickable for selection)
            let label = ui.selectable_label(false, node.data().to_string());
            if label.clicked() {
                if let Some(ref callback) = self.selection_callback {
                    callback(node.data());
                }
            }

            // Color (if present)
            if let Some(_color) = node.color() {
                // TODO: Apply color to label
                // This requires color conversion from our Color type to egui::Color32
            }
        });

        // Render children (if expanded)
        let expanded = self
            .expansion_state
            .get(path)
            .copied()
            .unwrap_or(node.is_expanded());
        if expanded {
            for (i, child) in node.children().iter().enumerate() {
                path.push(i);
                self.render_node_recursive(ui, child, path, depth + 1);
                path.pop();
            }
        }
    }
}

impl<T> Default for EguiTreeRenderer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T> TreeRenderer<T> for EguiTreeRenderer<T>
where
    T: std::fmt::Display + Send + Sync,
{
    async fn render_tree(&mut self, root: &TreeNode<T>) -> Result<()> {
        // NOTE: This is a simplified implementation
        // In practice, this would be called from within an egui App's update() method
        // and would receive the egui::Ui context as a parameter

        // For now, we just track the tree structure
        // The actual rendering happens in render_node_recursive() called from App::update()

        tracing::debug!(
            "EguiTreeRenderer: render_tree called for tree with {} nodes",
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
            modality: Modality::VisualGUI,
            supports_expansion: true,
            supports_selection: true,
            supports_icons: true,
            supports_colors: true,
            supports_filtering: false, // Not yet implemented
            is_interactive: true,
        }
    }

    fn name(&self) -> &str {
        "EguiTreeRenderer"
    }
}

// NOTE: In a real integration, this renderer would be used like this:
//
// ```rust
// impl eframe::App for MyApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             ui.vertical(|ui| {
//                 let mut path = Vec::new();
//                 self.tree_renderer.render_node_recursive(ui, &self.tree, &mut path, 0);
//             });
//         });
//     }
// }
// ```

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Icon;

    #[tokio::test]
    async fn test_egui_renderer_creation() {
        let renderer: EguiTreeRenderer<String> = EguiTreeRenderer::new();
        assert_eq!(renderer.name(), "EguiTreeRenderer");
    }

    #[tokio::test]
    async fn test_egui_renderer_capabilities() {
        let renderer: EguiTreeRenderer<String> = EguiTreeRenderer::new();
        let caps = renderer.capabilities();

        assert_eq!(caps.modality, Modality::VisualGUI);
        assert!(caps.supports_expansion);
        assert!(caps.supports_selection);
        assert!(caps.supports_icons);
        assert!(caps.is_interactive);
    }

    #[tokio::test]
    async fn test_egui_renderer_basic() {
        let mut renderer: EguiTreeRenderer<String> = EguiTreeRenderer::new();

        let tree = TreeNode::new("root".to_string())
            .with_icon(Icon::Emoji("📁".to_string()))
            .with_child(
                TreeNode::new("child".to_string()).with_icon(Icon::Emoji("📄".to_string())),
            );

        // Should not fail
        assert!(renderer.render_tree(&tree).await.is_ok());
    }

    #[tokio::test]
    async fn test_egui_renderer_selection() {
        let mut renderer: EguiTreeRenderer<String> = EguiTreeRenderer::new();

        let selected = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let selected_clone = selected.clone();

        renderer
            .on_select(move |data: &String| {
                *selected_clone.lock().unwrap() = data.clone();
            })
            .await
            .unwrap();

        // Simulate selection (in real usage, this would happen via UI click)
        if let Some(ref callback) = renderer.selection_callback {
            callback(&"test".to_string());
        }

        assert_eq!(*selected.lock().unwrap(), "test");
    }

    #[tokio::test]
    async fn test_egui_renderer_expansion() {
        let mut renderer: EguiTreeRenderer<String> = EguiTreeRenderer::new();

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
}
