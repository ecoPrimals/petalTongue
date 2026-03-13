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
    pub const fn with_transform(mut self, t: Transform2D) -> Self {
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
    #[must_use]
    pub fn new() -> Self {
        let root = SceneNode::new("root");
        let root_id = root.id.clone();
        let mut nodes = HashMap::new();
        nodes.insert(root_id.clone(), root);
        Self { nodes, root_id }
    }

    /// Get the root node ID.
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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
    #[must_use]
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

    /// Find all primitives whose `data_id` matches, returning their world transforms.
    #[must_use]
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
    use crate::primitive::{Color, Primitive};

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
        let child = g.get("child1").expect("child1 should exist");
        assert_eq!(child.id, "child1");
        assert!(
            g.get("root")
                .expect("root should exist")
                .children
                .contains(&"child1".to_string()),
            "root should have child1"
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
                .expect("parent should exist")
                .children
                .contains(&"child".to_string()),
            "parent should have child"
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

    #[test]
    fn flatten_skips_invisible_nodes() {
        let mut g = SceneGraph::new();
        let prim = Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 1.0,
            fill: None,
            stroke: None,
            data_id: None,
        };
        g.add_to_root(
            SceneNode::new("visible")
                .with_visible(true)
                .with_primitive(prim.clone()),
        );
        g.add_to_root(
            SceneNode::new("hidden")
                .with_visible(false)
                .with_primitive(prim),
        );
        let flat = g.flatten();
        assert_eq!(flat.len(), 1);
    }

    #[test]
    fn node_ids_iteration() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("a"));
        g.add_to_root(SceneNode::new("b"));
        let ids: Vec<_> = g.node_ids().collect();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"root"));
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
    }

    #[test]
    fn remove_returns_removed_node() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("to_remove"));
        let removed = g.remove("to_remove");
        assert!(removed.is_some());
        assert_eq!(removed.expect("removed node should exist").id, "to_remove");
        assert!(g.get("to_remove").is_none());
    }

    #[test]
    fn add_node_preserves_children_order() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("p"));
        g.add_node(SceneNode::new("c1"), "p");
        g.add_node(SceneNode::new("c2"), "p");
        let parent = g.get("p").expect("parent p should exist");
        assert_eq!(parent.children.len(), 2);
        assert_eq!(parent.children[0], "c1");
        assert_eq!(parent.children[1], "c2");
    }

    #[test]
    fn scene_node_builder_chain() {
        let node = SceneNode::new("id")
            .with_label("test")
            .with_opacity(0.5)
            .with_visible(false);
        assert_eq!(node.id, "id");
        assert_eq!(node.label.as_deref(), Some("test"));
        assert!((node.opacity - 0.5).abs() < f32::EPSILON);
        assert!(!node.visible);
    }

    #[test]
    fn scene_node_primitive_count() {
        let prim = Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 1.0,
            fill: None,
            stroke: None,
            data_id: None,
        };
        let node = SceneNode::new("n")
            .with_primitive(prim.clone())
            .with_primitive(prim);
        assert_eq!(node.primitive_count(), 2);
    }

    #[test]
    fn flatten_empty_primitives() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("empty"));
        let flat = g.flatten();
        assert_eq!(flat.len(), 0);
    }

    #[test]
    fn default_scene_graph() {
        let g = SceneGraph::default();
        assert_eq!(g.root_id(), "root");
        assert_eq!(g.node_count(), 1);
    }

    #[test]
    fn scene_node_with_transform() {
        let t = Transform2D::translate(50.0, 100.0);
        let node = SceneNode::new("n").with_transform(t);
        assert!((node.transform.tx - 50.0).abs() < EPS);
        assert!((node.transform.ty - 100.0).abs() < EPS);
    }

    #[test]
    fn flatten_missing_child_node_skipped() {
        let mut g = SceneGraph::new();
        let prim = Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 1.0,
            fill: None,
            stroke: None,
            data_id: None,
        };
        let root = g.get_mut("root").expect("root exists");
        root.children.push("phantom".to_string());
        root.primitives.push(prim);
        let flat = g.flatten();
        assert_eq!(
            flat.len(),
            1,
            "phantom child does not exist, only root primitives"
        );
    }

    #[test]
    fn remove_leaf_node() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("parent"));
        g.add_node(SceneNode::new("leaf"), "parent");
        assert_eq!(g.node_count(), 3);
        let removed = g.remove("leaf");
        assert!(removed.is_some());
        assert_eq!(g.node_count(), 2);
        assert!(g.get("leaf").is_none());
        let parent = g.get("parent").expect("parent exists");
        assert!(!parent.children.contains(&"leaf".to_string()));
    }

    #[test]
    fn get_mut_modifies_node() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("n").with_opacity(0.5));
        {
            let node = g.get_mut("n").expect("n exists");
            node.opacity = 0.8;
        }
        let node = g.get("n").expect("n exists");
        assert!((node.opacity - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn get_nonexistent_returns_none() {
        let mut g = SceneGraph::new();
        assert!(g.get("nonexistent").is_none());
        assert!(g.get_mut("nonexistent").is_none());
    }

    #[test]
    fn transform_composition_in_hierarchy() {
        let mut g = SceneGraph::new();
        let t1 = Transform2D::translate(10.0, 0.0);
        let t2 = Transform2D::scale(2.0, 2.0);
        let prim = Primitive::Point {
            x: 1.0,
            y: 1.0,
            radius: 1.0,
            fill: None,
            stroke: None,
            data_id: None,
        };
        g.add_to_root(
            SceneNode::new("a")
                .with_transform(t1)
                .with_primitive(prim.clone()),
        );
        g.add_node(
            SceneNode::new("b").with_transform(t2).with_primitive(prim),
            "a",
        );
        let flat = g.flatten();
        assert_eq!(flat.len(), 2, "should have 2 primitives from a and b");
        let (t_a, _) = &flat[0];
        let (t_ab, _) = &flat[1];
        let (wx1, wy1) = t_a.apply(1.0, 1.0);
        let (wx2, wy2) = t_ab.apply(1.0, 1.0);
        assert!(
            (wx1 - 11.0).abs() < EPS,
            "a: translate(10,0) + (1,1) = (11,1)"
        );
        assert!((wy1 - 1.0).abs() < EPS);
        assert!(
            (wx2 - 12.0).abs() < EPS,
            "a->b: composed transform scales then translates"
        );
        assert!((wy2 - 2.0).abs() < EPS);
    }

    #[test]
    fn find_by_data_id_empty_graph() {
        let g = SceneGraph::new();
        let found = g.find_by_data_id("any");
        assert!(found.is_empty());
    }

    #[test]
    fn scene_node_default_properties() {
        let node = SceneNode::new("id");
        assert_eq!(node.id, "id");
        assert_eq!(node.transform, Transform2D::IDENTITY);
        assert!(node.primitives.is_empty());
        assert!(node.children.is_empty());
        assert!(node.visible);
        assert!((node.opacity - 1.0).abs() < f32::EPSILON);
        assert!(node.label.is_none());
        assert!(node.data_source.is_none());
    }

    #[test]
    fn scene_node_serialization_roundtrip() {
        let node = SceneNode::new("test_node")
            .with_label("Test Label")
            .with_opacity(0.7)
            .with_visible(true)
            .with_primitive(Primitive::Point {
                x: 1.0,
                y: 2.0,
                radius: 3.0,
                fill: Some(Color::WHITE),
                stroke: None,
                data_id: Some("pt-1".to_string()),
            });
        let json = serde_json::to_string(&node).expect("serialization should succeed");
        let decoded: SceneNode =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(node.id, decoded.id);
        assert_eq!(node.label, decoded.label);
        assert!((node.opacity - decoded.opacity).abs() < f32::EPSILON);
        assert_eq!(node.visible, decoded.visible);
        assert_eq!(node.primitives.len(), decoded.primitives.len());
    }

    #[test]
    fn scene_graph_serialization_roundtrip() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("child1").with_label("Child").with_primitive(
            Primitive::Point {
                x: 0.0,
                y: 0.0,
                radius: 1.0,
                fill: None,
                stroke: None,
                data_id: None,
            },
        ));
        let json = serde_json::to_string(&g).expect("serialization should succeed");
        let decoded: SceneGraph =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(g.root_id(), decoded.root_id());
        assert_eq!(g.node_count(), decoded.node_count());
        assert!(decoded.get("child1").is_some());
    }

    #[test]
    fn scene_node_with_data_source() {
        let mut node = SceneNode::new("n");
        node.data_source = Some("ds-1".to_string());
        assert_eq!(node.data_source.as_deref(), Some("ds-1"));
    }

    #[test]
    fn scene_node_debug_formatting() {
        let node = SceneNode::new("debug_node");
        let s = format!("{node:?}");
        assert!(s.contains("debug_node"));
    }

    #[test]
    fn scene_graph_debug_formatting() {
        let g = SceneGraph::new();
        let s = format!("{g:?}");
        assert!(!s.is_empty());
    }

    #[test]
    fn scene_node_clone() {
        let node = SceneNode::new("orig").with_opacity(0.5);
        let cloned = node.clone();
        assert_eq!(node.id, cloned.id);
        assert!((node.opacity - cloned.opacity).abs() < f32::EPSILON);
    }

    #[test]
    fn scene_graph_clone() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("a"));
        let cloned = g.clone();
        assert_eq!(g.node_count(), cloned.node_count());
        assert!(cloned.get("a").is_some());
    }

    #[test]
    fn flatten_node_with_multiple_primitives() {
        let mut g = SceneGraph::new();
        let prim = Primitive::Point {
            x: 0.0,
            y: 0.0,
            radius: 1.0,
            fill: None,
            stroke: None,
            data_id: None,
        };
        g.add_to_root(
            SceneNode::new("multi")
                .with_primitive(prim.clone())
                .with_primitive(prim.clone())
                .with_primitive(prim),
        );
        let flat = g.flatten();
        assert_eq!(flat.len(), 3);
    }

    #[test]
    fn add_node_to_root_multiple_times() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("a"));
        g.add_to_root(SceneNode::new("b"));
        g.add_to_root(SceneNode::new("c"));
        assert_eq!(g.node_count(), 4);
        let root = g.get("root").expect("root exists");
        assert_eq!(root.children.len(), 3);
    }

    #[test]
    fn empty_graph_has_only_root() {
        let g = SceneGraph::new();
        assert_eq!(g.node_count(), 1);
        assert_eq!(g.root_id(), "root");
        let flat = g.flatten();
        assert!(flat.is_empty(), "root has no primitives");
    }

    #[test]
    fn deeply_nested_hierarchy() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("l1"));
        g.add_node(SceneNode::new("l2"), "l1");
        g.add_node(SceneNode::new("l3"), "l2");
        g.add_node(SceneNode::new("l4"), "l3");
        g.add_node(SceneNode::new("l5"), "l4");
        assert_eq!(g.node_count(), 6);
        let l5 = g.get("l5").expect("l5 exists");
        assert!(l5.children.is_empty());
        let l1 = g.get("l1").expect("l1 exists");
        assert_eq!(l1.children.len(), 1);
        assert_eq!(l1.children[0], "l2");
    }

    #[test]
    fn circular_reference_prevention_flat_storage() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("a"));
        g.add_node(SceneNode::new("b"), "a");
        let root = g.get_mut("root").expect("root");
        root.children.push("a".to_string());
        let flat = g.flatten();
        assert!(
            flat.len() <= 2,
            "flatten traverses children; phantom refs to a from root don't create cycles in flat storage"
        );
    }

    #[test]
    fn remove_deep_branch() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("p"));
        g.add_node(SceneNode::new("c1"), "p");
        g.add_node(SceneNode::new("c2"), "c1");
        g.add_node(SceneNode::new("c3"), "c2");
        assert_eq!(g.node_count(), 5);
        let removed = g.remove("c1");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, "c1");
        assert_eq!(g.node_count(), 2);
        assert!(g.get("c1").is_none());
        assert!(g.get("c2").is_none());
        assert!(g.get("c3").is_none());
    }

    #[test]
    fn add_node_overwrites_existing_same_id() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("dup").with_opacity(0.5).with_label("first"));
        g.add_to_root(SceneNode::new("dup").with_opacity(0.9).with_label("second"));
        let node = g.get("dup").expect("dup exists");
        assert!((node.opacity - 0.9).abs() < f32::EPSILON);
        assert_eq!(node.label.as_deref(), Some("second"));
        let root = g.get("root").expect("root");
        assert_eq!(root.children.iter().filter(|c| *c == "dup").count(), 2);
    }

    #[test]
    fn flatten_order_depth_first() {
        let mut g = SceneGraph::new();
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
                .with_transform(Transform2D::translate(1.0, 0.0))
                .with_primitive(prim.clone()),
        );
        g.add_node(
            SceneNode::new("b")
                .with_transform(Transform2D::translate(2.0, 0.0))
                .with_primitive(prim),
            "a",
        );
        let flat = g.flatten();
        assert_eq!(flat.len(), 2);
        assert!((flat[0].0.tx - 1.0).abs() < EPS);
        assert!((flat[1].0.tx - 3.0).abs() < EPS);
    }

    #[test]
    fn node_ids_no_duplicates_in_iteration() {
        let mut g = SceneGraph::new();
        g.add_to_root(SceneNode::new("x"));
        let ids: Vec<_> = g.node_ids().collect();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"root"));
        assert!(ids.contains(&"x"));
    }
}
