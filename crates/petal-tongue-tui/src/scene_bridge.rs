// SPDX-License-Identifier: AGPL-3.0-only
//! Bridge between the declarative scene engine and ratatui rendering.
//!
//! Uses `TerminalCompiler` from `petal-tongue-scene` to convert a `SceneGraph`
//! into a character grid, then renders that grid as a ratatui `Widget`.

use petal_tongue_scene::modality::{ModalityCompiler, ModalityOutput, TerminalCompiler};
use petal_tongue_scene::scene_graph::SceneGraph;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::Widget;

/// A ratatui widget that renders a `SceneGraph` as text art.
pub struct SceneWidget<'a> {
    scene: &'a SceneGraph,
    style: Style,
}

impl<'a> SceneWidget<'a> {
    /// Create a widget from a scene graph reference.
    #[must_use]
    pub fn new(scene: &'a SceneGraph) -> Self {
        Self {
            scene,
            style: Style::default(),
        }
    }

    /// Apply a base style (foreground/background colors).
    #[must_use]
    pub const fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for SceneWidget<'_> {
    #[expect(
        clippy::cast_possible_truncation,
        reason = "col_idx, row_idx bounded by area dimensions (u16)"
    )]
    fn render(self, area: Rect, buf: &mut Buffer) {
        let compiler = TerminalCompiler::new(area.width as usize, area.height as usize);
        let output = compiler.compile(self.scene);
        let ModalityOutput::TerminalCells(grid) = output else {
            return;
        };

        for (row_idx, row) in grid.iter().enumerate() {
            if row_idx >= area.height as usize {
                break;
            }
            for (col_idx, &ch) in row.iter().enumerate() {
                if col_idx >= area.width as usize {
                    break;
                }
                if ch != ' ' {
                    buf[(area.x + col_idx as u16, area.y + row_idx as u16)]
                        .set_char(ch)
                        .set_style(self.style);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use petal_tongue_scene::primitive::{Color, Primitive};
    use petal_tongue_scene::scene_graph::SceneNode;

    #[test]
    fn scene_widget_renders_to_buffer() {
        let mut scene = SceneGraph::new();
        scene.add_to_root(SceneNode::new("pt").with_primitive(Primitive::Point {
            x: 400.0,
            y: 300.0,
            radius: 5.0,
            fill: Some(Color::WHITE),
            stroke: None,
            data_id: None,
        }));

        let widget = SceneWidget::new(&scene);
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);

        let has_marker = buf.content().iter().any(|cell| cell.symbol() == "●");
        assert!(has_marker, "Buffer should contain a point marker");
    }

    #[test]
    fn scene_widget_empty_scene() {
        let scene = SceneGraph::new();
        let widget = SceneWidget::new(&scene);
        let area = Rect::new(0, 0, 40, 10);
        let mut buf = Buffer::empty(area);
        widget.render(area, &mut buf);
        // Should not panic on empty scene
    }
}
