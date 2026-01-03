//! Graph Engine - Core topology representation
//!
//! This module provides the modality-agnostic graph structure.
//! Renderers consume this graph and represent it in their own way.

use crate::types::{PrimalInfo, TopologyEdge};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Position in 2D or 3D space
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub struct Position {
    /// X coordinate in graph space
    pub x: f32,
    /// Y coordinate in graph space
    pub y: f32,
    /// Optional Z coordinate for 3D positioning
    pub z: Option<f32>,
}

impl Position {
    /// Create a 2D position
    #[must_use]
    pub const fn new_2d(x: f32, y: f32) -> Self {
        Self { x, y, z: None }
    }

    /// Create a 3D position
    #[must_use]
    pub const fn new_3d(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z: Some(z) }
    }

    /// Convert to 3D (uses z=0 if 2D)
    #[must_use]
    pub fn to_3d(self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: Some(self.z.unwrap_or(0.0)),
        }
    }

    /// Calculate distance to another position (2D)
    #[must_use]
    pub fn distance_to(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Calculate distance to another position (3D if available)
    #[must_use]
    pub fn distance_to_3d(self, other: Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z.unwrap_or(0.0) - other.z.unwrap_or(0.0);
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// A node in the graph (represents a primal)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    /// Node properties (health, capabilities, etc.)
    pub info: PrimalInfo,
    /// Current position (set by layout algorithm)
    pub position: Position,
    /// Velocity (used by physics-based layouts)
    #[serde(skip)]
    pub velocity: Position,
}

impl Node {
    /// Create a new node with default position
    #[must_use]
    pub fn new(info: PrimalInfo) -> Self {
        Self {
            info,
            position: Position::new_2d(0.0, 0.0),
            velocity: Position::new_2d(0.0, 0.0),
        }
    }

    /// Create a new node with specific position
    #[must_use]
    pub fn with_position(info: PrimalInfo, position: Position) -> Self {
        Self {
            info,
            position,
            velocity: Position::new_2d(0.0, 0.0),
        }
    }
}

/// The core graph engine
#[derive(Debug, Clone)]
pub struct GraphEngine {
    /// All nodes in the graph
    nodes: Vec<Node>,
    /// All edges in the graph
    edges: Vec<TopologyEdge>,
    /// Index mapping node ID to position in nodes vec
    node_index: HashMap<String, usize>,
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
    pub fn add_node(&mut self, info: PrimalInfo) {
        let node_id = info.id.clone();
        let node = Node::new(info);

        self.node_index.insert(node_id, self.nodes.len());
        self.nodes.push(node);
    }

    /// Remove a node from the graph
    pub fn remove_node(&mut self, node_id: &str) -> bool {
        if let Some(index) = self.node_index.remove(node_id) {
            self.nodes.remove(index);

            // Remove all edges connected to this node
            self.edges
                .retain(|edge| edge.from != node_id && edge.to != node_id);

            // Rebuild index
            self.rebuild_index();

            true
        } else {
            false
        }
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: TopologyEdge) {
        // Verify both nodes exist
        if self.node_index.contains_key(&edge.from) && self.node_index.contains_key(&edge.to) {
            self.edges.push(edge);
        }
    }

    /// Remove an edge from the graph
    pub fn remove_edge(&mut self, from: &str, to: &str) -> bool {
        let initial_len = self.edges.len();
        self.edges
            .retain(|edge| !(edge.from == from && edge.to == to));
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
                if edge.from == node_id {
                    self.get_node(&edge.to)
                } else if edge.to == node_id {
                    self.get_node(&edge.from)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Set the layout algorithm
    pub fn set_layout(&mut self, layout: LayoutAlgorithm) {
        self.layout = layout;
    }

    /// Get the current layout algorithm
    #[must_use]
    pub fn get_layout(&self) -> LayoutAlgorithm {
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
        #[allow(clippy::cast_precision_loss)]
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

/// Graph statistics
#[derive(Debug, Clone, Copy)]
pub struct GraphStats {
    /// Number of nodes in the graph
    pub node_count: usize,
    /// Number of edges in the graph
    pub edge_count: usize,
    /// Average node degree (connections per node)
    pub avg_degree: f32,
}

/// Layout algorithm types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayoutAlgorithm {
    /// Force-directed layout (physics-based)
    ForceDirected,
    /// Hierarchical layout (tree-like)
    Hierarchical,
    /// Circular layout (nodes in a circle)
    Circular,
    /// Random layout (for testing)
    Random,
}

// Layout algorithm implementations

/// Force-directed layout using Fruchterman-Reingold algorithm
fn force_directed_layout(nodes: &mut [Node], edges: &[TopologyEdge], iterations: usize) {
    const K: f32 = 100.0; // Optimal distance between nodes
    const AREA: f32 = 1000.0; // Layout area
    const COOLING_FACTOR: f32 = 0.95;

    // Initialize nodes with random positions if they're all at origin
    let all_at_origin = nodes
        .iter()
        .all(|n| n.position.x == 0.0 && n.position.y == 0.0);
    if all_at_origin {
        random_layout(nodes);
    }

    let mut temperature = AREA / 10.0;

    for _ in 0..iterations {
        // Calculate repulsive forces (all pairs)
        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let delta_x = nodes[i].position.x - nodes[j].position.x;
                let delta_y = nodes[i].position.y - nodes[j].position.y;
                let distance = (delta_x * delta_x + delta_y * delta_y).sqrt().max(0.01);

                let repulsion = K * K / distance;
                let force_x = (delta_x / distance) * repulsion;
                let force_y = (delta_y / distance) * repulsion;

                nodes[i].velocity.x += force_x;
                nodes[i].velocity.y += force_y;
                nodes[j].velocity.x -= force_x;
                nodes[j].velocity.y -= force_y;
            }
        }

        // Calculate attractive forces (edges)
        for edge in edges {
            if let (Some(from_idx), Some(to_idx)) = (
                nodes.iter().position(|n| n.info.id == edge.from),
                nodes.iter().position(|n| n.info.id == edge.to),
            ) {
                let delta_x = nodes[from_idx].position.x - nodes[to_idx].position.x;
                let delta_y = nodes[from_idx].position.y - nodes[to_idx].position.y;
                let distance = (delta_x * delta_x + delta_y * delta_y).sqrt().max(0.01);

                let attraction = distance * distance / K;
                let force_x = (delta_x / distance) * attraction;
                let force_y = (delta_y / distance) * attraction;

                nodes[from_idx].velocity.x -= force_x;
                nodes[from_idx].velocity.y -= force_y;
                nodes[to_idx].velocity.x += force_x;
                nodes[to_idx].velocity.y += force_y;
            }
        }

        // Apply velocities with cooling
        for node in nodes.iter_mut() {
            let v_len =
                (node.velocity.x * node.velocity.x + node.velocity.y * node.velocity.y).sqrt();
            if v_len > 0.0 {
                let displacement = v_len.min(temperature);
                node.position.x += (node.velocity.x / v_len) * displacement;
                node.position.y += (node.velocity.y / v_len) * displacement;
            }

            // Reset velocity
            node.velocity.x = 0.0;
            node.velocity.y = 0.0;
        }

        temperature *= COOLING_FACTOR;
    }
}

/// Hierarchical layout (simple tree-like layout)
fn hierarchical_layout(nodes: &mut [Node], edges: &[TopologyEdge]) {
    // Find root nodes (nodes with no incoming edges)
    let mut incoming_counts: HashMap<String, usize> = HashMap::new();
    for edge in edges {
        *incoming_counts.entry(edge.to.clone()).or_insert(0) += 1;
    }

    let roots: Vec<String> = nodes
        .iter()
        .filter(|node| !incoming_counts.contains_key(&node.info.id))
        .map(|node| node.info.id.clone())
        .collect();

    // Assign levels using BFS
    let mut levels: HashMap<String, usize> = HashMap::new();
    let mut queue = roots.clone();
    for root in &roots {
        levels.insert(root.clone(), 0);
    }

    while let Some(current) = queue.pop() {
        let current_level = levels[&current];
        for edge in edges {
            if edge.from == current && !levels.contains_key(&edge.to) {
                levels.insert(edge.to.clone(), current_level + 1);
                queue.push(edge.to.clone());
            }
        }
    }

    // Position nodes by level
    let mut level_counts: HashMap<usize, usize> = HashMap::new();
    for node in nodes.iter_mut() {
        let level = levels.get(&node.info.id).copied().unwrap_or(0);
        let count = level_counts.entry(level).or_insert(0);

        #[allow(clippy::cast_precision_loss)]
        {
            node.position.x = (*count as f32) * 150.0;
            node.position.y = (level as f32) * 150.0;
        }

        *count += 1;
    }
}

/// Circular layout (nodes arranged in a circle)
#[allow(clippy::cast_precision_loss)] // Precision loss acceptable for layout
fn circular_layout(nodes: &mut [Node]) {
    let radius = 300.0;
    let angle_step = 2.0 * std::f32::consts::PI / nodes.len() as f32;

    for (i, node) in nodes.iter_mut().enumerate() {
        let angle = (i as f32) * angle_step;
        node.position.x = angle.cos() * radius;
        node.position.y = angle.sin() * radius;
    }
}

/// Random layout (for testing)
#[allow(clippy::cast_precision_loss)] // Precision loss acceptable for layout
fn random_layout(nodes: &mut [Node]) {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    for node in nodes.iter_mut() {
        // Use node ID as seed for deterministic "random" layout
        let mut hasher = DefaultHasher::new();
        node.info.id.hash(&mut hasher);
        let hash = hasher.finish();

        let x = ((hash % 1000) as f32 - 500.0) * 2.0;
        let y = (((hash / 1000) % 1000) as f32 - 500.0) * 2.0;

        node.position.x = x;
        node.position.y = y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures::primals;

    fn create_test_primal(id: &str, name: &str) -> PrimalInfo {
        let mut primal = primals::test_primal(id);
        primal.name = name.to_string();
        primal
    }

    #[test]
    fn test_graph_creation() {
        let graph = GraphEngine::new();
        assert_eq!(graph.nodes().len(), 0);
        assert_eq!(graph.edges().len(), 0);
    }

    #[test]
    fn test_add_nodes() {
        let mut graph = GraphEngine::new();

        graph.add_node(create_test_primal("1", "Node 1"));
        graph.add_node(create_test_primal("2", "Node 2"));

        assert_eq!(graph.nodes().len(), 2);
        assert!(graph.get_node("1").is_some());
        assert!(graph.get_node("2").is_some());
    }

    #[test]
    fn test_add_edges() {
        let mut graph = GraphEngine::new();

        graph.add_node(create_test_primal("1", "Node 1"));
        graph.add_node(create_test_primal("2", "Node 2"));

        graph.add_edge(TopologyEdge {
            from: "1".to_string(),
            to: "2".to_string(),
            edge_type: "test".to_string(),
            label: None,
        });

        assert_eq!(graph.edges().len(), 1);
    }

    #[test]
    fn test_remove_node() {
        let mut graph = GraphEngine::new();

        graph.add_node(create_test_primal("1", "Node 1"));
        graph.add_node(create_test_primal("2", "Node 2"));
        graph.add_edge(TopologyEdge {
            from: "1".to_string(),
            to: "2".to_string(),
            edge_type: "test".to_string(),
            label: None,
        });

        assert!(graph.remove_node("1"));
        assert_eq!(graph.nodes().len(), 1);
        assert_eq!(graph.edges().len(), 0); // Edge should be removed too
    }

    #[test]
    fn test_neighbors() {
        let mut graph = GraphEngine::new();

        graph.add_node(create_test_primal("1", "Node 1"));
        graph.add_node(create_test_primal("2", "Node 2"));
        graph.add_node(create_test_primal("3", "Node 3"));

        graph.add_edge(TopologyEdge {
            from: "1".to_string(),
            to: "2".to_string(),
            edge_type: "test".to_string(),
            label: None,
        });
        graph.add_edge(TopologyEdge {
            from: "1".to_string(),
            to: "3".to_string(),
            edge_type: "test".to_string(),
            label: None,
        });

        let neighbors = graph.neighbors("1");
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_force_directed_layout() {
        let mut graph = GraphEngine::new();

        graph.add_node(create_test_primal("1", "Node 1"));
        graph.add_node(create_test_primal("2", "Node 2"));
        graph.add_node(create_test_primal("3", "Node 3"));
        graph.add_edge(TopologyEdge {
            from: "1".to_string(),
            to: "2".to_string(),
            edge_type: "test".to_string(),
            label: None,
        });
        graph.add_edge(TopologyEdge {
            from: "2".to_string(),
            to: "3".to_string(),
            edge_type: "test".to_string(),
            label: None,
        });

        graph.set_layout(LayoutAlgorithm::ForceDirected);
        graph.layout(50); // More iterations

        // Nodes should have moved from initial (0, 0) positions
        // With 3 nodes and repulsive forces, they should spread out
        let node1 = graph.get_node("1").unwrap();
        let node2 = graph.get_node("2").unwrap();
        let node3 = graph.get_node("3").unwrap();

        // At least one node should have moved significantly
        let total_movement = node1.position.x.abs()
            + node1.position.y.abs()
            + node2.position.x.abs()
            + node2.position.y.abs()
            + node3.position.x.abs()
            + node3.position.y.abs();

        assert!(
            total_movement > 10.0,
            "Nodes should have spread out, total movement: {total_movement}"
        );
    }

    #[test]
    fn test_circular_layout() {
        let mut graph = GraphEngine::new();

        for i in 0..5 {
            graph.add_node(create_test_primal(&i.to_string(), &format!("Node {i}")));
        }

        graph.set_layout(LayoutAlgorithm::Circular);
        graph.layout(1);

        // All nodes should be roughly the same distance from origin
        let radius = 300.0;
        for node in graph.nodes() {
            let dist =
                (node.position.x * node.position.x + node.position.y * node.position.y).sqrt();
            assert!((dist - radius).abs() < 0.1);
        }
    }

    #[test]
    fn test_graph_stats() {
        let mut graph = GraphEngine::new();

        graph.add_node(create_test_primal("1", "Node 1"));
        graph.add_node(create_test_primal("2", "Node 2"));
        graph.add_node(create_test_primal("3", "Node 3"));

        graph.add_edge(TopologyEdge {
            from: "1".to_string(),
            to: "2".to_string(),
            edge_type: "test".to_string(),
            label: None,
        });
        graph.add_edge(TopologyEdge {
            from: "2".to_string(),
            to: "3".to_string(),
            edge_type: "test".to_string(),
            label: None,
        });

        let stats = graph.stats();
        assert_eq!(stats.node_count, 3);
        assert_eq!(stats.edge_count, 2);
        assert!((stats.avg_degree - 1.33).abs() < 0.01);
    }
}
