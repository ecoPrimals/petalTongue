// SPDX-License-Identifier: AGPL-3.0-only

use serde::{Deserialize, Serialize};

use crate::primitive::Primitive;
use crate::scene_graph::SceneGraph;

use super::{ModalityCompiler, ModalityOutput};

// Re-export BrailleCell for use in ModalityOutput - but wait, BrailleCell is defined HERE.
// The mod.rs uses braille::BrailleCell. So we export BrailleCell from this module.
// The mod.rs has: pub use braille::BrailleCell;

/// A single Braille cell (3x2 dot matrix, Unicode Braille block U+2800..U+28FF).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrailleCell {
    /// 8-bit dot pattern (bits 0-7 = dots 1-8 per Unicode Braille).
    pub dots: u8,
}

impl BrailleCell {
    pub const BLANK: Self = Self { dots: 0 };

    /// Convert dot pattern to the corresponding Unicode Braille character.
    #[expect(clippy::cast_lossless, reason = "u8 to u32 is always lossless")]
    pub fn to_char(self) -> char {
        char::from_u32(0x2800 + self.dots as u32).unwrap_or('\u{2800}')
    }
}

/// Compiles scene graph to a Braille dot grid for tactile displays.
///
/// Maps the scene into a 2D grid where each cell is a 2x4 dot sub-grid
/// (standard 8-dot Braille). Data-carrying points become raised dots
/// at the corresponding grid position.
#[derive(Debug, Clone)]
pub struct BrailleCompiler {
    /// Grid width in Braille cells.
    cols: usize,
    /// Grid height in Braille cells.
    rows: usize,
}

impl BrailleCompiler {
    #[must_use]
    pub fn new(cols: usize, rows: usize) -> Self {
        Self { cols, rows }
    }

    fn to_braille_cell(
        col: usize,
        row: usize,
        dot_col: usize,
        dot_row: usize,
    ) -> (usize, usize, u8) {
        let _ = (col, row);
        let bit = match (dot_col, dot_row) {
            (0, 0) => 0,
            (0, 1) => 1,
            (0, 2) => 2,
            (1, 0) => 3,
            (1, 1) => 4,
            (1, 2) => 5,
            (0, 3) => 6,
            (1, 3) => 7,
            _ => return (0, 0, 0),
        };
        (col, row, 1 << bit)
    }
}

impl Default for BrailleCompiler {
    fn default() -> Self {
        Self::new(40, 12)
    }
}

impl ModalityCompiler for BrailleCompiler {
    fn name(&self) -> &'static str {
        "BrailleCompiler"
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "scene coordinates clamped to grid bounds"
    )]
    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let cols = self.cols.max(1);
        let rows = self.rows.max(1);
        let mut grid = vec![vec![BrailleCell::BLANK; cols]; rows];

        let pixel_w = cols as f64 * 2.0;
        let pixel_h = rows as f64 * 4.0;

        for (_transform, prim) in scene.flatten() {
            let (x, y) = match prim {
                Primitive::Point { x, y, .. } | Primitive::Rect { x, y, .. } => (*x, *y),
                _ => continue,
            };

            let px = (x / 800.0 * pixel_w).clamp(0.0, pixel_w - 1.0) as usize;
            let py = (y / 600.0 * pixel_h).clamp(0.0, pixel_h - 1.0) as usize;

            let cell_col = px / 2;
            let cell_row = py / 4;
            let dot_col = px % 2;
            let dot_row = py % 4;

            if cell_row < rows && cell_col < cols {
                let (_, _, bit) = Self::to_braille_cell(cell_col, cell_row, dot_col, dot_row);
                grid[cell_row][cell_col].dots |= bit;
            }
        }

        ModalityOutput::BrailleCells(grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{Color, Primitive};
    use crate::scene_graph::{SceneGraph, SceneNode};

    #[test]
    fn braille_compiler_produces_grid() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Point {
            x: 400.0,
            y: 300.0,
            radius: 5.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: Some("d1".to_string()),
        };
        graph.add_to_root(SceneNode::new("p").with_primitive(prim));
        let out = BrailleCompiler::new(40, 12).compile(&graph);
        let ModalityOutput::BrailleCells(grid) = &out else {
            panic!("expected BrailleCells");
        };
        assert_eq!(grid.len(), 12);
        assert_eq!(grid[0].len(), 40);
        let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
        assert!(has_raised, "Grid should have at least one raised dot");
    }

    #[test]
    fn braille_cell_to_char_blank() {
        assert_eq!(BrailleCell::BLANK.to_char(), '\u{2800}');
    }

    #[test]
    fn braille_cell_to_char_dot1() {
        let cell = BrailleCell { dots: 1 };
        assert_eq!(cell.to_char(), '\u{2801}');
    }
}
