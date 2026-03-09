// SPDX-License-Identifier: AGPL-3.0-only
//! Graph Builder Logic
//!
//! Visual graph manipulation for Neural API graph construction.

use chrono::Utc;
use std::collections::HashSet;

use super::types::{GraphEdge, GraphLayout, GraphNode, VisualGraph};

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
