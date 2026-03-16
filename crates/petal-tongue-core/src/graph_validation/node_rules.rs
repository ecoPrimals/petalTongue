// SPDX-License-Identifier: AGPL-3.0-or-later
//! Node validation rules: required parameters and type-specific checks.

use crate::graph_builder::{NodeType, VisualGraph};

use super::types::{ValidationIssue, ValidationResult};

/// Validate all nodes in the graph.
pub(super) fn validate_nodes(graph: &VisualGraph, result: &mut ValidationResult) {
    for node in &graph.nodes {
        let node_id = &node.id;
        // Check for missing required parameters
        if !node.has_all_required_parameters() {
            let missing = node.missing_parameters();
            result.add_issue(
                ValidationIssue::node_error(
                    node_id.clone(),
                    format!(
                        "Node '{}' missing required parameters: {}",
                        node_id,
                        missing.join(", ")
                    ),
                )
                .with_suggestion(
                    "Fill in all required parameters in the Property Panel".to_string(),
                ),
            );
        }

        // Type-specific validation
        match node.node_type {
            NodeType::PrimalStart => {
                // Should have primal_name and family_id
                if node.get_parameter("primal_name").is_none() {
                    result.add_issue(ValidationIssue::node_error(
                        node_id.clone(),
                        "PrimalStart node requires 'primal_name' parameter".to_string(),
                    ));
                }
            }
            NodeType::Verification => {
                // Should have primal_name
                if node.get_parameter("primal_name").is_none() {
                    result.add_issue(ValidationIssue::node_error(
                        node_id.clone(),
                        "Verification node requires 'primal_name' parameter".to_string(),
                    ));
                }
            }
            NodeType::WaitFor => {
                // Should have condition and timeout
                if node.get_parameter("condition").is_none() {
                    result.add_issue(ValidationIssue::node_error(
                        node_id.clone(),
                        "WaitFor node requires 'condition' parameter".to_string(),
                    ));
                }
            }
            NodeType::Conditional => {
                // Should have condition
                if node.get_parameter("condition").is_none() {
                    result.add_issue(ValidationIssue::node_error(
                        node_id.clone(),
                        "Conditional node requires 'condition' parameter".to_string(),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_builder::{GraphNode, NodeType, Vec2, VisualGraph};
    use crate::graph_validation::types::ValidationResult;

    #[test]
    fn test_valid_primal_start() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        node.set_parameter("primal_name".to_string(), "x".to_string());
        node.set_parameter("family_id".to_string(), "f1".to_string());
        graph.add_node(node);
        let mut result = ValidationResult::new();
        validate_nodes(&graph, &mut result);
        assert!(result.is_valid());
    }

    #[test]
    fn test_invalid_primal_start_missing_params() {
        let mut graph = VisualGraph::new("g".to_string());
        graph.add_node(GraphNode::new(NodeType::PrimalStart, Vec2::zero()));
        let mut result = ValidationResult::new();
        validate_nodes(&graph, &mut result);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_verification_requires_primal_name() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut node = GraphNode::new(NodeType::Verification, Vec2::zero());
        node.set_parameter("timeout".to_string(), "30".to_string());
        graph.add_node(node);
        let mut result = ValidationResult::new();
        validate_nodes(&graph, &mut result);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_waitfor_requires_condition() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut node = GraphNode::new(NodeType::WaitFor, Vec2::zero());
        node.set_parameter("timeout".to_string(), "30".to_string());
        graph.add_node(node);
        let mut result = ValidationResult::new();
        validate_nodes(&graph, &mut result);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_conditional_requires_condition() {
        let mut graph = VisualGraph::new("g".to_string());
        graph.add_node(GraphNode::new(NodeType::Conditional, Vec2::zero()));
        let mut result = ValidationResult::new();
        validate_nodes(&graph, &mut result);
        assert!(!result.is_valid());
    }

    #[test]
    fn test_primal_start_missing_primal_name_only() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        node.set_parameter("family_id".to_string(), "f1".to_string());
        graph.add_node(node);
        let mut result = ValidationResult::new();
        validate_nodes(&graph, &mut result);
        assert!(!result.is_valid());
        assert!(
            result
                .errors()
                .iter()
                .any(|e| e.message.contains("primal_name"))
        );
    }

    #[test]
    fn test_verification_valid() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut node = GraphNode::new(NodeType::Verification, Vec2::zero());
        node.set_parameter("primal_name".to_string(), "x".to_string());
        node.set_parameter("timeout".to_string(), "30".to_string());
        graph.add_node(node);
        let mut result = ValidationResult::new();
        validate_nodes(&graph, &mut result);
        assert!(result.is_valid());
    }

    #[test]
    fn test_waitfor_valid() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut node = GraphNode::new(NodeType::WaitFor, Vec2::zero());
        node.set_parameter("condition".to_string(), "healthy".to_string());
        node.set_parameter("timeout".to_string(), "30".to_string());
        graph.add_node(node);
        let mut result = ValidationResult::new();
        validate_nodes(&graph, &mut result);
        assert!(result.is_valid());
    }

    #[test]
    fn test_conditional_valid() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut node = GraphNode::new(NodeType::Conditional, Vec2::zero());
        node.set_parameter("condition".to_string(), "cpu > 80".to_string());
        graph.add_node(node);
        let mut result = ValidationResult::new();
        validate_nodes(&graph, &mut result);
        assert!(result.is_valid());
    }

    #[test]
    fn test_node_error_contains_suggestion() {
        let mut graph = VisualGraph::new("g".to_string());
        graph.add_node(GraphNode::new(NodeType::PrimalStart, Vec2::zero()));
        let mut result = ValidationResult::new();
        validate_nodes(&graph, &mut result);
        let errors = result.errors();
        let err = errors.first().expect("should have error");
        assert!(err.suggestion.is_some());
    }

    #[test]
    fn test_multiple_nodes_mixed_valid_invalid() {
        let mut graph = VisualGraph::new("g".to_string());
        let mut valid = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        valid.set_parameter("primal_name".to_string(), "x".to_string());
        valid.set_parameter("family_id".to_string(), "f1".to_string());
        graph.add_node(valid);
        graph.add_node(GraphNode::new(NodeType::Conditional, Vec2::zero()));
        let mut result = ValidationResult::new();
        validate_nodes(&graph, &mut result);
        assert!(!result.is_valid());
        assert!(!result.errors().is_empty());
    }

    #[test]
    fn test_empty_graph_valid() {
        let graph = VisualGraph::new("g".to_string());
        let mut result = ValidationResult::new();
        validate_nodes(&graph, &mut result);
        assert!(result.is_valid());
    }

    #[test]
    fn test_primal_start_requires_family_id_implied_by_has_all_required() {
        let mut node = GraphNode::new(NodeType::PrimalStart, Vec2::zero());
        node.set_parameter("primal_name".to_string(), "p".to_string());
        assert!(!node.has_all_required_parameters());
    }
}
