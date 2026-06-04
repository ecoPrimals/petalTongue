// SPDX-License-Identifier: AGPL-3.0-or-later
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
                        "Remove self-loop edges - they may cause infinite loops".to_owned(),
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
        let mut graph = VisualGraph::new("g".to_owned());
        let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        node.set_parameter("primal_name".to_owned(), "x".to_owned());
        node.set_parameter("family_id".to_owned(), "f1".to_owned());
        let id = node.id.clone();
        graph.add_node(node);
        graph
            .edges
            .push(GraphEdge::dependency("ghost".to_owned(), id));
        let mut result = ValidationResult::new();
        validate_edges(&graph, &mut result);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_edge_to_nonexistent_target() {
        let mut graph = VisualGraph::new("g".to_owned());
        let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        node.set_parameter("primal_name".to_owned(), "x".to_owned());
        node.set_parameter("family_id".to_owned(), "f1".to_owned());
        let id = node.id.clone();
        graph.add_node(node);
        graph
            .edges
            .push(GraphEdge::dependency(id, "ghost".to_owned()));
        let mut result = ValidationResult::new();
        validate_edges(&graph, &mut result);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_self_loop_warning() {
        let mut graph = VisualGraph::new("g".to_owned());
        let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        node.set_parameter("primal_name".to_owned(), "x".to_owned());
        node.set_parameter("family_id".to_owned(), "f1".to_owned());
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
        let mut graph = VisualGraph::new("g".to_owned());
        let mut n1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        n1.set_parameter("primal_name".to_owned(), "a".to_owned());
        n1.set_parameter("family_id".to_owned(), "f1".to_owned());
        let id1 = n1.id.clone();
        let mut n2 = GraphNode::new(NodeType::Verification, Vec2::zero());
        n2.set_parameter("primal_name".to_owned(), "b".to_owned());
        n2.set_parameter("timeout".to_owned(), "30".to_owned());
        let id2 = n2.id.clone();
        graph.add_node(n1);
        graph.add_node(n2);
        graph
            .add_edge(GraphEdge::dependency(id1, id2))
            .expect("add edge");
        let mut result = ValidationResult::new();
        validate_edges(&graph, &mut result);
        assert!(result.is_valid());
    }

    #[test]
    fn test_edge_both_source_and_target_nonexistent() {
        let graph = VisualGraph::new("g".to_owned());
        let mut g = graph;
        g.edges.push(GraphEdge::dependency(
            "ghost1".to_owned(),
            "ghost2".to_owned(),
        ));
        let mut result = ValidationResult::new();
        validate_edges(&g, &mut result);
        assert!(!result.is_valid());
        assert_eq!(result.errors().len(), 2, "both source and target errors");
    }

    #[test]
    fn test_edge_empty_graph_with_orphan_edge() {
        let mut graph = VisualGraph::new("g".to_owned());
        graph
            .edges
            .push(GraphEdge::dependency("a".to_owned(), "b".to_owned()));
        let mut result = ValidationResult::new();
        validate_edges(&graph, &mut result);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_multiple_valid_edges_no_issues() {
        let mut graph = VisualGraph::new("g".to_owned());
        let mut n1 = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        n1.set_parameter("primal_name".to_owned(), "a".to_owned());
        n1.set_parameter("family_id".to_owned(), "f1".to_owned());
        let id1 = n1.id.clone();
        let mut n2 = GraphNode::new(NodeType::Verification, Vec2::zero());
        n2.set_parameter("primal_name".to_owned(), "b".to_owned());
        n2.set_parameter("timeout".to_owned(), "30".to_owned());
        let id2 = n2.id.clone();
        let mut n3 = GraphNode::new(NodeType::Verification, Vec2::zero());
        n3.set_parameter("primal_name".to_owned(), "c".to_owned());
        n3.set_parameter("timeout".to_owned(), "30".to_owned());
        let id3 = n3.id.clone();
        graph.add_node(n1);
        graph.add_node(n2);
        graph.add_node(n3);
        graph
            .add_edge(GraphEdge::dependency(id1.clone(), id2))
            .expect("edge");
        graph
            .add_edge(GraphEdge::dependency(id1, id3))
            .expect("edge");
        let mut result = ValidationResult::new();
        validate_edges(&graph, &mut result);
        assert!(result.is_valid());
    }
}
