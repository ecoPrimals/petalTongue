// SPDX-License-Identifier: AGPL-3.0-only
//! Ratatui Command Palette Renderer

use crate::command_palette::CommandPalette;
use crate::renderer::{CommandPaletteRenderer, Modality, RendererCapabilities};
use anyhow::Result;
use async_trait::async_trait;
use std::marker::PhantomData;

/// Ratatui-based command palette renderer
pub struct RatatuiCommandPaletteRenderer<T> {
    _phantom: PhantomData<T>,
}

impl<T> RatatuiCommandPaletteRenderer<T> {
    /// Create a new ratatui command palette renderer
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for RatatuiCommandPaletteRenderer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T> CommandPaletteRenderer<T> for RatatuiCommandPaletteRenderer<T>
where
    T: Send + Sync + Clone,
{
    async fn render_palette(&mut self, _palette: &CommandPalette<T>) -> Result<()> {
        tracing::debug!("RatatuiCommandPaletteRenderer: render_palette called");
        Ok(())
    }

    fn capabilities(&self) -> RendererCapabilities {
        RendererCapabilities {
            modality: Modality::TerminalTUI,
            supports_expansion: false,
            supports_selection: true,
            supports_icons: false,
            supports_colors: true,
            supports_filtering: true,
            is_interactive: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command_palette::Command;

    #[tokio::test]
    async fn test_ratatui_command_palette_renderer() {
        let mut renderer: RatatuiCommandPaletteRenderer<String> =
            RatatuiCommandPaletteRenderer::new();

        let palette = CommandPalette::new().with_command(Command::new(
            "test",
            "Test",
            "Testing",
            "action".to_string(),
        ));

        assert!(renderer.render_palette(&palette).await.is_ok());
    }
}
