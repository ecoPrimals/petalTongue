//! Graph Builder Core Types
//!
//! Visual graph representation for Neural API graph construction.
//! Provides data structures for nodes, edges, and graph manipulation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A visual graph representation for Neural API graphs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualGraph {
    /// Unique graph ID
    pub id: String,

    /// Human-readable graph name
    pub name: String,

    /// Graph description
    pub description: Option<String>,

    /// All nodes in the graph
    pub nodes: Vec<GraphNode>,

    /// Edges connecting nodes
    pub edges: Vec<GraphEdge>,

    /// Layout metadata (positions, zoom, etc.)
    pub layout: GraphLayout,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
}

impl VisualGraph {
    /// Create a new empty graph
    #[must_use]
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            nodes: Vec::new(),
            edges: Vec::new(),
            layout: GraphLayout::default(),
            created_at: now,
            modified_at: now,
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: GraphNode) {
        self.nodes.push(node);
        self.modified_at = Utc::now();
    }

    /// Remove a node by ID
    pub fn remove_node(&mut self, node_id: &str) {
        self.nodes.retain(|n| n.id != node_id);
        // Remove all edges connected to this node
        self.edges.retain(|e| e.from != node_id && e.to != node_id);
        self.modified_at = Utc::now();
    }

    /// Get a node by ID
    #[must_use]
    pub fn get_node(&self, node_id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.id == node_id)
    }

    /// Get a mutable reference to a node by ID
    pub fn get_node_mut(&mut self, node_id: &str) -> Option<&mut GraphNode> {
        self.nodes.iter_mut().find(|n| n.id == node_id)
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: GraphEdge) -> Result<(), String> {
        // Validate that both nodes exist
        if !self.nodes.iter().any(|n| n.id == edge.from) {
            return Err(format!("Source node '{}' not found", edge.from));
        }
        if !self.nodes.iter().any(|n| n.id == edge.to) {
            return Err(format!("Target node '{}' not found", edge.to));
        }

        // Check for duplicate edges
        if self
            .edges
            .iter()
            .any(|e| e.from == edge.from && e.to == edge.to)
        {
            return Err(format!(
                "Edge from '{}' to '{}' already exists",
                edge.from, edge.to
            ));
        }

        self.edges.push(edge);
        self.modified_at = Utc::now();
        Ok(())
    }

    /// Remove an edge
    pub fn remove_edge(&mut self, from: &str, to: &str) {
        self.edges.retain(|e| !(e.from == from && e.to == to));
        self.modified_at = Utc::now();
    }

    /// Get all edges from a specific node
    #[must_use]
    pub fn get_outgoing_edges(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.from == node_id).collect()
    }

    /// Get all edges to a specific node
    #[must_use]
    pub fn get_incoming_edges(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.to == node_id).collect()
    }

    /// Check if graph contains a cycle using DFS
    #[must_use]
    pub fn has_cycle(&self) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in &self.nodes {
            if !visited.contains(&node.id)
                && self.has_cycle_util(&node.id, &mut visited, &mut rec_stack)
            {
                return true;
            }
        }
        false
    }

    /// DFS utility for cycle detection
    fn has_cycle_util(
        &self,
        node_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node_id.to_string());
        rec_stack.insert(node_id.to_string());

        // Check all outgoing edges
        for edge in self.get_outgoing_edges(node_id) {
            if !visited.contains(&edge.to) {
                if self.has_cycle_util(&edge.to, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(&edge.to) {
                return true; // Back edge found (cycle)
            }
        }

        rec_stack.remove(node_id);
        false
    }

    /// Get all nodes with no incoming edges (entry points)
    #[must_use]
    pub fn get_entry_nodes(&self) -> Vec<&GraphNode> {
        self.nodes
            .iter()
            .filter(|n| !self.edges.iter().any(|e| e.to == n.id))
            .collect()
    }

    /// Get all nodes with no outgoing edges (terminal nodes)
    #[must_use]
    pub fn get_terminal_nodes(&self) -> Vec<&GraphNode> {
        self.nodes
            .iter()
            .filter(|n| !self.edges.iter().any(|e| e.from == n.id))
            .collect()
    }
}

/// A node in the visual graph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node ID (within this graph)
    pub id: String,

    /// Node type (primal_start, verification, wait_for, conditional)
    pub node_type: NodeType,

    /// Display position on canvas
    pub position: Vec2,

    /// Node-specific parameters
    pub parameters: HashMap<String, String>,

    /// Visual state (transient, not persisted)
    #[serde(skip)]
    pub visual_state: NodeVisualState,
}

impl GraphNode {
    /// Create a new node
    #[must_use]
    pub fn new(node_type: NodeType, position: Vec2) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            node_type,
            position,
            parameters: HashMap::new(),
            visual_state: NodeVisualState::default(),
        }
    }

    /// Set a parameter value
    pub fn set_parameter(&mut self, key: String, value: String) {
        self.parameters.insert(key, value);
    }

    /// Get a parameter value
    #[must_use]
    pub fn get_parameter(&self, key: &str) -> Option<&String> {
        self.parameters.get(key)
    }

    /// Check if all required parameters are set
    #[must_use]
    pub fn has_all_required_parameters(&self) -> bool {
        let required = self.node_type.required_parameters();
        required.iter().all(|k| self.parameters.contains_key(*k))
    }

    /// Get missing required parameters
    #[must_use]
    pub fn missing_parameters(&self) -> Vec<&'static str> {
        let required = self.node_type.required_parameters();
        required
            .iter()
            .filter(|k| !self.parameters.contains_key(**k))
            .copied()
            .collect()
    }
}

/// Node types supported by Neural API
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// Start a primal
    PrimalStart,

    /// Verify primal health
    Verification,

    /// Wait for condition
    WaitFor,

    /// Conditional branch
    Conditional,
}

impl NodeType {
    /// Get human-readable name
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::PrimalStart => "Start Primal",
            Self::Verification => "Verify Health",
            Self::WaitFor => "Wait For",
            Self::Conditional => "Conditional",
        }
    }

    /// Get icon for node type
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::PrimalStart => "🚀",
            Self::Verification => "✅",
            Self::WaitFor => "⏳",
            Self::Conditional => "❓",
        }
    }

    /// Get description
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::PrimalStart => "Start a primal service",
            Self::Verification => "Verify primal health status",
            Self::WaitFor => "Wait for condition to be met",
            Self::Conditional => "Branch based on condition",
        }
    }

    /// Get required parameters
    #[must_use]
    pub const fn required_parameters(&self) -> &'static [&'static str] {
        match self {
            Self::PrimalStart => &["primal_name", "family_id"],
            Self::Verification => &["primal_name", "timeout"],
            Self::WaitFor => &["condition", "timeout"],
            Self::Conditional => &["condition"],
        }
    }
}

/// Edge connecting two nodes
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node ID
    pub from: String,

    /// Target node ID
    pub to: String,

    /// Edge type (dependency, data flow, etc.)
    pub edge_type: EdgeType,
}

impl GraphEdge {
    /// Create a new edge
    #[must_use]
    pub fn new(from: String, to: String, edge_type: EdgeType) -> Self {
        Self {
            from,
            to,
            edge_type,
        }
    }

    /// Create a dependency edge
    #[must_use]
    pub fn dependency(from: String, to: String) -> Self {
        Self::new(from, to, EdgeType::Dependency)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EdgeType {
    /// Node B depends on Node A (execution order)
    Dependency,

    /// Data flows from A to B
    DataFlow,
}

/// Layout metadata for the graph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphLayout {
    /// Camera position
    pub camera_position: Vec2,

    /// Zoom level (1.0 = 100%)
    pub zoom: f32,

    /// Grid enabled
    pub grid_enabled: bool,

    /// Grid size
    pub grid_size: f32,

    /// Snap to grid
    pub snap_to_grid: bool,
}

impl Default for GraphLayout {
    fn default() -> Self {
        Self {
            camera_position: Vec2 { x: 0.0, y: 0.0 },
            zoom: 1.0,
            grid_enabled: true,
            grid_size: 50.0,
            snap_to_grid: true,
        }
    }
}

/// 2D vector for positions
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Distance to another point
    #[must_use]
    pub fn distance(&self, other: &Self) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Snap to grid
    #[must_use]
    pub fn snap(&self, grid_size: f32) -> Self {
        Self {
            x: (self.x / grid_size).round() * grid_size,
            y: (self.y / grid_size).round() * grid_size,
        }
    }
}

/// Visual state for a node (transient)
#[derive(Clone, Debug, Default)]
pub struct NodeVisualState {
    /// Node is selected
    pub selected: bool,

    /// Node has validation errors
    pub has_error: bool,

    /// Error message (if any)
    pub error_message: Option<String>,

    /// Node is being hovered
    pub hovered: bool,

    /// Node is being dragged
    pub dragging: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_creation() {
        let graph = VisualGraph::new("test-graph".to_string());
        assert_eq!(graph.name, "test-graph");
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }

    #[test]
    fn test_add_node() {
        let mut graph = VisualGraph::new("test".to_string());
        let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        let node_id = node.id.clone();
        graph.add_node(node);

        assert_eq!(graph.nodes.len(), 1);
        assert!(graph.get_node(&node_id).is_some());
    }

    #[test]
    fn test_remove_node() {
        let mut graph = VisualGraph::new("test".to_string());
        let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        let node_id = node.id.clone();
        graph.add_node(node);

        graph.remove_node(&node_id);
        assert!(graph.nodes.is_empty());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = VisualGraph::new("test".to_string());
        let node1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        let node2 = GraphNode::new(NodeType::Verification, Vec2::new(100.0, 0.0));
        let id1 = node1.id.clone();
        let id2 = node2.id.clone();

        graph.add_node(node1);
        graph.add_node(node2);

        let edge = GraphEdge::dependency(id1.clone(), id2.clone());
        assert!(graph.add_edge(edge).is_ok());
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_add_edge_invalid_node() {
        let mut graph = VisualGraph::new("test".to_string());
        let node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        let id = node.id.clone();
        graph.add_node(node);

        let edge = GraphEdge::dependency(id, "nonexistent".to_string());
        assert!(graph.add_edge(edge).is_err());
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = VisualGraph::new("test".to_string());

        // Create three nodes
        let node1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        let node2 = GraphNode::new(NodeType::Verification, Vec2::new(100.0, 0.0));
        let node3 = GraphNode::new(NodeType::WaitFor, Vec2::new(200.0, 0.0));
        let id1 = node1.id.clone();
        let id2 = node2.id.clone();
        let id3 = node3.id.clone();

        graph.add_node(node1);
        graph.add_node(node2);
        graph.add_node(node3);

        // Create a cycle: 1 -> 2 -> 3 -> 1
        graph
            .add_edge(GraphEdge::dependency(id1.clone(), id2.clone()))
            .unwrap();
        graph
            .add_edge(GraphEdge::dependency(id2.clone(), id3.clone()))
            .unwrap();
        graph.add_edge(GraphEdge::dependency(id3, id1)).unwrap();

        assert!(graph.has_cycle());
    }

    #[test]
    fn test_no_cycle() {
        let mut graph = VisualGraph::new("test".to_string());

        let node1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        let node2 = GraphNode::new(NodeType::Verification, Vec2::new(100.0, 0.0));
        let node3 = GraphNode::new(NodeType::WaitFor, Vec2::new(200.0, 0.0));
        let id1 = node1.id.clone();
        let id2 = node2.id.clone();
        let id3 = node3.id.clone();

        graph.add_node(node1);
        graph.add_node(node2);
        graph.add_node(node3);

        // Linear: 1 -> 2 -> 3
        graph
            .add_edge(GraphEdge::dependency(id1, id2.clone()))
            .unwrap();
        graph.add_edge(GraphEdge::dependency(id2, id3)).unwrap();

        assert!(!graph.has_cycle());
    }

    #[test]
    fn test_entry_and_terminal_nodes() {
        let mut graph = VisualGraph::new("test".to_string());

        let node1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        let node2 = GraphNode::new(NodeType::Verification, Vec2::new(100.0, 0.0));
        let node3 = GraphNode::new(NodeType::WaitFor, Vec2::new(200.0, 0.0));
        let id1 = node1.id.clone();
        let id2 = node2.id.clone();
        let id3 = node3.id.clone();

        graph.add_node(node1);
        graph.add_node(node2);
        graph.add_node(node3);

        // 1 -> 2 -> 3
        graph
            .add_edge(GraphEdge::dependency(id1.clone(), id2.clone()))
            .unwrap();
        graph
            .add_edge(GraphEdge::dependency(id2, id3.clone()))
            .unwrap();

        let entry = graph.get_entry_nodes();
        let terminal = graph.get_terminal_nodes();

        assert_eq!(entry.len(), 1);
        assert_eq!(entry[0].id, id1);

        assert_eq!(terminal.len(), 1);
        assert_eq!(terminal[0].id, id3);
    }

    #[test]
    fn test_node_required_parameters() {
        let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        assert!(!node.has_all_required_parameters());

        node.set_parameter("primal_name".to_string(), "beardog".to_string());
        assert!(!node.has_all_required_parameters());

        node.set_parameter("family_id".to_string(), "nat0".to_string());
        assert!(node.has_all_required_parameters());
    }

    #[test]
    fn test_vec2_snap() {
        let pos = Vec2::new(47.0, 53.0);
        let snapped = pos.snap(50.0);
        assert_eq!(snapped, Vec2::new(50.0, 50.0));

        let pos2 = Vec2::new(23.0, 27.0);
        let snapped2 = pos2.snap(50.0);
        assert_eq!(snapped2, Vec2::new(0.0, 50.0));
    }
}
