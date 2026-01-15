//! Ratatui Panel Renderer
//!
//! Renders panel layouts in a terminal UI (TUI) context.

use crate::panel::Panel;
use crate::renderer::{Modality, PanelRenderer, RendererCapabilities};
use anyhow::Result;
use async_trait::async_trait;
use std::marker::PhantomData;

/// Ratatui-based panel renderer
pub struct RatatuiPanelRenderer<T> {
    _phantom: PhantomData<T>,
}

impl<T> RatatuiPanelRenderer<T> {
    /// Create a new ratatui panel renderer
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for RatatuiPanelRenderer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T> PanelRenderer<T> for RatatuiPanelRenderer<T>
where
    T: Send + Sync,
{
    async fn render_panel(&mut self, _panel: &Panel<T>) -> Result<()> {
        tracing::debug!("RatatuiPanelRenderer: render_panel called");
        Ok(())
    }

    fn capabilities(&self) -> RendererCapabilities {
        RendererCapabilities {
            modality: Modality::TerminalTUI,
            supports_expansion: true,
            supports_selection: true,
            supports_icons: false,
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
    async fn test_ratatui_panel_renderer() {
        let mut renderer: RatatuiPanelRenderer<String> = RatatuiPanelRenderer::new();

        let panel = Panel::split(
            Direction::Horizontal,
            0.5,
            Panel::Leaf(PanelContent::new("left", "Left", "Content".to_string())),
            Panel::Leaf(PanelContent::new("right", "Right", "Content".to_string())),
        );

        assert!(renderer.render_panel(&panel).await.is_ok());
    }
}
