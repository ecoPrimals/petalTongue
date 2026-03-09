// SPDX-License-Identifier: AGPL-3.0-only
//! Edge validation rules: connectivity and structural checks.

use crate::graph_builder::VisualGraph;
use std::collections::HashSet;

use super::types::{ValidationIssue, ValidationResult};

/// Validate all edges in the graph.
pub(super) fn validate_edges(graph: &VisualGraph, result: &mut ValidationResult) {
    // Build node ID set for quick lookups
    let node_ids: HashSet<_> = graph.nodes.iter().map(|n| &n.id).collect();

    for (idx, edge) in graph.edges.iter().enumerate() {
        // Check that source node exists
        if !node_ids.contains(&edge.from) {
            result.add_issue(ValidationIssue::error(format!(
                "Edge {} references non-existent source node: {}",
                idx, edge.from
            )));
        }

        // Check that target node exists
        if !node_ids.contains(&edge.to) {
            result.add_issue(ValidationIssue::error(format!(
                "Edge {} references non-existent target node: {}",
                idx, edge.to
            )));
        }

        // Check for self-loops
        if edge.from == edge.to {
            result.add_issue(
                ValidationIssue::warning(format!("Node '{}' has a self-loop edge", edge.from))
                    .with_suggestion(
                        "Remove self-loop edges - they may cause infinite loops".to_string(),
                    ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_builder::{GraphEdge, GraphNode, NodeType, Vec2, VisualGraph};
    use crate::graph_validation::types::ValidationResult;

    #[test]
    fn test_edge_to_nonexistent_source() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        node.set_parameter("primal_name".to_string(), "x".to_string());
        node.set_parameter("family_id".to_string(), "f1".to_string());
        let id = node.id.clone();
        graph.add_node(node);
        graph
            .edges
            .push(GraphEdge::dependency("ghost".to_string(), id));
        let mut result = ValidationResult::new();
        validate_edges(&graph, &mut result);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_edge_to_nonexistent_target() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        node.set_parameter("primal_name".to_string(), "x".to_string());
        node.set_parameter("family_id".to_string(), "f1".to_string());
        let id = node.id.clone();
        graph.add_node(node);
        graph
            .edges
            .push(GraphEdge::dependency(id, "ghost".to_string()));
        let mut result = ValidationResult::new();
        validate_edges(&graph, &mut result);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_self_loop_warning() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        node.set_parameter("primal_name".to_string(), "x".to_string());
        node.set_parameter("family_id".to_string(), "f1".to_string());
        let id = node.id.clone();
        graph.add_node(node);
        let _ = graph.add_edge(GraphEdge::dependency(id.clone(), id));
        let mut result = ValidationResult::new();
        validate_edges(&graph, &mut result);
        assert!(result.has_warnings());
        assert!(
            result
                .warnings()
                .iter()
                .any(|w| w.message.contains("self-loop"))
        );
    }

    #[test]
    fn test_valid_edges() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut n1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        n1.set_parameter("primal_name".to_string(), "a".to_string());
        n1.set_parameter("family_id".to_string(), "f1".to_string());
        let id1 = n1.id.clone();
        let mut n2 = GraphNode::new(NodeType::Verification, Vec2::zero());
        n2.set_parameter("primal_name".to_string(), "b".to_string());
        n2.set_parameter("timeout".to_string(), "30".to_string());
        let id2 = n2.id.clone();
        graph.add_node(n1);
        graph.add_node(n2);
        graph.add_edge(GraphEdge::dependency(id1, id2)).unwrap();
        let mut result = ValidationResult::new();
        validate_edges(&graph, &mut result);
        assert!(result.is_valid());
    }
}
