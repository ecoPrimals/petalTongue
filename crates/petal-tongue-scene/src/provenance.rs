// SPDX-License-Identifier: AGPL-3.0-only
//! Pixel-exact provenance tracking for scene graph primitives.

#![expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use crate::primitive::Primitive;
use crate::scene_graph::SceneGraph;
use crate::transform::Transform2D;

/// Provenance for a single pixel: which primitive owns it.
#[derive(Debug, Clone, PartialEq)]
pub struct PixelProvenance {
    /// The scene graph node that owns the primitive.
    pub node_id: String,
    /// Index in the flattened output.
    pub primitive_index: usize,
    /// The data_id from the primitive.
    pub data_id: Option<String>,
    /// World coordinates of the primitive origin.
    pub world_x: f64,
    pub world_y: f64,
}

/// Per-pixel provenance buffer.
#[derive(Debug, Clone)]
pub struct ProvenanceBuffer {
    width: u32,
    height: u32,
    buffer: Vec<Option<PixelProvenance>>,
}

impl ProvenanceBuffer {
    /// Creates an empty buffer (all pixels None).
    #[must_use]
    pub fn new(width: u32, height: u32) -> Self {
        let len = width.saturating_mul(height) as usize;
        Self {
            width,
            height,
            buffer: vec![None; len],
        }
    }

    /// Sets provenance for a pixel. Clamps coordinates to buffer bounds.
    pub fn set(&mut self, x: u32, y: u32, provenance: PixelProvenance) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.buffer[idx] = Some(provenance);
        }
    }

    /// Gets provenance for a pixel.
    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> Option<&PixelProvenance> {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.buffer[idx].as_ref()
        } else {
            None
        }
    }

    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }
}

/// Renders scene graph into a provenance buffer (pixel-exact tracking).
#[derive(Debug, Default)]
pub struct ProvenanceRenderer;

impl ProvenanceRenderer {
    /// Renders the scene into a provenance buffer. World coords map 1:1 to pixels.
    #[must_use]
    pub fn render(&self, scene: &SceneGraph, width: u32, height: u32) -> ProvenanceBuffer {
        let mut buf = ProvenanceBuffer::new(width, height);
        let flat = scene.flatten_with_ids();
        for (idx, (transform, prim, node_id)) in flat.into_iter().enumerate() {
            let prov = PixelProvenance {
                node_id: node_id.as_str().to_string(),
                primitive_index: idx,
                data_id: prim.data_id().map(String::from),
                world_x: 0.0,
                world_y: 0.0,
            };
            Self::fill_primitive(&mut buf, transform, prim, prov);
        }
        buf
    }

    fn fill_primitive(
        buf: &mut ProvenanceBuffer,
        transform: Transform2D,
        prim: &Primitive,
        mut prov: PixelProvenance,
    ) {
        match prim {
            Primitive::Rect {
                x,
                y,
                width,
                height,
                ..
            } => {
                let (wx, wy) = transform.apply(*x, *y);
                prov.world_x = wx;
                prov.world_y = wy;
                let (x1, y1) = transform.apply(*x + width, *y + height);
                let min_x = wx.min(x1).floor().max(0.0);
                let max_x = wx.max(x1).ceil();
                let min_y = wy.min(y1).floor().max(0.0);
                let max_y = wy.max(y1).ceil();
                Self::fill_rect(buf, min_x, min_y, max_x, max_y, &prov);
            }
            Primitive::Point { x, y, radius, .. } => {
                let (wx, wy) = transform.apply(*x, *y);
                prov.world_x = wx;
                prov.world_y = wy;
                Self::fill_circle(buf, wx, wy, *radius, &prov);
            }
            Primitive::Line { points, stroke, .. } => {
                let w = stroke.width as f64 / 2.0;
                for i in 0..points.len().saturating_sub(1) {
                    let (ax, ay) = transform.apply(points[i][0], points[i][1]);
                    let (bx, by) = transform.apply(points[i + 1][0], points[i + 1][1]);
                    prov.world_x = ax;
                    prov.world_y = ay;
                    let (min_x, min_y) = (ax.min(bx) - w, ay.min(by) - w);
                    let (max_x, max_y) = (ax.max(bx) + w, ay.max(by) + w);
                    Self::fill_rect(buf, min_x, min_y, max_x, max_y, &prov);
                }
            }
            _ => {
                let (min_x, min_y, max_x, max_y) = Self::primitive_bounds(prim, transform);
                prov.world_x = min_x;
                prov.world_y = min_y;
                Self::fill_rect(buf, min_x, min_y, max_x, max_y, &prov);
            }
        }
    }

    fn fill_rect(
        buf: &mut ProvenanceBuffer,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        prov: &PixelProvenance,
    ) {
        let x0 = min_x.floor().max(0.0) as u32;
        let y0 = min_y.floor().max(0.0) as u32;
        let x1 = (max_x.ceil() as u32).min(buf.width());
        let y1 = (max_y.ceil() as u32).min(buf.height());
        for y in y0..y1 {
            for x in x0..x1 {
                buf.set(x, y, prov.clone());
            }
        }
    }

    fn fill_circle(buf: &mut ProvenanceBuffer, cx: f64, cy: f64, r: f64, prov: &PixelProvenance) {
        let x0 = (cx - r).floor().max(0.0) as u32;
        let y0 = (cy - r).floor().max(0.0) as u32;
        let x1 = ((cx + r).ceil() as u32).min(buf.width());
        let y1 = ((cy + r).ceil() as u32).min(buf.height());
        let r2 = r * r;
        for y in y0..y1 {
            for x in x0..x1 {
                let dx = f64::from(x) + 0.5 - cx;
                let dy = f64::from(y) + 0.5 - cy;
                if dx * dx + dy * dy <= r2 {
                    buf.set(x, y, prov.clone());
                }
            }
        }
    }

    fn points_bounds(pts: impl Iterator<Item = [f64; 2]>) -> (f64, f64, f64, f64) {
        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;
        for p in pts {
            min_x = min_x.min(p[0]);
            min_y = min_y.min(p[1]);
            max_x = max_x.max(p[0]);
            max_y = max_y.max(p[1]);
        }
        if min_x > max_x {
            (0.0, 0.0, 0.0, 0.0)
        } else {
            (min_x, min_y, max_x, max_y)
        }
    }

    fn primitive_bounds(prim: &Primitive, transform: Transform2D) -> (f64, f64, f64, f64) {
        let (min_x, min_y, max_x, max_y) = match prim {
            Primitive::Text { x, y, .. } => (*x, *y, *x + 1.0, *y + 1.0),
            Primitive::Polygon { points, .. } => Self::points_bounds(points.iter().copied()),
            Primitive::Arc { cx, cy, radius, .. } => {
                (cx - radius, cy - radius, cx + radius, cy + radius)
            }
            Primitive::BezierPath {
                start, segments, ..
            } => Self::points_bounds(
                std::iter::once(*start).chain(segments.iter().flat_map(|s| [s.cp1, s.cp2, s.end])),
            ),
            Primitive::Mesh { vertices, .. } => {
                Self::points_bounds(vertices.iter().map(|v| [v.position[0], v.position[1]]))
            }
            _ => (0.0, 0.0, 0.0, 0.0),
        };
        let (tx_min, ty_min) = transform.apply(min_x, min_y);
        let (tx_max, ty_max) = transform.apply(max_x, max_y);
        (
            tx_min.min(tx_max),
            ty_min.min(ty_max),
            tx_min.max(tx_max),
            ty_min.max(ty_max),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::Color;
    use crate::scene_graph::SceneNode;

    fn rect(x: f64, y: f64, w: f64, h: f64, data_id: Option<&str>) -> Primitive {
        Primitive::Rect {
            x,
            y,
            width: w,
            height: h,
            fill: Some(Color::BLACK),
            stroke: None,
            corner_radius: 0.0,
            data_id: data_id.map(String::from),
        }
    }

    #[test]
    fn empty_scene_all_none() {
        let buf = ProvenanceRenderer.render(&SceneGraph::new(), 10, 10);
        for y in 0..10 {
            for x in 0..10 {
                assert!(buf.get(x, y).is_none(), "pixel ({x},{y}) should be None");
            }
        }
    }

    #[test]
    fn single_rect_pixels_have_provenance() {
        let mut s = SceneGraph::new();
        s.add_to_root(SceneNode::new("n1").with_primitive(rect(1.0, 2.0, 3.0, 2.0, None)));
        let buf = ProvenanceRenderer.render(&s, 10, 10);
        assert!(buf.get(1, 2).is_some() && buf.get(2, 2).is_some() && buf.get(3, 3).is_some());
        assert_eq!(buf.get(1, 2).unwrap().node_id, "n1");
        assert!(buf.get(0, 0).is_none());
    }

    #[test]
    fn overlapping_rects_topmost_wins() {
        let mut s = SceneGraph::new();
        s.add_to_root(SceneNode::new("bottom").with_primitive(rect(0.0, 0.0, 5.0, 5.0, None)));
        s.add_to_root(SceneNode::new("top").with_primitive(rect(2.0, 2.0, 3.0, 3.0, None)));
        let buf = ProvenanceRenderer.render(&s, 10, 10);
        assert_eq!(buf.get(1, 1).unwrap().node_id, "bottom");
        assert_eq!(buf.get(3, 3).unwrap().node_id, "top");
    }

    #[test]
    fn point_circle_center_has_provenance() {
        let mut s = SceneGraph::new();
        s.add_to_root(SceneNode::new("pt").with_primitive(Primitive::Point {
            x: 5.0,
            y: 5.0,
            radius: 2.0,
            fill: Some(Color::BLACK),
            stroke: None,
            data_id: None,
        }));
        let buf = ProvenanceRenderer.render(&s, 10, 10);
        assert!(buf.get(5, 5).is_some());
        assert_eq!(buf.get(5, 5).unwrap().node_id, "pt");
    }

    #[test]
    fn data_id_propagated_correctly() {
        let mut s = SceneGraph::new();
        s.add_to_root(SceneNode::new("n").with_primitive(rect(
            0.0,
            0.0,
            2.0,
            2.0,
            Some("data-42"),
        )));
        let buf = ProvenanceRenderer.render(&s, 10, 10);
        assert_eq!(buf.get(1, 1).unwrap().data_id.as_deref(), Some("data-42"));
    }
}
