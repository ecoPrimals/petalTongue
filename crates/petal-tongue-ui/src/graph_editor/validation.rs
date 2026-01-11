//! Graph Validation
//!
//! Validates graphs, nodes, and edges to ensure correctness.

use anyhow::{Context, Result};

use super::graph::Graph;
use super::node::GraphNode;

/// Graph validator
pub struct GraphValidator;

impl GraphValidator {
    /// Validate a single node
    pub fn validate_node(node: &GraphNode) -> Result<()> {
        // Validate ID
        if node.id.is_empty() {
            anyhow::bail!("Node ID cannot be empty");
        }

        // Validate node type
        if node.node_type.is_empty() {
            anyhow::bail!("Node type cannot be empty");
        }

        // Validate position (no NaN or infinite values)
        if !node.position.0.is_finite() || !node.position.1.is_finite() {
            anyhow::bail!("Node position must be finite");
        }

        Ok(())
    }

    /// Validate entire graph
    pub fn validate_graph(graph: &Graph) -> Result<()> {
        // Validate all nodes
        for node in graph.nodes.values() {
            Self::validate_node(node).with_context(|| format!("Invalid node '{}'", node.id))?;
        }

        // Validate all edges reference existing nodes
        for edge in &graph.edges {
            if !graph.nodes.contains_key(&edge.from) {
                anyhow::bail!(
                    "Edge '{}' references non-existent source node '{}'",
                    edge.id,
                    edge.from
                );
            }
            if !graph.nodes.contains_key(&edge.to) {
                anyhow::bail!(
                    "Edge '{}' references non-existent target node '{}'",
                    edge.id,
                    edge.to
                );
            }
        }

        // Validate no cycles (already done by topological_sort, but double-check)
        graph
            .topological_sort()
            .context("Graph validation failed: contains cycles")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_editor::edge::{DependencyType, GraphEdge};

    #[test]
    fn test_validate_valid_node() {
        let node = GraphNode::new("node-1".to_string(), "test-type".to_string());
        assert!(GraphValidator::validate_node(&node).is_ok());
    }

    #[test]
    fn test_validate_empty_id() {
        let node = GraphNode::new("".to_string(), "test-type".to_string());
        assert!(GraphValidator::validate_node(&node).is_err());
    }

    #[test]
    fn test_validate_empty_type() {
        let node = GraphNode::new("node-1".to_string(), "".to_string());
        assert!(GraphValidator::validate_node(&node).is_err());
    }

    #[test]
    fn test_validate_valid_graph() {
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

        assert!(GraphValidator::validate_graph(&graph).is_ok());
    }

    #[test]
    fn test_validate_graph_with_missing_node() {
        let mut graph = Graph::new("test-1".to_string(), "Test Graph".to_string());
        let node1 = GraphNode::new("node-1".to_string(), "test-type".to_string());

        graph.add_node(node1).unwrap();

        // Manually add edge (bypassing validation) to test validator
        let edge = GraphEdge::new(
            "edge-1".to_string(),
            "node-1".to_string(),
            "node-2".to_string(), // This node doesn't exist
            DependencyType::Sequential,
        );
        graph.edges.push(edge);

        assert!(GraphValidator::validate_graph(&graph).is_err());
    }
}
