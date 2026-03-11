// SPDX-License-Identifier: AGPL-3.0-only
//! Graph Engine - Core topology representation
//!
//! This module provides the modality-agnostic graph structure.
//! Renderers consume this graph and represent it in their own way.

mod layout;
mod types;

#[cfg(test)]
mod tests;

use crate::property::PropertyValue;
use crate::types::{PrimalId, PrimalInfo, TopologyEdge};
use std::collections::HashMap;

pub use types::{GraphStats, LayoutAlgorithm, Node, Position};

use layout::{circular_layout, force_directed_layout, hierarchical_layout, random_layout};

/// The core graph engine
#[derive(Debug, Clone)]
pub struct GraphEngine {
    /// All nodes in the graph
    nodes: Vec<Node>,
    /// All edges in the graph
    edges: Vec<TopologyEdge>,
    /// Index mapping node ID to position in nodes vec
    node_index: HashMap<PrimalId, usize>,
    /// Current layout algorithm
    layout: LayoutAlgorithm,
}

impl GraphEngine {
    /// Create a new empty graph engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_index: HashMap::new(),
            layout: LayoutAlgorithm::ForceDirected,
        }
    }

    /// Add a node to the graph
    ///
    /// If the primal has position data in properties ("x"/"y" or "position_x"/"position_y"),
    /// that position is used. Otherwise the node is placed at (0, 0) for layout algorithms.
    pub fn add_node(&mut self, info: PrimalInfo) {
        let position = Self::extract_position_from_primal(&info);
        let node = Node::with_position(info, position);
        // PrimalId clone is cheap (Arc)
        self.node_index
            .insert(node.info.id.clone(), self.nodes.len());
        self.nodes.push(node);
    }

    /// Extract position from primal properties if present.
    /// Supports "x"/"y" (scenario convert) and "`position_x"/"position_y`" (scenario provider).
    #[expect(clippy::cast_possible_truncation)]
    fn extract_position_from_primal(info: &PrimalInfo) -> Position {
        let get_f32 = |key: &str| {
            info.properties
                .get(key)
                .and_then(PropertyValue::as_number)
                .map(|n| n as f32)
        };

        let (x, y) = if let (Some(x), Some(y)) = (get_f32("x"), get_f32("y")) {
            (x, y)
        } else if let (Some(x), Some(y)) = (get_f32("position_x"), get_f32("position_y")) {
            (x, y)
        } else {
            (0.0, 0.0)
        };

        Position::new_2d(x, y)
    }

    /// Remove a node from the graph
    pub fn remove_node(&mut self, node_id: &str) -> bool {
        if let Some(index) = self.node_index.remove(node_id) {
            self.nodes.remove(index);

            // Remove all edges connected to this node (PrimalId implements PartialEq<str>)
            self.edges
                .retain(|edge| edge.from.as_str() != node_id && edge.to.as_str() != node_id);

            // Rebuild index
            self.rebuild_index();

            true
        } else {
            false
        }
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: TopologyEdge) {
        // Verify both nodes exist (PrimalId implements Borrow<str> for lookup)
        if self.node_index.contains_key(edge.from.as_str())
            && self.node_index.contains_key(edge.to.as_str())
        {
            self.edges.push(edge);
        }
    }

    /// Remove an edge from the graph
    pub fn remove_edge(&mut self, from: &str, to: &str) -> bool {
        let initial_len = self.edges.len();
        self.edges
            .retain(|edge| !(edge.from.as_str() == from && edge.to.as_str() == to));
        self.edges.len() != initial_len
    }

    /// Get a node by ID
    #[must_use]
    pub fn get_node(&self, node_id: &str) -> Option<&Node> {
        self.node_index
            .get(node_id)
            .and_then(|&index| self.nodes.get(index))
    }

    /// Get a mutable node by ID
    pub fn get_node_mut(&mut self, node_id: &str) -> Option<&mut Node> {
        if let Some(&index) = self.node_index.get(node_id) {
            self.nodes.get_mut(index)
        } else {
            None
        }
    }

    /// Get all nodes
    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Get all edges
    #[must_use]
    pub fn edges(&self) -> &[TopologyEdge] {
        &self.edges
    }

    /// Get neighbors of a node
    #[must_use]
    pub fn neighbors(&self, node_id: &str) -> Vec<&Node> {
        self.edges
            .iter()
            .filter_map(|edge| {
                if edge.from.as_str() == node_id {
                    self.get_node(edge.to.as_str())
                } else if edge.to.as_str() == node_id {
                    self.get_node(edge.from.as_str())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Set the layout algorithm
    pub const fn set_layout(&mut self, layout: LayoutAlgorithm) {
        self.layout = layout;
    }

    /// Get the current layout algorithm
    #[must_use]
    pub const fn get_layout(&self) -> LayoutAlgorithm {
        self.layout
    }

    /// Run layout algorithm to position nodes
    pub fn layout(&mut self, iterations: usize) {
        match self.layout {
            LayoutAlgorithm::ForceDirected => {
                force_directed_layout(&mut self.nodes, &self.edges, iterations);
            }
            LayoutAlgorithm::Hierarchical => {
                hierarchical_layout(&mut self.nodes, &self.edges);
            }
            LayoutAlgorithm::Circular => {
                circular_layout(&mut self.nodes);
            }
            LayoutAlgorithm::Random => {
                random_layout(&mut self.nodes);
            }
        }
    }

    /// Clear the graph
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.node_index.clear();
    }

    /// Rebuild the node index (after removals)
    fn rebuild_index(&mut self) {
        self.node_index.clear();
        for (index, node) in self.nodes.iter().enumerate() {
            self.node_index.insert(node.info.id.clone(), index);
        }
    }

    /// Get graph statistics
    #[must_use]
    pub fn stats(&self) -> GraphStats {
        // Note: Precision loss is acceptable for large graphs (>16M nodes)
        // as avg_degree is a statistical approximation
        #[expect(clippy::cast_precision_loss)]
        let avg_degree = if self.nodes.is_empty() {
            0.0
        } else {
            (self.edges.len() * 2) as f32 / self.nodes.len() as f32
        };

        GraphStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            avg_degree,
        }
    }
}

impl Default for GraphEngine {
    fn default() -> Self {
        Self::new()
    }
}
