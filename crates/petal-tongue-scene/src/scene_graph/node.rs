// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scene graph node - typed node with transform, primitives, and children.

use serde::{Deserialize, Serialize};

use crate::node_id::NodeId;
use crate::primitive::Primitive;
use crate::transform::{Transform2D, Transform3D};

/// A node in the scene graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: NodeId,
    pub transform: Transform2D,
    /// Optional 3D transform — when present, overrides `transform` for 3D-aware renderers.
    /// 2D renderers ignore this field and use `transform`. This enables the
    /// "2D-as-3D-slice" unification: existing 2D scenes remain unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform_3d: Option<Transform3D>,
    pub primitives: Vec<Primitive>,
    pub children: Vec<NodeId>,
    pub visible: bool,
    pub opacity: f32,
    /// Optional label for debugging and accessibility.
    pub label: Option<String>,
    /// Optional data source ID for interaction engine integration.
    pub data_source: Option<String>,
}

impl SceneNode {
    /// Create a new empty node.
    pub fn new(id: impl Into<NodeId>) -> Self {
        Self {
            id: id.into(),
            transform: Transform2D::IDENTITY,
            transform_3d: None,
            primitives: Vec::new(),
            children: Vec::new(),
            visible: true,
            opacity: 1.0,
            label: None,
            data_source: None,
        }
    }

    /// Builder: set transform.
    #[must_use]
    pub const fn with_transform(mut self, t: Transform2D) -> Self {
        self.transform = t;
        self
    }

    /// Builder: set 3D transform (for 3D-aware renderers).
    #[must_use]
    pub const fn with_transform_3d(mut self, t: Transform3D) -> Self {
        self.transform_3d = Some(t);
        self
    }

    /// Get the effective 3D transform — uses `transform_3d` if set,
    /// otherwise embeds the 2D transform at z=0.
    #[must_use]
    pub const fn effective_transform_3d(&self) -> Transform3D {
        match self.transform_3d {
            Some(t) => t,
            None => Transform3D::from_2d(&self.transform),
        }
    }

    /// Builder: add a primitive.
    #[must_use]
    pub fn with_primitive(mut self, p: Primitive) -> Self {
        self.primitives.push(p);
        self
    }

    /// Builder: set label.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Builder: set visibility.
    #[must_use]
    pub const fn with_visible(mut self, v: bool) -> Self {
        self.visible = v;
        self
    }

    /// Builder: set opacity.
    #[must_use]
    pub const fn with_opacity(mut self, o: f32) -> Self {
        self.opacity = o;
        self
    }

    /// Total primitive count including this node.
    #[must_use]
    pub const fn primitive_count(&self) -> usize {
        self.primitives.len()
    }
}
