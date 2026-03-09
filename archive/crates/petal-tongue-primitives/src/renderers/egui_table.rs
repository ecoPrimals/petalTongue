// SPDX-License-Identifier: AGPL-3.0-only
//! Egui Table Renderer
//!
//! Renders tables in an egui GUI context.

use crate::renderer::{Modality, RendererCapabilities, TableRenderer};
use crate::table::Table;
use anyhow::Result;
use async_trait::async_trait;
use std::marker::PhantomData;

/// Egui-based table renderer
pub struct EguiTableRenderer<T> {
    _phantom: PhantomData<T>,
}

impl<T> EguiTableRenderer<T> {
    /// Create a new egui table renderer
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Default for EguiTableRenderer<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T> TableRenderer<T> for EguiTableRenderer<T>
where
    T: Send + Sync,
{
    async fn render_table(&mut self, _table: &Table<T>) -> Result<()> {
        tracing::debug!("EguiTableRenderer: render_table called");
        Ok(())
    }

    fn capabilities(&self) -> RendererCapabilities {
        RendererCapabilities {
            modality: Modality::VisualGUI,
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
    async fn test_egui_table_renderer() {
        let mut renderer: EguiTableRenderer<Person> = EguiTableRenderer::new();

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
