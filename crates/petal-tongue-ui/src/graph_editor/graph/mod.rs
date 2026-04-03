// SPDX-License-Identifier: AGPL-3.0-or-later
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

mod serialization;
mod validation;

#[cfg(test)]
mod tests;

pub use serialization::GraphMetadata;

use crate::error::{GraphEditorError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

use super::edge::GraphEdge;
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
    ///
    /// # Errors
    ///
    /// Returns an error if the node already exists or validation fails.
    pub fn add_node(&mut self, node: GraphNode) -> Result<()> {
        // Validate node doesn't already exist
        if self.nodes.contains_key(&node.id) {
            return Err(GraphEditorError::NodeAlreadyExists(node.id).into());
        }

        // Validate node
        GraphValidator::validate_node(&node)?;

        debug!("Adding node '{}' (type: {})", node.id, node.node_type);
        let id = node.id.clone();
        self.nodes.insert(id, node);
        self.version += 1;

        Ok(())
    }

    /// Remove a node from the graph
    ///
    /// Also removes all edges connected to this node.
    ///
    /// # Errors
    ///
    /// Returns an error if the node does not exist.
    pub fn remove_node(&mut self, node_id: &str) -> Result<Vec<String>> {
        // Check node exists
        if !self.nodes.contains_key(node_id) {
            return Err(GraphEditorError::NodeNotFound(node_id.to_string()).into());
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
    ///
    /// # Errors
    ///
    /// Returns an error if the node does not exist, validation fails, or the node ID changes.
    pub fn modify_node(&mut self, node_id: &str, updated_node: GraphNode) -> Result<()> {
        // Check node exists
        if !self.nodes.contains_key(node_id) {
            return Err(GraphEditorError::NodeNotFound(node_id.to_string()).into());
        }

        // Validate updated node
        GraphValidator::validate_node(&updated_node)?;

        // Ensure ID hasn't changed
        if updated_node.id != node_id {
            return Err(GraphEditorError::NodeIdChange {
                from: node_id.to_string(),
                to: updated_node.id,
            }
            .into());
        }

        debug!("Modifying node '{}'", node_id);
        self.nodes.insert(node_id.to_string(), updated_node);
        self.version += 1;

        Ok(())
    }

    /// Add an edge (dependency) between nodes
    ///
    /// # Errors
    ///
    /// Returns an error if source or target node does not exist, the edge would create a cycle, or the edge already exists.
    pub fn add_edge(&mut self, edge: GraphEdge) -> Result<()> {
        // Validate nodes exist
        if !self.nodes.contains_key(&edge.from) {
            return Err(GraphEditorError::SourceNodeNotFound(edge.from).into());
        }
        if !self.nodes.contains_key(&edge.to) {
            return Err(GraphEditorError::TargetNodeNotFound(edge.to).into());
        }

        // Validate edge doesn't create cycle
        if self.would_create_cycle(&edge)? {
            return Err(GraphEditorError::EdgeWouldCreateCycle {
                from: edge.from,
                to: edge.to,
            }
            .into());
        }

        // Validate edge doesn't already exist
        if self
            .edges
            .iter()
            .any(|e| e.from == edge.from && e.to == edge.to)
        {
            return Err(GraphEditorError::EdgeAlreadyExists {
                from: edge.from,
                to: edge.to,
            }
            .into());
        }

        debug!("Adding edge from '{}' to '{}'", edge.from, edge.to);
        self.edges.push(edge);
        self.version += 1;

        Ok(())
    }

    /// Remove an edge
    ///
    /// # Errors
    ///
    /// Returns an error if the edge does not exist.
    pub fn remove_edge(&mut self, edge_id: &str) -> Result<()> {
        let initial_len = self.edges.len();
        self.edges.retain(|edge| edge.id != edge_id);

        if self.edges.len() == initial_len {
            return Err(GraphEditorError::EdgeNotFound(edge_id.to_string()).into());
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
        self.edges
            .iter()
            .filter(|edge| edge.to == node_id)
            .collect()
    }

    /// Get outgoing edges for a node
    #[must_use]
    pub fn get_outgoing_edges(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges
            .iter()
            .filter(|edge| edge.from == node_id)
            .collect()
    }

    /// Validate the entire graph
    ///
    /// # Errors
    ///
    /// Returns an error if any node is invalid, edges reference missing nodes, or the graph contains cycles.
    pub fn validate(&self) -> Result<()> {
        GraphValidator::validate_graph(self)
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
}

/// Graph statistics
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphStats {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Maximum depth of the graph
    pub max_depth: usize,
    /// Whether the graph contains cycles
    pub has_cycles: bool,
}
