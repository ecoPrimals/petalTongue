//! Ratatui Table Renderer
//!
//! Renders tables in a terminal UI (TUI) context.

use crate::renderer::{Modality, RendererCapabilities, TableRenderer};
use crate::table::Table;
use anyhow::Result;
use async_trait::async_trait;
use std::marker::PhantomData;

/// Ratatui-based table renderer
pub struct RatatuiTableRenderer<T> {
    _phantom: PhantomData<T>,
}

impl<T> RatatuiTableRenderer<T> {
    /// Create a new ratatui table renderer
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for RatatuiTableRenderer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T> TableRenderer<T> for RatatuiTableRenderer<T>
where
    T: Send + Sync,
{
    async fn render_table(&mut self, _table: &Table<T>) -> Result<()> {
        tracing::debug!("RatatuiTableRenderer: render_table called");
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
    use crate::table::Column;

    #[derive(Clone)]
    struct Person {
        name: String,
        age: u32,
    }

    #[tokio::test]
    async fn test_ratatui_table_renderer() {
        let mut renderer: RatatuiTableRenderer<Person> = RatatuiTableRenderer::new();

        let table = Table::new()
            .with_column(Column::new("Name", |p: &Person| p.name.clone()))
            .with_column(Column::new("Age", |p: &Person| p.age.to_string()))
            .with_data(vec![Person {
                name: "Alice".into(),
                age: 30,
            }]);

        assert!(renderer.render_table(&table).await.is_ok());
    }
}
