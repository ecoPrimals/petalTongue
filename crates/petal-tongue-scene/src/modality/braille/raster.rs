// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rasterization: scene primitives to Braille dot grid.

use crate::primitive::Primitive;
use crate::scene_graph::SceneGraph;
use crate::transform::Transform2D;

use super::super::{ModalityCompiler, ModalityOutput};
use super::types::BrailleCell;

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
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        reason = "clamped coordinates; Braille dimensions"
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
    #[expect(clippy::cast_sign_loss, reason = "grid indices bounded")]
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
        #[expect(clippy::cast_precision_loss, reason = "Braille cell dimensions")]
        let pixel_w = cols as f64 * 2.0;
        #[expect(clippy::cast_precision_loss, reason = "Braille cell dimensions")]
        let pixel_h = rows as f64 * 4.0;
        #[expect(clippy::cast_possible_truncation, reason = "clamped to pixel bounds")]
        let x_to_pixel = |x: f64| (x / viewport_w * pixel_w).clamp(0.0, pixel_w - 1.0) as i32;
        #[expect(clippy::cast_possible_truncation, reason = "clamped to pixel bounds")]
        let y_to_pixel = |y: f64| (y / viewport_h * pixel_h).clamp(0.0, pixel_h - 1.0) as i32;
        let mut px0 = x_to_pixel(x0);
        let mut py0 = y_to_pixel(y0);
        let px1 = x_to_pixel(x1);
        let py1 = y_to_pixel(y1);
        let dx = (px1 - px0).abs();
        let dy = -(py1 - py0).abs();
        let sx: i32 = if px0 < px1 { 1 } else { -1 };
        let sy: i32 = if py0 < py1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            let ux = px0.clamp(
                0,
                i32::try_from(cols * 2)
                    .unwrap_or(i32::MAX)
                    .saturating_sub(1),
            ) as usize;
            let uy = py0.clamp(
                0,
                i32::try_from(rows * 4)
                    .unwrap_or(i32::MAX)
                    .saturating_sub(1),
            ) as usize;
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
                            f64::from(dx).mul_add(2.0, tx),
                            f64::from(dy).mul_add(2.0, ty),
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
                    let t = angle_span.mul_add(f64::from(i) / f64::from(segments), *start_angle);
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
                        let t = f64::from(i) / f64::from(steps);
                        let mt = 1.0 - t;
                        let mt2 = mt * mt;
                        let mt3 = mt2 * mt;
                        let t2 = t * t;
                        let t3 = t2 * t;
                        let px = (3.0 * mt * t2)
                            .mul_add(seg.cp2[0], mt3 * p0[0] + 3.0 * mt2 * t * seg.cp1[0])
                            + t3 * seg.end[0];
                        let py = (3.0 * mt * t2)
                            .mul_add(seg.cp2[1], mt3 * p0[1] + 3.0 * mt2 * t * seg.cp1[1])
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
