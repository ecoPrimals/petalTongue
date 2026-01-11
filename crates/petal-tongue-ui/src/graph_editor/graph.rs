//! Graph Data Structure
//!
//! Core graph representation for collaborative intelligence.
//!
//! # Design Principles
//!
//! - **No hardcoding**: Node types discovered at runtime
//! - **Validation**: All operations validated before applying
//! - **Immutability**: Operations return new graphs (functional style)
//! - **Serializable**: Can save/load as templates

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};

use super::edge::{DependencyType, GraphEdge};
use super::node::GraphNode;
use super::validation::GraphValidator;

/// Graph - Collection of nodes and edges
///
/// Represents a complete execution graph for collaborative intelligence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Graph {
    /// Unique identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Nodes in the graph
    pub nodes: HashMap<String, GraphNode>,

    /// Edges (dependencies) between nodes
    pub edges: Vec<GraphEdge>,

    /// Metadata (tags, author, etc)
    pub metadata: GraphMetadata,

    /// Version (for optimistic locking)
    pub version: u64,
}

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphMetadata {
    /// Tags for categorization
    pub tags: Vec<String>,

    /// Author (if saved as template)
    pub author: Option<String>,

    /// Creation timestamp
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Last modified timestamp
    pub modified_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Template ID (if loaded from template)
    pub template_id: Option<String>,

    /// Custom metadata (extensible)
    pub custom: HashMap<String, serde_json::Value>,
}

impl Graph {
    /// Create a new empty graph
    #[must_use]
    pub fn new(id: String, name: String) -> Self {
        Self {
            id,
            name,
            description: String::new(),
            nodes: HashMap::new(),
            edges: Vec::new(),
            metadata: GraphMetadata::default(),
            version: 0,
        }
    }

    /// Add a node to the graph
    ///
    /// Validates the node before adding. Returns error if invalid.
    pub fn add_node(&mut self, node: GraphNode) -> Result<()> {
        // Validate node doesn't already exist
        if self.nodes.contains_key(&node.id) {
            anyhow::bail!("Node with id '{}' already exists", node.id);
        }

        // Validate node
        GraphValidator::validate_node(&node)?;

        debug!("Adding node '{}' (type: {})", node.id, node.node_type);
        self.nodes.insert(node.id.clone(), node);
        self.version += 1;

        Ok(())
    }

    /// Remove a node from the graph
    ///
    /// Also removes all edges connected to this node.
    pub fn remove_node(&mut self, node_id: &str) -> Result<Vec<String>> {
        // Check node exists
        if !self.nodes.contains_key(node_id) {
            anyhow::bail!("Node with id '{}' not found", node_id);
        }

        // Remove node
        self.nodes.remove(node_id);

        // Remove all edges connected to this node
        let mut affected_edges = Vec::new();
        self.edges.retain(|edge| {
            if edge.from == node_id || edge.to == node_id {
                affected_edges.push(edge.id.clone());
                false
            } else {
                true
            }
        });

        debug!(
            "Removed node '{}' and {} affected edges",
            node_id,
            affected_edges.len()
        );
        self.version += 1;

        Ok(affected_edges)
    }

    /// Modify a node in the graph
    pub fn modify_node(&mut self, node_id: &str, updated_node: GraphNode) -> Result<()> {
        // Check node exists
        if !self.nodes.contains_key(node_id) {
            anyhow::bail!("Node with id '{}' not found", node_id);
        }

        // Validate updated node
        GraphValidator::validate_node(&updated_node)?;

        // Ensure ID hasn't changed
        if updated_node.id != node_id {
            anyhow::bail!(
                "Cannot change node ID (from '{}' to '{}')",
                node_id,
                updated_node.id
            );
        }

        debug!("Modifying node '{}'", node_id);
        self.nodes.insert(node_id.to_string(), updated_node);
        self.version += 1;

        Ok(())
    }

    /// Add an edge (dependency) between nodes
    pub fn add_edge(&mut self, edge: GraphEdge) -> Result<()> {
        // Validate nodes exist
        if !self.nodes.contains_key(&edge.from) {
            anyhow::bail!("Source node '{}' not found", edge.from);
        }
        if !self.nodes.contains_key(&edge.to) {
            anyhow::bail!("Target node '{}' not found", edge.to);
        }

        // Validate edge doesn't create cycle
        if self.would_create_cycle(&edge)? {
            anyhow::bail!(
                "Edge from '{}' to '{}' would create a cycle",
                edge.from,
                edge.to
            );
        }

        // Validate edge doesn't already exist
        if self.edges.iter().any(|e| e.from == edge.from && e.to == edge.to) {
            anyhow::bail!("Edge from '{}' to '{}' already exists", edge.from, edge.to);
        }

        debug!("Adding edge from '{}' to '{}'", edge.from, edge.to);
        self.edges.push(edge);
        self.version += 1;

        Ok(())
    }

    /// Remove an edge
    pub fn remove_edge(&mut self, edge_id: &str) -> Result<()> {
        let initial_len = self.edges.len();
        self.edges.retain(|edge| edge.id != edge_id);

        if self.edges.len() == initial_len {
            anyhow::bail!("Edge with id '{}' not found", edge_id);
        }

        debug!("Removed edge '{}'", edge_id);
        self.version += 1;

        Ok(())
    }

    /// Get a node by ID
    #[must_use]
    pub fn get_node(&self, node_id: &str) -> Option<&GraphNode> {
        self.nodes.get(node_id)
    }

    /// Get all nodes
    #[must_use]
    pub fn get_nodes(&self) -> Vec<&GraphNode> {
        self.nodes.values().collect()
    }

    /// Get all edges
    #[must_use]
    pub fn get_edges(&self) -> &[GraphEdge] {
        &self.edges
    }

    /// Get incoming edges for a node
    #[must_use]
    pub fn get_incoming_edges(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|edge| edge.to == node_id).collect()
    }

    /// Get outgoing edges for a node
    #[must_use]
    pub fn get_outgoing_edges(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|edge| edge.from == node_id).collect()
    }

    /// Validate the entire graph
    pub fn validate(&self) -> Result<()> {
        GraphValidator::validate_graph(self)
    }

    /// Check if adding an edge would create a cycle
    fn would_create_cycle(&self, new_edge: &GraphEdge) -> Result<bool> {
        // Build adjacency list including the new edge
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for edge in &self.edges {
            adj.entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
        }

        // Add new edge
        adj.entry(new_edge.from.clone())
            .or_default()
            .push(new_edge.to.clone());

        // DFS to detect cycle starting from new_edge.to
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        Ok(Self::has_cycle_dfs(
            &new_edge.to,
            &adj,
            &mut visited,
            &mut rec_stack,
        ))
    }

    /// DFS helper for cycle detection
    fn has_cycle_dfs(
        node: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        if rec_stack.contains(node) {
            return true; // Cycle detected
        }

        if visited.contains(node) {
            return false; // Already processed
        }

        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = adj.get(node) {
            for neighbor in neighbors {
                if Self::has_cycle_dfs(neighbor, adj, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// Get topological sort of nodes (execution order)
    ///
    /// Returns nodes in execution order (dependencies first).
    pub fn topological_sort(&self) -> Result<Vec<String>> {
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize
        for node_id in self.nodes.keys() {
            in_degree.insert(node_id.clone(), 0);
            adj.insert(node_id.clone(), Vec::new());
        }

        // Build adjacency list and in-degree
        for edge in &self.edges {
            adj.entry(edge.from.clone())
                .or_default()
                .push(edge.to.clone());
            *in_degree.entry(edge.to.clone()).or_default() += 1;
        }

        // Kahn's algorithm
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|&(_, degree)| *degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(node) = queue.pop() {
            result.push(node.clone());

            if let Some(neighbors) = adj.get(&node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        // Check if all nodes were processed (no cycles)
        if result.len() != self.nodes.len() {
            anyhow::bail!("Graph contains cycles, cannot compute topological sort");
        }

        Ok(result)
    }

    /// Get statistics about the graph
    #[must_use]
    pub fn stats(&self) -> GraphStats {
        GraphStats {
            node_count: self.nodes.len(),
            edge_count: self.edges.len(),
            max_depth: self.max_depth(),
            has_cycles: self.topological_sort().is_err(),
        }
    }

    /// Calculate maximum depth of the graph
    fn max_depth(&self) -> usize {
        let adj: HashMap<String, Vec<String>> = self
            .edges
            .iter()
            .fold(HashMap::new(), |mut acc, edge| {
                acc.entry(edge.from.clone())
                    .or_default()
                    .push(edge.to.clone());
                acc
            });

        self.nodes
            .keys()
            .map(|node| self.calculate_depth(node, &adj, &mut HashSet::new()))
            .max()
            .unwrap_or(0)
    }

    /// Calculate depth for a node (DFS)
    fn calculate_depth(
        &self,
        node: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
    ) -> usize {
        if visited.contains(node) {
            return 0; // Cycle or already visited
        }

        visited.insert(node.to_string());

        let depth = if let Some(neighbors) = adj.get(node) {
            neighbors
                .iter()
                .map(|neighbor| self.calculate_depth(neighbor, adj, visited))
                .max()
                .unwrap_or(0)
                + 1
        } else {
            1
        };

        visited.remove(node);
        depth
    }
}

impl Default for GraphMetadata {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            author: None,
            created_at: Some(chrono::Utc::now()),
            modified_at: Some(chrono::Utc::now()),
            template_id: None,
            custom: HashMap::new(),
        }
    }
}

/// Graph statistics
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub max_depth: usize,
    pub has_cycles: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_graph() {
        let graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
        assert_eq!(graph.id, "test-1");
        assert_eq!(graph.name, "Test Graph");
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
        let node = GraphNode::new("node-1".to_string(), "test-type".to_string());

        assert!(graph.add_node(node).is_ok());
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn test_add_duplicate_node() {
        let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
        let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
        let node2 = GraphNode::new("node-1".to_string(), "test-type".to_string());

        assert!(graph.add_node(node1).is_ok());
        assert!(graph.add_node(node2).is_err());
    }

    #[test]
    fn test_remove_node() {
        let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
        let node = GraphNode::new("node-1".to_string(), "test-type".to_string());

        graph.add_node(node).unwrap();
        assert_eq!(graph.nodes.len(), 1);

        let affected = graph.remove_node("node-1").unwrap();
        assert_eq!(graph.nodes.len(), 0);
        assert_eq!(affected.len(), 0);
    }

    #[test]
    fn test_add_edge() {
        let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
        let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
        let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());

        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();

        let edge = GraphEdge::new(
            "edge-1".to_string(),
            "node-1".to_string(),
            "node-2".to_string(),
            DependencyType::Sequential,
        );

        assert!(graph.add_edge(edge).is_ok());
        assert_eq!(graph.edges.len(), 1);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
        let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
        let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());
        let node3 = GraphNode::new("node-3".to_string(), "test-type".to_string());

        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();
        graph.add_node(node3).unwrap();

        // Add edges: 1 → 2 → 3
        let edge1 = GraphEdge::new(
            "edge-1".to_string(),
            "node-1".to_string(),
            "node-2".to_string(),
            DependencyType::Sequential,
        );
        let edge2 = GraphEdge::new(
            "edge-2".to_string(),
            "node-2".to_string(),
            "node-3".to_string(),
            DependencyType::Sequential,
        );

        graph.add_edge(edge1).unwrap();
        graph.add_edge(edge2).unwrap();

        // Try to add edge 3 → 1 (would create cycle)
        let edge3 = GraphEdge::new(
            "edge-3".to_string(),
            "node-3".to_string(),
            "node-1".to_string(),
            DependencyType::Sequential,
        );

        assert!(graph.add_edge(edge3).is_err());
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
        let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
        let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());
        let node3 = GraphNode::new("node-3".to_string(), "test-type".to_string());

        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();
        graph.add_node(node3).unwrap();

        // Add edges: 1 → 2 → 3
        let edge1 = GraphEdge::new(
            "edge-1".to_string(),
            "node-1".to_string(),
            "node-2".to_string(),
            DependencyType::Sequential,
        );
        let edge2 = GraphEdge::new(
            "edge-2".to_string(),
            "node-2".to_string(),
            "node-3".to_string(),
            DependencyType::Sequential,
        );

        graph.add_edge(edge1).unwrap();
        graph.add_edge(edge2).unwrap();

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);

        // node-1 should come before node-2, node-2 before node-3
        let pos1 = sorted.iter().position(|id| id == "node-1").unwrap();
        let pos2 = sorted.iter().position(|id| id == "node-2").unwrap();
        let pos3 = sorted.iter().position(|id| id == "node-3").unwrap();

        assert!(pos1 < pos2);
        assert!(pos2 < pos3);
    }

    #[test]
    fn test_graph_stats() {
        let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
        let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());
        let node2 = GraphNode::new("node-2".to_string(), "test-type".to_string());

        graph.add_node(node1).unwrap();
        graph.add_node(node2).unwrap();

        let edge = GraphEdge::new(
            "edge-1".to_string(),
            "node-1".to_string(),
            "node-2".to_string(),
            DependencyType::Sequential,
        );
        graph.add_edge(edge).unwrap();

        let stats = graph.stats();
        assert_eq!(stats.node_count, 2);
        assert_eq!(stats.edge_count, 1);
        assert!(!stats.has_cycles);
    }
}

