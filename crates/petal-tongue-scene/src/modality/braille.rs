// SPDX-License-Identifier: AGPL-3.0-only

use serde::{Deserialize, Serialize};

use crate::primitive::Primitive;
use crate::scene_graph::SceneGraph;
use crate::transform::Transform2D;

use super::{ModalityCompiler, ModalityOutput};

/// A single Braille cell (3x2 dot matrix, Unicode Braille block U+2800..U+28FF).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrailleCell {
    pub dots: u8,
}

impl BrailleCell {
    pub const BLANK: Self = Self { dots: 0 };

    #[expect(clippy::cast_lossless, reason = "u8 to u32 is always lossless")]
    #[must_use]
    pub fn to_char(self) -> char {
        char::from_u32(0x2800 + self.dots as u32).unwrap_or('\u{2800}')
    }
}

/// Compiles scene graph to a Braille dot grid for tactile displays.
#[derive(Debug, Clone)]
pub struct BrailleCompiler {
    cols: usize,
    rows: usize,
    viewport_width: f64,
    viewport_height: f64,
}

impl BrailleCompiler {
    #[must_use]
    pub const fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            viewport_width: 800.0,
            viewport_height: 600.0,
        }
    }

    #[must_use]
    pub const fn with_viewport(mut self, width: f64, height: f64) -> Self {
        self.viewport_width = width;
        self.viewport_height = height;
        self
    }

    const fn to_braille_cell(
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

    #[expect(
        clippy::cast_sign_loss,
        reason = "clamped coordinates are non-negative"
    )]
    fn set_dot(
        grid: &mut [Vec<BrailleCell>],
        cols: usize,
        rows: usize,
        x: f64,
        y: f64,
        viewport_w: f64,
        viewport_h: f64,
    ) {
        let pixel_w = cols as f64 * 2.0;
        let pixel_h = rows as f64 * 4.0;
        let px = (x / viewport_w * pixel_w).clamp(0.0, pixel_w - 1.0) as usize;
        let py = (y / viewport_h * pixel_h).clamp(0.0, pixel_h - 1.0) as usize;
        let cell_col = px / 2;
        let cell_row = py / 4;
        let dot_col = px % 2;
        let dot_row = py % 4;
        if cell_row < rows && cell_col < cols {
            let (_, _, bit) = Self::to_braille_cell(cell_col, cell_row, dot_col, dot_row);
            grid[cell_row][cell_col].dots |= bit;
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "Bresenham needs grid and coordinate params"
    )]
    #[expect(
        clippy::cast_sign_loss,
        clippy::cast_possible_wrap,
        reason = "grid indices bounded"
    )]
    fn bresenham(
        grid: &mut [Vec<BrailleCell>],
        cols: usize,
        rows: usize,
        x0: f64,
        y0: f64,
        x1: f64,
        y1: f64,
        viewport_w: f64,
        viewport_h: f64,
    ) {
        let pixel_w = cols as f64 * 2.0;
        let pixel_h = rows as f64 * 4.0;
        let to_px = |x: f64| (x / viewport_w * pixel_w).clamp(0.0, pixel_w - 1.0) as i32;
        let to_py = |y: f64| (y / viewport_h * pixel_h).clamp(0.0, pixel_h - 1.0) as i32;
        let mut px0 = to_px(x0);
        let mut py0 = to_py(y0);
        let px1 = to_px(x1);
        let py1 = to_py(y1);
        let dx = (px1 - px0).abs();
        let dy = -(py1 - py0).abs();
        let sx: i32 = if px0 < px1 { 1 } else { -1 };
        let sy: i32 = if py0 < py1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            let ux = px0.clamp(0, (cols * 2) as i32 - 1) as usize;
            let uy = py0.clamp(0, (rows * 4) as i32 - 1) as usize;
            let cell_col = ux / 2;
            let cell_row = uy / 4;
            let dot_col = ux % 2;
            let dot_row = uy % 4;
            if cell_row < rows && cell_col < cols {
                let (_, _, bit) = Self::to_braille_cell(cell_col, cell_row, dot_col, dot_row);
                grid[cell_row][cell_col].dots |= bit;
            }
            if px0 == px1 && py0 == py1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                px0 += sx;
            }
            if e2 <= dx {
                err += dx;
                py0 += sy;
            }
        }
    }

    #[expect(clippy::too_many_lines, reason = "one match arm per primitive type")]
    fn rasterize_primitive(
        &self,
        grid: &mut [Vec<BrailleCell>],
        transform: &Transform2D,
        prim: &Primitive,
    ) {
        if !prim.carries_data() {
            return;
        }
        let cols = self.cols.max(1);
        let rows = self.rows.max(1);
        let vw = self.viewport_width;
        let vh = self.viewport_height;

        match prim {
            Primitive::Point { x, y, .. } => {
                let (tx, ty) = transform.apply(*x, *y);
                Self::set_dot(grid, cols, rows, tx, ty, vw, vh);
            }
            Primitive::Rect {
                x,
                y,
                width,
                height,
                ..
            } => {
                let (x0, y0) = transform.apply(*x, *y);
                let (x1, y1) = transform.apply(*x + *width, *y);
                let (x2, y2) = transform.apply(*x + *width, *y + *height);
                let (x3, y3) = transform.apply(*x, *y + *height);
                Self::bresenham(grid, cols, rows, x0, y0, x1, y1, vw, vh);
                Self::bresenham(grid, cols, rows, x1, y1, x2, y2, vw, vh);
                Self::bresenham(grid, cols, rows, x2, y2, x3, y3, vw, vh);
                Self::bresenham(grid, cols, rows, x3, y3, x0, y0, vw, vh);
            }
            Primitive::Line { points, closed, .. } => {
                if points.len() < 2 {
                    return;
                }
                for i in 0..points.len() - 1 {
                    let (x0, y0) = transform.apply(points[i][0], points[i][1]);
                    let (x1, y1) = transform.apply(points[i + 1][0], points[i + 1][1]);
                    Self::bresenham(grid, cols, rows, x0, y0, x1, y1, vw, vh);
                }
                if *closed && points.len() >= 3 {
                    let (x0, y0) = transform.apply(points[0][0], points[0][1]);
                    let (x1, y1) =
                        transform.apply(points[points.len() - 1][0], points[points.len() - 1][1]);
                    Self::bresenham(grid, cols, rows, x0, y0, x1, y1, vw, vh);
                }
            }
            Primitive::Text { x, y, .. } => {
                let (tx, ty) = transform.apply(*x, *y);
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        Self::set_dot(
                            grid,
                            cols,
                            rows,
                            tx + f64::from(dx) * 2.0,
                            ty + f64::from(dy) * 2.0,
                            vw,
                            vh,
                        );
                    }
                }
            }
            Primitive::Polygon {
                points, fill: _, ..
            } => {
                if points.len() < 2 {
                    return;
                }
                for i in 0..points.len() - 1 {
                    let (x0, y0) = transform.apply(points[i][0], points[i][1]);
                    let (x1, y1) = transform.apply(points[i + 1][0], points[i + 1][1]);
                    Self::bresenham(grid, cols, rows, x0, y0, x1, y1, vw, vh);
                }
                let (x0, y0) = transform.apply(points[0][0], points[0][1]);
                let (x1, y1) =
                    transform.apply(points[points.len() - 1][0], points[points.len() - 1][1]);
                Self::bresenham(grid, cols, rows, x0, y0, x1, y1, vw, vh);
            }
            Primitive::Arc {
                cx,
                cy,
                radius,
                start_angle,
                end_angle,
                ..
            } => {
                let segments = 32;
                let angle_span = end_angle - start_angle;
                let mut prev = transform.apply(
                    cx + radius * start_angle.cos(),
                    cy + radius * start_angle.sin(),
                );
                for i in 1..=segments {
                    let t = *start_angle + angle_span * (i as f64 / segments as f64);
                    let curr = transform.apply(cx + radius * t.cos(), cy + radius * t.sin());
                    Self::bresenham(grid, cols, rows, prev.0, prev.1, curr.0, curr.1, vw, vh);
                    prev = curr;
                }
            }
            Primitive::BezierPath {
                start, segments, ..
            } => {
                let mut pts = Vec::new();
                let (sx, sy) = transform.apply(start[0], start[1]);
                pts.push((sx, sy));
                let mut cur = *start;
                for seg in segments {
                    let steps = 16;
                    let p0 = cur;
                    for i in 1..=steps {
                        let t = i as f64 / steps as f64;
                        let mt = 1.0 - t;
                        let mt2 = mt * mt;
                        let mt3 = mt2 * mt;
                        let t2 = t * t;
                        let t3 = t2 * t;
                        let px = mt3 * p0[0]
                            + 3.0 * mt2 * t * seg.cp1[0]
                            + 3.0 * mt * t2 * seg.cp2[0]
                            + t3 * seg.end[0];
                        let py = mt3 * p0[1]
                            + 3.0 * mt2 * t * seg.cp1[1]
                            + 3.0 * mt * t2 * seg.cp2[1]
                            + t3 * seg.end[1];
                        pts.push(transform.apply(px, py));
                    }
                    cur = seg.end;
                }
                if pts.len() >= 2 {
                    for i in 0..pts.len() - 1 {
                        Self::bresenham(
                            grid,
                            cols,
                            rows,
                            pts[i].0,
                            pts[i].1,
                            pts[i + 1].0,
                            pts[i + 1].1,
                            vw,
                            vh,
                        );
                    }
                } else if pts.len() == 1 {
                    Self::set_dot(grid, cols, rows, pts[0].0, pts[0].1, vw, vh);
                }
            }
            Primitive::Mesh {
                vertices, indices, ..
            } => {
                for chunk in indices.chunks(3) {
                    if chunk.len() != 3 {
                        continue;
                    }
                    let i0 = chunk[0] as usize;
                    let i1 = chunk[1] as usize;
                    let i2 = chunk[2] as usize;
                    if i0 >= vertices.len() || i1 >= vertices.len() || i2 >= vertices.len() {
                        continue;
                    }
                    let v0 = &vertices[i0];
                    let v1 = &vertices[i1];
                    let v2 = &vertices[i2];
                    let (x0, y0) = transform.apply(v0.position[0], v0.position[1]);
                    let (x1, y1) = transform.apply(v1.position[0], v1.position[1]);
                    let (x2, y2) = transform.apply(v2.position[0], v2.position[1]);
                    Self::bresenham(grid, cols, rows, x0, y0, x1, y1, vw, vh);
                    Self::bresenham(grid, cols, rows, x1, y1, x2, y2, vw, vh);
                    Self::bresenham(grid, cols, rows, x2, y2, x0, y0, vw, vh);
                }
            }
        }
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

    fn compile(&self, scene: &SceneGraph) -> ModalityOutput {
        let cols = self.cols.max(1);
        let rows = self.rows.max(1);
        let mut grid = vec![vec![BrailleCell::BLANK; cols]; rows];

        for (transform, prim) in scene.flatten() {
            self.rasterize_primitive(&mut grid, &transform, prim);
        }

        ModalityOutput::BrailleCells(grid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::{AnchorPoint, Color, FillRule, Primitive, StrokeStyle};
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

    #[test]
    fn braille_compiler_with_viewport() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Point {
            x: 400.0,
            y: 300.0,
            radius: 5.0,
            fill: None,
            stroke: None,
            data_id: Some("d1".to_string()),
        };
        graph.add_to_root(SceneNode::new("p").with_primitive(prim));
        let compiler = BrailleCompiler::new(40, 12).with_viewport(1600.0, 1200.0);
        let out = compiler.compile(&graph);
        let ModalityOutput::BrailleCells(grid) = &out else {
            panic!("expected BrailleCells");
        };
        let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
        assert!(has_raised);
    }

    #[test]
    fn braille_compiler_skips_non_data_primitives() {
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
        let out = BrailleCompiler::new(40, 12).compile(&graph);
        let ModalityOutput::BrailleCells(grid) = &out else {
            panic!("expected BrailleCells");
        };
        let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
        assert!(!has_raised);
    }

    #[test]
    fn braille_compiler_line() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Line {
            points: vec![[0.0, 0.0], [800.0, 600.0]],
            stroke: StrokeStyle::default(),
            closed: false,
            data_id: Some("line1".to_string()),
        };
        graph.add_to_root(SceneNode::new("l").with_primitive(prim));
        let out = BrailleCompiler::new(40, 12).compile(&graph);
        let ModalityOutput::BrailleCells(grid) = &out else {
            panic!("expected BrailleCells");
        };
        let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
        assert!(has_raised);
    }

    #[test]
    fn braille_compiler_text() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Text {
            x: 100.0,
            y: 100.0,
            content: "Hello".to_string(),
            font_size: 16.0,
            color: Color::BLACK,
            anchor: AnchorPoint::TopLeft,
            bold: false,
            italic: false,
            data_id: Some("text1".to_string()),
        };
        graph.add_to_root(SceneNode::new("t").with_primitive(prim));
        let out = BrailleCompiler::new(40, 12).compile(&graph);
        let ModalityOutput::BrailleCells(grid) = &out else {
            panic!("expected BrailleCells");
        };
        let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
        assert!(has_raised);
    }

    #[test]
    fn braille_compiler_polygon() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Polygon {
            points: vec![[100.0, 100.0], [200.0, 100.0], [150.0, 200.0]],
            fill: Color::BLACK,
            stroke: None,
            fill_rule: FillRule::NonZero,
            data_id: Some("poly1".to_string()),
        };
        graph.add_to_root(SceneNode::new("p").with_primitive(prim));
        let out = BrailleCompiler::new(40, 12).compile(&graph);
        let ModalityOutput::BrailleCells(grid) = &out else {
            panic!("expected BrailleCells");
        };
        let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
        assert!(has_raised);
    }

    #[test]
    fn braille_compiler_arc() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Arc {
            cx: 400.0,
            cy: 300.0,
            radius: 100.0,
            start_angle: 0.0,
            end_angle: std::f64::consts::FRAC_PI_2,
            fill: None,
            stroke: None,
            data_id: Some("arc1".to_string()),
        };
        graph.add_to_root(SceneNode::new("a").with_primitive(prim));
        let out = BrailleCompiler::new(40, 12).compile(&graph);
        let ModalityOutput::BrailleCells(grid) = &out else {
            panic!("expected BrailleCells");
        };
        let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
        assert!(has_raised);
    }

    #[test]
    fn braille_compiler_bezier_path() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::BezierPath {
            start: [100.0, 100.0],
            segments: vec![crate::primitive::BezierSegment {
                cp1: [150.0, 50.0],
                cp2: [250.0, 150.0],
                end: [300.0, 100.0],
            }],
            stroke: StrokeStyle::default(),
            fill: None,
            fill_rule: FillRule::NonZero,
            data_id: Some("path1".to_string()),
        };
        graph.add_to_root(SceneNode::new("b").with_primitive(prim));
        let out = BrailleCompiler::new(40, 12).compile(&graph);
        let ModalityOutput::BrailleCells(grid) = &out else {
            panic!("expected BrailleCells");
        };
        let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
        assert!(has_raised);
    }

    #[test]
    fn braille_compiler_mesh() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Mesh {
            vertices: vec![
                crate::primitive::MeshVertex {
                    position: [100.0, 100.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: Color::BLACK,
                },
                crate::primitive::MeshVertex {
                    position: [200.0, 100.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: Color::BLACK,
                },
                crate::primitive::MeshVertex {
                    position: [150.0, 200.0, 0.0],
                    normal: [0.0, 0.0, 1.0],
                    color: Color::BLACK,
                },
            ],
            indices: vec![0, 1, 2],
            data_id: Some("mesh1".to_string()),
        };
        graph.add_to_root(SceneNode::new("m").with_primitive(prim));
        let out = BrailleCompiler::new(40, 12).compile(&graph);
        let ModalityOutput::BrailleCells(grid) = &out else {
            panic!("expected BrailleCells");
        };
        let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
        assert!(has_raised);
    }

    #[test]
    fn braille_compiler_rect() {
        let mut graph = SceneGraph::new();
        let prim = Primitive::Rect {
            x: 100.0,
            y: 100.0,
            width: 200.0,
            height: 150.0,
            fill: Some(Color::BLACK),
            stroke: None,
            corner_radius: 0.0,
            data_id: Some("rect1".to_string()),
        };
        graph.add_to_root(SceneNode::new("r").with_primitive(prim));
        let out = BrailleCompiler::new(40, 12).compile(&graph);
        let ModalityOutput::BrailleCells(grid) = &out else {
            panic!("expected BrailleCells");
        };
        let has_raised = grid.iter().any(|row| row.iter().any(|c| c.dots != 0));
        assert!(has_raised);
    }
}
