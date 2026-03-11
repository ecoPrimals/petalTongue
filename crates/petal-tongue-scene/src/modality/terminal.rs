// SPDX-License-Identifier: AGPL-3.0-only

use crate::primitive::Primitive;
use crate::scene_graph::SceneGraph;

use super::{ModalityCompiler, ModalityOutput};

/// Compiles scene graph to a terminal character grid (for ratatui rendering).
///
/// Maps primitives to a 2D character array: points become markers, lines become
/// line-drawing characters, rectangles become boxes, and text is placed directly.
#[derive(Debug, Clone, Default)]
pub struct TerminalCompiler {
    width: usize,
    height: usize,
}

impl TerminalCompiler {
    /// Create a terminal compiler with the given dimensions (columns x rows).
    #[must_use]
    pub const fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }

    fn plot_point(grid: &mut [Vec<char>], col: usize, row: usize, ch: char) {
        if row < grid.len() && col < grid[0].len() {
            grid[row][col] = ch;
        }
    }

    fn plot_line(grid: &mut [Vec<char>], points: &[[f64; 2]], w: usize, h: usize) {
        for pair in points.windows(2) {
            let (x0, y0) = Self::to_cell(pair[0][0], pair[0][1], w, h);
            let (x1, y1) = Self::to_cell(pair[1][0], pair[1][1], w, h);
            Self::bresenham(grid, x0, y0, x1, y1);
        }
    }

    fn to_cell(x: f64, y: f64, w: usize, h: usize) -> (usize, usize) {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped to [0, w-1] and [0, h-1] before cast"
        )]
        let col = (x / 800.0 * w as f64).clamp(0.0, (w - 1) as f64) as usize;
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "clamped to [0, w-1] and [0, h-1] before cast"
        )]
        let row = (y / 600.0 * h as f64).clamp(0.0, (h - 1) as f64) as usize;
        (col, row)
    }

    #[expect(
        clippy::cast_possible_wrap,
        reason = "Bresenham uses i64 for signed arithmetic"
    )]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "y,x are bounds-checked in loop condition before indexing"
    )]
    fn bresenham(grid: &mut [Vec<char>], x0: usize, y0: usize, x1: usize, y1: usize) {
        let (mut x, mut y) = (x0 as i64, y0 as i64);
        let x1i = x1 as i64;
        let y1i = y1 as i64;
        let (dx, dy) = ((x1i - x).abs(), (y1i - y).abs());
        let sx: i64 = if x0 < x1 { 1 } else { -1 };
        let sy: i64 = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        let rows = grid.len() as i64;
        let cols = if grid.is_empty() {
            0
        } else {
            grid[0].len() as i64
        };

        loop {
            if y >= 0 && y < rows && x >= 0 && x < cols {
                let ch = if dx > dy * 2 {
                    '─'
                } else if dy > dx * 2 {
                    '│'
                } else {
                    '·'
                };
                grid[y as usize][x as usize] = ch;
            }
            if x == x1i && y == y1i {
                break;
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }
}

impl ModalityCompiler for TerminalCompiler {
    fn name(&self) -> &'static str {
        "TerminalCompiler"
    }

    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let w = if self.width > 0 { self.width } else { 80 };
        let h = if self.height > 0 { self.height } else { 24 };
        let mut grid = vec![vec![' '; w]; h];

        for (_transform, prim) in scene.flatten() {
            match prim {
                Primitive::Point { x, y, .. } => {
                    let (col, row) = Self::to_cell(*x, *y, w, h);
                    Self::plot_point(&mut grid, col, row, '●');
                }
                Primitive::Line { points, .. } => {
                    Self::plot_line(&mut grid, points, w, h);
                }
                Primitive::Rect {
                    x,
                    y,
                    width,
                    height,
                    ..
                } => {
                    let (c0, r0) = Self::to_cell(*x, *y, w, h);
                    let (c1, r1) = Self::to_cell(x + width, y + height, w, h);
                    for col in c0..=c1.min(w - 1) {
                        Self::plot_point(&mut grid, col, r0, '─');
                        Self::plot_point(&mut grid, col, r1.min(h - 1), '─');
                    }
                    for row in r0..=r1.min(h - 1) {
                        Self::plot_point(&mut grid, c0, row, '│');
                        Self::plot_point(&mut grid, c1.min(w - 1), row, '│');
                    }
                }
                Primitive::Text { x, y, content, .. } => {
                    let (col, row) = Self::to_cell(*x, *y, w, h);
                    for (i, ch) in content.chars().enumerate() {
                        if col + i < w && row < h {
                            grid[row][col + i] = ch;
                        }
                    }
                }
                _ => {}
            }
        }

        ModalityOutput::TerminalCells(grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{AnchorPoint, Color, Primitive};
    use crate::scene_graph::{SceneGraph, SceneNode};

    #[test]
    fn terminal_compiler_plots_points() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Point {
            x: 400.0,
            y: 300.0,
            radius: 5.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: None,
        };
        graph.add_to_root(SceneNode::new("p").with_primitive(prim));
        let out = TerminalCompiler::new(80, 24).compile(&graph);
        let ModalityOutput::TerminalCells(grid) = &out else {
            panic!("expected TerminalCells");
        };
        assert_eq!(grid.len(), 24);
        assert_eq!(grid[0].len(), 80);
        let has_marker = grid.iter().any(|row| row.contains(&'●'));
        assert!(has_marker, "Grid should contain a point marker");
    }

    #[test]
    fn terminal_compiler_renders_text() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Text {
            x: 0.0,
            y: 0.0,
            content: "Hi".to_string(),
            font_size: 12.0,
            color: Color::WHITE,
            anchor: AnchorPoint::TopLeft,
            bold: false,
            italic: false,
            data_id: None,
        };
        graph.add_to_root(SceneNode::new("t").with_primitive(prim));
        let out = TerminalCompiler::new(40, 10).compile(&graph);
        let ModalityOutput::TerminalCells(grid) = &out else {
            panic!("expected TerminalCells");
        };
        assert_eq!(grid[0][0], 'H');
        assert_eq!(grid[0][1], 'i');
    }

    #[test]
    fn terminal_compiler_zero_dimensions_uses_defaults() {
        let graph = SceneGraph::new();
        let out = TerminalCompiler::new(0, 0).compile(&graph);
        let ModalityOutput::TerminalCells(grid) = &out else {
            panic!("expected TerminalCells");
        };
        assert_eq!(grid.len(), 24);
        assert_eq!(grid[0].len(), 80);
    }
}
