// SPDX-License-Identifier: AGPL-3.0-only
//! Egui Panel Renderer
//!
//! Renders panel layouts in an egui GUI context.

use crate::panel::Panel;
use crate::renderer::{Modality, PanelRenderer, RendererCapabilities};
use anyhow::Result;
use async_trait::async_trait;
use std::marker::PhantomData;

/// Egui-based panel renderer
pub struct EguiPanelRenderer<T> {
    _phantom: PhantomData<T>,
}

impl<T> EguiPanelRenderer<T> {
    /// Create a new egui panel renderer
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for EguiPanelRenderer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T> PanelRenderer<T> for EguiPanelRenderer<T>
where
    T: Send + Sync,
{
    async fn render_panel(&mut self, _panel: &Panel<T>) -> Result<()> {
        tracing::debug!("EguiPanelRenderer: render_panel called");
        Ok(())
    }

    fn capabilities(&self) -> RendererCapabilities {
        RendererCapabilities {
            modality: Modality::VisualGUI,
            supports_expansion: true,
            supports_selection: true,
            supports_icons: true,
            supports_colors: true,
            supports_filtering: false,
            is_interactive: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::panel::{Direction, PanelContent};

    #[tokio::test]
    async fn test_egui_panel_renderer() {
        let mut renderer: EguiPanelRenderer<String> = EguiPanelRenderer::new();

        let panel = Panel::split(
            Direction::Horizontal,
            0.5,
            Panel::Leaf(PanelContent::new("left", "Left", "Content".to_string())),
            Panel::Leaf(PanelContent::new("right", "Right", "Content".to_string())),
        );

        assert!(renderer.render_panel(&panel).await.is_ok());
    }
}
