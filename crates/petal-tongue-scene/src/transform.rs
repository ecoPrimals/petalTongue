// SPDX-License-Identifier: AGPL-3.0-or-later
//! Spatial transforms for scene nodes.

use serde::{Deserialize, Serialize};
use std::ops::Mul;

/// A 2D affine transform represented as a 3x3 matrix (row-major).
///
/// The matrix is stored as [a, b, tx, c, d, ty] where:
/// ```text
/// | a  b  tx |
/// | c  d  ty |
/// | 0  0  1  |
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform2D {
    pub a: f64,
    pub b: f64,
    pub tx: f64,
    pub c: f64,
    pub d: f64,
    pub ty: f64,
}

impl Transform2D {
    pub const IDENTITY: Self = Self {
        a: 1.0,
        b: 0.0,
        tx: 0.0,
        c: 0.0,
        d: 1.0,
        ty: 0.0,
    };

    #[must_use]
    pub const fn translate(x: f64, y: f64) -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            tx: x,
            c: 0.0,
            d: 1.0,
            ty: y,
        }
    }

    #[must_use]
    pub const fn scale(sx: f64, sy: f64) -> Self {
        Self {
            a: sx,
            b: 0.0,
            tx: 0.0,
            c: 0.0,
            d: sy,
            ty: 0.0,
        }
    }

    #[must_use]
    pub const fn uniform_scale(s: f64) -> Self {
        Self::scale(s, s)
    }

    #[must_use]
    pub fn rotate(angle_rad: f64) -> Self {
        let (sin, cos) = angle_rad.sin_cos();
        Self {
            a: cos,
            b: -sin,
            tx: 0.0,
            c: sin,
            d: cos,
            ty: 0.0,
        }
    }

    /// Apply this transform to a point.
    #[must_use]
    pub fn apply(&self, x: f64, y: f64) -> (f64, f64) {
        (
            self.a.mul_add(x, self.b * y) + self.tx,
            self.c.mul_add(x, self.d * y) + self.ty,
        )
    }

    /// Compose with another transform (self * other).
    #[must_use]
    pub fn then(self, other: Self) -> Self {
        Self {
            a: self.a.mul_add(other.a, self.b * other.c),
            b: self.a.mul_add(other.b, self.b * other.d),
            tx: self.a.mul_add(other.tx, self.b * other.ty) + self.tx,
            c: self.c.mul_add(other.a, self.d * other.c),
            d: self.c.mul_add(other.b, self.d * other.d),
            ty: self.c.mul_add(other.tx, self.d * other.ty) + self.ty,
        }
    }

    /// Compute the inverse transform, if possible.
    #[must_use]
    pub fn inverse(&self) -> Option<Self> {
        let det = self.a.mul_add(self.d, -(self.b * self.c));
        if det.abs() < f64::EPSILON {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Self {
            a: self.d * inv_det,
            b: -self.b * inv_det,
            tx: self.b.mul_add(self.ty, -(self.d * self.tx)) * inv_det,
            c: -self.c * inv_det,
            d: self.a * inv_det,
            ty: self.c.mul_add(self.tx, -(self.a * self.ty)) * inv_det,
        })
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Mul for Transform2D {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        self.then(rhs)
    }
}

/// A 3D transform for scenes delegated to GPU compute providers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform3D {
    /// 4x4 column-major matrix.
    pub matrix: [f64; 16],
}

impl Transform3D {
    pub const IDENTITY: Self = Self {
        matrix: [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ],
    };

    #[must_use]
    pub const fn translate(x: f64, y: f64, z: f64) -> Self {
        let mut m = Self::IDENTITY;
        m.matrix[12] = x;
        m.matrix[13] = y;
        m.matrix[14] = z;
        m
    }

    #[must_use]
    pub const fn uniform_scale(s: f64) -> Self {
        let mut m = Self::IDENTITY;
        m.matrix[0] = s;
        m.matrix[5] = s;
        m.matrix[10] = s;
        m
    }
}

impl Default for Transform3D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-10;

    #[test]
    fn identity_apply() {
        let t = Transform2D::IDENTITY;
        let (x, y) = t.apply(3.0, 4.0);
        assert!((x - 3.0).abs() < EPS);
        assert!((y - 4.0).abs() < EPS);
    }

    #[test]
    fn translate_apply() {
        let t = Transform2D::translate(10.0, -5.0);
        let (x, y) = t.apply(1.0, 2.0);
        assert!((x - 11.0).abs() < EPS);
        assert!((y - (-3.0)).abs() < EPS);
    }

    #[test]
    fn scale_apply() {
        let t = Transform2D::scale(2.0, 3.0);
        let (x, y) = t.apply(4.0, 5.0);
        assert!((x - 8.0).abs() < EPS);
        assert!((y - 15.0).abs() < EPS);
    }

    #[test]
    fn rotate_90_degrees() {
        let t = Transform2D::rotate(std::f64::consts::FRAC_PI_2);
        let (x, y) = t.apply(1.0, 0.0);
        assert!((x - 0.0).abs() < EPS);
        assert!((y - 1.0).abs() < EPS);
    }

    #[test]
    fn compose_translate_then_scale() {
        let t = Transform2D::translate(5.0, 10.0).then(Transform2D::scale(2.0, 2.0));
        let (x, y) = t.apply(1.0, 1.0);
        // First scale: (2, 2), then translate: (7, 12)
        assert!((x - 7.0).abs() < EPS);
        assert!((y - 12.0).abs() < EPS);
    }

    #[test]
    fn inverse_of_translate() {
        let t = Transform2D::translate(3.0, 4.0);
        let inv = t.inverse().unwrap();
        let (x, y) = inv.apply(5.0, 6.0);
        assert!((x - 2.0).abs() < EPS);
        assert!((y - 2.0).abs() < EPS);
    }

    #[test]
    fn mul_operator() {
        let t1 = Transform2D::translate(1.0, 2.0);
        let t2 = Transform2D::scale(3.0, 4.0);
        let composed = t1 * t2;
        let (x, y) = composed.apply(1.0, 1.0);
        assert!((x - 4.0).abs() < EPS);
        assert!((y - 6.0).abs() < EPS);
    }

    #[test]
    fn transform3d_translate() {
        let t = Transform3D::translate(1.0, 2.0, 3.0);
        assert!((t.matrix[12] - 1.0).abs() < EPS);
        assert!((t.matrix[13] - 2.0).abs() < EPS);
        assert!((t.matrix[14] - 3.0).abs() < EPS);
    }

    #[test]
    fn transform3d_scale() {
        let t = Transform3D::uniform_scale(5.0);
        assert!((t.matrix[0] - 5.0).abs() < EPS);
        assert!((t.matrix[5] - 5.0).abs() < EPS);
        assert!((t.matrix[10] - 5.0).abs() < EPS);
    }
}
