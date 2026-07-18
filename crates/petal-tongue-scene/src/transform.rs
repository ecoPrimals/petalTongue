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

    /// Construct from a 2D transform (embeds at z=0).
    #[must_use]
    pub const fn from_2d(t: &Transform2D) -> Self {
        let mut m = Self::IDENTITY;
        m.matrix[0] = t.a;
        m.matrix[1] = t.c;
        m.matrix[4] = t.b;
        m.matrix[5] = t.d;
        m.matrix[12] = t.tx;
        m.matrix[13] = t.ty;
        m
    }
}

impl Default for Transform3D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Camera projection model for scene rendering.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Projection {
    /// Orthographic (parallel) projection — used for 2D scenes.
    /// Width/height define the visible area in world units.
    Orthographic {
        width: f64,
        height: f64,
        near: f64,
        far: f64,
    },
    /// Perspective projection — used for 3D scenes.
    /// `fov_y` is the vertical field of view in radians.
    Perspective {
        fov_y: f64,
        aspect: f64,
        near: f64,
        far: f64,
    },
}

impl Projection {
    /// Standard 2D orthographic projection at z=0.
    /// The visible area is `width × height` world units centered at origin.
    #[must_use]
    pub const fn ortho_2d(width: f64, height: f64) -> Self {
        Self::Orthographic {
            width,
            height,
            near: -1.0,
            far: 1.0,
        }
    }

    /// Standard 3D perspective with 60° vertical FOV.
    #[must_use]
    pub const fn perspective_default(aspect: f64) -> Self {
        Self::Perspective {
            fov_y: 1.047_197_551_196_597_6, // 60° in radians
            aspect,
            near: 0.1,
            far: 1000.0,
        }
    }
}

impl Default for Projection {
    fn default() -> Self {
        Self::ortho_2d(800.0, 600.0)
    }
}

/// Camera defining the viewpoint for a scene.
///
/// For 2D scenes (the default), the camera sits at z=0 looking down the -Z axis
/// with orthographic projection. This preserves full backward compatibility:
/// existing 2D scenes render identically.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    /// Camera position and orientation in world space.
    pub transform: Transform3D,
    /// Projection model.
    pub projection: Projection,
}

impl Camera {
    /// Default 2D camera: identity transform, orthographic projection.
    pub const ORTHO_2D: Self = Self {
        transform: Transform3D::IDENTITY,
        projection: Projection::Orthographic {
            width: 800.0,
            height: 600.0,
            near: -1.0,
            far: 1.0,
        },
    };

    /// Create a 2D camera with specific viewport dimensions.
    #[must_use]
    pub const fn ortho(width: f64, height: f64) -> Self {
        Self {
            transform: Transform3D::IDENTITY,
            projection: Projection::ortho_2d(width, height),
        }
    }

    /// Create a 3D perspective camera at a given position.
    #[must_use]
    pub const fn perspective(position: Transform3D, aspect: f64) -> Self {
        Self {
            transform: position,
            projection: Projection::perspective_default(aspect),
        }
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

    #[test]
    fn transform3d_from_2d_preserves_translation() {
        let t2d = Transform2D::translate(3.0, 7.0);
        let t3d = Transform3D::from_2d(&t2d);
        assert!((t3d.matrix[12] - 3.0).abs() < EPS);
        assert!((t3d.matrix[13] - 7.0).abs() < EPS);
        assert!((t3d.matrix[14] - 0.0).abs() < EPS);
    }

    #[test]
    fn transform3d_from_2d_preserves_rotation() {
        let t2d = Transform2D::rotate(std::f64::consts::FRAC_PI_2);
        let t3d = Transform3D::from_2d(&t2d);
        // Column-major: col0 = [a, c, 0, 0], col1 = [b, d, 0, 0]
        assert!((t3d.matrix[0] - t2d.a).abs() < EPS);
        assert!((t3d.matrix[1] - t2d.c).abs() < EPS);
        assert!((t3d.matrix[4] - t2d.b).abs() < EPS);
        assert!((t3d.matrix[5] - t2d.d).abs() < EPS);
    }

    #[test]
    fn camera_ortho_2d_default() {
        let cam = Camera::ORTHO_2D;
        assert_eq!(cam.transform, Transform3D::IDENTITY);
        match cam.projection {
            Projection::Orthographic { width, height, .. } => {
                assert!((width - 800.0).abs() < EPS);
                assert!((height - 600.0).abs() < EPS);
            }
            Projection::Perspective { .. } => panic!("expected orthographic"),
        }
    }

    #[test]
    fn camera_perspective_has_fov() {
        let cam = Camera::perspective(Transform3D::translate(0.0, 0.0, 5.0), 16.0 / 9.0);
        match cam.projection {
            Projection::Perspective { fov_y, aspect, .. } => {
                assert!((fov_y - 1.047_197_551_196_597_6).abs() < EPS);
                assert!((aspect - 16.0 / 9.0).abs() < EPS);
            }
            Projection::Orthographic { .. } => panic!("expected perspective"),
        }
    }

    #[test]
    fn projection_default_is_ortho() {
        let proj = Projection::default();
        assert!(matches!(proj, Projection::Orthographic { .. }));
    }
}
