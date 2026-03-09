// SPDX-License-Identifier: AGPL-3.0-only
//! Hierarchical scene graph with typed nodes and spatial transforms.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::primitive::Primitive;
use crate::transform::Transform2D;

/// Stable identifier for a scene node.
pub type NodeId = String;

/// A node in the scene graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneNode {
    pub id: NodeId,
    pub transform: Transform2D,
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
    pub fn with_transform(mut self, t: Transform2D) -> Self {
        self.transform = t;
        self
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
    pub fn with_visible(mut self, v: bool) -> Self {
        self.visible = v;
        self
    }

    /// Builder: set opacity.
    #[must_use]
    pub fn with_opacity(mut self, o: f32) -> Self {
        self.opacity = o;
        self
    }

    /// Total primitive count including this node.
    pub fn primitive_count(&self) -> usize {
        self.primitives.len()
    }
}

/// The scene graph: a flat map of nodes with a root.
///
/// Nodes reference children by ID. The graph is acyclic by construction
/// (callers must not create cycles). Flat storage enables O(1) lookup
/// and efficient serialization for IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneGraph {
    nodes: HashMap<NodeId, SceneNode>,
    root_id: NodeId,
}

impl SceneGraph {
    /// Create a new scene graph with a root node.
    pub fn new() -> Self {
        let root = SceneNode::new("root");
        let root_id = root.id.clone();
        let mut nodes = HashMap::new();
        nodes.insert(root_id.clone(), root);
        Self { nodes, root_id }
    }

    /// Get the root node ID.
    pub fn root_id(&self) -> &str {
        &self.root_id
    }

    /// Add a node to the graph, parented to the given parent.
    /// Returns false if the parent doesn't exist.
    pub fn add_node(&mut self, node: SceneNode, parent_id: &str) -> bool {
        let child_id = node.id.clone();
        if !self.nodes.contains_key(parent_id) {
            return false;
        }
        self.nodes.insert(child_id.clone(), node);
        if let Some(parent) = self.nodes.get_mut(parent_id) {
            parent.children.push(child_id);
        }
        true
    }

    /// Add a node directly under root.
    pub fn add_to_root(&mut self, node: SceneNode) {
        let child_id = node.id.clone();
        self.nodes.insert(child_id.clone(), node);
        if let Some(root) = self.nodes.get_mut(&self.root_id) {
            root.children.push(child_id);
        }
    }

    /// Get a node by ID.
    pub fn get(&self, id: &str) -> Option<&SceneNode> {
        self.nodes.get(id)
    }

    /// Get a mutable node by ID.
    pub fn get_mut(&mut self, id: &str) -> Option<&mut SceneNode> {
        self.nodes.get_mut(id)
    }

    /// Remove a node and all its descendants.
    pub fn remove(&mut self, id: &str) -> Option<SceneNode> {
        if id == self.root_id {
            return None; // cannot remove root
        }
        // Remove from parent's children
        for node in self.nodes.values_mut() {
            node.children.retain(|c| c != id);
        }
        // Collect descendants
        let mut to_remove = vec![id.to_string()];
        let mut i = 0;
        while i < to_remove.len() {
            if let Some(node) = self.nodes.get(&to_remove[i]) {
                to_remove.extend(node.children.clone());
            }
            i += 1;
        }
        let mut removed = None;
        for rid in &to_remove {
            let n = self.nodes.remove(rid);
            if rid == id {
                removed = n;
            }
        }
        removed
    }

    /// Total number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Total primitive count across all nodes.
    pub fn total_primitives(&self) -> usize {
        self.nodes.values().map(SceneNode::primitive_count).sum()
    }

    /// Iterate all node IDs.
    pub fn node_ids(&self) -> impl Iterator<Item = &str> {
        self.nodes.keys().map(String::as_str)
    }

    /// Collect the flattened list of (transform, primitive) pairs via depth-first traversal.
    /// The transform is the composed world transform from root to the node.
    pub fn flatten(&self) -> Vec<(Transform2D, &Primitive)> {
        let mut result = Vec::new();
        self.flatten_node(&self.root_id, Transform2D::IDENTITY, &mut result);
        result
    }

    fn flatten_node<'a>(
        &'a self,
        node_id: &str,
        parent_transform: Transform2D,
        out: &mut Vec<(Transform2D, &'a Primitive)>,
    ) {
        let Some(node) = self.nodes.get(node_id) else {
            return;
        };
        if !node.visible {
            return;
        }
        let world_transform = parent_transform.then(node.transform);
        for prim in &node.primitives {
            out.push((world_transform, prim));
        }
        for child_id in &node.children {
            self.flatten_node(child_id, world_transform, out);
        }
    }

    /// Find all primitives whose data_id matches, returning their world transforms.
    pub fn find_by_data_id(&self, data_id: &str) -> Vec<(Transform2D, &Primitive)> {
        self.flatten()
            .into_iter()
            .filter(|(_, p)| p.data_id() == Some(data_id))
            .collect()
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::Primitive;

    const EPS: f64 = 1e-10;

    #[test]
    fn new_creates_root() {
        let g = SceneGraph::new();
        assert_eq!(g.root_id(), "root");
        assert_eq!(g.node_count(), 1);
        assert!(g.get("root").is_some());
    }

    #[test]
    fn add_to_root_and_get() {
        let mut g = SceneGraph::new();
        let node = SceneNode::new("child1");
        g.add_to_root(node);
        assert_eq!(g.node_count(), 2);
        let child = g.get("child1").unwrap();
        assert_eq!(child.id, "child1");
        assert!(
            g.get("root")
                .unwrap()
                .children
                .contains(&"child1".to_string())
        );
    }

    #[test]
    fn add_node_with_parent() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("parent"));
        let child = SceneNode::new("child");
        let ok = g.add_node(child, "parent");
        assert!(ok);
        assert_eq!(g.node_count(), 3);
        assert!(
            g.get("parent")
                .unwrap()
                .children
                .contains(&"child".to_string())
        );
    }

    #[test]
    fn add_node_with_invalid_parent_returns_false() {
        let mut g = SceneGraph::new();
        let node = SceneNode::new("orphan");
        let ok = g.add_node(node, "nonexistent");
        assert!(!ok);
        assert_eq!(g.node_count(), 1);
    }

    #[test]
    fn remove_node_and_descendants() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("a"));
        g.add_node(SceneNode::new("b"), "a");
        g.add_node(SceneNode::new("c"), "b");
        assert_eq!(g.node_count(), 4);
        let removed = g.remove("a");
        assert!(removed.is_some());
        assert_eq!(g.node_count(), 1);
        assert!(g.get("a").is_none());
        assert!(g.get("b").is_none());
        assert!(g.get("c").is_none());
    }

    #[test]
    fn cannot_remove_root() {
        let mut g = SceneGraph::new();
        let removed = g.remove("root");
        assert!(removed.is_none());
        assert_eq!(g.node_count(), 1);
    }

    #[test]
    fn node_count() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("a"));
        g.add_to_root(SceneNode::new("b"));
        assert_eq!(g.node_count(), 3);
    }

    #[test]
    fn total_primitives() {
        let mut g = SceneGraph::new();
        let prim = Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 1.0,
            fill: None,
            stroke: None,
            data_id: None,
        };
        g.add_to_root(SceneNode::new("a").with_primitive(prim.clone()));
        g.add_to_root(
            SceneNode::new("b")
                .with_primitive(prim.clone())
                .with_primitive(prim),
        );
        assert_eq!(g.total_primitives(), 3);
    }

    #[test]
    fn flatten_produces_correct_transforms() {
        let mut g = SceneGraph::new();
        let t_a = Transform2D::translate(10.0, 20.0);
        let t_b = Transform2D::translate(5.0, 0.0);
        let prim = Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 1.0,
            fill: None,
            stroke: None,
            data_id: None,
        };
        g.add_to_root(
            SceneNode::new("a")
                .with_transform(t_a)
                .with_primitive(prim.clone()),
        );
        g.add_node(
            SceneNode::new("b").with_transform(t_b).with_primitive(prim),
            "a",
        );
        let flat = g.flatten();
        assert_eq!(flat.len(), 2);
        // First: root->a, so world = t_a
        assert!((flat[0].0.tx - 10.0).abs() < EPS);
        assert!((flat[0].0.ty - 20.0).abs() < EPS);
        // Second: root->a->b, so world = t_a.then(t_b)
        let composed = t_a.then(t_b);
        assert!((flat[1].0.tx - composed.tx).abs() < EPS);
        assert!((flat[1].0.ty - composed.ty).abs() < EPS);
    }

    #[test]
    fn find_by_data_id() {
        let mut g = SceneGraph::new();
        let prim_foo = Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 1.0,
            fill: None,
            stroke: None,
            data_id: Some("foo".to_string()),
        };
        let prim_bar = Primitive::Point {
            x: 1.0,
            y: 1.0,
            radius: 1.0,
            fill: None,
            stroke: None,
            data_id: Some("bar".to_string()),
        };
        let prim_foo2 = Primitive::Point {
            x: 2.0,
            y: 2.0,
            radius: 1.0,
            fill: None,
            stroke: None,
            data_id: Some("foo".to_string()),
        };
        g.add_to_root(SceneNode::new("a").with_primitive(prim_foo));
        g.add_to_root(SceneNode::new("b").with_primitive(prim_bar));
        g.add_to_root(SceneNode::new("c").with_primitive(prim_foo2));
        let found = g.find_by_data_id("foo");
        assert_eq!(found.len(), 2);
        let found_bar = g.find_by_data_id("bar");
        assert_eq!(found_bar.len(), 1);
        let found_none = g.find_by_data_id("baz");
        assert_eq!(found_none.len(), 0);
    }
}
