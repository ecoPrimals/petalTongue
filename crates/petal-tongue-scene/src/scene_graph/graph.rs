// SPDX-License-Identifier: AGPL-3.0-only
//! Scene graph - flat map of nodes with hierarchical structure.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::primitive::Primitive;
use crate::transform::Transform2D;

use super::node::SceneNode;
use crate::node_id::NodeId;

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
        self.root_id.as_str()
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
        if let Some(root) = self.nodes.get_mut(self.root_id.as_str()) {
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
        if self.root_id == id {
            return None; // cannot remove root
        }
        // Remove from parent's children
        for node in self.nodes.values_mut() {
            node.children.retain(|c| c.as_str() != id);
        }
        // Collect descendants (reuse Arc clones, no string allocation)
        let mut to_remove: Vec<NodeId> = vec![id.into()];
        let mut i = 0;
        while i < to_remove.len() {
            if let Some(node) = self.nodes.get(to_remove[i].as_str()) {
                to_remove.extend(node.children.iter().cloned());
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
        self.nodes.keys().map(NodeId::as_str)
    }

    /// Collect the flattened list of (transform, primitive) pairs via depth-first traversal.
    /// The transform is the composed world transform from root to the node.
    #[must_use]
    pub fn flatten(&self) -> Vec<(Transform2D, &Primitive)> {
        self.flatten_with_ids()
            .into_iter()
            .map(|(t, p, _)| (t, p))
            .collect()
    }

    /// Like `flatten`, but also returns the owning node ID for each primitive.
    #[must_use]
    pub fn flatten_with_ids(&self) -> Vec<(Transform2D, &Primitive, &NodeId)> {
        let mut result = Vec::new();
        self.flatten_node(self.root_id.as_str(), Transform2D::IDENTITY, &mut result);
        result
    }

    fn flatten_node<'a>(
        &'a self,
        node_id: &str,
        parent_transform: Transform2D,
        out: &mut Vec<(Transform2D, &'a Primitive, &'a NodeId)>,
    ) {
        let Some(node) = self.nodes.get(node_id) else {
            return;
        };
        if !node.visible {
            return;
        }
        let world_transform = parent_transform.then(node.transform);
        for prim in &node.primitives {
            out.push((world_transform, prim, &node.id));
        }
        for child_id in &node.children {
            self.flatten_node(child_id.as_str(), world_transform, out);
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
