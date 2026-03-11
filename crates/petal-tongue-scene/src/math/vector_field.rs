// SPDX-License-Identifier: AGPL-3.0-only
//! Vector field: arrows at grid points.

use serde::{Deserialize, Serialize};

use crate::math::MathObject;
use crate::primitive::{Color, FillRule, LineCap, LineJoin, Primitive, StrokeStyle};

/// A single arrow in a vector field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrowSpec {
    /// Start point (x, y).
    pub start: [f64; 2],
    /// Direction vector (dx, dy).
    pub direction: [f64; 2],
}

/// A vector field: arrows at grid points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorField {
    /// Arrows: start + direction.
    pub arrows: Vec<ArrowSpec>,
    /// Scale factor for arrow length.
    pub scale: f64,
    /// Color for arrows.
    pub color: Color,
}

impl VectorField {
    /// Build from a field function F(x,y) -> (dx, dy).
    pub fn from_fn(
        field: impl Fn(f64, f64) -> (f64, f64),
        x_range: (f64, f64),
        y_range: (f64, f64),
        density: usize,
        scale: f64,
        color: Color,
    ) -> Self {
        let (x_min, x_max) = x_range;
        let (y_min, y_max) = y_range;
        let mut arrows = Vec::new();
        for i in 0..density {
            for j in 0..density {
                let t_x = (i as f64 + 0.5) / density as f64;
                let t_y = (j as f64 + 0.5) / density as f64;
                let x = x_min + t_x * (x_max - x_min);
                let y = y_min + t_y * (y_max - y_min);
                let (dx, dy) = field(x, y);
                arrows.push(ArrowSpec {
                    start: [x, y],
                    direction: [dx * scale, dy * scale],
                });
            }
        }
        Self {
            arrows,
            scale,
            color,
        }
    }
}

impl MathObject for VectorField {
    fn to_primitives(&self) -> Vec<Primitive> {
        let mut prims = Vec::new();
        let stroke = StrokeStyle {
            color: self.color,
            width: 1.0,
            cap: LineCap::Butt,
            join: LineJoin::Miter,
        };

        let head_size = 4.0;
        for arrow in &self.arrows {
            let [sx, sy] = arrow.start;
            let [dx, dy] = arrow.direction;
            let ex = sx + dx;
            let ey = sy + dy;

            // Shaft
            prims.push(Primitive::Line {
                points: vec![[sx, sy], [ex, ey]],
                stroke,
                closed: false,
                data_id: None,
            });

            // Arrow head (triangle)
            let len = (dx * dx + dy * dy).sqrt();
            if len > 1e-10 {
                let ux = dx / len;
                let uy = dy / len;
                let perp_x = -uy;
                let perp_y = ux;
                let tip = [ex, ey];
                let base1 = [
                    ex - ux * head_size + perp_x * head_size * 0.5,
                    ey - uy * head_size + perp_y * head_size * 0.5,
                ];
                let base2 = [
                    ex - ux * head_size - perp_x * head_size * 0.5,
                    ey - uy * head_size - perp_y * head_size * 0.5,
                ];
                prims.push(Primitive::Polygon {
                    points: vec![tip, base1, base2],
                    fill: self.color,
                    stroke: None,
                    fill_rule: FillRule::NonZero,
                    data_id: None,
                });
            }
        }

        prims
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-10;

    #[test]
    fn arrow_spec_construction() {
        let arrow = ArrowSpec {
            start: [1.0, 2.0],
            direction: [3.0, 4.0],
        };
        assert!((arrow.start[0] - 1.0).abs() < EPS);
        assert!((arrow.direction[1] - 4.0).abs() < EPS);
    }

    #[test]
    fn vector_field_from_fn_basic() {
        let vf = VectorField::from_fn(
            |x, y| (x + y, x - y),
            (-1.0, 1.0),
            (-1.0, 1.0),
            2,
            1.0,
            Color::BLACK,
        );
        assert_eq!(vf.arrows.len(), 4);
        assert!((vf.scale - 1.0).abs() < EPS);
    }

    #[test]
    fn vector_field_zero_length_arrows_no_head() {
        let vf = VectorField {
            arrows: vec![ArrowSpec {
                start: [0.0, 0.0],
                direction: [0.0, 0.0],
            }],
            scale: 1.0,
            color: Color::BLACK,
        };
        let prims = vf.to_primitives();
        assert_eq!(
            prims.len(),
            1,
            "zero-length arrow produces only shaft, no head"
        );
        assert!(matches!(prims[0], Primitive::Line { .. }));
    }

    #[test]
    fn vector_field_nonzero_arrow_has_head() {
        let vf = VectorField {
            arrows: vec![ArrowSpec {
                start: [0.0, 0.0],
                direction: [10.0, 0.0],
            }],
            scale: 1.0,
            color: Color::BLACK,
        };
        let prims = vf.to_primitives();
        assert!(
            prims.len() >= 2,
            "non-zero arrow should have shaft + head polygon"
        );
        let has_polygon = prims.iter().any(|p| matches!(p, Primitive::Polygon { .. }));
        assert!(has_polygon, "arrow head should be a polygon");
    }

    #[test]
    fn vector_field_to_primitives_empty() {
        let vf = VectorField {
            arrows: vec![],
            scale: 1.0,
            color: Color::BLACK,
        };
        let prims = vf.to_primitives();
        assert!(prims.is_empty());
    }

    #[test]
    fn vector_field_scale_applied() {
        let vf = VectorField::from_fn(
            |_, _| (1.0, 0.0),
            (0.0, 1.0),
            (0.0, 1.0),
            1,
            2.0,
            Color::BLACK,
        );
        assert!((vf.arrows[0].direction[0] - 2.0).abs() < EPS);
    }
}
