// SPDX-License-Identifier: AGPL-3.0-only
// Ratatui Form Renderer - TUI form rendering
//
// Deep Debt Principles:
// - No unsafe code
// - Generic over data type T
// - Async-safe (capability-based)
// - Terminal-friendly

use crate::form::{FieldType, Form};
use crate::renderer::{FormRenderer, RendererCapabilities};
use anyhow::Result;
use async_trait::async_trait;

/// Ratatui implementation of FormRenderer
pub struct RatatuiFormRenderer {
    /// Frame index for rendering
    frame_index: usize,
}

impl RatatuiFormRenderer {
    /// Create a new ratatui form renderer
    pub fn new() -> Self {
        Self { frame_index: 0 }
    }
}

impl Default for RatatuiFormRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<T: Send + Sync> FormRenderer<T> for RatatuiFormRenderer {
    async fn render_form(&mut self, form: &mut Form<T>) -> Result<()> {
        self.frame_index += 1;

        // In a real implementation, this would use ratatui widgets
        // For now, we simulate the rendering logic

        // Would render:
        // - Block with title
        // - List of fields
        // - Input widgets per field type
        // - Error messages
        // - Submit button

        for field in &form.fields {
            match &field.field_type {
                FieldType::Text { .. } | FieldType::TextArea { .. } => {
                    // Paragraph::new(value).block(Block::default().title(&field.label))
                }
                FieldType::Number { .. } | FieldType::Integer { .. } => {
                    // Paragraph::new(format!("{}", value))
                }
                FieldType::Select { options, .. } => {
                    // List::new(options).block(Block::default().title(&field.label))
                }
                FieldType::MultiSelect { options, .. } => {
                    // Multiple items with checkboxes: "[X] option" / "[ ] option"
                    let _ = options; // Suppress unused warning
                }
                FieldType::Checkbox { .. } => {
                    // "[X] label" or "[ ] label"
                }
                FieldType::Radio { options, .. } => {
                    // "( ) option1" / "(o) option2"
                    let _ = options; // Suppress unused warning
                }
                FieldType::Slider { min, max, .. } => {
                    // Gauge showing value between min and max
                    let _ = (min, max); // Suppress unused warnings
                }
                FieldType::Color { .. } => {
                    // Show color as "#RRGGBBAA" with preview block
                }
            }

            // Show errors for this field
            let errors = form.field_errors(&field.id);
            for error in errors {
                // Paragraph::new(&error.message).style(Style::default().fg(Color::Red))
                let _ = error; // Suppress unused warning
            }
        }

        // Render submit button
        // "[ Submit ]" with highlight if focused

        Ok(())
    }

    fn capabilities(&self) -> RendererCapabilities {
        RendererCapabilities {
            modality: crate::renderer::Modality::TerminalTUI,
            supports_expansion: false,
            supports_selection: false,
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
    use crate::form::Field;

    #[derive(Debug, Clone)]
    struct TestData {
        name: String,
    }

    #[tokio::test]
    async fn test_ratatui_form_renderer_creation() {
        let renderer: RatatuiFormRenderer = RatatuiFormRenderer::new();
        let caps = <RatatuiFormRenderer as FormRenderer<TestData>>::capabilities(&renderer);
        assert!(caps.is_interactive);
        assert_eq!(caps.modality, crate::renderer::Modality::TerminalTUI);
    }

    #[tokio::test]
    async fn test_ratatui_form_render() {
        let mut renderer = RatatuiFormRenderer::new();
        let mut form =
            Form::<TestData>::new("Test Form").with_field(Field::text("name", "Name").required());

        let result = renderer.render_form(&mut form).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ratatui_form_multiple_renders() {
        let mut renderer = RatatuiFormRenderer::new();
        let mut form = Form::<TestData>::new("Test Form")
            .with_field(Field::text("name", "Name"))
            .with_field(Field::checkbox("active", "Active"));

        // Render multiple times (simulating animation frames)
        for _ in 0..5 {
            let result = renderer.render_form(&mut form).await;
            assert!(result.is_ok());
        }

        assert_eq!(renderer.frame_index, 5);
    }
}
